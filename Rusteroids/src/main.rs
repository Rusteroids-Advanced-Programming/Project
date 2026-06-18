mod modules;
use std::sync::{Arc, RwLock};
use crate::modules::orchestrator::initializer::Initializer;
use crate::modules::orchestrator::orchestator::Orchestrator;

#[tokio::main]
async fn main() {
    env_logger::init();

    println!("Listening on http://localhost:3000");
    let orch = Orchestrator::new(1);
    let shared_orch = Arc::new(RwLock::new(orch));
    modules::visualizer::server::start_server(shared_orch.clone()).await;
}