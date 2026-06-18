use std::sync::{Arc, RwLock};
use axum::extract::State;
use axum::Json;
use axum::http::StatusCode;
use common_game::logging::Participant;
use serde::Deserialize;
use crate::modules::visualizer::dto::GalaxyResponse;
use crate::modules::orchestrator::orchestator::Orchestrator;
use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;
use crate::modules::read_galaxy::stats::{ExplorerDataDTO, PlanetDataDTO, PlanetStatsDTO};
use crate::modules::read_galaxy::galaxy_generator::generate_galaxy_file;

#[derive(Deserialize)]
pub struct StartGamePayload {
    pub difficulty: String,
}

#[derive(serde::Serialize)]
pub struct StructuredLogDTO {
    pub timestamp: u64,
    pub sender: String,
    pub receiver: String,
    pub event_type: String,
    pub channel: String,
    pub message: String,
}





pub async fn start_game(
    State(orch): State<Arc<RwLock<Orchestrator>>>,
    Json(payload): Json<StartGamePayload>,
) -> Result<&'static str, StatusCode> {

    let diff_u8 = match payload.difficulty.as_str() {
        "easy" => 0,
        "medium" => 1,
        "hard" => 2,
        "peaceful" => 3,
        _ => 1,
    };

    println!(" Difficoltà scelta : {} ({})", payload.difficulty, diff_u8);

    if let Err(e) = generate_galaxy_file(30) {
        eprintln!("=== ERRORE CRITICO: Impossibile generare galaxy-initialization.txt: {:?} ===", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    use crate::modules::orchestrator::initializer::Initializer;
    use crate::modules::orchestrator::explorer_initializer::ExplorerInitializer;

    {
        let mut orch_write = orch.write().expect("Lock Orchestrator poisoned in write");

        orch_write.initialize();
        orch_write.initialize_explorers(vec![2, 3], orch.clone());
    }

    let orch_for_run = orch.clone();
    std::thread::spawn(move || {
        println!("=== SYSTEM: Thread di simulazione dell'Orchestrator AVVIATO ===");
        orch_for_run.read().expect("Lock Orchestrator poisoned in run thread").run();
    });

    Ok("OK")
}

pub async fn get_logs(State(orch): State<Arc<RwLock<Orchestrator>>>) -> Json<Vec<String>> {
    let logs = orch.read().unwrap().logs.read().unwrap().clone();
    Json(logs.into_iter().collect())
}

pub async fn get_structured_logs(State(orch): State<Arc<RwLock<Orchestrator>>>) -> Json<Vec<StructuredLogDTO>> {
    let guard = orch.read().unwrap();
    let logs_guard = guard.structured_logs.read().unwrap();

    let dto_list = logs_guard.iter().map(|log| {
        let fmt_participant = |p: &Option<Participant>| {
            p.as_ref().map_or_else(|| "None".to_string(), |act| format!("{:?} #{}", act.actor_type, act.id))
        };

        StructuredLogDTO {
            timestamp: log.timestamp_unix,
            sender: fmt_participant(&log.sender),
            receiver: fmt_participant(&log.receiver),
            event_type: format!("{:?}", log.event_type),
            channel: format!("{:?}", log.channel),
            message: log.payload.get("message").cloned().unwrap_or_else(|| "".to_string()),
        }
    }).collect();

    Json(dto_list)
}

pub async fn get_galaxy_status(State(orch): State<Arc<RwLock<Orchestrator>>>) -> Json<GalaxyResponse> {
    let (stats_snapshot, resources_snapshot, explorer_snapshot) = {
        let orch_guard = orch.read().expect("Lock Orchestrator poisoned");
        let stats_guard = orch_guard.stats_map.read().expect("Lock StatsMap poisoned");

        (
            stats_guard.clone(),
            orch_guard.planet_resources.clone(),
            orch_guard.explorer_planet.read().unwrap().clone(),
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