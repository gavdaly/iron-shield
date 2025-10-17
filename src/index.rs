use crate::config::{Clock, Config};
use askama_axum::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

/// Template structure for the index page
///
/// Contains the configuration data needed to render the dashboard
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    /// Configuration data for the dashboard
    config: Config,
}

/// Generates the index template with loaded configuration
///
/// # Returns
///
/// An HTML response with the index template or an error response
pub async fn generate_index() -> impl IntoResponse {
    tracing::debug!("Generating index template");

    match Config::load() {
        Ok(config) => {
            let template = IndexTemplate { config };
            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    tracing::error!("Template rendering error: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Template rendering error",
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Configuration loading error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration loading error",
            )
                .into_response()
        }
    }
}
