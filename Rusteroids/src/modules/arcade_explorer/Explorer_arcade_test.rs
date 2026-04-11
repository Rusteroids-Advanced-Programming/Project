#[cfg(test)]
mod tests {

    use std::sync::{Arc, RwLock};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    use crossbeam_channel::unbounded;
    use std::io::{self, Write};
    use rand::Rng;
    use crate::modules::orchestrator::orchestator::Orchestrator;
    use common_game::protocols::orchestrator_explorer::{OrchestratorToExplorer, ExplorerToOrchestrator};
    use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
    use common_game::protocols::planet_explorer::PlanetToExplorer;
    use common_game::components::resource::{BasicResourceType, ComplexResourceType};
    use crate::modules::arcade_explorer::Explorer_arcade::{ExplorerGoal, SmartExplorer};
    use crate::modules::manual_explorer::bag_type::DummyBag;
    use crate::modules::orchestrator::event_manager::ManageEvents;
    use crate::modules::orchestrator::handler_explorer_ai::HandlerExplorer;
    use crate::modules::orchestrator::initializer::Initializer;
    // --- SCORING CONFIG ---
    const PTS_MAP_COMPLETE: u32 = 80;
    const PTS_NET: u32 = 100;
    const PTS_NOMAD: u32 = 100;
    const PTS_HUB: u32 = 100;
    const PTS_HOARD: u32 = 1000;
    const PTS_BASIC_30: u32 = 50;
    const PTS_BASIC_5_EACH: u32 = 50;
    const PTS_COMPLEX_ALL: u32 = 900;
    const PTS_VICTORY_LAP: u32 = 900;

    const PTS_MONOPOLIST: u32 = 150;
    const PTS_SURVEYOR: u32 = 300;
    const PTS_EFFICIENCY: u32 = 500;
    const PTS_SPEEDRUN: u32 = 300;
    const PTS_SCAVENGER: u32 = 0;
    const PTS_BACKPACKER: u32 = 400;
    const PTS_DAREDEVIL: u32 = 250;
    const PTS_LOCALVORE: u32 = 200;
    const PTS_INDUSTRIALIST: u32 = 300;
    const PTS_BIZARRE: u32 = 100_000;

    const PTS_PER_PLANET: u32 = 10;
    const PTS_PER_EDGE: u32 = 15;

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

    fn get_user_settings() -> (ExplorerGoal, u32) {
        println!("\n========================================");
        println!("   🚀  SMART EXPLORER SIMULATION  🚀");
        println!("========================================");

        let mode = loop {
            print!("> Select Mission (1=Normal, 2=Secret): ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            match input.trim() {
                "1" => break ExplorerGoal::Normal,
                "2" => break ExplorerGoal::Secret,
                _ => println!("Invalid input. Please enter 1 or 2."),
            }
        };

        let difficulty = loop {
            print!("> Select Difficulty (0=Easy, 1=Medium, 2=Hard, 3=Modular): ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            match input.trim().parse::<u32>() {
                Ok(n) if n <= 3 => break n,
                _ => println!("Invalid input. Please enter 0, 1, 2, or 3."),
            }
        };

        println!("----------------------------------------");
        println!("> Configuration Accepted: {:?} Mode | Difficulty {}", mode, difficulty);
        println!("----------------------------------------\n");

        println!("📜 MISSION BRIEFING DOWNLOADING...");
        thread::sleep(Duration::from_millis(500));

        if mode == ExplorerGoal::Normal {
            println!("\n[OBJECTIVE: TRUE ENDING]");
            println!("1. Map the entire galaxy.");
            println!("2. Hoard resources (>67 items).");
            println!("3. Craft ONE of every complex tech.");
            println!("4. Survey every planet (3x visits).");
            println!("5. Return to Spawn Point safely.");
        } else {
            println!("\n[OBJECTIVE: CRAZY NOISY BIZARRE EXPLORER]");
            println!("⚠ WARNING: EXTREME DIFFICULTY DETECTED ⚠");
            println!("1. COMPLETE ALL NORMAL OBJECTIVES.");
            println!("2. MONOPOLIST: Hold 15+ of ALL basic resources.");
            println!("3. SCAVENGER: Extract from 80% of the galaxy.");
            println!("4. INDUSTRIALIST: Utilize EVERY factory.");
            println!("5. WAIFU COLLECTOR: Craft TWO AI Partners at every AI Factory.");
        }

        println!("\n> SYSTEM: Launch in 3 seconds...");
        thread::sleep(Duration::from_secs(3));
        println!("> SYSTEM: 🚀 LAUNCH INITIATED 🚀\n");

        (mode, difficulty)
    }

