use crate::config::Config;
use crate::error::Result;
use crate::index::generate_index;
use crate::uptime::{uptime_stream, UptimeState};
use axum::{routing::get, Router};
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::mpsc;
use tower_http::services::ServeDir;
use tracing::{debug, error, info};

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

    // Load the initial configuration
    let config = Config::load()?;
    info!("Configuration loaded successfully");

    // Create uptime state with the config wrapped in RwLock
    let config_rwlock = Arc::new(std::sync::RwLock::new(config));
    let history_map = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let uptime_state = Arc::new(UptimeState {
        config: config_rwlock,
        history: history_map,
    });

    // Set up file watcher for config changes
    let (tx, rx) = mpsc::unbounded_channel();
    let watcher_state = uptime_state.clone(); // Clone for the watcher
    let config_path = std::path::Path::new("config.json5").to_path_buf();
    let config_path_for_watcher = config_path.clone(); // Clone for the watcher closure

    // Create the file watcher - use full std::result::Result to avoid type conflict
    let mut watcher = recommended_watcher(move |res: std::result::Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                match event.kind {
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                        for path in event.paths {
                            if path == config_path_for_watcher {
                                debug!("Configuration file change detected: {:?}", path);
                                if tx.send(()).is_err() {
                                    error!("Failed to send config reload signal");
                                }
                                break;
                            }
                        }
                    }
                    _ => {} // Ignore other event types
                }
            }
            Err(e) => error!("Watch error: {:?}", e),
        }
    })
    .map_err(|e| crate::error::IronShieldError::Generic(e.to_string()))?;

    // Add the config file to the watcher
    watcher
        .watch(&config_path, RecursiveMode::NonRecursive)
        .map_err(|e| crate::error::IronShieldError::Generic(e.to_string()))?;

    info!("Started config file watcher for: {:?}", config_path);

    // Spawn a task to handle config reloads
    tokio::spawn({
        let uptime_state = watcher_state;
        let mut reload_rx = rx; // Receive channel for config reload signals
        async move {
            loop {
                if reload_rx.recv().await.is_some() {
                    match Config::load() {
                        Ok(new_config) => {
                            let numbe_of_sites = new_config.sites.len();
                            info!("Reloading configuration with {numbe_of_sites} sites",);

                            {
                                if let Ok(mut config_guard) = uptime_state.config.write() {
                                    *config_guard = new_config;
                                    info!("Configuration updated successfully");
                                } else {
                                    error!("Failed to acquire config write lock");
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to reload configuration: {e}");
                        }
                    }
                }
            }
        }
    });

    let app = Router::new()
        .route("/", get(generate_index))
        .route("/uptime", get(uptime_stream))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(uptime_state);

    std::mem::forget(watcher);

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
