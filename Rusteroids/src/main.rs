mod modules;

use std::io;
use std::sync::{Arc, RwLock};
use crate::modules::orchestrator::initializer::Initializer;
use crate::modules::orchestrator::orchestator::Orchestrator;

fn ask_difficulty() -> Result<u8, String> {
    println!("Choose game difficulty: [0] EASY    [1] MEDIUM     [2] HARD   [3] PEACEFUL");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).unwrap_or(0);
    let difficulty: u8 = user_input
        .trim()
        .parse()
        .expect("Please insert a valid difficulty option");

    if difficulty <= 3 {
        Ok(difficulty)
    } else {
        Err("Please insert a valid difficulty option".to_string())
    }
}

#[tokio::main]
async fn main() {
    let diff = ask_difficulty().unwrap();
    let mut orch = Orchestrator::new(diff);
    orch.initialize();

    let shared_orch = Arc::new(RwLock::new(orch));

    {
        use crate::modules::orchestrator::explorer_initializer::ExplorerInitializer;
        let mut orch_write = shared_orch.write().unwrap();
        orch_write.initialize_explorers(vec![2,3], shared_orch.clone());
    }

    let orch_for_run = shared_orch.clone();
    std::thread::spawn(move || {
        orch_for_run.read().unwrap().run();
    });

    modules::visualizer::server::start_server(shared_orch.clone()).await;
}