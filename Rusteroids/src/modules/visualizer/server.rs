use axum::{Router, routing::{get, post}};
use std::sync::{Arc, RwLock};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use crate::modules::orchestrator::orchestator::Orchestrator;
use crate::modules::visualizer::routes;

pub async fn start_server(shared_orch: Arc<RwLock<Orchestrator>>) {
    let app = Router::new()
        .route("/galaxy", get(routes::get_galaxy_status))
        .route("/logs", get(routes::get_logs))
        .route("/start-game", post(routes::start_game))
        .fallback_service(ServeDir::new("visualizer"))
        .layer(CorsLayer::permissive())
        .with_state(shared_orch);

    println!("Visualizer Server in ascolto su http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}