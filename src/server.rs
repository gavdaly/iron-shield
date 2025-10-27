use crate::config::ConfigWatcher;
use crate::error::Result;
use crate::index::generate_index;
use crate::uptime::{uptime_stream, UptimeState};
use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::signal;
use tower_http::services::ServeDir;
use tracing::info;

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

    // Create the config watcher which handles loading and watching the config file
    let config_path = std::path::Path::new("config.json5").to_path_buf();
    let config_watcher = ConfigWatcher::new(&config_path)?;
    let config_rwlock = config_watcher.get_config(); // Get the Arc<RwLock<Config>>

    info!("Configuration loaded and watcher initialized successfully");

    // Create uptime state with the config from ConfigWatcher
    let history_map = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let uptime_state = Arc::new(UptimeState {
        config: config_rwlock,
        history: history_map,
    });

    let app = Router::new()
        .route("/", get(generate_index))
        .route("/uptime", get(uptime_stream))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(uptime_state);

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
            tracing::error!("Failed to install Ctrl+C handler: {e}");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut terminate_signal) => {
                terminate_signal.recv().await;
            }
            Err(e) => {
                tracing::error!("Failed to install SIGTERM handler: {e}");
                // Continue with only Ctrl+C handler if SIGTERM fails
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("Received shutdown signal, starting graceful shutdown");
}
