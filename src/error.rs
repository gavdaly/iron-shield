//! Custom error types for the Iron Shield application
//!
//! This module defines custom error types and implements the necessary traits
//! to properly handle errors throughout the application. It provides a unified
//! error type that can represent various error conditions that may occur during
//! the operation of the Iron Shield application.
//!
//! The main error type, `IronShieldError`, encapsulates different categories of
//! errors that can occur, from configuration issues to server runtime problems.
//! This allows for consistent error handling across the application while
//! preserving the specific error details when needed.

use std::fmt;

/// Main error type for the Iron Shield application
///
/// This enum encapsulates all possible error types that can occur during the
/// operation of the Iron Shield application. Each variant represents a specific
/// category of error with its associated error type.
///
/// # Variants
///
/// * `AddressParse` - Error occurred while parsing network addresses
/// * `ServerRun` - Error occurred during server runtime operations
/// * `ConfigRead` - Error occurred while reading configuration files
/// * `ConfigParse` - Error occurred while parsing configuration data
/// * `JsonParse` - Error occurred while serializing/deserializing JSON data
/// * `Generic` - Generic error with a string message for unspecified errors
///
/// # Examples
///
/// ```
/// use iron_shield::error::{IronShieldError, Result};
///
/// fn example_function() -> Result<String> {
///     // Simulating an error condition
///     Err(IronShieldError::Generic("Something went wrong".to_string()))
/// }
///
/// match example_function() {
///     Ok(value) => println!("Success: {}", value),
///     Err(IronShieldError::Generic(msg)) => eprintln!("Error: {}", msg),
///     Err(e) => eprintln!("Unexpected error: {}", e),
/// }
/// ```
#[derive(Debug)]
pub enum IronShieldError {
    /// Error occurred while parsing network addresses
    AddressParse(std::net::AddrParseError),
    /// Error occurred while running the server
    ServerRun(axum::Error),
    /// Error occurred while reading configuration files
    ConfigRead(std::io::Error),
    /// Error occurred while parsing configuration data
    ConfigParse(json5::Error),
    /// Error occurred while serializing/deserializing JSON data
    JsonParse(serde_json::Error),
    /// Generic error with a string message for unspecified errors
    Generic(String),
}

impl fmt::Display for IronShieldError {
    /// Formats the error for display purposes
    ///
    /// This implementation provides user-friendly error messages that describe
    /// what type of error occurred and includes the underlying error details
    /// when available. This is important for logging and debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to a Formatter used to format the error
    ///
    /// # Returns
    ///
    /// Returns `fmt::Result` indicating whether the formatting was successful
    ///
    /// # Examples
    ///
    /// ```
    /// use iron_shield::error::IronShieldError;
    /// use std::io;
    ///
    /// let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    /// let error = IronShieldError::ConfigRead(io_error);
    ///
    /// println!("{}", error); // Displays: "Failed to read configuration file: File not found"
    /// ```
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
    /// Returns the lower-level source of this error, if any
    ///
    /// This method is important for error chaining, allowing access to the
    /// underlying error that caused this error. For variants that wrap other
    /// errors (like `ConfigRead`, `ServerRun`, etc.), this returns the wrapped
    /// error. For the `Generic` variant, this returns `None` as it doesn't
    /// wrap another error.
    ///
    /// # Returns
    ///
    /// * `Some(error)` - If the error wraps another error that can be accessed
    /// * `None` - If the error doesn't wrap another error (e.g., `Generic` variant)
    ///
    /// # Examples
    ///
    /// ```
    /// use iron_shield::error::IronShieldError;
    /// use std::error::Error;
    /// use std::io;
    ///
    /// let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    /// let error = IronShieldError::ConfigRead(io_error);
    ///
    /// // Check if there's an underlying error
    /// if let Some(source) = error.source() {
    ///     println!("Underlying error: {}", source);
    /// }
    /// ```
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
    /// Converts from a network address parsing error to `IronShieldError`
    ///
    /// This conversion allows network address parsing errors to be seamlessly
    /// converted to the application's custom error type, making error handling
    /// more consistent throughout the application.
    ///
    /// # Arguments
    ///
    /// * `error` - The network address parsing error to convert
    ///
    /// # Returns
    ///
    /// Returns an `IronShieldError::AddressParse` variant containing the original error
    fn from(error: std::net::AddrParseError) -> Self {
        IronShieldError::AddressParse(error)
    }
}

