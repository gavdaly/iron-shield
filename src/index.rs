use crate::config::Config;
use crate::sites::Site;
use askama_axum::Template;
use tracing;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    sites: Vec<Site>,
    config: Config,
}

pub async fn generate_index() -> IndexTemplate {
    tracing::debug!("Generating index template");
    let config = Config::load();
    let sites = Site::load();

    tracing::info!("Index template generated with {} sites and config", sites.len());
    IndexTemplate { config, sites }
}
