use crate::error::Result;
use crate::index::generate_index;
use axum::{routing::get, Router};
use tower_http::services::ServeDir;

/// Run the web server on the specified port.
///
/// # Arguments
///
/// * `port` - The port number to bind the server to
///
/// # Returns
///
/// Returns `Ok(())` if the server runs successfully, or an `IronShieldError` if an error occurs
///
/// # Errors
///
/// Returns an error if:
/// - The address string cannot be parsed into a valid `SocketAddr`
/// - The server fails to bind to the specified address
pub async fn run(port: u16) -> Result<()> {
    tracing::info!("Initializing server");

    let app = Router::new()
        .route("/", get(generate_index))
        .nest_service("/static", ServeDir::new("static"));

    tracing::debug!("Routes configured");

    let addr = format!("0.0.0.0:{port}");
    let address = &addr.parse()?;
    tracing::info!("Binding server to address: {address}");

    tracing::info!("Site launched on: http://{addr}");

    if let Err(e) = axum::Server::bind(address)
        .serve(app.into_make_service())
        .await
    {
        return Err(crate::error::IronShieldError::Generic(format!(
            "Server error: {e}"
        )));
    }

    tracing::info!("Server shutdown complete");
    Ok(())
}
