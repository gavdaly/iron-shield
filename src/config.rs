use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Default configuration file name
///
/// This constant defines the default name for the configuration file.
/// The application expects to find a file with this name in the root directory.
pub const CONFIG_FILE: &str = "config.json5";

/// Application configuration structure
///
/// Contains all configuration parameters for the Iron Shield dashboard.
/// This struct supports serialization and deserialization for JSON5 format
/// and provides default values for optional fields.
///
/// # Fields
///
/// * `site_name` - The name of the site displayed in the page title
/// * `clock` - The format in which to display the clock
/// * `opentelemetry_endpoint` - Optional HTTP endpoint to send uptime telemetry to
/// * `sites` - A vector of bookmarked sites to display on the dashboard
///
/// # Examples
///
/// ```
/// use iron_shield::config::{Config, Clock};
/// use iron_shield::config::Site;
///
/// let config = Config {
///     site_name: "My Dashboard".to_string(),
///     clock: Clock::Hour24,
///     opentelemetry_endpoint: None,
///     sites: vec![
///         Site {
///             name: "Google".to_string(),
///             url: "https://google.com".to_string(),
///             category: "Search".to_string(),
///             tags: vec!["important".to_string()],
///             uptime_percentage: 0.0, // Not required when initializing manually
///         }
///     ],
/// };
///
/// assert_eq!(config.site_name, "My Dashboard");
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Name of the site displayed in the page title
    #[serde(default = "default_site_name")]
    pub site_name: String,
    /// Clock format to use (24-hour, 12-hour, or no clock)
    #[serde(default)]
    pub clock: Clock,
    /// Optional endpoint to forward uptime telemetry snapshots to
    #[serde(default)]
    pub opentelemetry_endpoint: Option<String>,
    /// List of bookmarked sites to display
    #[serde(default)]
    pub sites: Vec<Site>,
}

/// Represents a bookmarked website in the dashboard
///
/// Contains the essential information for displaying and accessing a bookmarked site.
/// This struct supports serialization and deserialization for JSON5 format.
///
/// # Fields
///
/// * `name` - The display name for the site shown in the UI
/// * `url` - The URL of the site to link to
/// * `category` - An optional category for grouping sites (defaults to empty string)
/// * `tags` - A vector of tags for categorization and filtering
/// * `uptime_percentage` - The uptime percentage for display in the UI (not in config file)
///
/// # Examples
///
/// ```
/// use iron_shield::config::Site;
///
/// let site = Site {
///     name: "Google".to_string(),
///     url: "https://google.com".to_string(),
///     category: "Search Engines".to_string(),
///     tags: vec!["search".to_string(), "important".to_string()],
///     uptime_percentage: 99.9,
/// };
///
/// assert_eq!(site.name, "Google");
/// assert_eq!(site.url, "https://google.com");
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Site {
    /// Display name for the site
    pub name: String,
    /// URL of the site
    pub url: String,
    /// Category for grouping sites
    #[serde(default)]
    pub category: String,
    /// List of tags for categorization and filtering
    pub tags: Vec<String>,
    /// The uptime percentage for display in the UI (not in config file)
    /// This field is populated at runtime with data from the uptime monitoring system
    #[serde(default, skip_serializing, skip_deserializing)]
    pub uptime_percentage: f64,
}

impl Default for Config {
    /// Provides a default configuration with pre-filled values
    ///
    /// The default configuration includes:
    /// - Site name: "Iron Shield Dashboard" (using the `default_site_name` function)
    /// - Clock: `Clock::None` (no clock displayed)
    /// - Sites: An empty vector of sites
    fn default() -> Self {
        Config {
            site_name: default_site_name(),
            clock: Clock::None,
            opentelemetry_endpoint: None,
            sites: Vec::new(),
        }
    }
}

