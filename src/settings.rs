use crate::config::{Clock, Config};
use crate::error::Result;
use crate::uptime::UptimeState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use tracing::{error, info};
use url::Url;

/// Structure to receive configuration updates from the API
///
/// This struct represents the data structure for updating the application configuration
/// via the API. It contains all the necessary fields to update site settings, clock format,
/// and monitored sites.
///
/// # Fields
///
/// * `site_name` - The name of the site to display in the UI
/// * `clock` - The clock format as a string ("24hour", "12hour", or "none")
/// * `sites` - A vector of site updates to monitor
///
/// # Examples
///
/// ```
/// use iron_shield::settings::ConfigUpdate;
/// use iron_shield::settings::SiteUpdate;
///
/// let config_update = ConfigUpdate {
///     site_name: "My Site".to_string(),
///     clock: "24hour".to_string(),
///     sites: vec![
///         SiteUpdate {
///             name: "Example".to_string(),
///             url: "https://example.com".to_string(),
///             category: "Web".to_string(),
///             tags: vec!["important".to_string()],
///         }
///     ],
/// };
///
/// assert!(config_update.validate().is_ok());
/// ```
#[derive(Deserialize, Serialize, Clone)]
pub struct ConfigUpdate {
    /// The name of the site to display in the UI
    pub site_name: String,
    /// The clock format as a string ("24hour", "12hour", or "none")
    pub clock: String,
    /// A vector of site updates to monitor
    pub sites: Vec<SiteUpdate>,
}

impl ConfigUpdate {
    /// Validates the configuration update
    ///
    /// This method checks that all required fields are properly set and that URLs have valid formats.
    /// It performs comprehensive validation to ensure the configuration is safe to save.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the configuration is valid, or an error if validation fails
    ///
    /// # Errors
    ///
    /// This function returns an `IronShieldError` if:
    /// - The site name is empty.
    /// - The clock format is invalid (not "24hour", "12hour", or "none").
    /// - A site name or URL is empty.
    /// - A site URL has an invalid format.
    ///
    /// # Examples
    ///
    /// ```
    /// use iron_shield::settings::ConfigUpdate;
    /// use iron_shield::settings::SiteUpdate;
    ///
    /// let mut config_update = ConfigUpdate {
    ///     site_name: "My Site".to_string(),
    ///     clock: "24hour".to_string(),
    ///     sites: vec![
    ///         SiteUpdate {
    ///             name: "Example".to_string(),
    ///             url: "https://example.com".to_string(),
    ///             category: "Web".to_string(),
    ///             tags: vec!["important".to_string()],
    ///         }
    ///     ],
    /// };
    ///
    /// assert!(config_update.validate().is_ok());
    ///
    /// config_update.site_name = "".to_string(); // Invalid - empty site name
    /// assert!(config_update.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<()> {
        if self.site_name.trim().is_empty() {
            return Err(crate::error::IronShieldError::from(
                "Site name cannot be empty",
            ));
        }

        match self.clock.as_str() {
            "24hour" | "12hour" | "none" => {}
            _ => return Err(crate::error::IronShieldError::from("Invalid clock format")),
        }

        for site in &self.sites {
            if site.name.trim().is_empty() {
                return Err(crate::error::IronShieldError::from(
                    "Site name cannot be empty",
                ));
            }
            if site.url.trim().is_empty() {
                return Err(crate::error::IronShieldError::from(
                    "Site URL cannot be empty",
                ));
            }

            // Validate URL format
            if Url::parse(&site.url).is_err() {
                return Err(crate::error::IronShieldError::from(format!(
                    "Invalid URL format: {}",
                    site.url
                )));
            }
        }

        Ok(())
    }
}

/// Structure to receive site updates from the API
///
/// This struct represents the data structure for updating individual site information
/// via the API. It contains all the necessary fields to define a site to monitor.
///
/// # Fields
///
/// * `name` - The display name for the site
/// * `url` - The URL to monitor
/// * `category` - The category of the site for organizational purposes
/// * `tags` - A vector of tags to associate with the site
///
/// # Examples
///
/// ```
/// use iron_shield::settings::SiteUpdate;
///
/// let site_update = SiteUpdate {
///     name: "Example Site".to_string(),
///     url: "https://example.com".to_string(),
///     category: "Web Services".to_string(),
///     tags: vec!["important".to_string(), "external".to_string()],
/// };
///
/// assert_eq!(site_update.name, "Example Site");
/// assert_eq!(site_update.url, "https://example.com");
/// ```
#[derive(Deserialize, Serialize, Clone)]
pub struct SiteUpdate {
    /// The display name for the site
    pub name: String,
    /// The URL to monitor
    pub url: String,
    /// The category of the site for organizational purposes
    pub category: String,
    /// A vector of tags to associate with the site
    pub tags: Vec<String>,
}