    #[test]
    fn test_autonomous_explorer_behavior() {
        let (user_mode, user_difficulty) = get_user_settings();
        println!("--- TEST SIMULATION STARTING ---");

        let mut orchestrator = Orchestrator::new(user_difficulty as u8);
        let orch_arc = Arc::new(RwLock::new(orchestrator));

        {
            let mut orch_guard = orch_arc.write().unwrap();
            orch_guard.initialize();
            println!("[Test] Planets Initialized.");
        }

        let explorer_id = 10;

        // Random Spawn
        let galaxy_size = orch_arc.read().unwrap().galaxy_graph.read().unwrap().nodes.len() as u32;
        let mut rng = rand::thread_rng();
        let spawn_planet_id = rng.gen_range(1..=galaxy_size);
        println!("🎲 [RNG] Detected Galaxy Size: {}", galaxy_size);
        println!("🎲 [RNG] Explorer spawning at Planet #{}", spawn_planet_id);

        let (to_exp_tx, to_exp_rx) = unbounded::<OrchestratorToExplorer>();
        let (to_orch_tx, to_orch_rx) = unbounded::<ExplorerToOrchestrator<DummyBag>>();
        let (to_exp_planet_tx, to_exp_planet_rx) = unbounded::<PlanetToExplorer>();

        let smart_explorer = SmartExplorer::new(explorer_id, spawn_planet_id, to_exp_rx, to_orch_tx);
        smart_explorer.set_goal(user_mode);

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

        orch_arc.read().unwrap().start_explorer(explorer_id);

        let mut tick = 0;
        let mut watchdog_timer = 0;
        let max_stagnation_ticks = 400;
        let mut last_bag_count = 0;
        let mut last_pos = 0;
        let mut early_exit_title: Option<&str> = None;
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

            if tick % 40 == 0 {
                let orch = orch_arc.read().unwrap();
                let planet_threads = &orch.planet_threads;

                // --- NEW PLANET DISPLAY LOGIC ---
                let mut all_ids: Vec<_> = planet_threads.keys().cloned().collect();
                all_ids.sort();

                let planet_status_str: String = all_ids.iter().map(|id| {
                    if let Some(handle) = planet_threads.get(id) {
                        if handle.is_finished() {
                            format!("💀({})", id)
                        } else {
                            format!("{}", id)
                        }
                    } else {
                        format!("?")
                    }
                }).collect::<Vec<String>>().join(", ");
                // --------------------------------

                let bag = smart_explorer.bag.read().unwrap().to_dummy();
                let current_score = calc_resource_score(&bag);
                let risk_display = smart_explorer.get_risk_display();
                let p = smart_explorer.get_mission_progress_strings();

                println!("\n--- 🌌 GALAXY STATUS (T={:.1}s) 🌌 ---", tick as f32 * 0.05);
                println!(" Explorer Pos: Planet #{} [{}]", current_pos, risk_display);
                println!(" Planets: [{}]", planet_status_str); // Display all in one vector
                println!(" Bag: {:?} | {:?}", bag.basic, bag.complex);
                println!(" Current Score: {}", current_score);

                println!("\n > MISSION PROGRESS:");
                println!("   [Map: {}] [Net: {}] [Nomad: {}] [Hub: {}] [30+: {}] [5ea: {}]",
                         p.get("Map").unwrap(), p.get("Net").unwrap(), p.get("Nomad").unwrap(),
                         p.get("Hub").unwrap(), p.get("30+").unwrap(), p.get("5ea").unwrap());
                println!("   [Hoard: {}] [Craft: {}] [Survey: {}] [Eff: {}] [Scav: {}] [Loc: {}]",
                         p.get("Hoard").unwrap(), p.get("Craft").unwrap(), p.get("Survey").unwrap(),
                         p.get("Eff").unwrap(), p.get("Scav").unwrap(), p.get("Loc").unwrap());
                println!("   [Speed: {}] [Dare: {}] [Mono: {}]",
                         p.get("Speed").unwrap(), p.get("Dare").unwrap(), p.get("Mono").unwrap());
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

            // --- CONDITIONS ---
            let complexes = vec![ComplexResourceType::Water, ComplexResourceType::Diamond, ComplexResourceType::Life, ComplexResourceType::Robot, ComplexResourceType::Dolphin, ComplexResourceType::AIPartner];
            let t4_complex = complexes.iter().all(|c| *bag_dummy.complex.get(c).unwrap_or(&0) >= 1);
            let t2_30_basic = bag_dummy.basic.values().sum::<usize>() >= 30;
            let t8_hoard = current_bag_count >= 67;
            let basics = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen];
            let t3_5_each = basics.iter().all(|b| *bag_dummy.basic.get(b).unwrap_or(&0) >= 5);
            let t9_home = current_pos == spawn_planet_id;

            let kb_size = smart_explorer.knowledge_base.read().unwrap().len();
            let visits = smart_explorer.planet_visit_counts.read().unwrap();
            let t_survey = kb_size >= 8 && visits.values().filter(|&&v| v >= 3).count() >= 8;

            let needed_scav = (kb_size as f32 * 0.8).ceil() as usize;
            let t_scav = smart_explorer.extraction_sites.read().unwrap().len() >= needed_scav && needed_scav > 0;

            // NOTE: Localvore removed from mandatory list
            let all_mandatory = t4_complex && t2_30_basic && t8_hoard && t3_5_each && t9_home && t_survey && t_scav;

            // --- FIXED: EXIT LOGIC FOR SECRET MODE ---
            if all_mandatory {
                if user_mode == ExplorerGoal::Normal {
                    early_exit_title = Some("True Ending");
                    break;
                }
                // If Secret Mode, continue running...
            }

            let hist = smart_explorer.crafted_history.read().unwrap();
            let t_mono = basics.iter().all(|b| *bag_dummy.basic.get(b).unwrap_or(&0) >= 15);

            let mut all_factories_used = true;
            let kb = smart_explorer.knowledge_base.read().unwrap();
            for (pid, data) in kb.iter() {
                if data.combines.as_ref().map_or(false, |c| !c.is_empty()) {
                    if !hist.contains_key(pid) { all_factories_used = false; break; }
                }
            }
            let t_indus = all_factories_used && kb_size >= 8;

            let mut bizarre_condition = true;
            for (pid, data) in kb.iter() {
                if data.combines.as_ref().map_or(false, |c| c.contains(&ComplexResourceType::AIPartner)) {
                    let count = hist.get(pid).map_or(0, |list| list.iter().filter(|&&x| x == ComplexResourceType::AIPartner).count());
                    if count < 2 { bizarre_condition = false; break; }
                }
            }

            if all_mandatory && t_mono && t_indus && bizarre_condition {
                early_exit_title = Some("CRAZY NOISY BIZZARE EXPLORER");
                break;
            }

            if watchdog_timer >= max_stagnation_ticks {
                println!("\n❌ MISSION ENDED (STAGNATION) ❌");
                break;
            }
        }

