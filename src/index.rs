use crate::config::Clock;
use crate::settings::{ConfigUpdate, SiteUpdate};
use crate::uptime::UptimeState;
use crate::utils;
use askama_axum::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use std::sync::Arc;
use tracing::error;

/// Template structure for the index page
///
/// This struct is used by the Askama templating engine to render the main dashboard page.
/// It contains all the necessary data needed to populate the HTML template, including
/// the application configuration and the current time for display purposes.
///
/// # Fields
///
/// * `config` - The application configuration containing site name, clock format, and monitored sites
/// * `current_time` - The current UTC time as a formatted string for display in the template
///
/// # Examples
///
/// ```
/// use iron_shield::config::{Config, Clock, Site};
/// use iron_shield::index::IndexTemplate;
/// use iron_shield::utils;
///
/// let config = Config {
///     site_name: "My Dashboard".to_string(),
///     clock: Clock::Hour24,
///     sites: vec![Site {
///         name: "Example".to_string(),
///         url: "https://example.com".to_string(),
///         category: "Web".to_string(),
///         tags: vec!["important".to_string()],
///         uptime_percentage: 99.5,
///     }],
/// };
///
/// let current_time = utils::get_current_time_string();
/// // Note: IndexTemplate is used internally by the generate_index function
/// // and is not typically constructed directly in user code
/// ```
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// Application configuration containing site information and settings
    config: crate::config::Config,
    /// Current UTC time as a formatted string for display in the template
    current_time: String,
    /// JSON representation of the configuration for the frontend settings modal
    config_json: String,
}

/// Generates the index template with loaded configuration
///
/// This function handles the main page request by retrieving the current configuration
/// from shared state, formatting the current time, and rendering the index template.
/// It's designed to be used as an Axum handler for the main dashboard endpoint.
///
/// The function acquires a read lock on the shared configuration, clones the data
/// to avoid holding the lock during template rendering, and then generates the
/// appropriate HTML response based on the configuration and current time.
///
/// # Arguments
///
/// * `State(state)` - The uptime state containing the shared configuration
///
/// # Returns
///
/// An HTML response with the rendered index template, or an error response if
/// the configuration could not be accessed or if template rendering failed.
///
/// # Errors
///
/// This function returns an HTTP 500 error response if:
/// - The configuration read lock cannot be acquired
/// - The template cannot be rendered
///
/// # Examples
///
/// Using this in an Axum router:
///
/// ```rust,no_run
/// use axum::{Router, routing::get};
/// use iron_shield::index::generate_index;
/// use iron_shield::uptime::UptimeState;
/// use std::sync::{Arc, RwLock};
/// use std::collections::HashMap;
///
/// // Assuming you have an uptime_state set up
/// let app = Router::new()
///     .route("/", get(generate_index));
/// ```
pub async fn generate_index(State(state): State<Arc<UptimeState>>) -> impl IntoResponse {
    tracing::debug!("Generating index template");

    // Get the config from the shared state
    match state.config.read() {
        Ok(config_guard) => {
            let config = config_guard.clone(); // Clone the config to avoid holding the lock
            drop(config_guard); // Explicitly drop the lock as soon as possible

            // Get current UTC time from utility function
            let current_time = utils::get_current_time_string();

            let config_for_client = ConfigUpdate {
                site_name: config.site_name.clone(),
                clock: config.clock.to_string(),
                sites: config
                    .sites
                    .iter()
                    .map(|site| SiteUpdate {
                        name: site.name.clone(),
                        url: site.url.clone(),
                        category: site.category.clone(),
                        tags: site.tags.clone(),
                    })
                    .collect(),
            };

            let config_json = match serde_json::to_string(&config_for_client) {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize config for settings modal: {e}");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to serialize configuration",
                    )
                        .into_response();
                }
            };

            let template = IndexTemplate {
                config,
                current_time,
                config_json,
            };
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
        Err(e) => {
            tracing::error!("Configuration read lock error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration read lock error",
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Site};
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use std::collections::HashMap;
    use std::sync::RwLock;
    use tokio::sync::broadcast;
    use tokio_util::sync::CancellationToken;

    /// Helper function to build a configuration for testing
    ///
    /// Creates a simple configuration with a given site name and clock format,
    /// including a single example site for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `site_name` - The name to use for the test site
    /// * `clock` - The clock format to use in the test configuration
    ///
    /// # Returns
    ///
    /// Returns a `Config` instance suitable for testing
    fn build_config(site_name: &str, clock: Clock) -> Config {
        Config {
            site_name: site_name.to_string(),
            clock,
            sites: vec![Site {
                name: "Docs".to_string(),
                url: "https://docs.example.com".to_string(),
                category: "Reference".to_string(),
                tags: vec!["docs".to_string()],
                uptime_percentage: 99.9,
            }],
        }
    }

    /// Helper function to build an uptime state for testing
    ///
    /// Creates an uptime state containing the given configuration wrapped in
    /// the necessary Arc and `RwLock` structures for sharing between threads.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to wrap in the uptime state
    ///
    /// # Returns
    ///
    /// Returns an `Arc<UptimeState>` instance suitable for testing
    fn build_state(config: Config) -> Arc<UptimeState> {
        let (shutdown_events, _) = broadcast::channel(1);
        Arc::new(UptimeState {
            config: Arc::new(RwLock::new(config)),
            history: Arc::new(RwLock::new(HashMap::new())),
            config_file_path: std::path::PathBuf::from("test-config.json5"),
            shutdown_events,
            shutdown_token: CancellationToken::new(),
        })
    }

    #[test]
    /// Test that the index template correctly renders site name and clock settings
    ///
    /// This test verifies that the template properly includes the site name in the title,
    /// respects the clock format setting, and includes the site URL in the output.
    /// It ensures that basic template functionality works as expected.
    fn index_template_renders_site_name_and_clock() {
        let config = build_config("Test Dashboard", Clock::Hour12);
        let template = IndexTemplate {
            config,
            current_time: "10:00:00 UTC".to_string(),
            config_json: "{}".to_string(),
        };

        let rendered = template
            .render()
            .expect("Template rendering should succeed in test");

        assert!(
            rendered.contains("<title>Test Dashboard</title>"),
            "rendered template should include site title"
        );
        assert!(
            rendered.contains("data-format=\"12hour\""),
            "rendered template should respect clock format"
        );
        assert!(
            rendered.contains("https://docs.example.com"),
            "rendered template should include site URL"
        );
    }

    #[tokio::test]
    /// Test that the `generate_index` function returns a proper HTML response
    ///
    /// This async test verifies that the `generate_index` function properly handles
    /// requests and returns valid HTML responses with the correct site information.
    /// It checks both the status code and content of the response.
    async fn generate_index_returns_html_response() {
        let config = build_config("Async Dashboard", Clock::Hour24);
        let state = build_state(config);

        let response = generate_index(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let (parts, body) = response.into_parts();
        let body_bytes = body
            .collect()
            .await
            .expect("Failed to collect response body")
            .to_bytes();
        let body_string =
            String::from_utf8(body_bytes.to_vec()).expect("Body should contain valid UTF-8");

        assert!(
            body_string.contains("Async Dashboard"),
            "response should include site title"
        );
        assert!(
            body_string.contains("Docs"),
            "response should include site entry"
        );
        assert_eq!(parts.status, StatusCode::OK);
    }
}
