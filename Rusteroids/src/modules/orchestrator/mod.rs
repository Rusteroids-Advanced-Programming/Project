mod event_manager;
pub(crate) mod explorer_initializer;
mod handler_explorer_ai;
pub mod initializer;
pub mod orchestator;
pub(crate) mod orchestrator_ai;

mod tests {
    use crate::modules::orchestrator::explorer_initializer::ExplorerInitializer;
    use crate::modules::orchestrator::initializer::Initializer;
    use crate::modules::orchestrator::orchestator::Orchestrator;
    use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;
    use std::sync::{Arc, RwLock};
    use std::thread;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn simulate_planet() {
        let diff = 0;
        let mut orch = Orchestrator::new(diff);
        orch.initialize();

        // println!("{:?}", orch.planet_channels);
        println!("{:?}", orch.planet_threads);
        // println!("{:?}", orch.planet_map);
        // orch.start_planet_ai(8);
        let mut id = orch.send_sunray(8);
        println!("id del sunray {:?}", id);
        id = orch.send_sunray(8);

        sleep(Duration::from_secs(1));
        println!("id del sunray {:?}", id);
        let state = orch.get_planet_state(8).unwrap();
        println!("{:?}", state);

        orch.send_asteroid(8);
        sleep(Duration::from_secs(1));

        orch.send_asteroid(8);
        sleep(Duration::from_secs(1));

        orch.send_asteroid(8);

        sleep(Duration::from_secs(1));

        orch.send_sunray(8);

        // println!("{:?}", orch.planet_channels.get(&1));
        println!("{:?}", orch.planet_threads);

        // let planet_1 = orch.planet_map.get_planet_by_id(1).unwrap();
    }

    #[test]
    fn simulate_explorer() {
        let _ = env_logger::builder()
            .is_test(true) // Assicura che l'output sia catturato correttamente da cargo test
            .try_init();

        let diff = 0;
        let orch = Orchestrator::new(diff);
        let arc_orch = Arc::new(RwLock::new(orch));
        arc_orch.write().unwrap().initialize();
        let vec_explorers = vec![(10, 1)];
        // for i in 1..9 {
        //     arc_orch.read().unwrap().send_sunray(i);
        // }

        let orc_clone1 = arc_orch.clone();
        let orc_clone2 = arc_orch.clone();

        let _handle = thread::spawn(move || {
            orc_clone1.read().unwrap().run();
        });

        arc_orch
            .write()
            .unwrap()
            .initialize_explorers(vec_explorers, orc_clone2);
        //
        // println!("{:?}", orch.explorer_threads);
        // println!("{:?}", orch.explorer_channels);
        //
        // orch.send_sunray(8);
        // orch.send_sunray(8);
        //
        // orch.start_explorer(10);
        //
        // orch.generate_resource(10, BasicResourceType::Carbon); //--> SENZA QUESTA NON VA UN CAZZO DIO MAIALE NON VA AVANTI,. I NEIGBOURS NON VENGONO SETTATi IN OGNI CASO
        sleep(Duration::from_secs(60));
    }
}