/// Saves the configuration to the config.json5 file
///
/// This function handles the API request to update and save the application configuration.
/// It validates the incoming configuration data, converts it to the appropriate format,
/// writes it to the configuration file, and updates the in-memory configuration.
///
/// The function performs validation of the configuration data before saving, converts
/// string-based clock format to the appropriate enum, and updates both the file system
/// and in-memory configuration.
///
/// # Arguments
///
/// * `State(state)` - The uptime state containing the configuration and file path
/// * `Json(payload)` - The updated configuration data as a JSON payload
///
/// # Returns
///
/// An HTTP response indicating success (200 OK) or failure (500 Internal Server Error).
/// On success, returns the message "Configuration saved successfully".
/// On failure, returns the error message.
///
/// # Errors
///
/// This function returns an HTTP 500 error response if:
/// - The configuration fails validation
/// - The configuration cannot be serialized to JSON5 format
/// - The configuration file cannot be written to disk
/// - The in-memory configuration cannot be updated due to a lock error
///
/// # Examples
///
/// Using this in an Axum router:
///
/// ```rust,no_run
/// use axum::{Router, routing::post};
/// use iron_shield::settings::save_config;
///
/// let app = Router::new()
///     .route("/api/config", post(save_config));
/// ```
pub async fn save_config(
    State(state): State<Arc<UptimeState>>,
    Json(payload): Json<ConfigUpdate>,
) -> impl IntoResponse {
    tracing::info!("Saving configuration");

    let result = (|| -> Result<()> {
        // Validate the configuration update
        payload.validate()?;

        // Convert the clock format string to the Clock enum
        let clock = match payload.clock.as_str() {
            "24hour" => Clock::Hour24,
            "12hour" => Clock::Hour12,
            "none" => Clock::None,
            _ => return Err(crate::error::IronShieldError::from("Invalid clock format")),
        };

        // Convert SiteUpdate to Site
        let sites: Vec<crate::config::Site> = payload
            .sites
            .into_iter()
            .map(|site_update| crate::config::Site {
                name: site_update.name,
                url: site_update.url,
                category: site_update.category,
                tags: site_update.tags,
                uptime_percentage: 0.0, // Initialize to 0.0, will be updated by uptime service
            })
            .collect();

        // Create a new configuration based on the payload
        let new_config = Config {
            site_name: payload.site_name,
            clock,
            sites,
        };

        // Write the updated configuration to the file
        let config_json = json5::to_string(&new_config).map_err(|e| {
            crate::error::IronShieldError::from(format!("Failed to serialize config: {e}"))
        })?;

        fs::write(&state.config_file_path, config_json).map_err(|e| {
            crate::error::IronShieldError::from(format!("Failed to write config file: {e}"))
        })?;

        // Update the config in memory
        {
            let mut config_guard = state.config.write().map_err(|_| {
                crate::error::IronShieldError::from("Failed to acquire config write lock")
            })?;
            *config_guard = new_config;
            info!("Configuration updated successfully in memory");
        }

        Ok(())
    })();

    match result {
        Ok(()) => (StatusCode::OK, "Configuration saved successfully").into_response(),
        Err(e) => {
            error!("Error saving configuration: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_update_validate_valid_data() {
        let config_update = ConfigUpdate {
            site_name: "Test Site".to_string(),
            clock: "24hour".to_string(),
            sites: vec![SiteUpdate {
                name: "Example".to_string(),
                url: "https://example.com".to_string(),
                category: "Web".to_string(),
                tags: vec!["test".to_string()],
            }],
        };

        assert!(config_update.validate().is_ok());
    }

    #[test]
    fn test_config_update_validate_empty_site_name() {
        let config_update = ConfigUpdate {
            site_name: String::new(),
            clock: "24hour".to_string(),
            sites: vec![],
        };

        assert!(config_update.validate().is_err());
    }

    #[test]
    fn test_config_update_validate_invalid_clock() {
        let config_update = ConfigUpdate {
            site_name: "Test Site".to_string(),
            clock: "invalid".to_string(),
            sites: vec![],
        };

        assert!(config_update.validate().is_err());
    }

    #[test]
    fn test_config_update_validate_empty_site_fields() {
        let mut config_update = ConfigUpdate {
            site_name: "Test Site".to_string(),
            clock: "24hour".to_string(),
            sites: vec![SiteUpdate {
                name: String::new(),
                url: "https://example.com".to_string(),
                category: "Web".to_string(),
                tags: vec!["test".to_string()],
            }],
        };

        assert!(config_update.validate().is_err());

        config_update.sites[0].name = "Example".to_string();
        config_update.sites[0].url = String::new();

        assert!(config_update.validate().is_err());
    }

    #[test]
    fn test_config_update_validate_invalid_url() {
        let config_update = ConfigUpdate {
            site_name: "Test Site".to_string(),
            clock: "24hour".to_string(),
            sites: vec![SiteUpdate {
                name: "Example".to_string(),
                url: "invalid-url".to_string(),
                category: "Web".to_string(),
                tags: vec!["test".to_string()],
            }],
        };

        assert!(config_update.validate().is_err());
    }

    #[test]
    fn test_config_update_validate_valid_clock_formats() {
        let valid_clocks = vec!["24hour", "12hour", "none"];

        for clock_format in valid_clocks {
            let config_update = ConfigUpdate {
                site_name: "Test Site".to_string(),
                clock: clock_format.to_string(),
                sites: vec![],
            };

            assert!(
                config_update.validate().is_ok(),
                "Clock format '{clock_format}' should be valid"
            );
        }
    }

    #[test]
    fn test_site_update_creation() {
        let site_update = SiteUpdate {
            name: "Test Site".to_string(),
            url: "https://test.com".to_string(),
            category: "Test Category".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        assert_eq!(site_update.name, "Test Site");
        assert_eq!(site_update.url, "https://test.com");
        assert_eq!(site_update.category, "Test Category");
        assert_eq!(
            site_update.tags,
            vec!["tag1".to_string(), "tag2".to_string()]
        );
    }

    #[test]
    fn test_config_update_creation() {
        let config_update = ConfigUpdate {
            site_name: "Test Site".to_string(),
            clock: "12hour".to_string(),
            sites: vec![],
        };

        assert_eq!(config_update.site_name, "Test Site");
        assert_eq!(config_update.clock, "12hour");
        assert!(config_update.sites.is_empty());
    }
}
