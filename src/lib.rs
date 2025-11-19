//! # Iron Shield Library
//!
//! This library provides the core functionality for the Iron Shield dashboard application.
//! Iron Shield is a customizable launcher & uptime monitor with a minimal UI that displays
//! a clock and a list of bookmarked websites with their current status.
//!
//! ## Overview
//!
//! The library is organized into several modules that handle different aspects of the application:
//!
//! - `config`: Handles application configuration and settings
//! - `error`: Defines custom error types for consistent error handling
//! - `index`: Renders the main dashboard page
//! - `server`: Runs the web server and manages routes
//! - `settings`: Handles the settings page and API
//! - `uptime`: Manages uptime monitoring and status updates
//! - `utils`: Provides utility functions used throughout the application
//!
//! ## Getting Started
//!
//! To use this library in your own application, you'll typically use the server module to start
//! the web server with the `run` function:
//!
//! ```no_run
//! use iron_shield::{config::CONFIG_FILE, server};
//! use std::path::PathBuf;
//! use tokio_util::sync::CancellationToken;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), iron_shield::error::IronShieldError> {
//!     let cancel_token = CancellationToken::new();
//!     let config_path = Some(PathBuf::from(CONFIG_FILE));
//!
//!     server::run(3000, config_path, cancel_token).await
//! }
//! ```
//!
//! ## Features
//!
//! - **Configurable Dashboard**: Customize the site name and clock format
//! - **Uptime Monitoring**: Real-time monitoring of website availability with status indicators
//! - **Settings API**: Dynamic configuration updates via a web interface
//! - **Responsive UI**: Clean, minimal design that works on different screen sizes
//! - **File Watching**: Automatic configuration reload when config file changes
//!
//! ## Architecture
//!
//! The application follows a modular architecture with clear separation of concerns:
//! - The server module handles HTTP requests and routing
//! - The config module manages application settings
//! - The uptime module monitors site availability
//! - The index module generates the main dashboard
//! - The settings module allows configuration updates
//! - The utils module provides common utilities

/// ## Error Handling
///
/// The library uses a custom error type `IronShieldError` for consistent error handling
/// throughout the application. This error type can represent various error conditions
/// such as configuration loading errors, server runtime errors, and more.
///
/// Custom error types module
///
/// Defines the `IronShieldError` enum and related functionality for consistent error
/// handling across the application. This module provides a unified error type that
/// can represent various error conditions that may occur during application operation.
pub mod error;

/// Configuration management module
///
/// Handles application configuration loading, validation, and watching. The config module
/// provides functionality for loading settings from a JSON5 file, validating them,
/// and automatically reloading when changes are detected. It supports various configuration
/// options including site names, clock formats, and bookmarked sites.
pub mod config;

/// Index page generation module
///
/// Responsible for rendering the main dashboard page. This module combines configuration
/// data with a template to generate the HTML for the main dashboard view, including
/// site bookmarks and current time display.
pub mod index;

/// Server operations module
///
/// Contains the main web server implementation using the Axum framework. This module
/// sets up routes, handles HTTP requests, serves static files, and manages graceful
/// shutdown of the server. It coordinates between other modules to provide the complete
/// web application functionality.
pub mod server;

/// Uptime monitoring module
///
/// Implements website monitoring functionality that checks the availability of configured
/// sites and reports their status in real-time. This module uses Server-Sent Events (SSE)
/// to push status updates to the frontend and maintains historical uptime data.
pub mod uptime;

/// Settings page module
///
/// Handles the settings page and API for updating application configuration. This module
/// provides both a web interface for configuration management and an API endpoint for
/// dynamic updates to site configurations and settings.
pub mod settings;

/// Telemetry helpers
///
/// Provides utilities for shipping uptime snapshots to external collectors so the
/// dashboard can maintain a long-term history outside of the local runtime.
pub mod telemetry;

/// Utility functions module
///
/// Contains common utility functions used throughout the application, such as time
/// formatting utilities that are used for displaying current time on the dashboard.
pub mod utils;
