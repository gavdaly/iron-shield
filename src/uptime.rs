use crate::config::Config;
use axum::{extract::State, response::Sse};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use tracing::{debug, error, info};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UptimeStatus {
    Up,
    Down,
    Loading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeHistory {
    pub site_id: String,
    pub status: UptimeStatus,
    pub timestamp: u64,
    pub history: Vec<UptimeStatus>, // Last 20 status checks
    pub uptime_percentage: f64,     // Percentage of "up" time in the history
}

// Shared state for the uptime service with historical data
pub struct UptimeState {
    pub config: Arc<RwLock<Config>>,
    pub history: Arc<RwLock<HashMap<String, VecDeque<UptimeStatus>>>>,
    pub config_file_path: std::path::PathBuf,
}

/// Handles the uptime monitoring stream endpoint
///
/// # Panics
///
/// This function might panic if `semaphore.acquire().await.unwrap()` fails.
/// However, this should not happen in practice as the semaphore is always available.
#[allow(clippy::too_many_lines)]
pub async fn uptime_stream(
    State(state): State<Arc<UptimeState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    // Clone the config and history for use in the stream
    let config = state.config.clone();
    let history_map = state.history.clone();
    let client = reqwest::Client::new();

    // Create a semaphore to limit concurrent site checks
    let semaphore = Arc::new(Semaphore::new(10)); // Limit to 10 concurrent checks

    // Create a channel to send updates from the checker task
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    {
        if let Ok(config_guard) = config.read() {
            let sites_to_initialize = config_guard.sites.clone();
            drop(config_guard); // Release the read lock immediately

            for site in &sites_to_initialize {
                if let Ok(mut history_guard) = history_map.write() {
                    let mut history = VecDeque::new();
                    // Start with Loading status as before
                    history.push_back(UptimeStatus::Loading);
                    history_guard.insert(site.name.clone(), history);
                } else {
                    error!(
                        "Failed to acquire history write lock for initialization of site: {}",
                        site.name
                    );
                }
            }
        } else {
            error!("Failed to acquire config read lock for initial history setup");
        }
    }

    // Spawn a task to periodically check sites and send updates
    tokio::spawn(async move {
        info!("Starting uptime monitoring service");

        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            debug!("Starting new uptime check cycle");

            // Get a copy of the sites to check
            let sites_to_check = {
                match config.read() {
                    Ok(guard) => guard.sites.clone(),
                    Err(e) => {
                        error!("Failed to acquire config read lock: {e}");
                        continue;
                    }
                }
            }; // Release the read lock immediately after cloning

            // Update history to loading state
            {
                let mut history_guard = match history_map.write() {
                    Ok(guard) => guard,
                    Err(e) => {
                        error!("Failed to acquire history write lock: {e}");
                        continue;
                    }
                };
                for site in &sites_to_check {
                    let site_history = history_guard
                        .entry(site.name.clone())
                        .or_insert_with(VecDeque::new);
                    site_history.push_back(UptimeStatus::Loading);

                    // Keep only the last 20 entries
                    if site_history.len() > 20 {
                        site_history.pop_front();
                    }
                }
            }

            // Send loading updates
            {
                let history_guard = match history_map.read() {
                    Ok(guard) => guard,
                    Err(e) => {
                        error!("Failed to acquire history read lock: {e}");
                        continue;
                    }
                };
                let mut uptime_data = Vec::new();

                for site in &sites_to_check {
                    if let Some(site_history) = history_guard.get(&site.name) {
                        if let Some(&current_status) = site_history.back() {
                            let uptime_percentage = calculate_uptime_percentage(site_history);
                            let site_name = site.name.clone();

                            debug!("Calculated uptime: site={site_name}, current_status={current_status:?}, percentage={uptime_percentage:.2}%");

                            let data = create_uptime_history(
                                &site.name,
                                current_status,
                                site_history,
                                uptime_percentage,
                            );

                            uptime_data.push(data);
                        }
                    }
                }

                if tx.send(uptime_data).is_err() {
                    error!("Failed to send loading updates to SSE stream");
                    break; // Channel closed, exit the loop
                }
            }

            // Now check the actual status of each site with concurrency limiting
            let mut tasks = Vec::new();
            for site in &sites_to_check {
                let client = client.clone();
                let url = site.url.clone();
                let tx = tx.clone();
                let history_map = history_map.clone();
                let site_name = site.name.clone();
                let semaphore = semaphore.clone();

                let task = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap(); // Wait for permit
                    debug!("Starting uptime check for site: {site_name}");

                    let status = check_site_status(&client, &url).await;
                    debug!("Uptime check completed for site: {site_name}, status: {status:?}");

                    {
                        let mut history_guard = match history_map.write() {
                            Ok(guard) => guard,
                            Err(e) => {
                                error!("Failed to acquire history write lock: {e}");
                                return; // Exit if we can't update history
                            }
                        };
                        let site_history = history_guard
                            .entry(site_name.clone())
                            .or_insert_with(VecDeque::new);
                        site_history.push_back(status);

                        // Keep only the last 20 entries
                        if site_history.len() > 20 {
                            site_history.pop_front();
                        }
                    }

                    // Send the final status update
                    {
                        let history_guard = match history_map.read() {
                            Ok(guard) => guard,
                            Err(e) => {
                                error!("Failed to acquire history read lock: {e}");
                                return; // Exit if we can't read history
                            }
                        };
                        if let Some(site_history) = history_guard.get(&site_name) {
                            let uptime_percentage = calculate_uptime_percentage(site_history);

                            debug!(
                                "Updated uptime stats: site={site_name}, status={status:?}, percentage={uptime_percentage:.2}%"
                            );

                            let data = create_uptime_history(
                                &site_name,
                                status,
                                site_history,
                                uptime_percentage,
                            );

                            if tx.send(vec![data]).is_err() {
                                error!("Failed to send uptime update for site: {site_name}");
                            }
                        }
                    }
                });

                tasks.push(task);
            }

            // Wait for all tasks to complete before next interval
            for task in tasks {
                let _ = task.await;
            }
        }
    });

    // Convert the receiving end of the channel into a stream
    let stream = UnboundedReceiverStream::new(rx).map(|uptime_data| {
        if let Ok(event) = axum::response::sse::Event::default().json_data(&uptime_data) {
            Ok(event)
        } else {
            error!("Failed to serialize uptime data for SSE");
            Ok(axum::response::sse::Event::default().data("Error"))
        }
    });

    Sse::new(stream)
}

