use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;
mod index;
mod server;

#[tokio::main]
async fn main() {
    // Initialize tracing with env filter
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

    server::run(port).await;

    tracing::info!("Iron Shield application shutting down");
}