impl Config {
    /// Load the application configuration from the config.json5 file
    ///
    /// This method reads the specified configuration file and parses it as JSON5 format
    /// into a Config struct. It handles errors for file reading and parsing, providing
    /// appropriate error messages with context about the file that failed to load.
    ///
    /// # Arguments
    ///
    /// * `config_file_path` - The path to the configuration file to load
    ///
    /// # Returns
    ///
    /// Returns `Ok(Config)` if the configuration was successfully loaded and parsed,
    /// or an `IronShieldError` if an error occurs during reading or parsing.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The configuration file cannot be read (e.g., file doesn't exist, no permissions)
    /// - The configuration file contains invalid JSON5 syntax
    ///
    /// # Examples
    ///
    /// ```
    /// use iron_shield::config::Config;
    /// use std::path::PathBuf;
    ///
    /// // This would load a real config file in practice
    /// // let config_path = PathBuf::from("config.json5");
    /// // let config = Config::load(&config_path).unwrap();
    /// ```
    pub fn load(config_file_path: &PathBuf) -> crate::error::Result<Self> {
        tracing::debug!(
            "Loading application configuration from {:?}",
            config_file_path
        );
        let config_str = fs::read_to_string(config_file_path).map_err(|e| {
            crate::error::IronShieldError::Generic(format!(
                "Failed to read config file {}: {e}",
                config_file_path.display()
            ))
        })?;
        let config: Config = json5::from_str(&config_str).map_err(|e| {
            crate::error::IronShieldError::Generic(format!(
                "Failed to parse config file {}: {e}",
                config_file_path.display()
            ))
        })?;

        tracing::info!(
            "Configuration loaded successfully from {}",
            config_file_path.display()
        );
        Ok(config)
    }
}

/// Provides a default site name if not specified in config
///
/// This function returns the default site name used when the configuration doesn't
/// specify one. It's used by the serde `default` attribute on the `site_name` field.
///
/// # Returns
///
/// Returns the default site name as a String: "Iron Shield Dashboard"
fn default_site_name() -> String {
    "Iron Shield Dashboard".to_string()
}

/// Clock format options
///
/// Defines the format in which to display the time on the dashboard.
/// The enum supports serialization and deserialization for configuration persistence.
///
/// # Variants
///
/// * `None` - No clock is displayed on the dashboard
/// * `Hour24` - Time is displayed in 24-hour format (e.g., 13:00)
/// * `Hour12` - Time is displayed in 12-hour format with AM/PM (e.g., 1:00 PM)
///
/// # Examples
///
/// ```
/// use iron_shield::config::Clock;
///
/// let clock_format = Clock::Hour24;
/// match clock_format {
///     Clock::None => println!("No clock displayed"),
///     Clock::Hour24 => println!("Using 24-hour format"),
///     Clock::Hour12 => println!("Using 12-hour format"),
/// }
///
/// assert_eq!(clock_format, Clock::Hour24);
/// ```
#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Clone)]
pub enum Clock {
    /// No clock displayed
    #[default]
    None,
    /// 24-hour format (e.g., 13:00)
    Hour24,
    /// 12-hour format with AM/PM (e.g., 1:00 PM)
    Hour12,
}

impl std::fmt::Display for Clock {
    /// Formats the Clock enum to its string representation
    ///
    /// This implementation allows the Clock enum to be converted to its string representation,
    /// which is used for serialization and in API responses. The mapping is:
    /// - `Clock::Hour24` becomes "24hour"
    /// - `Clock::Hour12` becomes "12hour"
    /// - `Clock::None` becomes "none"
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write the string representation to
    ///
    /// # Returns
    ///
    /// Returns `std::fmt::Result` indicating success or failure of the formatting operation
    ///
    /// # Examples
    ///
    /// ```
    /// use iron_shield::config::Clock;
    ///
    /// assert_eq!(Clock::Hour24.to_string(), "24hour");
    /// assert_eq!(Clock::Hour12.to_string(), "12hour");
    /// assert_eq!(Clock::None.to_string(), "none");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Clock::Hour24 => f.write_str("24hour"),
            Clock::Hour12 => f.write_str("12hour"),
            Clock::None => f.write_str("none"),
        }
    }
}

/// A wrapper around Config that provides interior mutability and file watching capabilities
///
/// This struct handles loading the configuration file and automatically reloading it
/// when changes are detected. It uses Rust's `notify` crate to monitor the filesystem
/// for changes to the configuration file.
///
/// The `ConfigWatcher` maintains an Arc<`RwLock`<Config>> for thread-safe access to the
/// configuration from multiple parts of the application. When the config file changes,
/// it automatically reloads the configuration in the background.
///
/// # Fields
///
/// * `config` - The configuration wrapped in Arc<`RwLock`<>> for thread-safe access
/// * `_watcher` - The file watcher that monitors the config file for changes (kept to prevent dropping)
///
/// # Examples
///
/// ```
/// use iron_shield::config::ConfigWatcher;
/// use std::path::PathBuf;
///
/// // This would work with a real config file in practice
/// // let config_path = PathBuf::from("config.json5");
/// // let config_watcher = ConfigWatcher::new(&config_path).unwrap();
/// // let config = config_watcher.get_config();
/// ```
pub struct ConfigWatcher {
    /// The actual configuration wrapped in `RwLock` for interior mutability
    pub config: Arc<RwLock<Config>>,
    /// The file system watcher that automatically reloads config on changes
    /// Kept as a field to ensure it stays alive during the lifetime of `ConfigWatcher`
    _watcher: notify::RecommendedWatcher, // Keep watcher alive via ownership
}

