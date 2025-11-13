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
    /// Error occurred while serializing/deserializing JSON
    JsonParse(serde_json::Error),
    /// Generic error with a message
    Generic(String),
}

impl fmt::Display for IronShieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IronShieldError::AddressParse(e) => write!(f, "Failed to parse network address: {e}"),
            IronShieldError::ServerRun(e) => write!(f, "Server runtime error: {e}"),
            IronShieldError::ConfigRead(e) => write!(f, "Failed to read configuration file: {e}"),
            IronShieldError::ConfigParse(e) => write!(f, "Failed to parse configuration: {e}"),
            IronShieldError::JsonParse(e) => write!(f, "Failed to parse JSON: {e}"),
            IronShieldError::Generic(msg) => write!(f, "Error: {msg}"),
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
            IronShieldError::JsonParse(e) => Some(e),
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

impl From<serde_json::Error> for IronShieldError {
    fn from(error: serde_json::Error) -> Self {
        IronShieldError::JsonParse(error)
    }
}

impl From<&str> for IronShieldError {
    fn from(error: &str) -> Self {
        IronShieldError::Generic(error.to_string())
    }
}

impl From<String> for IronShieldError {
    fn from(error: String) -> Self {
        IronShieldError::Generic(error)
    }
}

/// Result type alias using our custom error type
pub type Result<T> = std::result::Result<T, IronShieldError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io;

    #[test]
    fn test_iron_shield_error_display() {
        // Test AddressParse error
        let addr_parse_error = "invalid".parse::<std::net::IpAddr>().unwrap_err();
        let addr_error = IronShieldError::AddressParse(addr_parse_error);
        assert!(format!("{addr_error}").contains("Failed to parse network address"));

        // Test ServerRun error
        let server_error = IronShieldError::ServerRun(axum::Error::new(std::io::Error::other(
            "test server error",
        )));
        assert!(format!("{server_error}").contains("Server runtime error"));

        // Test ConfigRead error
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let config_read_error = IronShieldError::ConfigRead(io_error);
        assert!(format!("{config_read_error}").contains("Failed to read configuration file"));

        // Test ConfigParse error - creating a JSON5 error by attempting to parse invalid JSON5
        let config_parse_error = IronShieldError::ConfigParse(
            json5::from_str::<serde_json::Value>("{invalid json5").unwrap_err(),
        );
        assert!(format!("{config_parse_error}").contains("Failed to parse configuration"));

        // Test JsonParse error
        let json_parse_error = IronShieldError::JsonParse(
            serde_json::from_str::<serde_json::Value>("{invalid json").unwrap_err(),
        );
        assert!(format!("{json_parse_error}").contains("Failed to parse JSON"));

        // Test Generic error
        let generic_error = IronShieldError::Generic("Test error".to_string());
        assert!(format!("{generic_error}").contains("Error: Test error"));
    }

    #[test]
    fn test_iron_shield_error_source() {
        // Test that errors with sources return Some
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let config_read_error = IronShieldError::ConfigRead(io_error);
        assert!(config_read_error.source().is_some());

        // Test that Generic error returns None as source
        let generic_error = IronShieldError::Generic("Test error".to_string());
        assert!(generic_error.source().is_none());
    }

    #[test]
    fn test_from_implementations() {
        // Test From<std::net::AddrParseError>
        let addr_parse_error: std::net::AddrParseError =
            "invalid".parse::<std::net::IpAddr>().unwrap_err();
        let iron_shield_error: IronShieldError = addr_parse_error.into();
        assert!(matches!(
            iron_shield_error,
            IronShieldError::AddressParse(_)
        ));

        // Test From<axum::Error>
        let axum_error = axum::Error::new(std::io::Error::other("test server error"));
        let iron_shield_error: IronShieldError = axum_error.into();
        assert!(matches!(iron_shield_error, IronShieldError::ServerRun(_)));

        // Test From<std::io::Error>
        let io_error = io::Error::new(io::ErrorKind::InvalidInput, "Test IO error");
        let iron_shield_error: IronShieldError = io_error.into();
        assert!(matches!(iron_shield_error, IronShieldError::ConfigRead(_)));

        // Test From<json5::Error>
        let json5_error = json5::from_str::<serde_json::Value>("{invalid json5").unwrap_err();
        let iron_shield_error: IronShieldError = json5_error.into();
        assert!(matches!(iron_shield_error, IronShieldError::ConfigParse(_)));

        // Test From<serde_json::Error>
        let serde_error = serde_json::from_str::<serde_json::Value>("{invalid json").unwrap_err();
        let iron_shield_error: IronShieldError = serde_error.into();
        assert!(matches!(iron_shield_error, IronShieldError::JsonParse(_)));

        // Test From<&str>
        let str_error: IronShieldError = "test error".into();
        assert!(matches!(str_error, IronShieldError::Generic(_)));

        // Test From<String>
        let string_error: IronShieldError = "test error".to_string().into();
        assert!(matches!(string_error, IronShieldError::Generic(_)));
    }

    #[test]
    fn test_iron_shield_error_debug() {
        let addr_parse_error = "invalid".parse::<std::net::IpAddr>().unwrap_err();
        let config_read_error = IronShieldError::AddressParse(addr_parse_error);

        let debug_output = format!("{config_read_error:?}");
        assert!(debug_output.contains("AddressParse"));
    }

    #[test]
    fn test_result_type_alias() {
        // Test that the Result type alias works as expected
        let success_result: Result<String> = Ok("success".to_string());
        assert!(success_result.is_ok());

        let error_result: Result<String> = Err(IronShieldError::Generic("test".to_string()));
        assert!(error_result.is_err());
    }
}
