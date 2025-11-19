//! Server module for the Iron Shield dashboard application
//!
//! This module contains the main server functionality for the Iron Shield application.
//! It handles setting up and running the web server, configuring routes, serving static files,
//! and managing graceful shutdowns.
//!
//! The server uses Axum as the web framework and provides endpoints for:
//! - Main dashboard page
//! - Configuration API
//! - Uptime monitoring stream
//! - Static file serving

use crate::config::{ConfigWatcher, CONFIG_FILE};
use crate::error::Result;
use crate::index::generate_index;
use crate::settings::save_config;
use crate::uptime::{uptime_stream, UptimeState};
use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::{signal, sync::broadcast};
use tokio_util::sync::CancellationToken;
use tower_http::services::ServeDir;
use tracing::info;

/// Default location for the bundled frontend assets.
const FRONTEND_DIST_DEFAULT: &str = "frontend/dist";

/// Run the web server on the specified port with graceful shutdown capabilities.
///
/// This function initializes the web server, sets up routes, configures static file serving,
/// and starts listening on the specified port. It handles configuration loading through
/// the `ConfigWatcher` which monitors the config file for changes.
///
/// The server serves the following endpoints:
/// - / - Main dashboard page
/// - /api/config - Settings API endpoint for updating configuration
/// - /uptime - Server-Sent Events endpoint for real-time uptime updates
/// - /static/\* - Static file serving for CSS, JS, and assets
///
/// # Arguments
///
/// * `port` - The TCP port number on which to bind the server
/// * `config_file_path_option` - Optional path to the configuration file
///   (uses default path if None is provided)
/// * `cancel_token` - A cancellation token for graceful shutdown signals
///
/// # Returns
///
/// Returns `Ok(())` when the server shuts down gracefully, or an `IronShieldError` if an error occurs
/// during startup, runtime, or shutdown.
///
/// # Errors
///
/// This function returns an error if:
/// - The configuration file cannot be loaded or watched
/// - The server cannot bind to the specified address
/// - The server encounters an error during serving
///
/// # Examples
///
/// ```
/// # use tokio_util::sync::CancellationToken;
/// # async fn example() -> Result<(), iron_shield::error::IronShieldError> {
/// use std::path::PathBuf;
///
/// let cancel_token = CancellationToken::new();
/// let config_path = Some(PathBuf::from("config.json5"));
///
/// iron_shield::server::run(3000, config_path, cancel_token).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run(
    port: u16,
    config_file_path_option: Option<PathBuf>,
    cancel_token: CancellationToken,
) -> Result<()> {
    tracing::info!("Initializing server");

    // Determine the config file path to use
    let config_path =
        config_file_path_option.unwrap_or_else(|| std::path::Path::new(CONFIG_FILE).to_path_buf());

    // Create the config watcher which handles loading and watching the config file
    let config_watcher = ConfigWatcher::new(&config_path)?;
    let config_rwlock = config_watcher.get_config(); // Get the Arc<RwLock<Config>>

    info!("Configuration loaded and watcher initialized successfully");

    // Create uptime state with the config from ConfigWatcher
    let history_map = Arc::new(RwLock::new(HashMap::new()));
    let (shutdown_tx, _) = broadcast::channel(16);

    let uptime_state = Arc::new(UptimeState {
        config: config_rwlock,
        history: history_map,
        config_file_path: config_path.clone(), // Clone for UptimeState
        shutdown_events: shutdown_tx.clone(),
        shutdown_token: cancel_token.clone(),
    });

    let static_dir = resolve_static_dir();
    info!(
        "Serving static assets from: {}",
        static_dir.to_string_lossy()
    );

    let app = Router::new()
        .route("/", get(generate_index))
        .route("/api/config", post(save_config))
        .route("/uptime", get(uptime_stream))
        .nest_service("/static", ServeDir::new(static_dir))
        .with_state(uptime_state.clone());

    tracing::debug!("Routes configured");

    let addr = format!("0.0.0.0:{port}");
    let address = addr.parse::<std::net::SocketAddr>().map_err(|e| {
        crate::error::IronShieldError::Generic(format!("Failed to parse address: {e}"))
    })?;
    tracing::info!("Binding server to address: {address}");

    tracing::info!("Site launched on: http://{addr}");

    // Spawn the shutdown signal handler
    tokio::spawn(shutdown_signal(cancel_token.clone(), shutdown_tx.clone()));

    let listener = tokio::net::TcpListener::bind(address).await.map_err(|e| {
        crate::error::IronShieldError::Generic(format!("Failed to bind to address: {e}"))
    })?;
    if let Err(e) = axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async move {
            cancel_token.cancelled().await;
        })
        .await
    {
        return Err(crate::error::IronShieldError::Generic(format!(
            "Server error: {e}"
        )));
    }

    tracing::info!("Server shutdown complete");
    Ok(())
}

fn resolve_static_dir() -> PathBuf {
    std::env::var("FRONTEND_DIST_DIR")
        .map_or_else(|_| PathBuf::from(FRONTEND_DIST_DEFAULT), PathBuf::from)
}

/// A helper function that awaits a shutdown signal to trigger graceful shutdown.
///
/// This function listens for termination signals (Ctrl+C on all platforms, SIGTERM on Unix systems)
/// and triggers the cancellation token when a signal is received. This allows the server to
/// perform a graceful shutdown, finishing ongoing requests before terminating.
///
/// # Arguments
///
/// * `cancel_token` - The cancellation token that will be cancelled when a shutdown signal is received
///
/// # Behavior
///
/// On Unix systems, the function handles both SIGTERM and Ctrl+C (SIGINT) signals.
/// On non-Unix systems, it only handles Ctrl+C.
/// When either signal is received, the function logs the event and cancels the provided token.
///
/// # Note
///
/// This function is designed to be spawned as a separate task using `tokio::spawn`.
async fn shutdown_signal(
    cancel_token: CancellationToken,
    shutdown_events: broadcast::Sender<String>,
) {
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
    if let Err(err) = shutdown_events.send("Server is shutting down for maintenance".to_string()) {
        tracing::warn!("Failed to notify clients about shutdown: {err}");
    }
    cancel_token.cancel();
}
