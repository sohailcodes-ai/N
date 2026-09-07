use n_simulation::{AgentStatus, CompanyType, EventType, World, TICKS_PER_DAY, TICKS_PER_HOUR};

fn format_clock(tick: u64) -> String {
    let minute = tick % TICKS_PER_HOUR;
    let hour = (tick / TICKS_PER_HOUR) % 24;
    let day_of_week = (tick / TICKS_PER_DAY) % 7;
    let day_of_month = (tick / TICKS_PER_DAY) % 30 + 1;
    let month = (tick / (TICKS_PER_DAY * 30)) % 12 + 1;
    let year = tick / (TICKS_PER_DAY * 30 * 12) + 1;
    let weekday = match day_of_week {
        0 => "Mon",
        1 => "Tue",
        2 => "Wed",
        3 => "Thu",
        4 => "Fri",
        5 => "Sat",
        6 => "Sun",
        _ => "???",
    };
    format!(
        "Y{:04}-M{:02}-D{:02}({}) {:02}:{:02}",
        year, month, day_of_month, weekday, hour, minute
    )
}

fn main() {
    let seed = 1234;
    let num_agents = 50;
    let num_companies = 10;
    let days = 7;
    let total_ticks = TICKS_PER_DAY * days;

    println!("N Genesis -- Phase 5 Employment & Labor Markets");
    println!("Seed: {}", seed);
    println!("Agents: {}", num_agents);
    println!("Companies: {}", num_companies);
    println!("Simulating {} days ({} ticks)...", days, total_ticks);
    println!("---");

    let mut world = World::initialize(seed, num_agents, num_companies);

    let agent_ids: Vec<String> = world.state.agents.keys().cloned().collect();
    for id in &agent_ids {
        let agent = world.state.agents.get_mut(id).unwrap();
        let _ = agent.inventory.add_resource("food", 5);
        let _ = agent.inventory.add_resource("water", 5);
    }

    let initial_total_money = {
        let agents_money: f64 = world.state.agents.values().map(|a| a.money).sum();
        let companies_cash: f64 = world.state.companies.values().map(|c| c.cash).sum();
        agents_money + companies_cash
    };

    let mut total_food_produced = 0u64;
    let mut total_food_consumed = 0u64;
    let mut total_food_purchased = 0u64;
    let mut total_water_produced = 0u64;
    let mut total_water_consumed = 0u64;
    let mut total_water_purchased = 0u64;
    let mut total_hires = 0u64;
    let mut total_fires = 0u64;
    let mut total_wages_paid = 0.0f64;

    for tick_num in 0..total_ticks {
        world.tick();

        let events_before = world.state.events.len();
        for event in &world.state.events[events_before.saturating_sub(30)..] {
            match event.event_type {
                EventType::ProductionCompleted => {
                    if event.state_snapshot.contains("food") {
                        total_food_produced += 1;
                    } else if event.state_snapshot.contains("water") {
                        total_water_produced += 1;
                    }
                }
                EventType::AgentAte => total_food_consumed += 1,
                EventType::FoodPurchased => total_food_purchased += 1,
                EventType::AgentDrank => total_water_consumed += 1,
                EventType::WaterPurchased => total_water_purchased += 1,
                EventType::AgentHired => total_hires += 1,
                EventType::AgentFired => total_fires += 1,
                EventType::WagePaid => {
                    if let Some(amount) = event
                        .state_snapshot
                        .split("Paid ")
                        .nth(1)
                        .and_then(|s| s.split(" N").next())
                        .and_then(|s| s.trim().parse::<f64>().ok())
                    {
                        total_wages_paid += amount;
                    }
                }
                _ => {}
            }
        }

        if tick_num % (TICKS_PER_DAY) == 0 && tick_num > 0 {
            let clock_str = format_clock(world.clock.tick);
            let sleeping = world
                .state
                .agents
                .values()
                .filter(|a| a.status == AgentStatus::Sleeping)
                .count();
            let working = world
                .state
                .agents
                .values()
                .filter(|a| a.status == AgentStatus::Working)
                .count();
            let eating = world
                .state
                .agents
                .values()
                .filter(|a| a.status == AgentStatus::Eating)
                .count();
            let drinking = world
                .state
                .agents
                .values()
                .filter(|a| a.status == AgentStatus::Drinking)
                .count();
            let idle = world
                .state
                .agents
                .values()
                .filter(|a| a.status == AgentStatus::Idle)
                .count();

            let employed = world.state.labor_market.active_employments().len();
            let unemployed = world
                .state
                .labor_market
                .unemployed_count(world.state.agents.len());
            let open_pos = world.state.labor_market.open_positions();
            let avg_wage = world.state.labor_market.average_wage();

            let food_price = world
                .state
                .markets
                .prices
                .get("food")
                .copied()
                .unwrap_or(0.0);
            let water_price = world
                .state
                .markets
                .prices
                .get("water")
                .copied()
                .unwrap_or(0.0);

            println!(
                "{} | sleep: {:>2} work: {:>2} eat: {:>2} drink: {:>2} idle: {:>2} | employed: {:>2} unemployed: {:>2} open: {:>2} avg_wage: {:.1} | food: {:.2} water: {:.2}",
                clock_str, sleeping, working, eating, drinking, idle, employed, unemployed, open_pos, avg_wage, food_price, water_price
            );
        }
    }

    println!("---");
    println!(
        "Simulation complete. {} ticks processed.",
        world.tick_count()
    );

    let final_total_money = {
        let agents_money: f64 = world.state.agents.values().map(|a| a.money).sum();
        let companies_cash: f64 = world.state.companies.values().map(|c| c.cash).sum();
        agents_money + companies_cash
    };

    let employed = world.state.labor_market.active_employments().len();
    let unemployed = world
        .state
        .labor_market
        .unemployed_count(world.state.agents.len());
    let open_pos = world.state.labor_market.open_positions();
    let avg_wage = world.state.labor_market.average_wage();
    let median_wage = world.state.labor_market.median_wage();

    println!("\n=== N WORLD TIME ===");
    println!("Date: {}", format_clock(world.clock.tick));
    println!(
        "Day {} of year {}",
        world.clock.day_of_month(),
        world.clock.year()
    );

    println!("\n=== POPULATION ===");
    println!("Total agents: {}", world.state.agents.len());
    println!("Employed: {}", employed);
    println!("Unemployed: {}", unemployed);
    println!("Open positions: {}", open_pos);
    println!("Average wage: {:.2} N/tick", avg_wage);
    println!("Median wage: {:.2} N/tick", median_wage);

    println!("\n=== COMPANIES ===");
    let total_companies = world.state.companies.len();
    let operating = world
        .state
        .companies
        .values()
        .filter(|c| c.active && c.status != n_simulation::CompanyStatus::Insolvent)
        .count();
    let closed = world.state.companies.values().filter(|c| !c.active).count();
    let profitable = world
        .state
        .companies
        .values()
        .filter(|c| c.active && c.profit_loss > 0.0)
        .count();
    let unprofitable = world
        .state
        .companies
        .values()
        .filter(|c| c.active && c.profit_loss < 0.0)
        .count();

    println!("Total companies: {}", total_companies);
    println!("Operating: {}", operating);
    println!("Closed: {}", closed);
    println!("Profitable: {}", profitable);
    println!("Unprofitable: {}", unprofitable);

    println!("\n=== PRODUCTION ===");
    println!("Food produced: {}", total_food_produced);
    println!("Food consumed (eating): {}", total_food_consumed);
    println!("Food purchased (market): {}", total_food_purchased);
    println!("Water produced: {}", total_water_produced);
    println!("Water consumed (drinking): {}", total_water_consumed);
    println!("Water purchased (market): {}", total_water_purchased);

    println!("\n=== PRICES ===");
    let food_price = world
        .state
        .markets
        .prices
        .get("food")
        .copied()
        .unwrap_or(0.0);
    let water_price = world
        .state
        .markets
        .prices
        .get("water")
        .copied()
        .unwrap_or(0.0);
    println!("Food price: {:.2} N", food_price);
    println!("Water price: {:.2} N", water_price);

    println!("\n=== EMPLOYMENT FLOWS ===");
    println!("Total hires: {}", total_hires);
    println!("Total fires: {}", total_fires);
    println!("Total wages paid: {:.2} N", total_wages_paid);

    println!("\n=== MONEY ===");
    let agents_money: f64 = world.state.agents.values().map(|a| a.money).sum();
    let companies_cash: f64 = world.state.companies.values().map(|c| c.cash).sum();
    let total_revenue: f64 = world
        .state
        .companies
        .values()
        .map(|c| c.cumulative_profit)
        .sum();
    println!("Agent money: {:.2} N", agents_money);
    println!("Company cash: {:.2} N", companies_cash);
    println!("Total world money: {:.2} N", final_total_money);
    println!(
        "Money conservation: initial={:.2} final={:.2} delta={:.2}",
        initial_total_money,
        final_total_money,
        final_total_money - initial_total_money
    );

    println!("\n=== COMPANY HIGHLIGHTS ===");
    if let Some((best_id, best_wage)) = world.state.labor_market.highest_paying_company() {
        let name = world
            .state
            .companies
            .get(&best_id)
            .map(|c| c.name.clone())
            .unwrap_or_default();
        println!(
            "Highest-paying company: {} ({}) at {:.1} N/tick",
            name, best_id, best_wage
        );
    }
    if let Some((emp_id, emp_count)) = world.state.labor_market.largest_employer() {
        let name = world
            .state
            .companies
            .get(&emp_id)
            .map(|c| c.name.clone())
            .unwrap_or_default();
        println!(
            "Largest employer: {} ({}) with {} employees",
            name, emp_id, emp_count
        );
    }

    let mut best_revenue_company = None;
    let mut best_revenue = f64::NEG_INFINITY;
    let mut best_profit_company = None;
    let mut best_profit = f64::NEG_INFINITY;
    for company in world.state.companies.values() {
        if company.active {
            if company.revenue > best_revenue {
                best_revenue = company.revenue;
                best_revenue_company = Some(company.name.clone());
            }
            if company.profit_loss > best_profit {
                best_profit = company.profit_loss;
                best_profit_company = Some(company.name.clone());
            }
        }
    }
    if let Some(name) = best_revenue_company {
        println!("Highest-revenue company: {} ({:.2} N)", name, best_revenue);
    }
    if let Some(name) = best_profit_company {
        println!("Most profitable company: {} ({:.2} N)", name, best_profit);
    }

    println!("\n=== MARKET DEPTH ===");
    let depth = &world.state.markets.depth;
    println!(
        "Food:  bid_depth={} ask_depth={}",
        depth.food_bid_depth, depth.food_ask_depth
    );
    println!(
        "Water: bid_depth={} ask_depth={}",
        depth.water_bid_depth, depth.water_ask_depth
    );

    let water_volume = world
        .state
        .markets
        .total_volume
        .get("water")
        .copied()
        .unwrap_or(0);
    let food_volume = world
        .state
        .markets
        .total_volume
        .get("food")
        .copied()
        .unwrap_or(0);
    println!("Food trade volume: {}", food_volume);
    println!("Water trade volume: {}", water_volume);

    println!("\n=== INVARIANT CHECK ===");
    let mut violations = 0;
    for agent in world.state.agents.values() {
        if agent.money < 0.0 {
            println!(
                "VIOLATION: {} has negative money: {:.2}",
                agent.id, agent.money
            );
            violations += 1;
        }
        if agent.needs.hunger < 0.0 || agent.needs.hunger > 1.0 {
            println!(
                "VIOLATION: {} hunger out of range: {:.2}",
                agent.id, agent.needs.hunger
            );
            violations += 1;
        }
        if agent.needs.thirst < 0.0 || agent.needs.thirst > 1.0 {
            println!(
                "VIOLATION: {} thirst out of range: {:.2}",
                agent.id, agent.needs.thirst
            );
            violations += 1;
        }
        if agent.needs.fatigue < 0.0 || agent.needs.fatigue > 1.0 {
            println!(
                "VIOLATION: {} fatigue out of range: {:.2}",
                agent.id, agent.needs.fatigue
            );
            violations += 1;
        }
    }
    for company in world.state.companies.values() {
        if company.cash < 0.0 {
            println!(
                "VIOLATION: {} has negative cash: {:.2}",
                company.id, company.cash
            );
            violations += 1;
        }
    }

    // Employment invariant checks
    for emp in world.state.labor_market.active_employments() {
        let agent = world.state.agents.get(&emp.agent_id).unwrap();
        if agent.employer.as_deref() != Some(emp.company_id.as_str()) {
            println!(
                "VIOLATION: Agent {} employed at {} but employer={:?}",
                emp.agent_id, emp.company_id, agent.employer
            );
            violations += 1;
        }
    }

    // Money conservation
    let money_check = (final_total_money - initial_total_money).abs();
    if money_check > 0.01 {
        println!(
            "VIOLATION: Money not conserved: initial={:.2} final={:.2}",
            initial_total_money, final_total_money
        );
        violations += 1;
    }

    if violations == 0 {
        println!("ALL PHASE 5 ECONOMIC INVARIANTS PASSED.");
    } else {
        println!("{} INVARIANT VIOLATIONS!", violations);
    }
}