impl From<axum::Error> for IronShieldError {
    /// Converts from an Axum framework error to `IronShieldError`
    ///
    /// This conversion allows errors from the Axum web framework to be converted
    /// to the application's custom error type, maintaining consistency in error
    /// handling across all parts of the application.
    ///
    /// # Arguments
    ///
    /// * `error` - The Axum error to convert
    ///
    /// # Returns
    ///
    /// Returns an `IronShieldError::ServerRun` variant containing the original error
    fn from(error: axum::Error) -> Self {
        IronShieldError::ServerRun(error)
    }
}

impl From<std::io::Error> for IronShieldError {
    /// Converts from an IO error to `IronShieldError`
    ///
    /// This conversion allows input/output errors (such as file read errors)
    /// to be seamlessly converted to the application's custom error type,
    /// making error handling more consistent throughout the application.
    ///
    /// # Arguments
    ///
    /// * `error` - The IO error to convert
    ///
    /// # Returns
    ///
    /// Returns an `IronShieldError::ConfigRead` variant containing the original error
    fn from(error: std::io::Error) -> Self {
        IronShieldError::ConfigRead(error)
    }
}

impl From<json5::Error> for IronShieldError {
    /// Converts from a JSON5 parsing error to `IronShieldError`
    ///
    /// This conversion allows errors during JSON5 parsing (e.g., when loading
    /// the configuration file) to be converted to the application's custom error type.
    ///
    /// # Arguments
    ///
    /// * `error` - The JSON5 parsing error to convert
    ///
    /// # Returns
    ///
    /// Returns an `IronShieldError::ConfigParse` variant containing the original error
    fn from(error: json5::Error) -> Self {
        IronShieldError::ConfigParse(error)
    }
}

impl From<serde_json::Error> for IronShieldError {
    /// Converts from a Serde JSON error to `IronShieldError`
    ///
    /// This conversion allows errors during JSON serialization/deserialization
    /// to be converted to the application's custom error type.
    ///
    /// # Arguments
    ///
    /// * `error` - The Serde JSON error to convert
    ///
    /// # Returns
    ///
    /// Returns an `IronShieldError::JsonParse` variant containing the original error
    fn from(error: serde_json::Error) -> Self {
        IronShieldError::JsonParse(error)
    }
}

impl From<&str> for IronShieldError {
    /// Converts from a string slice to `IronShieldError`
    ///
    /// This conversion creates a generic error with the provided string message.
    /// It's useful for creating simple error messages without having to manually
    /// construct the `IronShieldError::Generic` variant.
    ///
    /// # Arguments
    ///
    /// * `error` - The string slice containing the error message
    ///
    /// # Returns
    ///
    /// Returns an `IronShieldError::Generic` variant containing the error message
    fn from(error: &str) -> Self {
        IronShieldError::Generic(error.to_string())
    }
}

impl From<String> for IronShieldError {
    /// Converts from a String to `IronShieldError`
    ///
    /// This conversion creates a generic error with the provided string message.
    /// It's useful for creating simple error messages without having to manually
    /// construct the `IronShieldError::Generic` variant.
    ///
    /// # Arguments
    ///
    /// * `error` - The String containing the error message
    ///
    /// # Returns
    ///
    /// Returns an `IronShieldError::Generic` variant containing the error message
    fn from(error: String) -> Self {
        IronShieldError::Generic(error)
    }
}

/// Result type alias using our custom error type
///
/// This type alias provides a convenient shorthand for `Result<T, IronShieldError>`
/// throughout the application. Using this alias makes the code more concise and
/// consistent in its error handling approach.
///
/// # Type Parameters
///
/// * `T` - The type of the value returned in the `Ok` variant
///
/// # Examples
///
/// ```
/// use iron_shield::error::Result;
///
/// fn example_function() -> Result<String> {
///     // This function returns Result<String, IronShieldError>
///     Ok("success".to_string())
/// }
///
/// match example_function() {
///     Ok(value) => println!("Success: {}", value),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
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
