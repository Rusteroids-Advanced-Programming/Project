pub(crate) mod event_manager;
pub(crate) mod explorer_initializer;
pub(crate) mod handler_explorer_ai;
pub mod initializer;
pub mod orchestrator;
pub(crate) mod orchestrator_ai;

#[allow(unused)]
mod tests {
    use crate::modules::orchestrator::explorer_initializer::ExplorerInitializer;
    use crate::modules::orchestrator::initializer::Initializer;
    use crate::modules::orchestrator::orchestrator::Orchestrator;
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

        let mut _id = orch.send_sunray(8);
        _id = orch.send_sunray(8);

        sleep(Duration::from_secs(1));
        let state = orch.get_planet_state(8).unwrap();

        orch.send_asteroid(8);
        sleep(Duration::from_secs(1));

        let guard = orch.stats_map.read().unwrap();
        let stats = guard.get(&8).unwrap();

        assert_eq!(
            "DummyPlanetState { energy_cells: [true, true, false, false, false], charged_cells_count: 2, has_rocket: false }",
            format!("{:?}", state)
        );
        assert_eq!(false, stats.alive);
        assert_eq!(2, stats.sunray_count);
        assert_eq!(1, stats.asteroid_count as i32);
    }

    #[test]
    fn simulate_explorer() {
        let _ = env_logger::builder().is_test(true).try_init();

        let diff = 0;
        let orch = Orchestrator::new(diff);
        let arc_orch = Arc::new(RwLock::new(orch));
        arc_orch.write().unwrap().initialize();
        let vec_explorers = vec![1];

        let orc_clone1 = arc_orch.clone();
        let orc_clone2 = arc_orch.clone();

        let _handle = thread::spawn(move || {
            orc_clone1.read().unwrap().run();
        });

        arc_orch
            .write()
            .unwrap()
            .initialize_explorers(vec_explorers, orc_clone2);

        sleep(Duration::from_secs(60));
    }
}