impl ConfigWatcher {
    /// Create a new `ConfigWatcher` by loading the initial configuration and setting up a file watcher
    ///
    /// This function loads the configuration from the specified file and starts a file watcher
    /// to automatically reload the configuration when changes are detected. The configuration
    /// is loaded into an `RwLock` to allow safe concurrent access from multiple threads.
    ///
    /// The function spawns an asynchronous task that listens for file change events and
    /// reloads the configuration when the file is modified, created, or removed.
    ///
    /// # Arguments
    ///
    /// * `config_path` - The path to the configuration file to watch and load
    ///
    /// # Returns
    ///
    /// Returns `Ok(ConfigWatcher)` with the loaded configuration and active file watcher,
    /// or an `IronShieldError` if loading or setting up the watcher fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The configuration file cannot be read or parsed (using `Config::load`)
    /// - The file watcher cannot be created or set up to watch the specified file
    ///
    /// # Examples
    ///
    /// ```
    /// use iron_shield::config::ConfigWatcher;
    /// use std::path::PathBuf;
    ///
    /// // This would work with a real config file in practice
    /// // let config_path = PathBuf::from("config.json5");
    /// // let config_watcher = ConfigWatcher::new(&config_path).unwrap();
    /// ```
    pub fn new(config_path: &PathBuf) -> crate::error::Result<Self> {
        // Load initial configuration
        let config = Config::load(config_path)?;
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
            .map_err(|e| {
                crate::error::IronShieldError::Generic(format!(
                    "Failed to create file watcher: {e}"
                ))
            })?;

        // Add the config file to the watcher
        watcher
            .watch(config_path, RecursiveMode::NonRecursive)
            .map_err(|e| {
                crate::error::IronShieldError::Generic(format!("Failed to watch config file: {e}"))
            })?;

        info!("Started config file watcher for: {}", config_path.display());

        // Spawn a task to handle config reloads
        tokio::spawn({
            let config_inner = watcher_config;
            let mut reload_rx = rx;
            let reload_config_path = config_path.clone(); // Clone for reload task
            async move {
                loop {
                    if reload_rx.recv().await.is_some() {
                        match Config::load(&reload_config_path) {
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

        Ok(ConfigWatcher {
            config: config_rwlock,
            _watcher: watcher,
        })
    }

    /// Get a clone of the Arc<`RwLock`<Config>> for sharing with other components
    ///
    /// This method provides access to the shared configuration by returning a clone
    /// of the Arc<wrapped around the `RwLock`<Config>>. This allows multiple parts of
    /// the application to safely access and read the configuration concurrently.
    ///
    /// Since the return value is wrapped in Arc and `RwLock`, callers can use
    /// `read().unwrap()` to read the config or `write().unwrap()` to modify it
    /// (though modifications should typically go through the `ConfigWatcher` mechanism).
    ///
    /// # Returns
    ///
    /// Returns an `Arc<RwLock<Config>>` that can be shared across threads safely.
    /// The Arc allows multiple owners of the same configuration data, while the
    /// `RwLock` ensures safe concurrent access.
    ///
    /// # Examples
    ///
    /// ```
    /// use iron_shield::config::ConfigWatcher;
    /// use std::path::PathBuf;
    ///
    /// // Example of how to use the returned config
    /// // let config_path = PathBuf::from("config.json5");
    /// // let config_watcher = ConfigWatcher::new(&config_path).unwrap();
    /// // let shared_config = config_watcher.get_config();
    /// //
    /// // {
    /// //     let config_guard = shared_config.read().unwrap();
    /// //     println!("Site name: {}", config_guard.site_name);
    /// // }
    /// ```
    #[must_use]
    pub fn get_config(&self) -> Arc<RwLock<Config>> {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn default_site_name_returns_expected_value() {
        assert_eq!(default_site_name(), "Iron Shield Dashboard");
    }

    #[test]
    fn clock_display_formats_all_variants() {
        assert_eq!(Clock::Hour24.to_string(), "24hour");
        assert_eq!(Clock::Hour12.to_string(), "12hour");
        assert_eq!(Clock::None.to_string(), "none");
        assert_eq!(Clock::default(), Clock::None);
    }

    #[test]
    fn config_load_parses_complete_configuration() {
        let mut temp_file =
            NamedTempFile::new().expect("Failed to create temporary config file for test");
        write!(
            temp_file,
            r#"{{
                site_name: "Custom Site",
                clock: "Hour12",
                sites: [
                    {{
                        name: "Docs",
                        url: "https://example.com",
                        category: "Reference",
                        tags: ["docs", "reference"],
                    }},
                ],
            }}"#
        )
        .expect("Failed to write configuration contents");
        temp_file
            .flush()
            .expect("Failed to flush configuration data");

        let config = Config::load(&temp_file.path().to_path_buf())
            .expect("Expected configuration to load successfully");

        assert_eq!(config.site_name, "Custom Site");
        assert_eq!(config.clock, Clock::Hour12);
        assert_eq!(config.sites.len(), 1);
        assert_eq!(config.sites[0].name, "Docs");
        assert_eq!(config.sites[0].category, "Reference");
        assert_eq!(
            config.sites[0].tags,
            vec!["docs".to_string(), "reference".to_string()]
        );
    }

    #[test]
    fn config_load_applies_defaults_when_fields_missing() {
        let mut temp_file =
            NamedTempFile::new().expect("Failed to create temporary config file for defaults test");
        write!(
            temp_file,
            r#"{{
                sites: [
                    {{
                        name: "Example",
                        url: "https://example.org",
                        tags: [],
                    }},
                ],
            }}"#
        )
        .expect("Failed to write configuration contents");
        temp_file
            .flush()
            .expect("Failed to flush configuration data");

        let config = Config::load(&temp_file.path().to_path_buf())
            .expect("Expected configuration to load successfully");

        assert_eq!(config.site_name, default_site_name());
        assert_eq!(config.clock, Clock::None);
        assert_eq!(config.sites.len(), 1);
        assert_eq!(config.sites[0].category, "");
    }

    #[test]
    fn config_load_returns_error_for_invalid_json() {
        let mut temp_file =
            NamedTempFile::new().expect("Failed to create temporary config file for error test");
        write!(temp_file, "{{ invalid json").expect("Failed to write invalid configuration data");
        temp_file
            .flush()
            .expect("Failed to flush configuration data");

        let err = Config::load(&temp_file.path().to_path_buf()).expect_err("Expected load to fail");
        assert!(
            err.to_string().contains("Failed to parse config file"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn config_load_returns_error_for_nonexistent_file() {
        let nonexistent_path = PathBuf::from("/nonexistent/config.json5");

        let err = Config::load(&nonexistent_path).expect_err("Expected load to fail");
        assert!(
            err.to_string().contains("Failed to read config file"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn site_creation_with_all_fields() {
        let site = Site {
            name: "Test Site".to_string(),
            url: "https://test.com".to_string(),
            category: "Test Category".to_string(),
            tags: vec!["test".to_string(), "example".to_string()],
            uptime_percentage: 99.5,
        };

        assert_eq!(site.name, "Test Site");
        assert_eq!(site.url, "https://test.com");
        assert_eq!(site.category, "Test Category");
        assert_eq!(site.tags, vec!["test".to_string(), "example".to_string()]);
        assert!((site.uptime_percentage - 99.5).abs() < f64::EPSILON);
    }

    #[test]
    fn site_creation_with_default_category() {
        let site = Site {
            name: "Test Site".to_string(),
            url: "https://test.com".to_string(),
            category: String::default(), // This will use the serde default
            tags: vec![],
            uptime_percentage: Default::default(), // This uses the serde default (0.0)
        };

        assert_eq!(site.category, "");
        assert!((site.uptime_percentage - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn config_default_values() {
        let config = Config::default();

        assert_eq!(config.site_name, "Iron Shield Dashboard");
        assert_eq!(config.clock, Clock::None);
        assert!(config.sites.is_empty());
    }

    #[test]
    fn site_default_uptime_percentage() {
        // Create a blank site and check that uptime_percentage defaults to 0.0
        let site = Site {
            name: "Test Site".to_string(),
            url: "https://example.com".to_string(),
            category: "Test".to_string(),
            tags: vec![],
            uptime_percentage: Default::default(),
        };

        assert!((site.uptime_percentage - 0.0).abs() < f64::EPSILON);
    }
}
