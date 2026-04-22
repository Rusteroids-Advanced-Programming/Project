use std::sync::{Arc, RwLock};
use axum::extract::State;
use axum::Json;
use crate::modules::visualizer::dto::GalaxyResponse;
use crate::modules::orchestrator::orchestator::Orchestrator;
use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;
use crate::modules::read_galaxy::stats::{ExplorerDataDTO, PlanetDataDTO, PlanetStatsDTO};

pub async fn get_logs(State(orch): State<Arc<RwLock<Orchestrator>>>) -> Json<Vec<String>> {
    let logs = orch.read().unwrap().logs.read().unwrap().clone();
    Json(logs.into_iter().collect())
}

pub async fn get_galaxy_status(State(orch): State<Arc<RwLock<Orchestrator>>>) -> Json<GalaxyResponse> {
    let (stats_snapshot, resources_snapshot, explorer_snapshot) = {
        let orch_guard = orch.read().expect("Lock Orchestrator poisoned");
        let stats_guard = orch_guard.stats_map.read().expect("Lock StatsMap poisoned");

        (
            stats_guard.clone(),
            orch_guard.planet_resources.clone(),
            orch_guard.explorer_planet.read().unwrap().clone(), // da checkare, read.unwrap di un rwlock
        )
    };

    let mut planets_data = Vec::new();

    let mut sorted_ids: Vec<_> = stats_snapshot.keys().collect();
    sorted_ids.sort();

    for &id in sorted_ids {
        let s = stats_snapshot.get(&id).unwrap();

        let dynamic_state = {
            let orch_guard = orch.read().unwrap();
            orch_guard.get_planet_state(id)
        };

        let has_rocket = dynamic_state
            .as_ref()
            .map(|ds| ds.has_rocket)
            .unwrap_or(false);

        let (base_res, complex_res) = resources_snapshot
            .get(&id)
            .cloned()
            .unwrap_or((vec![], vec![]));

        let neighbors = {
            let orch_guard = orch.read().unwrap();
            let graph_guard = orch_guard.galaxy_graph.read().unwrap();
            graph_guard
                .nodes
                .iter()
                .find(|n| n.read().unwrap().value == id)
                .map(|n| {
                    n.read()
                        .unwrap()
                        .adjacent_nodes
                        .iter()
                        .map(|adj| adj.read().unwrap().value)
                        .collect::<Vec<u32>>()
                })
                .unwrap_or_default()
        };

        planets_data.push(PlanetDataDTO {
            id,
            name: s.planet_name.clone(),
            planet_type: format!("{:?}", s.planet_type),
            alive: s.alive,
            energy_cells: dynamic_state
                .map(|ds| ds.charged_cells_count as u32)
                .unwrap_or(0),
            resources_base: base_res,
            resources_complex: complex_res,
            neighbors,
            has_rocket,
            stats: PlanetStatsDTO {
                asteroids: s.asteroid_count,
                sunrays: s.sunray_count,
                rockets: s.rocket_used_count,
            },
        });
    }

    let mut explorers_data = Vec::new();
    let orch_guard = orch.read().unwrap();

    for (&exp_id, &planet_id) in explorer_snapshot.iter() {
        if let Some(explorer_obj) = orch_guard.explorers.get(&exp_id) {
            let guard = explorer_obj.get_base();
            let is_alive = *guard.alive.read().unwrap();

            let bag_content = if is_alive {
                //let bag_guard = explorer_obj.dummy_bag.read().unwrap();
                let bag = guard.bag.read().unwrap().to_dummy();
                let mut content = Vec::new();
                for (res_type, count) in &bag.basic {
                    for _ in 0..*count {
                        content.push(format!("{:?}", res_type));
                    }
                }
                for (res_type, count) in &bag.complex {
                    for _ in 0..*count {
                        content.push(format!("{:?}", res_type));
                    }
                }
                content
            } else {
                vec![]
            };

            explorers_data.push(ExplorerDataDTO {
                id: exp_id,
                current_planet: planet_id,
                bag: bag_content,
                alive: is_alive,
            });
        } else {
            explorers_data.push(ExplorerDataDTO {
                id: exp_id,
                current_planet: planet_id,
                bag: vec![],
                alive: false,
            });
        }
    }

    Json(GalaxyResponse {
        planets: planets_data,
        explorers: explorers_data,
    })
}