use serde::{Deserialize, Serialize};
use std::fs;
use tracing;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub site_name: String,
    pub clock: Option<Clock>,
    pub sites: Vec<Site>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Site {
    pub name: String,
    pub url: String,
    pub tags: Vec<String>,
}

impl Config {
    pub fn load() -> Self {
        tracing::debug!("Loading application configuration");
        let config_str = fs::read_to_string("config.json5").expect("Failed to read config.json5");
        let config: Config = json5::from_str(&config_str).expect("Failed to parse config.json5");

        tracing::info!("Configuration loaded successfully");
        config
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum Clock {
    Military,
    #[default]
    Standard,
}

impl std::fmt::Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Clock::*;
        match self {
            Military => f.write_str("military"),
            Standard => f.write_str("standard"),
        }
    }
}
