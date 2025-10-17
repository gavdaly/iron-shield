use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;
mod error;
mod index;
mod server;

/// Main entry point for the Iron Shield application
///
/// Initializes tracing, parses command line arguments for the port,
/// starts the web server, and handles application lifecycle
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    tracing::info!("Starting Iron Shield application");
    tracing::debug!("Application initialized with tracing enabled");

    if let Err(e) = server::run(port).await {
        eprintln!("Application error: {e}");
        tracing::error!("Server error: {e}");
    }

    tracing::info!("Iron Shield application shutting down");
}
