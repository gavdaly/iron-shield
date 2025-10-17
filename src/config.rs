use serde::{Deserialize, Serialize};
use std::fs;

/// Application configuration structure
///
/// Contains all configuration parameters for the Iron Shield dashboard
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    /// Name of the site displayed in the page title
    pub site_name: String,
    /// Clock format to use (24-hour or 12-hour)
    pub clock: Option<Clock>,
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
    /// # Panics
    ///
    /// This function will panic if:
    /// - The config.json5 file cannot be read
    /// - The config.json5 file does not contain valid JSON5
    /// - The JSON5 does not match the expected Config structure
    pub fn load() -> Self {
        tracing::debug!("Loading application configuration");
        let config_str = fs::read_to_string("config.json5").expect("Failed to read config.json5");
        let config: Config = json5::from_str(&config_str).expect("Failed to parse config.json5");

        tracing::info!("Configuration loaded successfully");
        config
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
    #[default]
    TwelveHour,
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
        use Clock::{TwelveHour, TwentyFourHour};
        match self {
            TwentyFourHour => f.write_str("24hour"),
            TwelveHour => f.write_str("12hour"),
        }
    }
}