        // --- FINAL EVALUATION ---
        let final_bag = smart_explorer.bag.read().unwrap().to_dummy();
        let total_basic: usize = final_bag.basic.values().sum();
        let kb_size = smart_explorer.knowledge_base.read().unwrap().len();

        let t1 = smart_explorer.visited.read().unwrap().len() >= 8;
        let t2 = total_basic >= 30;
        let basics = vec![BasicResourceType::Carbon, BasicResourceType::Silicon, BasicResourceType::Hydrogen, BasicResourceType::Oxygen];
        let t3 = basics.iter().all(|b| *final_bag.basic.get(b).unwrap_or(&0) >= 5);
        let complexes = vec![ComplexResourceType::Water, ComplexResourceType::Diamond, ComplexResourceType::Life, ComplexResourceType::Robot, ComplexResourceType::Dolphin, ComplexResourceType::AIPartner];
        let t4 = complexes.iter().all(|c| *final_bag.complex.get(c).unwrap_or(&0) >= 1);
        let mut total_known_edges = 0;
        for data in smart_explorer.knowledge_base.read().unwrap().values() { total_known_edges += data.neighbors.len(); }
        let visited_edges_count = smart_explorer.visited_edges.read().unwrap().len();
        let t5 = visited_edges_count >= total_known_edges && total_known_edges > 0;
        let t6 = smart_explorer.extraction_sites.read().unwrap().len() >= 5;
        let (hub_id, _) = *smart_explorer.hub_candidate.read().unwrap();
        let hub_visits = *smart_explorer.hub_visits.read().unwrap().get(&hub_id).unwrap_or(&0);
        let t7 = hub_visits >= 3;
        let max_bag = *smart_explorer.max_bag_size.read().unwrap();
        let t8 = max_bag >= 67;
        let current_pos = *smart_explorer.current_planet_id.read().unwrap();
        let spawn_alive = {
            let orch = orch_arc.read().unwrap();
            if let Some(handle) = orch.planet_threads.get(&spawn_planet_id) { !handle.is_finished() } else { false }
        };
        let t9 = current_pos == spawn_planet_id && !explorer_died;

