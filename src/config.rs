use serde::{Deserialize, Serialize};
use std::fs;

/// Application configuration structure
///
/// Contains all configuration parameters for the Iron Shield dashboard
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    /// Name of the site displayed in the page title
    pub site_name: String,
    /// Clock format to use (24-hour, 12-hour, or no clock)
    pub clock: Clock,
    /// List of bookmarked sites to display
    pub sites: Vec<Site>,
}

/// Represents a bookmarked website in the dashboard
///
/// Contains the essential information for displaying and accessing a bookmarked site
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Site {
    /// Display name for the site
    pub name: String,
    /// URL of the site
    pub url: String,
    /// List of tags for categorization and filtering
    pub tags: Vec<String>,
}

impl Config {
    /// Load the application configuration from the config.json5 file.
    ///
    /// # Returns
    ///
    /// Returns the loaded Config if successful, or an `IronShieldError` if an error occurs
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be read or parsed
    pub fn load() -> crate::error::Result<Self> {
        tracing::debug!("Loading application configuration");
        let config_str = fs::read_to_string("config.json5")?;

        // Deserialize to a JSON Value first to check if clock field exists
        let mut value: serde_json::Value = json5::from_str(&config_str)?;

        // If clock field is missing, set it to the default
        if value.get("clock").is_none() {
            if let Some(obj) = value.as_object_mut() {
                obj.insert(
                    "clock".to_string(),
                    serde_json::Value::String("NoClock".to_string()),
                );
            } else {
                return Err(crate::error::IronShieldError::Generic(
                    "Config is not an object".to_string(),
                ));
            }
        }

        let config: Config = serde_json::from_value(value)?;

        tracing::info!("Configuration loaded successfully");
        Ok(config)
    }
}

/// Clock format options
///
/// Defines the format in which to display the time on the dashboard
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum Clock {
    /// 24-hour format (e.g., 13:00)
    TwentyFourHour,
    /// 12-hour format with AM/PM (e.g., 1:00 PM)
    TwelveHour,
    /// No clock displayed
    #[default]
    NoClock,
}

impl std::fmt::Display for Clock {
    /// Formats the Clock enum to its string representation
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter to write the string to
    ///
    /// # Returns
    ///
    /// Result of the formatting operation
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Clock::TwentyFourHour => f.write_str("24hour"),
            Clock::TwelveHour => f.write_str("12hour"),
            Clock::NoClock => f.write_str("noclock"),
        }
    }
}
