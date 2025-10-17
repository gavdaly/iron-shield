use crate::index::generate_index;
use axum::{routing::get, Router};
use tower_http::services::ServeDir;
use tracing;

pub async fn run() {
    tracing::info!("Initializing server");
    
    let app = Router::new()
        .route("/", get(generate_index))
        .nest_service("/static", ServeDir::new("static"));

    tracing::debug!("Routes configured");

    let address = &"0.0.0.0:3000".parse().expect("Error parsing address");
    tracing::info!("Binding server to address: {}", address);

    tracing::info!("Site launched on: http://0.0.0.0:3000");
    
    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .expect("Run Server");

    tracing::info!("Server shutdown complete");
}
