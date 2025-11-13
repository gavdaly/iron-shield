use crate::config::Clock;
use crate::uptime::UptimeState;
use crate::utils;
use askama_axum::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use std::sync::Arc;

/// Template structure for the index page
///
/// Contains the configuration data needed to render the dashboard
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// Configuration data for the dashboard
    config: crate::config::Config,
    /// Current UTC time as a formatted string
    current_time: String,
}

/// Generates the index template with loaded configuration
///
/// # Returns
///
/// An HTML response with the index template or an error response
pub async fn generate_index(State(state): State<Arc<UptimeState>>) -> impl IntoResponse {
    tracing::debug!("Generating index template");

    // Get the config from the shared state
    match state.config.read() {
        Ok(config_guard) => {
            let config = config_guard.clone(); // Clone the config to avoid holding the lock
            drop(config_guard); // Explicitly drop the lock as soon as possible

            // Get current UTC time from utility function
            let current_time = utils::get_current_time_string();

            let template = IndexTemplate {
                config,
                current_time,
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

    fn build_config(site_name: &str, clock: Clock) -> Config {
        Config {
            site_name: site_name.to_string(),
            clock,
            sites: vec![Site {
                name: "Docs".to_string(),
                url: "https://docs.example.com".to_string(),
                category: "Reference".to_string(),
                tags: vec!["docs".to_string()],
            }],
        }
    }

    fn build_state(config: Config) -> Arc<UptimeState> {
        Arc::new(UptimeState {
            config: Arc::new(RwLock::new(config)),
            history: Arc::new(RwLock::new(HashMap::new())),
            config_file_path: std::path::PathBuf::from("test-config.json5"),
        })
    }

    #[test]
    fn index_template_renders_site_name_and_clock() {
        let config = build_config("Test Dashboard", Clock::Hour12);
        let template = IndexTemplate {
            config,
            current_time: "10:00:00 UTC".to_string(),
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