        let visits = smart_explorer.planet_visit_counts.read().unwrap();
        let t_survey = kb_size >= 8 && visits.values().filter(|&&v| v >= 3).count() >= 8;

        // --- FIXED: Read values here to fix Scope Error ---
        let total_visits = *smart_explorer.total_visits.read().unwrap();
        let total_steps = *smart_explorer.total_steps.read().unwrap();

        let t_eff = *smart_explorer.efficiency_achieved.read().unwrap();
        let t_speed = !*smart_explorer.triggered_danger.read().unwrap();
        let needed_scav = (kb_size as f32 * 0.8).ceil() as usize;
        let t_scav = smart_explorer.extraction_sites.read().unwrap().len() >= needed_scav && needed_scav > 0;
        let t_back = total_basic == 0;
        let t_dare = *smart_explorer.daredevil_event.read().unwrap();
        let t_mono = basics.iter().all(|b| *final_bag.basic.get(b).unwrap_or(&0) >= 15);
        let t_local = *smart_explorer.localvore_event.read().unwrap();

        let hist = smart_explorer.crafted_history.read().unwrap();
        let mut all_factories_used = true;
        let kb = smart_explorer.knowledge_base.read().unwrap();
        for (pid, data) in kb.iter() {
            if data.combines.as_ref().map_or(false, |c| !c.is_empty()) {
                if !hist.contains_key(pid) { all_factories_used = false; break; }
            }
        }
        let t_indus = all_factories_used && kb_size >= 8;

