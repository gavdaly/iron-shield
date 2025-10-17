use crate::config::Config;
use askama_axum::Template;

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
/// An `IndexTemplate` instance with the loaded configuration
pub async fn generate_index() -> IndexTemplate {
    tracing::debug!("Generating index template");
    let config = Config::load();

    tracing::info!("Index template generated with config");
    IndexTemplate { config }
}
