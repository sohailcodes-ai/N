# N Genesis — Economy & Market Design

## Overview

Phase 3 implements the first complete economic loop in N:

```
RESOURCE → PRODUCTION → INVENTORY → MARKET → PURCHASE → CONSUMPTION
```

Food has a causal lifecycle inside the simulation. Agents need food, food must be produced, production consumes inputs, produced food enters inventories, food reaches a market, agents purchase food using money, purchased food enters agent inventory, and agents consume food according to the Phase 2 needs system.

## Resource Model

Resources are physical quantities tracked in inventories.

### Resource Types

| Resource | Name | Description |
|----------|------|-------------|
| `Food` | `"food"` | Edible food consumed by agents |
| `Water` | `"water"` | Drinking water consumed by agents |
| `RawFood` | `"raw_food"` | Agricultural input for food production |

### ResourceInventory

Generic typed inventory with overflow protection:
- `add(resource, amount)` → `Result<u32, String>`
- `remove(resource, amount)` → `Result<u32, String>`
- `quantity(resource)` → `u32`
- `has(resource)` → `bool`

All operations enforce: no negative quantities, no overflow.

## Production System

### Recipe

A recipe defines inputs → outputs with labor requirements:

```rust
Recipe {
    name: "food_production",
    inputs: [RawFood × 10],
    outputs: [Food × 10],
    labor_required: 1,
    cooldown_ticks: 1,
}
```

### Production Cycle

1. Check company has the recipe
2. Check enough workers (status == Working)
3. Check inputs available in company inventory
4. Check cooldown is 0
5. Consume inputs from company inventory
6. Compute output quantity: `base × (1.0 + skill × 0.3)` (capped at 1.5×)
7. Add outputs to company inventory
8. Set cooldown
9. Emit ProductionCompleted event

### Skill Multiplier

```
productivity_multiplier = 1.0 + average_worker_skill × 0.3
```

Capped at 1.5×. High skill produces more food per cycle.

## Company Production

Companies with `CompanyType::FoodProducer`:
- Own inventory (HashMap<String, u32>)
- Have a recipe (`recipe_name: Option<String>`)
- Track production cooldown
- Employees provide labor when Working

### Genesis Configuration

- 5 food producer companies (company-000 through company-004)
- 5 service companies (company-005 through company-009)
- Each food producer starts with 500 raw_food + 50 food

## Labor

Agents work for companies during work hours (9-17).

When an agent has `status == Working` and `employer == Some(company_id)`:
- Their productivity skill contributes to production
- They receive wages (10 N per work tick)
- Company pays wages from cash

### Wage Payment

- Only paid to agents with `status == Working`
- Amount: 10 N per tick per working employee
- Company cash → agent money
- If company can't afford all wages, pays as many as possible

## Market System

### MarketListing

```rust
MarketListing {
    seller_id: "company-000",
    resource: "food",
    quantity: 10,
    price: 2.0,
    original_quantity: 10,
}
```

### Market Transaction

```rust
MarketTransaction {
    buyer_id: "agent-0000",
    seller_id: "company-000",
    resource: "food",
    quantity: 3,
    price: 2.0,
    total_cost: 6.0,
    tick: 42,
}
```

### Market Update (per tick)

1. Companies with food in inventory add listings
2. Supply/demand computed from recent transactions
3. Price adjusts: `new_price = current + demand_pressure - supply_pressure`
4. Price bounded to [0.50, 10.00]

## Agent Food Purchasing

### Decision Logic

```
if hunger > 0.4 AND food_in_inventory < 5:
    desired = min(5 - current_food, max(deficit, urgency), 3)
    affordable = min(desired, money / price)
    buy from cheapest nearby listing
```

### Transaction Execution

1. Validate seller has enough quantity
2. Validate buyer has enough money
3. Atomically: buyer money -= cost, seller money += cost
4. Atomically: buyer inventory += quantity, seller listing -= quantity
5. Emit FoodPurchased event

## Money Flow

```
Agent works → company pays wage → agent money increases
Company produces food → lists on market → agent buys food → company revenue increases
Agent consumes food → hunger decreases
```

### Conservation

Total money in the world = agents money + companies cash.

Money never appears from nowhere. All money movement is traceable through events.

### Genesis Metrics (7-day soak)

- Total money: 15,000 N (conserved, delta = 0.00)
- Agents: 10,300 N | Companies: 4,700 N
- Food produced: 160 units
- Food consumed: 224 units
- Food purchased: 60 transactions
- Zero invariant violations

## Economic Invariants

1. No negative inventory (u32 quantities, Result-based errors)
2. No negative money (f64 validated before transfers)
3. No money from nowhere (total money conserved)
4. No food from nowhere (production consumes inputs)
5. No duplicated resources (inventory is authoritative)
6. Invalid transactions rejected (seller has goods, buyer has money)
7. Failed transactions are atomic (no partial state mutation)
8. Failed production is atomic (inputs not consumed on failure)
9. Company revenue comes only from transactions
10. Prices remain positive and bounded [0.50, 10.00]

## Simulation Tick Order

```
1. Clock advancement
2. Needs update (hunger/thirst/fatigue increase)
3. Agent purchasing (buy food from market if hungry)
4. Agent routines (eat/drink/sleep/work/idle)
5. Labor contribution (working agents → company labor)
6. Production (companies with recipes produce goods)
7. Market update (listings, pricing, cleanup)
8. Wage payment (companies pay working employees)
```

Deterministic: same seed + same tick count = identical state.

## Events

Economic events recorded:
- `ProductionStarted` — company begins production
- `ProductionCompleted` — production outputs generated
- `MarketListingCreated` — company lists food on market
- `FoodPurchased` — agent buys food from market
- `WagePaid` — company pays employee wages
- `PriceChanged` — market price adjusted by supply/demand
