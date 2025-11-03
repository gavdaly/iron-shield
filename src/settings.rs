use crate::config::{Clock, Config, CONFIG_FILE};
use crate::error::Result;
use crate::uptime::UptimeState;
use askama_axum::Template;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use tracing::{error, info};
use url::Url;

// Template structure for the settings page
#[derive(Template)]
#[template(path = "settings.html")]
pub struct SettingsTemplate {
    config: Config,
}

/// Generates the settings template with loaded configuration
///
/// # Returns
///
/// An HTML response with the settings template or an error response
pub async fn generate_settings(State(state): State<Arc<UptimeState>>) -> impl IntoResponse {
    tracing::debug!("Generating settings template");

    // Get the config from the shared state
    let config = match state.config.read() {
        Ok(config_guard) => config_guard.clone(), // Clone the config to avoid holding the lock
        Err(_) => {
            tracing::error!("Configuration read lock error");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration read lock error",
            )
                .into_response();
        }
    };

    let template = SettingsTemplate { config };
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Template rendering error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Template rendering error",
            )
                .into_response()
        }
    }
}

/// Structure to receive configuration updates from the API
#[derive(Deserialize, Serialize, Clone)]
pub struct ConfigUpdate {
    pub site_name: String,
    pub clock: String,
    pub sites: Vec<SiteUpdate>,
}

impl ConfigUpdate {
    /// Validates the configuration update
    ///
    /// # Returns
    ///
    /// `Ok(())` if the configuration is valid, or an error if validation fails
    pub fn validate(&self) -> Result<()> {
        if self.site_name.trim().is_empty() {
            return Err(crate::error::IronShieldError::from("Site name cannot be empty"));
        }

        match self.clock.as_str() {
            "24hour" | "12hour" | "none" => {},
            _ => return Err(crate::error::IronShieldError::from("Invalid clock format")),
        }

        for site in &self.sites {
            if site.name.trim().is_empty() {
                return Err(crate::error::IronShieldError::from("Site name cannot be empty"));
            }
            if site.url.trim().is_empty() {
                return Err(crate::error::IronShieldError::from("Site URL cannot be empty"));
            }
            
            // Validate URL format
            if let Err(_) = Url::parse(&site.url) {
                return Err(crate::error::IronShieldError::from(format!("Invalid URL format: {}", site.url)));
            }
        }

        Ok(())
    }
}

/// Structure to receive site updates from the API
#[derive(Deserialize, Serialize, Clone)]
pub struct SiteUpdate {
    pub name: String,
    pub url: String,
    pub category: String,
    pub tags: Vec<String>,
}

/// Saves the configuration to the config.json5 file
///
/// # Arguments
///
/// * `State(state)` - The uptime state containing the configuration
/// * `Json(payload)` - The updated configuration data
///
/// # Returns
///
/// An HTTP response indicating success or failure
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
            })
            .collect();

        // Create a new configuration based on the payload
        let new_config = Config {
            site_name: payload.site_name,
            clock,
            sites,
        };

        // Write the updated configuration to the file
        let config_json = json5::to_string(&new_config)
            .map_err(|e| crate::error::IronShieldError::from(format!("Failed to serialize config: {e}")))?;
        
        fs::write(CONFIG_FILE, config_json)
            .map_err(|e| crate::error::IronShieldError::from(format!("Failed to write config file: {e}")))?;

        // Update the config in memory
        {
            let mut config_guard = state.config.write()
                .map_err(|_| crate::error::IronShieldError::from("Failed to acquire config write lock"))?;
            *config_guard = new_config;
            info!("Configuration updated successfully in memory");
        }

        Ok(())
    })();

    match result {
        Ok(_) => (StatusCode::OK, "Configuration saved successfully").into_response(),
        Err(e) => {
            error!("Error saving configuration: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}