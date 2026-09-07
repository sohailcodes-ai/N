use n_simulation::{AgentStatus, EventType, World, TICKS_PER_DAY, TICKS_PER_HOUR};

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

    println!("N Genesis -- Phase 4 Economy Soak Test");
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

    let company_ids: Vec<String> = world.state.companies.keys().cloned().collect();
    for cid in &company_ids {
        let employees: Vec<String> = world
            .state
            .companies
            .get(cid)
            .unwrap()
            .employees
            .keys()
            .cloned()
            .collect();
        for emp_id in &employees {
            if let Some(agent) = world.state.agents.get_mut(emp_id) {
                agent.employer = Some(cid.clone());
            }
        }
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

    for tick_num in 0..total_ticks {
        world.tick();

        let events_before = world.state.events.len();
        for event in &world.state.events[events_before.saturating_sub(20)..] {
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
                "{} | sleep: {:>2} work: {:>2} eat: {:>2} drink: {:>2} idle: {:>2} | food: {:.2} water: {:.2} | events: {}",
                clock_str, sleeping, working, eating, drinking, idle, food_price, water_price, world.state.events.len()
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

    println!("\n=== PHASE 4 ECONOMY METRICS ===");
    println!("Total events: {}", world.state.events.len());
    println!("Food produced: {}", total_food_produced);
    println!("Food consumed (eating): {}", total_food_consumed);
    println!("Food purchased (market): {}", total_food_purchased);
    println!("Water produced: {}", total_water_produced);
    println!("Water consumed (drinking): {}", total_water_consumed);
    println!("Water purchased (market): {}", total_water_purchased);
    println!(
        "Total money: initial={:.2} final={:.2} delta={:.2}",
        initial_total_money,
        final_total_money,
        final_total_money - initial_total_money
    );

    let agents_money: f64 = world.state.agents.values().map(|a| a.money).sum();
    let companies_cash: f64 = world.state.companies.values().map(|c| c.cash).sum();
    let companies_revenue: f64 = world.state.companies.values().map(|c| c.revenue).sum();
    println!("Agents total money: {:.2}", agents_money);
    println!("Companies total cash: {:.2}", companies_cash);
    println!("Companies total revenue: {:.2}", companies_revenue);

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
    println!("Final food price: {:.2}", food_price);
    println!("Final water price: {:.2}", water_price);

    let agents_with_zero_food = world
        .state
        .agents
        .values()
        .filter(|a| a.inventory.resource_quantity("food") == 0)
        .count();
    let agents_with_zero_water = world
        .state
        .agents
        .values()
        .filter(|a| a.inventory.resource_quantity("water") == 0)
        .count();
    let agents_with_zero_money = world
        .state
        .agents
        .values()
        .filter(|a| a.money <= 0.0)
        .count();
    println!("Agents with zero food: {}", agents_with_zero_food);
    println!("Agents with zero water: {}", agents_with_zero_water);
    println!("Agents with zero money: {}", agents_with_zero_money);

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
        for (res, qty) in &agent.inventory.resources {
            if *qty == 0 && agent.inventory.has_resource(res) {
                println!(
                    "VIOLATION: {} has zero-but-present inventory for {}",
                    agent.id, res
                );
                violations += 1;
            }
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

    let money_check = (final_total_money - initial_total_money).abs();
    if money_check > 0.01 {
        println!(
            "VIOLATION: Money not conserved: initial={:.2} final={:.2}",
            initial_total_money, final_total_money
        );
        violations += 1;
    }

    if violations == 0 {
        println!("ALL PHASE 4 ECONOMIC INVARIANTS PASSED.");
    } else {
        println!("{} INVARIANT VIOLATIONS!", violations);
    }

    println!("\n=== N WORLD CLOCK (Final) ===");
    println!("Date: {}", format_clock(world.clock.tick));
    println!(
        "Day {} of year {}",
        world.clock.day_of_month(),
        world.clock.year()
    );
}
