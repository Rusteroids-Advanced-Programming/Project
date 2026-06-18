use crate::modules::orchestrator::orchestrator::Orchestrator;
use crate::modules::orchestrator::orchestrator_ai::OrchestratorAI;
use crate::modules::read_galaxy::galaxy_generator::generate_galaxy_file;
use crate::modules::read_galaxy::stats::{ExplorerDataDTO, PlanetDataDTO, PlanetStatsDTO};
use crate::modules::visualizer::dto::GalaxyResponse;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use common_game::logging::Participant;
use serde::Deserialize;
use std::sync::{Arc, RwLock};

/// Incoming request payload containing setup parameters to initialize a game instance.
#[derive(Deserialize)]
pub struct StartGamePayload {
    pub difficulty: String,
    pub planets_count: Option<u32>,
}

/// Data Transfer Object representing a single formatted log entry optimized for frontend visualization.
#[derive(serde::Serialize)]
pub struct StructuredLogDTO {
    pub timestamp: u64,
    pub sender: String,
    pub receiver: String,
    pub event_type: String,
    pub channel: String,
    pub message: String,
}

/// Axum handler responsible for tearing down active game loops and resetting the universe under new parameters.
pub async fn start_game(
    State(orch): State<Arc<RwLock<Orchestrator>>>,
    Json(payload): Json<StartGamePayload>,
) -> Result<&'static str, StatusCode> {
    {
        // Stop current execution and safely wipe logs using scoped locks to avoid deadlocks
        let orch_read = orch.read().expect("Lock Orchestrator poisoned in shutdown");
        orch_read.stop();

        if let Ok(mut logs) = orch_read.logs.write() {
            logs.clear();
        }
        if let Ok(mut s_logs) = orch_read.structured_logs.write() {
            s_logs.clear();
        }
    }

    // Yield control back to the async executor to guarantee the previous game loop thread terminates
    tokio::time::sleep(std::time::Duration::from_millis(30)).await;

    let diff_u8 = match payload.difficulty.as_str() {
        "easy" => 0,
        "medium" => 1,
        "hard" => 2,
        "peaceful" => 3,
        _ => 1,
    };

    let mut num_planets = payload.planets_count.unwrap_or(30);
    if num_planets < 7 {
        num_planets = 7;
    }
    if num_planets > 50 {
        num_planets = 50;
    }

    if let Err(e) = generate_galaxy_file(num_planets as usize) {
        eprintln!(
            "ERROR: Unable to generate galaxy-initialization.txt with {} planets: {:?}",
            num_planets, e
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    use crate::modules::orchestrator::explorer_initializer::ExplorerInitializer;
    use crate::modules::orchestrator::initializer::Initializer;

    {
        let mut orch_write = orch.write().expect("Lock Orchestrator poisoned in write");

        use crate::modules::orchestrator::orchestrator::Difficulty;
        orch_write.difficulty = match diff_u8 {
            0 => Difficulty::Easy,
            1 => Difficulty::Medium,
            2 => Difficulty::Hard,
            3 => Difficulty::Peaceful,
            _ => Difficulty::Medium,
        };

        // Fully flush historical game metrics from cross-game tracked states
        if let Ok(mut stats) = orch_write.stats_map.write() {
            stats.clear();
        }

        orch_write.planet_resources.clear();

        if let Ok(mut exp_planet) = orch_write.explorer_planet.write() {
            exp_planet.clear();
        }

        orch_write.initialize();
        orch_write.initialize_explorers(vec![1, 2], orch.clone());
    }

    // Detach the synchronous physics simulation loop onto a native OS thread
    let orch_for_run = orch.clone();
    std::thread::spawn(move || {
        orch_for_run
            .read()
            .expect("Lock Orchestrator poisoned in run thread")
            .run();
    });

    Ok("OK")
}

/// Axum handler that fetches and outputs the raw vector of diagnostic text logs.
pub async fn get_logs(State(orch): State<Arc<RwLock<Orchestrator>>>) -> Json<Vec<String>> {
    let logs = orch.read().unwrap().logs.read().unwrap().clone();
    Json(logs.into_iter().collect())
}

/// Axum handler returning a compiled, human-readable list of structural game action payloads.
pub async fn get_structured_logs(
    State(orch): State<Arc<RwLock<Orchestrator>>>,
) -> Json<Vec<StructuredLogDTO>> {
    let guard = orch.read().unwrap();
    let logs_guard = guard.structured_logs.read().unwrap();

    let dto_list = logs_guard
        .iter()
        .map(|log| {
            let fmt_participant = |p: &Option<Participant>| {
                p.as_ref().map_or_else(
                    || "None".to_string(),
                    |act| format!("{:?} #{}", act.actor_type, act.id),
                )
            };

            StructuredLogDTO {
                timestamp: log.timestamp_unix,
                sender: fmt_participant(&log.sender),
                receiver: fmt_participant(&log.receiver),
                event_type: format!("{:?}", log.event_type),
                channel: format!("{:?}", log.channel),
                message: log
                    .payload
                    .get("message")
                    .cloned()
                    .unwrap_or_else(|| "".to_string()),
            }
        })
        .collect();

    Json(dto_list)
}

/// Axum handler providing a comprehensive real-time snapshot of astronomical nodes and active explorers.
pub async fn get_galaxy_status(
    State(orch): State<Arc<RwLock<Orchestrator>>>,
) -> Json<GalaxyResponse> {
    let (stats_snapshot, resources_snapshot, explorer_snapshot) = {
        // Isolate short-lived snapshots to release the main orchestrator read lock quickly
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
            // Traverse the synchronized celestial graph pointers to resolve adjacency arrays
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

            // Unpack internal inventory variants into flat string structures for JSON transport
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

    let mut game_won = false;
    let mut winner_id: Option<u32> = None;

    // Scan for victory conditions across all instantiated explorer instances
    for (&exp_id, _) in explorer_snapshot.iter() {
        if let Some(explorer_obj) = orch_guard.explorers.get(&exp_id) {
            if explorer_obj.all_tasks_finished() {
                game_won = true;
                winner_id = Some(exp_id as u32);
                orch_guard.stop();
                break;
            }
        }
    }

    let all_dead = !explorers_data.is_empty() && explorers_data.iter().all(|ex| !ex.alive);

    // Enforce instant simulation freezing upon terminal match outcomes
    if game_won || all_dead {
        orch_guard.stop();
    }

    Json(GalaxyResponse {
        planets: planets_data,
        explorers: explorers_data,
        game_won,
        winner_id,
    })
}
