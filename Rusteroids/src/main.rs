mod modules;
use crate::modules::orchestrator::orchestrator::Orchestrator;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() {
    env_logger::init();

    println!("Listening on http://localhost:3000");
    let orch = Orchestrator::new(1);
    let shared_orch = Arc::new(RwLock::new(orch));
    modules::visualizer::server::start_server(shared_orch.clone()).await;
}
