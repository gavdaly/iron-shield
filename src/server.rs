use crate::index::generate_index;
use axum::{routing::get, Router};
use tower_http::services::ServeDir;

/// Run the web server on the specified port.
///
/// # Panics
///
/// This function will panic if:
/// - The address string cannot be parsed into a valid `SocketAddr`
/// - The server fails to bind to the specified address
pub async fn run(port: u16) {
    tracing::info!("Initializing server");

    let app = Router::new()
        .route("/", get(generate_index))
        .nest_service("/static", ServeDir::new("static"));

    tracing::debug!("Routes configured");

    let addr = format!("0.0.0.0:{port}");
    let address = &addr.parse().expect("Error parsing address");
    tracing::info!("Binding server to address: {address}");

    tracing::info!("Site launched on: http://{addr}");

    axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
        .expect("Run Server");

    tracing::info!("Server shutdown complete");
}
