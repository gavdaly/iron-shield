use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use http_body_util::BodyExt; // For .collect()
use iron_shield::config::{Clock, Config, Site, DEFAULT_MONITOR_INTERVAL_SECS};
use iron_shield::error::IronShieldError;
use iron_shield::settings::{ConfigUpdate, SiteUpdate};
use iron_shield::uptime::UptimeState;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock; // Use std::sync::RwLock
use tempfile::tempdir;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_config_update_validate_valid() {
    let config_update = ConfigUpdate {
        site_name: "Test Site".to_string(),
        clock: "24hour".to_string(),
        opentelemetry_endpoint: None,
        sites: vec![
            SiteUpdate {
                name: "Google".to_string(),
                url: "https://www.google.com".to_string(),
                category: "Search".to_string(),
                tags: vec!["web".to_string()],
                monitor_interval_secs: DEFAULT_MONITOR_INTERVAL_SECS,
                disabled: false,
            },
            SiteUpdate {
                name: "Rust-lang".to_string(),
                url: "https://www.rust-lang.org".to_string(),
                category: "Programming".to_string(),
                tags: vec!["dev".to_string(), "oss".to_string()],
                monitor_interval_secs: DEFAULT_MONITOR_INTERVAL_SECS,
                disabled: false,
            },
        ],
    };

    assert!(config_update.validate().is_ok());
}

#[tokio::test]
async fn test_config_update_validate_empty_site_name() {
    let config_update = ConfigUpdate {
        site_name: String::new(),
        clock: "24hour".to_string(),
        opentelemetry_endpoint: None,
        sites: vec![],
    };

    let err = config_update.validate().unwrap_err();
    assert_eq!(
        err.to_string(),
        IronShieldError::from("Site name cannot be empty").to_string()
    );
}

#[tokio::test]
async fn test_config_update_validate_invalid_clock_format() {
    let config_update = ConfigUpdate {
        site_name: "Test Site".to_string(),
        clock: "invalid".to_string(),
        opentelemetry_endpoint: None,
        sites: vec![],
    };

    let err = config_update.validate().unwrap_err();
    assert_eq!(
        err.to_string(),
        IronShieldError::from("Invalid clock format").to_string()
    );
}

#[tokio::test]
async fn test_config_update_validate_empty_site_entry_name() {
    let config_update = ConfigUpdate {
        site_name: "Test Site".to_string(),
        clock: "24hour".to_string(),
        opentelemetry_endpoint: None,
        sites: vec![SiteUpdate {
            name: String::new(),
            url: "https://www.google.com".to_string(),
            category: "Search".to_string(),
            tags: vec![],
            monitor_interval_secs: DEFAULT_MONITOR_INTERVAL_SECS,
            disabled: false,
        }],
    };

    let err = config_update.validate().unwrap_err();
    assert_eq!(
        err.to_string(),
        IronShieldError::from("Site name cannot be empty").to_string()
    );
}

#[tokio::test]
async fn test_config_update_validate_empty_site_entry_url() {
    let config_update = ConfigUpdate {
        site_name: "Test Site".to_string(),
        clock: "24hour".to_string(),
        opentelemetry_endpoint: None,
        sites: vec![SiteUpdate {
            name: "Google".to_string(),
            url: String::new(),
            category: "Search".to_string(),
            tags: vec![],
            monitor_interval_secs: DEFAULT_MONITOR_INTERVAL_SECS,
            disabled: false,
        }],
    };

    let err = config_update.validate().unwrap_err();
    assert_eq!(
        err.to_string(),
        IronShieldError::from("Site URL cannot be empty").to_string()
    );
}

#[tokio::test]
async fn test_config_update_validate_invalid_site_entry_url() {
    let config_update = ConfigUpdate {
        site_name: "Test Site".to_string(),
        clock: "24hour".to_string(),
        opentelemetry_endpoint: None,
        sites: vec![SiteUpdate {
            name: "Google".to_string(),
            url: "invalid-url".to_string(),
            category: "Search".to_string(),
            tags: vec![],
            monitor_interval_secs: DEFAULT_MONITOR_INTERVAL_SECS,
            disabled: false,
        }],
    };

    let err = config_update.validate().unwrap_err();
    assert_eq!(
        err.to_string(),
        IronShieldError::from("Invalid URL format: invalid-url").to_string()
    );
}

