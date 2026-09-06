use n_simulation::{World, HOURS_PER_DAY};

fn main() {
    let seed = 1234;
    let num_agents = 50;
    let num_companies = 10;
    let days = 7;
    let total_ticks = HOURS_PER_DAY * days;

    println!("N Genesis — Soak Test");
    println!("Seed: {}", seed);
    println!("Agents: {}", num_agents);
    println!("Companies: {}", num_companies);
    println!("Simulating {} days ({} ticks)...", days, total_ticks);
    println!("---");

    let mut world = World::initialize(seed, num_agents, num_companies);

    // Give every agent starting food and water
    let agent_ids: Vec<String> = world.state.agents.keys().cloned().collect();
    for id in &agent_ids {
        let agent = world.state.agents.get_mut(id).unwrap();
        let _ = agent.inventory.add_resource("food", 30);
        let _ = agent.inventory.add_resource("water", 15);
    }

    for tick in 0..total_ticks {
        world.tick();

        if tick % 24 == 0 {
            let day = world.clock.day();
            let hour = world.clock.hour_of_day();
            let sleeping = world
                .state
                .agents
                .values()
                .filter(|a| a.status == n_simulation::AgentStatus::Sleeping)
                .count();
            let working = world
                .state
                .agents
                .values()
                .filter(|a| a.status == n_simulation::AgentStatus::Working)
                .count();
            let eating = world
                .state
                .agents
                .values()
                .filter(|a| a.status == n_simulation::AgentStatus::Eating)
                .count();
            let drinking = world
                .state
                .agents
                .values()
                .filter(|a| a.status == n_simulation::AgentStatus::Drinking)
                .count();

            println!(
                "Day {:>2} Hour {:>2} | sleeping: {:>2} working: {:>2} eating: {:>2} drinking: {:>2} | events: {}",
                day, hour, sleeping, working, eating, drinking, world.state.events.len()
            );
        }
    }

    println!("---");
    println!(
        "Simulation complete. {} ticks processed.",
        world.tick_count()
    );
    println!("Total events: {}", world.state.events.len());
    println!("Agents: {}", world.state.agents.len());
    println!("Companies: {}", world.state.companies.len());

    // Verify invariants
    let mut violations = 0;
    for agent in world.state.agents.values() {
        if agent.money < 0.0 {
            println!("VIOLATION: {} has negative money", agent.id);
            violations += 1;
        }
        if agent.needs.hunger < 0.0 || agent.needs.hunger > 1.0 {
            println!("VIOLATION: {} hunger out of range", agent.id);
            violations += 1;
        }
        if agent.needs.thirst < 0.0 || agent.needs.thirst > 1.0 {
            println!("VIOLATION: {} thirst out of range", agent.id);
            violations += 1;
        }
        if agent.needs.fatigue < 0.0 || agent.needs.fatigue > 1.0 {
            println!("VIOLATION: {} fatigue out of range", agent.id);
            violations += 1;
        }
    }
    for company in world.state.companies.values() {
        if company.cash < 0.0 {
            println!("VIOLATION: {} has negative cash", company.id);
            violations += 1;
        }
    }

    if violations == 0 {
        println!("All economic invariants PASSED.");
    } else {
        println!("{} INVARIANT VIOLATIONS!", violations);
    }
}
