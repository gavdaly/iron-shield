use crate::config::Config;
use askama_axum::Template;
use tracing;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    config: Config,
}

pub async fn generate_index() -> IndexTemplate {
    tracing::debug!("Generating index template");
    let config = Config::load();

    tracing::info!("Index template generated with config");
    IndexTemplate { config }
}
