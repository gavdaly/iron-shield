use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;
mod error;
mod index;
mod server;
mod uptime;

use crate::error::IronShieldError;

/// Main entry point for the Iron Shield application
///
/// Initializes tracing, parses command line arguments for the port,
/// starts the web server, and handles application lifecycle.
/// The application will listen for Ctrl+C (SIGINT) and SIGTERM signals
/// to perform a graceful shutdown.
#[tokio::main]
async fn main() -> Result<(), IronShieldError> {
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

    server::run(port).await?;

    tracing::info!("Iron Shield application shutting down");
    Ok(())
}