        let mut bizarre_condition = true;
        for (pid, data) in kb.iter() {
            if data.combines.as_ref().map_or(false, |c| c.contains(&ComplexResourceType::AIPartner)) {
                let count = hist.get(pid).map_or(0, |list| list.iter().filter(|&&x| x == ComplexResourceType::AIPartner).count());
                if count < 2 { bizarre_condition = false; break; }
            }
        }
        let all_original_done = t1 && t2 && t3 && t4 && t5 && t6 && t7 && t8 && t9;
        let t_bizarre = bizarre_condition && all_original_done && t_mono && t_scav && t_indus;

        let mut total_score = 0;
        if t1 { total_score += PTS_MAP_COMPLETE; }
        if t2 { total_score += PTS_BASIC_30; }
        if t3 { total_score += PTS_BASIC_5_EACH; }
        if t4 { total_score += PTS_COMPLEX_ALL; }
        if t5 { total_score += PTS_NET; }
        if t6 { total_score += PTS_NOMAD; }
        if t7 { total_score += PTS_HUB; }
        if t8 { total_score += PTS_HOARD; }
        if t9 { total_score += PTS_VICTORY_LAP; }
        if t_survey { total_score += PTS_SURVEYOR; }
        if t_eff { total_score += PTS_EFFICIENCY; }
        if t_speed { total_score += PTS_SPEEDRUN; }
        if t_scav { total_score += (kb_size as u32) * 50; }
        if t_back { total_score += PTS_BACKPACKER; }
        if t_dare { total_score += PTS_DAREDEVIL; }
        if t_mono { total_score += PTS_MONOPOLIST; }
        if t_local { total_score += PTS_LOCALVORE; }
        if t_indus { total_score += PTS_INDUSTRIALIST; }
        if t_bizarre { total_score += PTS_BIZARRE; }

        let planets_visited = smart_explorer.visited.read().unwrap().len() as u32;
        let edges_visited = smart_explorer.visited_edges.read().unwrap().len() as u32;
        total_score += planets_visited * PTS_PER_PLANET;
        total_score += edges_visited * PTS_PER_EDGE;
        let res_score = calc_resource_score(&final_bag);
        total_score += res_score;

        let mut tasks_completed = 0;
        if t1 { tasks_completed += 1; }
        if t2 { tasks_completed += 1; }
        if t3 { tasks_completed += 1; }
        if t4 { tasks_completed += 1; }
        if t5 { tasks_completed += 1; }
        if t6 { tasks_completed += 1; }
        if t7 { tasks_completed += 1; }
        if t8 { tasks_completed += 1; }
        if t9 { tasks_completed += 1; }

        let mapping_tasks_done = t1 && t5 && t6 && t7;
        let big_obj_done = t4 || t8 || t9;
        let all_except_home = t1 && t2 && t3 && t4 && t5 && t6 && t7 && t8;
        let all_mandatory = all_original_done && t_survey && t_scav; // Localvore optional

        let (ending_title, is_pass) = if let Some(title) = early_exit_title {
            (title, true)
        } else if t_bizarre {
            ("CRAZY NOISY BIZZARE EXPLORER", true)
        } else if all_mandatory {
            ("True Ending", true)
        } else if explorer_died {
            ("Fallen Comrade", false)
        } else if all_except_home && !spawn_alive {
            ("No Turning Back", true)
        } else if !mapping_tasks_done && big_obj_done {
            ("Blind Runner", true)
        } else if t4 {
            ("You Win", true)
        } else if tasks_completed >= 4 {
            ("Game Results", true)
        } else {
            ("Catastrophic Failure", false)
        };

        let status_msg = if is_pass { "PASS" } else { "FAIL" };
        let bizarre_label = if t_bizarre { "BIZARRE" } else { "SECRET TASK" };
        let bizarre_pts_str = if t_bizarre { format!("{}", PTS_BIZARRE) } else { "??????".to_string() };

