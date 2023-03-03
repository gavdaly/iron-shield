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

impl std::fmt::Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Clock::*;
        match self {
            None => f.write_str("none"),
            Military => f.write_str("military"),
            Standard => f.write_str("standard"),
        }
    }
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
