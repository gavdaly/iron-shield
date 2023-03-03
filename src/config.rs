use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub site_name: String,
    pub clock: Clock,
    pub search_engines: Option<Vec<SearchEngine>>,
    pub weather: Option<Weather>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum Clock {
    None,
    Military,
    #[default]
    Standard,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Weather {
    pub api: String,
    pub lat: f32,
    pub lng: f32,
    pub metric: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SearchEngine {
    pub name: String,
    pub url: String,
    pub icon: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            site_name: "test".to_string(),
            ..Default::default()
        }
    }
}
