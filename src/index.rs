use crate::config::Config;
use crate::sites::Site;
use askama_axum::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    sites: Vec<Site>,
    config: Config,
}

pub async fn generate_index() -> IndexTemplate {
    let config = Config::load();
    let sites = Site::load();

    IndexTemplate { config, sites }
}