        let report = format!(
            "\n=========================================\n\
             ENDING: {} [{}]\n\
             =========================================\n\
             MANDATORY TASKS: {}/9 (Original)\n\
             [+{:4}] Map Galaxy:        {} ({}/8)\n\
             [+{:4}] >30 Basic:         {} ({})\n\
             [+{:4}] >5 Each Basic:     {}\n\
             [+{:4}] All Complex:       {}\n\
             [+{:4}] Network Tech:      {} ({}/{})\n\
             [+{:4}] Nomad:             {} ({}/5)\n\
             [+{:4}] Hub Hunter:        {} ({} visits)\n\
             [+{:4}] Hoarder >67:       {} (Max: {})\n\
             [+{:4}] Victory Lap:       {}\n\
             -----------------------------------------\n\
             NEW MANDATORY (True Ending):\n\
             [+{:4}] Surveyor (3x):     {}\n\
             [+{:4}] Efficiency (<70):  {}\n\
             [+{:4}] Scavenger (80%):   {}\n\
             [+{:4}] Localvore:         {}\n\
             -----------------------------------------\n\
             OPTIONAL BADGES:\n\
             [+{:4}] Speedrunner:       {}\n\
             [+{:4}] Backpacker:        {}\n\
             [+{:4}] Daredevil:         {}\n\
             [+{:4}] Monopolist (15+):  {}\n\
             [+{:4}] Industrialist:     {}\n\
             [+{:6}] {:<18} {}\n\
             -----------------------------------------\n\
             BONUSES:\n\
             [+{:4}] Planets Visited:   {} ({})\n\
             [+{:4}] Edges Traversed:   {} ({})\n\
             [INFO ] Total Activity:    {} Visits | {} Steps\n\
             -----------------------------------------\n\
             RESOURCES:\n\
             [+{:4}] Inventory Value:   {}\n\
             -----------------------------------------\n\
             TOTAL SCORE: {}\n\
             =========================================",
            ending_title, status_msg,
            tasks_completed,
            PTS_MAP_COMPLETE, if t1 { "✅" } else { "❌" }, planets_visited,
            PTS_BASIC_30, if t2 { "✅" } else { "❌" }, total_basic,
            PTS_BASIC_5_EACH, if t3 { "✅" } else { "❌" },
            PTS_COMPLEX_ALL, if t4 { "✅" } else { "❌" },
            PTS_NET, if t5 { "✅" } else { "❌" }, edges_visited, total_known_edges,
            PTS_NOMAD, if t6 { "✅" } else { "❌" }, smart_explorer.extraction_sites.read().unwrap().len(),
            PTS_HUB, if t7 { "✅" } else { "❌" }, hub_visits,
            PTS_HOARD, if t8 { "✅" } else { "❌" }, max_bag,
            PTS_VICTORY_LAP, if t9 { "✅" } else { "❌" },
            PTS_SURVEYOR, if t_survey { "✅" } else { "❌" },
            PTS_EFFICIENCY, if t_eff { "✅" } else { "❌" },
            (kb_size as u32 * 50), if t_scav { "✅" } else { "❌" },
            PTS_LOCALVORE, if t_local { "✅" } else { "❌" },
            PTS_SPEEDRUN, if t_speed { "✅" } else { "❌" },
            PTS_BACKPACKER, if t_back { "✅" } else { "❌" },
            PTS_DAREDEVIL, if t_dare { "✅" } else { "❌" },
            PTS_MONOPOLIST, if t_mono { "✅" } else { "❌" },
            PTS_INDUSTRIALIST, if t_indus { "✅" } else { "❌" },
            bizarre_pts_str, bizarre_label, if t_bizarre { "✅" } else { "❌" },
            (planets_visited * PTS_PER_PLANET), planets_visited, PTS_PER_PLANET,
            (edges_visited * PTS_PER_EDGE), edges_visited, PTS_PER_EDGE,
            total_visits, total_steps,
            res_score, res_score,
            total_score
        );

        println!("{}", report);

        if !is_pass {
            panic!("\n❌ GAME OVER: {}. See Scorecard above.\n", ending_title);
        }
    }
}