use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Application configuration structure
///
/// Contains all configuration parameters for the Iron Shield dashboard
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Name of the site displayed in the page title
    pub site_name: String,
    /// Clock format to use (24-hour, 12-hour, or no clock)
    pub clock: Clock,
    /// List of bookmarked sites to display
    pub sites: Vec<Site>,
}

/// Represents a bookmarked website in the dashboard
///
/// Contains the essential information for displaying and accessing a bookmarked site
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Site {
    /// Display name for the site
    pub name: String,
    /// URL of the site
    pub url: String,
    /// List of tags for categorization and filtering
    pub tags: Vec<String>,
}

impl Config {
    /// Load the application configuration from the config.json5 file.
    ///
    /// # Returns
    ///
    /// Returns the loaded Config if successful, or an `IronShieldError` if an error occurs
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be read or parsed
    pub fn load() -> crate::error::Result<Self> {
        tracing::debug!("Loading application configuration");
        let config_str = fs::read_to_string("config.json5")?;

        // Deserialize to a JSON Value first to check if clock field exists
        let mut value: serde_json::Value = json5::from_str(&config_str)?;

        // If clock field is missing, set it to the default
        if value.get("clock").is_none() {
            if let Some(obj) = value.as_object_mut() {
                obj.insert(
                    "clock".to_string(),
                    serde_json::Value::String("None".to_string()),
                );
            } else {
                return Err(crate::error::IronShieldError::Generic(
                    "Config is not an object".to_string(),
                ));
            }
        }

        let config: Config = serde_json::from_value(value)?;

        tracing::info!("Configuration loaded successfully");
        Ok(config)
    }
}

/// Clock format options
///
/// Defines the format in which to display the time on the dashboard
#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Clone)]
pub enum Clock {
    /// 24-hour format (e.g., 13:00)
    Hour24,
    /// 12-hour format with AM/PM (e.g., 1:00 PM)
    Hour12,
    /// No clock displayed
    #[default]
    None,
}

impl std::fmt::Display for Clock {
    /// Formats the Clock enum to its string representation
    ///
    /// # Arguments
    ///
    /// * `f` - Formatter to write the string to
    ///
    /// # Returns
    ///
    /// Result of the formatting operation
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Clock::Hour24 => f.write_str("24hour"),
            Clock::Hour12 => f.write_str("12hour"),
            Clock::None => f.write_str("none"),
        }
    }
}

/// A wrapper around Config that provides interior mutability and file watching capabilities
pub struct ConfigWatcher {
    /// The actual configuration wrapped in `RwLock` for interior mutability
    pub config: Arc<RwLock<Config>>,
}

impl ConfigWatcher {
    /// Create a new `ConfigWatcher` by loading the initial configuration and setting up a file watcher
    ///
    /// # Errors
    ///
    /// This function will return an error if the configuration file cannot be read or parsed,
    /// or if the file watcher cannot be set up.
    pub fn new(config_path: &PathBuf) -> crate::error::Result<Self> {
        // Load initial configuration
        let config = Config::load()?;
        let config_rwlock = Arc::new(RwLock::new(config));

        // Create the config watcher
        let watcher_config = config_rwlock.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        let config_path_for_watcher = config_path.clone();

        let mut watcher =
            recommended_watcher(move |res: std::result::Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        match event.kind {
                            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                                for path in event.paths {
                                    if path == config_path_for_watcher {
                                        debug!("Configuration file change detected: {:?}", path);
                                        if tx.send(()).is_err() {
                                            error!("Failed to send config reload signal");
                                        }
                                        break;
                                    }
                                }
                            }
                            _ => {} // Ignore other event types
                        }
                    }
                    Err(e) => error!("Watch error: {:?}", e),
                }
            })
            .map_err(|e| crate::error::IronShieldError::Generic(e.to_string()))?;

        // Add the config file to the watcher
        watcher
            .watch(config_path, RecursiveMode::NonRecursive)
            .map_err(|e| crate::error::IronShieldError::Generic(e.to_string()))?;

        info!("Started config file watcher for: {:?}", config_path);

        // Spawn a task to handle config reloads
        tokio::spawn({
            let config_inner = watcher_config;
            let mut reload_rx = rx;
            async move {
                loop {
                    if reload_rx.recv().await.is_some() {
                        match Config::load() {
                            Ok(new_config) => {
                                let number_of_sites = new_config.sites.len();
                                info!("Reloading configuration with {number_of_sites} sites");

                                {
                                    if let Ok(mut config_guard) = config_inner.write() {
                                        *config_guard = new_config;
                                        info!("Configuration updated successfully");
                                    } else {
                                        error!("Failed to acquire config write lock");
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to reload configuration: {e}");
                            }
                        }
                    }
                }
            }
        });

        // Since the watcher needs to be kept alive for the duration of the ConfigWatcher
        std::mem::forget(watcher);

        Ok(ConfigWatcher {
            config: config_rwlock,
        })
    }

    /// Get a clone of the Arc<`RwLock`<Config>> for sharing with other components
    #[must_use]
    pub fn get_config(&self) -> Arc<RwLock<Config>> {
        self.config.clone()
    }
}
