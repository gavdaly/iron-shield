use crate::error::Result;
use crate::index::generate_index;
use axum::{routing::get, Router};
use tokio::signal;
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
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        return Err(crate::error::IronShieldError::Generic(format!(
            "Server error: {e}"
        )));
    }

    tracing::info!("Server shutdown complete");
    Ok(())
}

/// A helper function that awaits a shutdown signal
async fn shutdown_signal() {
    // Handle Ctrl+C
    let ctrl_c = async {
        if let Err(e) = signal::ctrl_c().await {
            tracing::error!("Failed to install Ctrl+C handler: {}", e);
            // This is a critical error, so we panic as the application can't function correctly without shutdown capability
            panic!("Failed to install Ctrl+C handler: {}", e);
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut terminate_signal) => {
                terminate_signal.recv().await;
            }
            Err(e) => {
                tracing::error!("Failed to install SIGTERM handler: {}", e);
                // Continue with only Ctrl+C handler if SIGTERM fails
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Received shutdown signal, starting graceful shutdown");
}
