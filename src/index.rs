use crate::config::Clock;
use crate::uptime::UptimeState;
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
            let template = IndexTemplate { config };
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