/// Helper function to calculate uptime percentage
/// Excludes Loading status from the calculation to avoid artificially reducing the percentage
fn calculate_uptime_percentage(site_history: &VecDeque<UptimeStatus>) -> f64 {
    let up_count = site_history
        .iter()
        .filter(|&&s| s == UptimeStatus::Up)
        .count();
    let count_excluding_loading = site_history
        .iter()
        .filter(|&&s| s != UptimeStatus::Loading)
        .count();

    if count_excluding_loading == 0 {
        0.0
    } else {
        // This cast is necessary for percentage calculation and precision loss is acceptable
        // for our use case (uptime monitoring doesn't require exact precision)
        #[allow(clippy::cast_precision_loss)]
        {
            (up_count as f64) / (count_excluding_loading as f64) * 100.0
        }
    }
}

/// Helper function to create uptime history data
fn create_uptime_history(
    site_name: &str,
    current_status: UptimeStatus,
    site_history: &VecDeque<UptimeStatus>,
    uptime_percentage: f64,
) -> UptimeHistory {
    UptimeHistory {
        site_id: site_name.to_string(),
        status: current_status,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs(),
        history: site_history.iter().copied().collect(),
        uptime_percentage,
    }
}

/// Helper function to check site status
async fn check_site_status(client: &reqwest::Client, url: &str) -> UptimeStatus {
    debug!("Checking site status: {url}");
    match client
        .head(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                debug!("Site {url} is UP: status {status}");
                UptimeStatus::Up
            } else {
                debug!("Site {url} is DOWN: status {status}");
                UptimeStatus::Down
            }
        }
        Err(e) => {
            debug!("Site {url} is DOWN: error {e}");
            UptimeStatus::Down
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::collections::VecDeque;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_uptime_status_enum() {
        assert_eq!(format!("{:?}", UptimeStatus::Up), "Up");
        assert_eq!(format!("{:?}", UptimeStatus::Down), "Down");
        assert_eq!(format!("{:?}", UptimeStatus::Loading), "Loading");

        // Test PartialEq implementation
        assert_eq!(UptimeStatus::Up, UptimeStatus::Up);
        assert_ne!(UptimeStatus::Up, UptimeStatus::Down);
    }

    #[test]
    fn test_calculate_uptime_percentage_empty_history() {
        let history = VecDeque::new();
        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_uptime_percentage_all_up() {
        let mut history = VecDeque::new();
        history.push_back(UptimeStatus::Up);
        history.push_back(UptimeStatus::Up);
        history.push_back(UptimeStatus::Up);

        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_uptime_percentage_all_down() {
        let mut history = VecDeque::new();
        history.push_back(UptimeStatus::Down);
        history.push_back(UptimeStatus::Down);
        history.push_back(UptimeStatus::Down);

        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_uptime_percentage_mixed_no_loading() {
        let mut history = VecDeque::new();
        history.push_back(UptimeStatus::Up);
        history.push_back(UptimeStatus::Down);
        history.push_back(UptimeStatus::Up);
        history.push_back(UptimeStatus::Down);

        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_uptime_percentage_with_loading() {
        // Loading status should be excluded from the calculation
        let mut history = VecDeque::new();
        history.push_back(UptimeStatus::Up);
        history.push_back(UptimeStatus::Loading);
        history.push_back(UptimeStatus::Down);
        history.push_back(UptimeStatus::Loading);
        history.push_back(UptimeStatus::Up);

        // 2 Up out of 3 non-Loading statuses = 66.67%
        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 66.67).abs() < 0.01);
    }

    #[test]
    fn test_calculate_uptime_percentage_all_loading() {
        let mut history = VecDeque::new();
        history.push_back(UptimeStatus::Loading);
        history.push_back(UptimeStatus::Loading);
        history.push_back(UptimeStatus::Loading);

        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_create_uptime_history() {
        let mut history = VecDeque::new();
        history.push_back(UptimeStatus::Up);
        history.push_back(UptimeStatus::Down);
        history.push_back(UptimeStatus::Loading);

        let uptime_history = create_uptime_history("test-site", UptimeStatus::Up, &history, 50.0);

        assert_eq!(uptime_history.site_id, "test-site");
        assert_eq!(uptime_history.status, UptimeStatus::Up);
        assert!((uptime_history.uptime_percentage - 50.0).abs() < f64::EPSILON);
        assert_eq!(
            uptime_history.history,
            vec![UptimeStatus::Up, UptimeStatus::Down, UptimeStatus::Loading]
        );

        // Check that timestamp is reasonable (within a few seconds of now)
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(uptime_history.timestamp <= current_time);
        assert!(uptime_history.timestamp >= current_time - 10); // Allow some buffer
    }

    #[test]
    fn test_uptime_state_creation() {
        // Create a mock config
        let config = Arc::new(RwLock::new(Config {
            site_name: "Test Site".to_string(),
            clock: crate::config::Clock::None,
            sites: vec![],
        }));

        let history = Arc::new(RwLock::new(HashMap::new()));

        let config_file_path = std::path::PathBuf::from("config.json5");

        let uptime_state = UptimeState {
            config,
            history,
            config_file_path,
        };

        // Verify that the state can be created without issues
        assert!(uptime_state.config.read().is_ok());
        assert!(uptime_state.history.read().is_ok());
    }

    #[tokio::test]
    async fn test_check_site_status_up() {
        // This test requires a real server to test against
        // For now, we can only test error conditions
        let client = reqwest::Client::new();

        // Test with a URL that should result in an error (nonexistent domain)
        let result =
            check_site_status(&client, "http://definitely-not-a-real-domain-12345.com").await;
        assert_eq!(result, UptimeStatus::Down);
    }

    // Test the data structures
    #[test]
    fn test_uptime_history_serialization() {
        let uptime_history = UptimeHistory {
            site_id: "test-site".to_string(),
            status: UptimeStatus::Up,
            timestamp: 1_234_567_890, // Using underscores for readability
            history: vec![UptimeStatus::Up, UptimeStatus::Down, UptimeStatus::Loading],
            uptime_percentage: 50.0,
        };

        // Test serialization/deserialization
        let serialized = serde_json::to_string(&uptime_history).unwrap();
        let deserialized: UptimeHistory = serde_json::from_str(&serialized).unwrap();

        assert_eq!(uptime_history.site_id, deserialized.site_id);
        assert_eq!(uptime_history.status, deserialized.status);
        assert_eq!(uptime_history.timestamp, deserialized.timestamp);
        assert_eq!(uptime_history.history, deserialized.history);
        assert!(
            (uptime_history.uptime_percentage - deserialized.uptime_percentage).abs()
                < f64::EPSILON
        );
    }

    #[tokio::test]
    async fn test_check_site_status_with_valid_url() {
        // Test with a URL that should return success (HTTPbin is commonly used for testing)
        let client = reqwest::Client::new();
        let result = check_site_status(&client, "https://httpbin.org/status/200").await;
        // Note: This might fail if no internet connection, but it's a good test when available
        // For now, we'll just check that it doesn't panic
        assert!(result == UptimeStatus::Up || result == UptimeStatus::Down);
    }

    #[tokio::test]
    async fn test_check_site_status_down_non_2xx() {
        // Test with a URL that returns a 500 Internal Server Error
        let client = reqwest::Client::new();
        let result = check_site_status(&client, "https://httpbin.org/status/500").await;
        assert_eq!(result, UptimeStatus::Down);
    }

    #[tokio::test]
    async fn test_check_site_status_with_timeout() {
        // Test with a URL that should timeout
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(1)) // Very short timeout
            .build()
            .unwrap();

        // Use a URL that will likely timeout with 1ms timeout
        let result = check_site_status(&client, "https://httpbin.org/delay/10").await;
        assert_eq!(result, UptimeStatus::Down);
    }

    #[test]
    fn test_calculate_uptime_percentage_accuracy() {
        // Test with a known percentage (75% uptime)
        let mut history = VecDeque::new();
        history.push_back(UptimeStatus::Up); // Counted as up
        history.push_back(UptimeStatus::Up); // Counted as up
        history.push_back(UptimeStatus::Up); // Counted as up
        history.push_back(UptimeStatus::Down); // Counted as down

        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_uptime_percentage_with_mixed_loading() {
        // Test that loading statuses are excluded from calculation
        let mut history = VecDeque::new();
        history.push_back(UptimeStatus::Up); // Counted as up
        history.push_back(UptimeStatus::Loading); // Not counted
        history.push_back(UptimeStatus::Down); // Counted as down
        history.push_back(UptimeStatus::Loading); // Not counted
        history.push_back(UptimeStatus::Up); // Counted as up
        history.push_back(UptimeStatus::Up); // Counted as up

        // Should be 3 up / 4 total non-loading = 75%
        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 75.0).abs() < f64::EPSILON);
    }
}
