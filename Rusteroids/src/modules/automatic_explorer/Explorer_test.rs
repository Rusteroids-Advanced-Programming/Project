#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    use crossbeam_channel::unbounded;
    use crate::modules::orchestrator::orchestator::Orchestrator;
    use common_game::protocols::orchestrator_explorer::{OrchestratorToExplorer, ExplorerToOrchestrator};
    use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
    use common_game::protocols::planet_explorer::PlanetToExplorer;
    use common_game::components::resource::{BasicResourceType, ComplexResourceType};
    use crate::modules::automatic_explorer::Explorer::SmartExplorer;
    use crate::modules::manual_explorer::bag_type::DummyBag;
    use crate::modules::orchestrator::event_manager::ManageEvents;
    use crate::modules::orchestrator::handler_explorer_ai::HandlerExplorer;
    use crate::modules::orchestrator::initializer::Initializer;

    // --- SCORING CONFIG ---
    const PTS_MAP: u32 = 50;
    const PTS_NET: u32 = 50;
    const PTS_NOMAD: u32 = 30;
    const PTS_HUB: u32 = 30;
    const PTS_HOARD: u32 = 50;
    const PTS_BASIC_30: u32 = 20;
    const PTS_BASIC_5_EACH: u32 = 20;
    const PTS_COMPLEX_ALL: u32 = 100;
    const PTS_VICTORY_LAP: u32 = 100;

    fn calc_resource_score(bag: &DummyBag) -> u32 {
        let mut score = 0;
        score += bag.basic.values().sum::<usize>() as u32;
        for (ctype, count) in &bag.complex {
            let val = match ctype {
                ComplexResourceType::Water => 12,
                ComplexResourceType::Diamond => 12,
                ComplexResourceType::Life => 23,
                ComplexResourceType::Robot => 34,
                ComplexResourceType::Dolphin => 45,
                ComplexResourceType::AIPartner => 56,
            };
            score += (*count as u32) * val;
        }
        score
    }

    #[test]
    fn test_autonomous_explorer_behavior() {
        println!("--- TEST SIMULATION STARTING ---");

        let mut orchestrator = Orchestrator::new(0);
        let orch_arc = Arc::new(RwLock::new(orchestrator));

        {
            let mut orch_guard = orch_arc.write().unwrap();
            orch_guard.initialize();
            println!("[Test] Planets Initialized.");
        }

        let explorer_id = 10;
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let spawn_planet_id = (since_epoch.subsec_nanos() % 8) + 1;
        println!("🎲 [RNG] Explorer spawning at Planet #{}", spawn_planet_id);

        let (to_exp_tx, to_exp_rx) = unbounded::<OrchestratorToExplorer>();
        let (to_orch_tx, to_orch_rx) = unbounded::<ExplorerToOrchestrator<DummyBag>>();
        let (to_exp_planet_tx, to_exp_planet_rx) = unbounded::<PlanetToExplorer>();

        let smart_explorer = SmartExplorer::new(explorer_id, spawn_planet_id, to_exp_rx, to_orch_tx);

        {
            let mut orch = orch_arc.write().unwrap();
            orch.explorer_channels.insert(explorer_id, (to_exp_tx.clone(), to_orch_rx, to_exp_planet_tx, to_exp_planet_rx));
            orch.explorer_planet.write().unwrap().insert(explorer_id, spawn_planet_id);
            let planet_channels = orch.planet_channels.read().unwrap();
            let target_planet_ch = planet_channels.get(&spawn_planet_id).unwrap();
            *smart_explorer.to_planet.write().unwrap() = Some(target_planet_ch.2.clone());
            *smart_explorer.from_planet.write().unwrap() = Some(orch.explorer_channels.get(&explorer_id).unwrap().3.clone());
        }

        let expl_clone = smart_explorer.clone();
        thread::spawn(move || { if let Err(e) = expl_clone.run() { println!("[Test] Explorer stopped: {}", e); } });

        // Listener
        let orch_clone_for_listener = orch_arc.clone();
        thread::spawn(move || {
            let orch = orch_clone_for_listener.read().unwrap();
            let expl_channels = Arc::new(RwLock::new(orch.explorer_channels.get(&explorer_id).unwrap().clone()));
            let planet_channels = orch.planet_channels.clone();
            let galaxy = orch.galaxy_graph.clone();
            let (_, rx_from_exp, _, _) = &*expl_channels.read().unwrap();
            loop {
                match rx_from_exp.recv() {
                    Ok(ExplorerToOrchestrator::NeighborsRequest { explorer_id: _, current_planet_id }) => {
                        let mut neighbors = Vec::new();
                        let graph = galaxy.read().unwrap();
                        for node in &graph.nodes {
                            let n_guard = node.read().unwrap();
                            if n_guard.value == current_planet_id {
                                for adj in &n_guard.adjacent_nodes { neighbors.push(adj.read().unwrap().value); }
                            }
                        }
                        let (tx_to_exp, _, _, _) = &*expl_channels.read().unwrap();
                        tx_to_exp.send(OrchestratorToExplorer::NeighborsResponse { neighbors }).unwrap();
                    },
                    Ok(ExplorerToOrchestrator::TravelToPlanetRequest { explorer_id: eid, current_planet_id: _, dst_planet_id }) => {
                        let p_chans = planet_channels.read().unwrap();
                        if let Some((p_sender, p_receiver, p_expl_sender)) = p_chans.get(&dst_planet_id) {
                            let (tx_to_exp, _, tx_planet_to_expl, _) = &*expl_channels.read().unwrap();
                            let send_res = p_sender.send(OrchestratorToPlanet::IncomingExplorerRequest { explorer_id: eid, new_sender: tx_planet_to_expl.clone() });
                            if send_res.is_ok() {
                                if let Ok(PlanetToOrchestrator::IncomingExplorerResponse{ res, .. }) = p_receiver.recv_timeout(Duration::from_millis(500)) {
                                    if res.is_ok() {
                                        tx_to_exp.send(OrchestratorToExplorer::MoveToPlanet { sender_to_new_planet: Some(p_expl_sender.clone()), planet_id: dst_planet_id }).unwrap();
                                    }
                                }
                            }
                        }
                    },
                    Ok(_) => {},
                    Err(_) => break,
                }
            }
        });

        orch_arc.read().unwrap().reset_explorer(explorer_id);

        let mut tick = 0;
        let mut watchdog_timer = 0;
        let max_stagnation_ticks = 400;
        let mut last_bag_count = 0;
        let mut last_pos = 0;
        let mut victory = false;
        let mut explorer_died = false;

        loop {
            if tick % 5 == 0 { orch_arc.read().unwrap().manage(); }
            thread::sleep(Duration::from_millis(50));
            tick += 1;

            let current_pos = *smart_explorer.current_planet_id.read().unwrap();
            {
                let orch = orch_arc.read().unwrap();
                if let Some(handle) = orch.planet_threads.get(&current_pos) {
                    if handle.is_finished() {
                        explorer_died = true;
                        println!("\n💥 CRITICAL FAILURE: Planet #{} exploded with Explorer on it! 💥", current_pos);
                        break;
                    }
                }
            }

            // --- UPDATED GALAXY REPORT ---
            if tick % 40 == 0 {
                let orch = orch_arc.read().unwrap();
                let planet_threads = &orch.planet_threads;
                let mut alive = Vec::new();
                let mut dead = Vec::new();
                for (id, handle) in planet_threads.iter() {
                    if handle.is_finished() { dead.push(*id); } else { alive.push(*id); }
                }
                alive.sort(); dead.sort();
                let bag = smart_explorer.bag.read().unwrap().to_dummy();
                let current_score = calc_resource_score(&bag);

                // Get the colored risk string
                let risk_display = smart_explorer.get_risk_display();

                println!("\n--- 🌌 GALAXY STATUS (T={:.1}s) 🌌 ---", tick as f32 * 0.05);
                println!(" Explorer Pos: Planet #{} [{}]", current_pos, risk_display);
                println!(" Alive: {:?}", alive);
                if !dead.is_empty() { println!(" 💀 DEAD:  {:?}", dead); }


                //[Image of Solar System]

                println!(" Bag Basic:   {:?}", bag.basic);
                println!(" Bag Complex: {:?}", bag.complex);
                println!(" Current Score: {}", current_score);
                println!("--------------------------------------\n");
            }

            let bag_dummy = smart_explorer.bag.read().unwrap().to_dummy();
            let current_bag_count = bag_dummy.basic.values().sum::<usize>() + bag_dummy.complex.values().sum::<usize>();

            if current_bag_count != last_bag_count || current_pos != last_pos {
                watchdog_timer = 0;
                last_bag_count = current_bag_count;
                last_pos = current_pos;
            } else {
                watchdog_timer += 1;
            }

            let complexes = vec![ComplexResourceType::Water, ComplexResourceType::Diamond, ComplexResourceType::Life, ComplexResourceType::Robot, ComplexResourceType::Dolphin, ComplexResourceType::AIPartner];
            let has_all_complex = complexes.iter().all(|c| *bag_dummy.complex.get(c).unwrap_or(&0) >= 1);
            let total_basic: usize = bag_dummy.basic.values().sum();
            let has_30_basic = total_basic >= 30;
            let has_67_total = current_bag_count >= 67;
            let basics = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen];
            let has_5_each_basic = basics.iter().all(|b| *bag_dummy.basic.get(b).unwrap_or(&0) >= 5);
            let home = current_pos == spawn_planet_id && tick > 100;

            if has_all_complex && has_30_basic && has_67_total && has_5_each_basic && home {
                victory = true;
                break;
            }

            if watchdog_timer >= max_stagnation_ticks {
                println!("\n❌ MISSION ENDED (STAGNATION) ❌");
                break;
            }
        }

        // Final Scoring
        let final_bag = smart_explorer.bag.read().unwrap().to_dummy();
        let total_basic: usize = final_bag.basic.values().sum();
        let total_items = total_basic + final_bag.complex.values().sum::<usize>();

        let t1 = smart_explorer.visited.read().unwrap().len() >= 8;
        let t2 = total_basic >= 30;
        let basics = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen];
        let t3 = basics.iter().all(|b| *final_bag.basic.get(b).unwrap_or(&0) >= 5);
        let complexes = vec![ComplexResourceType::Water, ComplexResourceType::Diamond, ComplexResourceType::Life, ComplexResourceType::Robot, ComplexResourceType::Dolphin, ComplexResourceType::AIPartner];
        let t4 = complexes.iter().all(|c| *final_bag.complex.get(c).unwrap_or(&0) >= 1);
        let kb = smart_explorer.knowledge_base.read().unwrap();
        let mut total_known_edges = 0;
        for data in kb.values() { total_known_edges += data.neighbors.len(); }
        let visited_edges_count = smart_explorer.visited_edges.read().unwrap().len();
        let t5 = visited_edges_count >= total_known_edges && total_known_edges > 0;
        let t6 = smart_explorer.extraction_sites.read().unwrap().len() >= 5;
        let (hub_id, _) = *smart_explorer.hub_candidate.read().unwrap();
        let hub_visits = *smart_explorer.hub_visits.read().unwrap().get(&hub_id).unwrap_or(&0);
        let t7 = hub_visits >= 3;
        let max_bag = *smart_explorer.max_bag_size.read().unwrap();
        let t8 = max_bag >= 67;
        let current_pos = *smart_explorer.current_planet_id.read().unwrap();
        let t9 = victory && current_pos == spawn_planet_id;

        let mut total_score = 0;
        if t1 { total_score += PTS_MAP; }
        if t2 { total_score += PTS_BASIC_30; }
        if t3 { total_score += PTS_BASIC_5_EACH; }
        if t4 { total_score += PTS_COMPLEX_ALL; }
        if t5 { total_score += PTS_NET; }
        if t6 { total_score += PTS_NOMAD; }
        if t7 { total_score += PTS_HUB; }
        if t8 { total_score += PTS_HOARD; }
        if t9 { total_score += PTS_VICTORY_LAP; }

        let res_score = calc_resource_score(&final_bag);
        total_score += res_score;

        let status_msg = if explorer_died { "DIED" } else if victory { "VICTORY" } else { "TIME UP" };

        let report = format!(
            "\n=========================================\n\
             FINAL SCORECARD [{}]\n\
             =========================================\n\
             TASKS:\n\
             [+{:3}] Map Galaxy:        {} ({}/8)\n\
             [+{:3}] >30 Basic:         {} ({})\n\
             [+{:3}] >5 Each Basic:     {}\n\
             [+{:3}] All Complex:       {}\n\
             [+{:3}] Network Tech:      {} ({}/{})\n\
             [+{:3}] Nomad:             {} ({}/5)\n\
             [+{:3}] Hub Hunter:        {} ({} visits)\n\
             [+{:3}] Hoarder >67:       {} (Max: {})\n\
             [+{:3}] Victory Lap:       {}\n\
             -----------------------------------------\n\
             RESOURCES:\n\
             [+{:3}] Inventory Value:   {}\n\
             -----------------------------------------\n\
             TOTAL SCORE: {}\n\
             =========================================",
            status_msg,
            PTS_MAP, if t1 { "✅" } else { "❌" }, smart_explorer.visited.read().unwrap().len(),
            PTS_BASIC_30, if t2 { "✅" } else { "❌" }, total_basic,
            PTS_BASIC_5_EACH, if t3 { "✅" } else { "❌" },
            PTS_COMPLEX_ALL, if t4 { "✅" } else { "❌" },
            PTS_NET, if t5 { "✅" } else { "❌" }, visited_edges_count, total_known_edges,
            PTS_NOMAD, if t6 { "✅" } else { "❌" }, smart_explorer.extraction_sites.read().unwrap().len(),
            PTS_HUB, if t7 { "✅" } else { "❌" }, hub_visits,
            PTS_HOARD, if t8 { "✅" } else { "❌" }, max_bag,
            PTS_VICTORY_LAP, if t9 { "✅" } else { "❌" },
            res_score, res_score,
            total_score
        );

        println!("{}", report);

        if !victory {
            panic!("\n❌ MISSION FAILED: Grand Slam criteria not met. See Scorecard above for details.\n");
        }
    }
}