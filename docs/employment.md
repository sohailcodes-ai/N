# N Genesis — Employment & Labor Markets

## Overview

Phase 5 implements dynamic employment: companies hire/fire workers, agents apply for jobs, wages are paid, and companies track profitability. The labor market is competitive — agents choose the best available offer, and companies that can't attract workers or go insolvent close down.

## Labor Market State

```rust
pub struct LaborMarketState {
    pub employments: Vec<Employment>,
    pub job_openings: Vec<JobOffer>,
    pub last_update_tick: u64,
}
```

### Employment

```rust
pub struct Employment {
    pub agent_id: String,
    pub company_id: String,
    pub role: String,
    pub wage: f64,
    pub start_tick: u64,
    pub active: bool,
}
```

Active employments are pruned at end of each tick. Dead entries (active=false) are removed to prevent O(T²) scan growth.

### JobOffer

```rust
pub struct JobOffer {
    pub company_id: String,
    pub role: String,
    pub wage: f64,
    pub openings: u32,
    pub min_skill_level: f64,
    pub posted_tick: u64,
    pub filled: u32,
}
```

Openings with `openings == 0` are pruned at end of `process_hiring`.

## Tick Lifecycle (Employment Steps)

```
process_job_evaluation  →  unemployed agents apply to best openings
process_hiring          →  understaffed companies create new job offers
process_wages           →  companies pay employed agents (work hours, hourly)
process_firing          →  unprofitable companies fire workers
process_company_profit  →  track company P&L, trigger insolvency checks
process_company_insolvency → close companies below cash threshold
```

### process_job_evaluation

For each unemployed agent (sorted by id):
1. Find the highest-paying opening with `min_skill_level <= agent_skill`
2. Deduct `HIRE_SKILL_COST` (0.02) from agent's relevant skill
3. Create employment record
4. Set `agent.employer = Some(company_id)`
5. Emit `Hired` event
6. Decrement `openings` on the offer

### process_hiring

For each company (sorted by id):
1. Skip if `last_hire` cooldown not elapsed (60 ticks)
2. Compute `needed = desired_employees - current_employee_count`
3. If existing openings >= needed, skip
4. Otherwise create a new `JobOffer` with `openings = needed`
5. Set `last_hire = current_tick`

### process_wages

Wages are paid **only during work hours** (ticks 540–959 per day) and **only once per hour** (when `minute_of_hour == 0`).

For each company (sorted by id):
1. Find all active employments for this company
2. Pay `employment.wage` per working employee
3. `company.cash -= total_wages`
4. `agent.money += employment.wage` for each paid agent
5. Emit `WagePaid` events
6. If company can't afford wages, pays as many as possible (sorted by agent id)

### process_firing

For each company:
1. Compute profit = revenue - expenses
2. If profit < 0, increment `loss_streak`
3. If profit >= 0, reset `loss_streak` to 0
4. If `loss_streak >= COMPANY_LOSS_STREAK_TO_FIRE` (3):
   - Terminate all employments for this company
   - Mark company inactive
   - Emit `CompanyClosed` event

### process_company_profit / insolvency

Separate from firing: companies below `COMPANY_INSOLVENCY_THRESHOLD` (100 N) with sufficient loss streak are forcibly closed.

## Company State

```rust
pub struct CompanyState {
    pub id: String,
    pub name: String,
    pub company_type: CompanyType,
    pub cash: f64,
    pub revenue: f64,
    pub expenses: f64,
    pub profit: f64,
    pub loss_streak: u32,
    pub desired_employees: u32,
    pub last_hire_tick: u64,
    pub employees: Vec<String>,
    pub active: bool,
    // ... inventory, recipe, etc.
}
```

### CompanyType

- `FoodProducer` — produces food from raw_food
- `WaterProducer` — produces water
- `Service` — no production, employs agents for future use

### CompanyStatus

- `Active` — operating normally
- `Struggling` — loss_streak > 0 but below firing threshold
- `Insolvent` — below cash threshold, pending closure
- `Closed` — no longer operating

## Genesis Bootstrap

- 3 companies: company-000 (Food), company-001 (Food), company-002 (Service)
- Seed 42: company-000 hires agent-0000 (skill 0.5), company-001 hires agent-0005 (skill 0.5)
- Wages: 10 N/tick (paid hourly during work hours = 90 N/day)
- Cash: companies start with 1000 N
- Desired employees: 3 per company

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `WAGE_PER_TICK` | 10.0 | Default wage per work tick |
| `HIRE_SKILL_COST` | 0.02 | Skill deduction when hired |
| `HIRE_COOLDOWN` | 60 | Ticks between job postings |
| `COMPANY_INSOLVENCY_THRESHOLD` | 100.0 | Cash below this = insolvent |
| `COMPANY_LOSS_STREAK_TO_FIRE` | 3 | Consecutive losses before firing |
| `MIN_EMPLOYEES_FOR_PRODUCTION` | 1 | Minimum workers to run recipe |

## Economic Flow

```
Company needs workers
→ Creates JobOffer with openings
→ Unemployed agents evaluate openings
→ Best agent gets hired (highest wage, meets skill)
→ Agent works during work hours (9-17)
→ Company pays wages hourly
→ Company tracks revenue vs expenses
→ Profitable: hire more, grow
→ Unprofitable: loss_streak++
→ Loss streak >= 3: fire all, close company
```

## Invariants

1. An agent cannot be employed by two companies simultaneously
2. Employed agents receive wages from their employer only
3. Companies cannot pay more wages than they have in cash
4. Firing releases all employees for that company
5. Closed companies have zero active employments
6. Money is conserved: total world money = agent money + company cash
7. Job openings are pruned when filled (openings == 0)
8. Dead employments are pruned at end of each tick

## Tests

91 unit tests covering:
- Employment creation and consistency
- Agent cannot be double-employed
- Firing releases agents
- Wage payment reduces company cash and increases agent money
- Insolvency closes company and releases employees
- Money conservation over long runs
- Job openings created for understaffed companies
- Deterministic replay (Phase 3/4/5)
- 7-day and 30-day soak tests with all invariants enforced
