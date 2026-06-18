use crate::modules::orchestrator::orchestrator::Orchestrator;
use crate::modules::visualizer::routes;
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::{Arc, RwLock};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

/// Initializes the Axum web server, configures API routes, binds static assets, and begins listening on port 3000.
pub async fn start_server(shared_orch: Arc<RwLock<Orchestrator>>) {
    let app = Router::new()
        // API Endpoints for pulling simulation data and pushing system state commands
        .route("/galaxy", get(routes::get_galaxy_status))
        .route("/logs", get(routes::get_logs))
        .route("/api/structured-logs", get(routes::get_structured_logs))
        .route("/start-game", post(routes::start_game))
        // Serve frontend files (HTML, CSS, TS/JS, Media) from the local directory when no API routes match
        .fallback_service(ServeDir::new("visualizer"))
        // Permissive CORS layer added to ease local development integration testing
        .layer(CorsLayer::permissive())
        // Inject the thread-safe reference to the Orchestrator as the shared global state
        .with_state(shared_orch);

    println!("Server running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