// Helper function to create a test UptimeState
fn create_test_uptime_state(config_file_path: PathBuf) -> Arc<UptimeState> {
    let config = Config {
        site_name: "Initial Site".to_string(),
        clock: Clock::Hour24,
        opentelemetry_endpoint: None,
        sites: vec![],
    };
    let (shutdown_events, _) = broadcast::channel(1);
    Arc::new(UptimeState {
        config: Arc::new(RwLock::new(config)), // Use std::sync::RwLock
        history: Arc::new(RwLock::new(std::collections::HashMap::new())),
        config_file_path,
        shutdown_events,
        shutdown_token: CancellationToken::new(),
    })
}

#[tokio::test]
async fn test_save_config_success() {
    let temp_dir =
        tempdir().expect("Failed to create temporary directory for test_save_config_success");
    let temp_config_path = temp_dir.path().join("config.json5");

    // Initialize the config file with some content
    fs::write(&temp_config_path, "{}")
        .expect("Failed to write initial config file in test_save_config_success");

    let state = create_test_uptime_state(temp_config_path.clone());

    let payload = ConfigUpdate {
        site_name: "Updated Site Name".to_string(),
        clock: "12hour".to_string(),
        opentelemetry_endpoint: None,
        sites: vec![SiteUpdate {
            name: "New Site".to_string(),
            url: "https://new.example.com".to_string(),
            category: "Test".to_string(),
            tags: vec!["new".to_string()],
            monitor_interval_secs: DEFAULT_MONITOR_INTERVAL_SECS,
            disabled: false,
        }],
    };

    let response =
        iron_shield::settings::save_config(State(state.clone()), Json(payload.clone())).await;
    let (parts, body) = response.into_response().into_parts();
    let body_bytes = body
        .collect()
        .await
        .expect("Failed to collect response body in save_config test")
        .to_bytes();
    let body_string = String::from_utf8(body_bytes.to_vec())
        .expect("Failed to convert response body to string in save_config test");

    assert_eq!(parts.status, StatusCode::OK);
    assert_eq!(body_string, "Configuration saved successfully");

    // Verify file content
    let file_content = fs::read_to_string(&temp_config_path)
        .expect("Failed to read updated config file in test_save_config_success");
    let expected_config = Config {
        site_name: "Updated Site Name".to_string(),
        clock: Clock::Hour12,
        opentelemetry_endpoint: None,
        sites: vec![Site {
            name: "New Site".to_string(),
            url: "https://new.example.com".to_string(),
            category: "Test".to_string(),
            tags: vec!["new".to_string()],
            monitor_interval_secs: DEFAULT_MONITOR_INTERVAL_SECS,
            disabled: false,
            uptime_percentage: 0.0,
        }],
    };
    let expected_json = json5::to_string(&expected_config)
        .expect("Failed to serialize expected config to JSON5 in test_save_config_success");
    assert_eq!(file_content, expected_json);

    // Verify in-memory config update
    let config_guard = state
        .config
        .read()
        .expect("Failed to acquire config read lock in test_save_config_success"); // Removed .await
    assert_eq!(config_guard.site_name, "Updated Site Name");
    assert_eq!(config_guard.clock, Clock::Hour12);
    assert_eq!(config_guard.sites.len(), 1);
    assert_eq!(config_guard.sites[0].name, "New Site");
}

#[tokio::test]
async fn test_save_config_invalid_payload() {
    let temp_dir = tempdir()
        .expect("Failed to create temporary directory for test_save_config_invalid_payload");
    let temp_config_path = temp_dir.path().join("config.json5");
    fs::write(&temp_config_path, "{}")
        .expect("Failed to write initial config file in test_save_config_invalid_payload");

    let state = create_test_uptime_state(temp_config_path.clone());

    let payload = ConfigUpdate {
        site_name: String::new(), // Invalid site name
        clock: "24hour".to_string(),
        opentelemetry_endpoint: None,
        sites: vec![],
    };

    let response =
        iron_shield::settings::save_config(State(state.clone()), Json(payload.clone())).await;
    let (parts, body) = response.into_response().into_parts();
    let body_bytes = body
        .collect()
        .await
        .expect("Failed to collect response body in save_config_invalid_payload test")
        .to_bytes();
    let body_string = String::from_utf8(body_bytes.to_vec())
        .expect("Failed to convert response body to string in save_config_invalid_payload test");

    assert_eq!(parts.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body_string, "Error: Site name cannot be empty");

    // Verify file content is unchanged (or still initial empty json)
    let file_content = fs::read_to_string(&temp_config_path)
        .expect("Failed to read config file in test_save_config_invalid_payload");
    assert_eq!(file_content, "{}");

    // Verify in-memory config is unchanged
    let config_guard = state
        .config
        .read()
        .expect("Failed to acquire config read lock in test_save_config_invalid_payload"); // Removed .await
    assert_eq!(config_guard.site_name, "Initial Site");
}
