//! Custom error types for the Iron Shield application
//!
//! This module defines custom error types and implements the necessary traits
//! to properly handle errors throughout the application.

use std::fmt;

/// Main error type for the Iron Shield application
#[derive(Debug)]
pub enum IronShieldError {
    /// Error occurred while parsing address
    AddressParse(std::net::AddrParseError),

    /// Error occurred while running the server
    ServerRun(axum::Error),

    /// Error occurred while reading configuration file
    ConfigRead(std::io::Error),

    /// Error occurred while parsing configuration
    ConfigParse(json5::Error),

    /// Generic error with a message
    Generic(String),
}

impl fmt::Display for IronShieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IronShieldError::AddressParse(e) => {
                write!(f, "Failed to parse network address: {e}")
            }
            IronShieldError::ServerRun(e) => {
                write!(f, "Server runtime error: {e}")
            }
            IronShieldError::ConfigRead(e) => {
                write!(f, "Failed to read configuration file: {e}")
            }
            IronShieldError::ConfigParse(e) => {
                write!(f, "Failed to parse configuration: {e}")
            }
            IronShieldError::Generic(msg) => {
                write!(f, "Error: {msg}")
            }
        }
    }
}

impl std::error::Error for IronShieldError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IronShieldError::AddressParse(e) => Some(e),
            IronShieldError::ServerRun(e) => Some(e),
            IronShieldError::ConfigRead(e) => Some(e),
            IronShieldError::ConfigParse(e) => Some(e),
            IronShieldError::Generic(_) => None,
        }
    }
}

impl From<std::net::AddrParseError> for IronShieldError {
    fn from(error: std::net::AddrParseError) -> Self {
        IronShieldError::AddressParse(error)
    }
}

impl From<axum::Error> for IronShieldError {
    fn from(error: axum::Error) -> Self {
        IronShieldError::ServerRun(error)
    }
}

impl From<std::io::Error> for IronShieldError {
    fn from(error: std::io::Error) -> Self {
        IronShieldError::ConfigRead(error)
    }
}

impl From<json5::Error> for IronShieldError {
    fn from(error: json5::Error) -> Self {
        IronShieldError::ConfigParse(error)
    }
}

/// Result type alias using our custom error type
pub type Result<T> = std::result::Result<T, IronShieldError>;
