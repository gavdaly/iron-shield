//! # Iron Shield Main Application Entry Point
//!
//! This is the main executable for the Iron Shield dashboard application.
//! It handles command-line argument parsing, tracing initialization,
//! server startup, and application lifecycle management.
//!
//! The application can be launched with optional command-line arguments:
//!
//! - First argument: Port number (defaults to 3000)
//! - Second argument: Path to configuration file (defaults to "config.json5")
//!
//! ## Example Usage
//!
//! ```bash
//! # Run with default settings (port 3000, default config)
//! cargo run
//!
//! # Run on a specific port
//! cargo run 8080
//!
//! # Run with a specific port and configuration file
//! cargo run 8080 my-config.json5
//! ```
//!
//! The application includes comprehensive logging using the tracing framework.
//! Log levels can be controlled through the `RUST_LOG` environment variable.

use std::env;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config;
mod error;
mod index;
mod server;
mod settings;
mod telemetry;
mod uptime;
mod utils;

use crate::error::IronShieldError;

/// Main entry point for the Iron Shield application
///
/// This function:
/// 1. Initializes the tracing subscriber for application logging
/// 2. Parses command line arguments for port and configuration file path
/// 3. Creates a cancellation token for graceful shutdown
/// 4. Starts the web server with the specified parameters
/// 5. Handles application lifecycle and graceful shutdown
///
/// The function uses the `#[tokio::main]` macro to create a runtime and execute
/// the async block, making it the entry point for the application's async operations.
///
/// # Arguments
///
/// This function takes no explicit arguments as it reads command-line arguments using `std::env::args()`.
///
/// # Returns
///
/// Returns `Ok(())` if the application shuts down successfully, or an `IronShieldError` if an error occurs.
///
/// # Errors
///
/// The function returns an error if:
/// - The server fails to start
/// - Configuration cannot be loaded
/// - Any unrecoverable error occurs during execution
///
/// # Command Line Arguments
///
/// * First argument (optional): Port number to run the server on (defaults to 3000)
/// * Second argument (optional): Path to the configuration file (defaults to "config.json5")
///
/// # Examples
///
/// ```bash
/// # Run with default port (3000) and default config file (config.json5)
/// cargo run
///
/// # Run on port 8080 with default config
/// cargo run 8080
///
/// # Run on port 8080 with custom config file
/// cargo run 8080 /path/to/config.json5
/// ```
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

    let mut config_file_path: Option<PathBuf> = None;

    if let Some(arg2) = env::args().nth(2) {
        config_file_path = Some(PathBuf::from(arg2));
    }

    tracing::info!("Starting Iron Shield application");
    tracing::debug!("Application initialized with tracing enabled");

    let cancel_token = CancellationToken::new();
    server::run(port, config_file_path, cancel_token).await?;

    tracing::info!("Iron Shield application shutting down");
    Ok(())
}
