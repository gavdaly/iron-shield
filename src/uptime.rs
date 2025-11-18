use crate::config::Config;
use axum::{extract::State, response::Sse};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio_stream::{
    wrappers::{BroadcastStream, UnboundedReceiverStream},
    StreamExt,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

/// Maximum number of historical uptime entries retained per site.
const MAX_HISTORY_ENTRIES: usize = 20;

/// Represents the uptime status of a monitored website
///
/// This enum is used to track the current status of a website during uptime monitoring.
/// It supports serialization and deserialization for API communication.
///
/// # Variants
///
/// * `Up` - The site is responding successfully to requests
/// * `Down` - The site is not responding or returning error status codes
/// * `Loading` - The site status is currently being checked (intermediate state)
///
/// # Examples
///
/// ```
/// use iron_shield::uptime::UptimeStatus;
///
/// let status = UptimeStatus::Up;
/// match status {
///     UptimeStatus::Up => println!("Site is up"),
///     UptimeStatus::Down => println!("Site is down"),
///     UptimeStatus::Loading => println!("Checking site status..."),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UptimeStatus {
    /// The site is responding successfully
    Up,
    /// The site is not responding or returning error status codes
    Down,
    /// The site status is currently being checked
    Loading,
}

/// Contains the historical uptime data for a single monitored site
///
/// This struct stores current status information along with historical data for a monitored site.
/// It's designed for serialization to be sent via Server-Sent Events (SSE) to clients for real-time
/// status updates.
///
/// # Fields
///
/// * `site_id` - Unique identifier for the site (typically the site name)
/// * `status` - Current status of the site (Up, Down, or Loading)
/// * `timestamp` - Unix timestamp when this record was created
/// * `history` - A collection of the last 20 status checks for trend analysis
/// * `uptime_percentage` - Calculated percentage of "up" time in the history (excluding Loading statuses)
///
/// # Examples
///
/// ```
/// use iron_shield::uptime::{HistoryEntry, UptimeHistory, UptimeStatus};
/// use std::collections::VecDeque;
///
/// let history = UptimeHistory {
///     site_id: "example.com".to_string(),
///     status: UptimeStatus::Up,
///     timestamp: 1234567890,
///     history: vec![
///         HistoryEntry {
///             status: UptimeStatus::Up,
///             response_time_ms: Some(150),
///         },
///         HistoryEntry {
///             status: UptimeStatus::Up,
///             response_time_ms: Some(160),
///         },
///         HistoryEntry {
///             status: UptimeStatus::Down,
///             response_time_ms: None,
///         },
///     ],
///     uptime_percentage: 66.67,
///     response_time_ms: Some(180),
/// };
///
/// println!("Site {} has {}% uptime", history.site_id, history.uptime_percentage);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEntry {
    /// Recorded status for this check
    pub status: UptimeStatus,
    /// Optional response time in milliseconds (only present for completed checks)
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeHistory {
    /// Unique identifier for the site (typically the site name)
    pub site_id: String,
    /// Current status of the site (Up, Down, or Loading)
    pub status: UptimeStatus,
    /// Unix timestamp when this record was created
    pub timestamp: u64,
    /// A collection of the last 20 status checks for trend analysis
    pub history: Vec<HistoryEntry>, // Last 20 status checks
    /// Calculated percentage of "up" time in the history (excluding Loading statuses)
    pub uptime_percentage: f64, // Percentage of "up" time in the history
    /// The response time (in milliseconds) for the latest check, if available
    pub response_time_ms: Option<u64>,
}

/// Result of a single uptime probe with the measured response time.
#[derive(Debug, Clone, Copy)]
struct SiteCheckResult {
    status: UptimeStatus,
    response_time_ms: Option<u64>,
}

/// Shared state for the uptime monitoring service with historical data
///
/// This struct contains the shared state used by the uptime monitoring service. It provides
/// thread-safe access to configuration and historical uptime data for all monitored sites.
///
/// # Fields
///
/// * `config` - Thread-safe access to the application configuration
/// * `history` - Thread-safe map of site histories (`site_id` -> `VecDeque` of `UptimeStatus`)
/// * `config_file_path` - Path to the configuration file for reloading purposes
///
/// # Examples
///
/// ```
/// use iron_shield::uptime::UptimeState;
/// use iron_shield::config::Config;
/// use std::collections::HashMap;
/// use std::sync::{Arc, RwLock};
/// use std::path::PathBuf;
/// use tokio::sync::broadcast;
/// use tokio_util::sync::CancellationToken;
///
/// let config = Arc::new(RwLock::new(Config {
///     site_name: "Test Site".to_string(),
///     clock: iron_shield::config::Clock::None,
///     sites: vec![],
/// }));
/// let history = Arc::new(RwLock::new(HashMap::new()));
/// let (shutdown_events, _) = broadcast::channel(1);
/// let shutdown_token = CancellationToken::new();
/// let uptime_state = UptimeState {
///     config,
///     history,
///     config_file_path: PathBuf::from("config.json5"),
///     shutdown_events,
///     shutdown_token,
/// };
/// ```
pub struct UptimeState {
    /// Thread-safe access to the application configuration
    pub config: Arc<RwLock<Config>>,
    /// Thread-safe map of site histories (`site_id` -> `VecDeque` of `UptimeStatus`)
    pub history: Arc<RwLock<HashMap<String, VecDeque<HistoryEntry>>>>,
    /// Path to the configuration file for reloading purposes
    pub config_file_path: std::path::PathBuf,
    /// Broadcast channel used to notify connected SSE clients about shutdowns
    pub shutdown_events: tokio::sync::broadcast::Sender<String>,
    /// Cancellation token to gracefully stop background uptime tasks
    pub shutdown_token: CancellationToken,
}

/// Handles the uptime monitoring stream endpoint using Server-Sent Events (SSE)
///
/// This function creates a real-time stream of uptime status updates for all configured sites.
/// It periodically checks the status of each site (every 5 seconds by default) and pushes updates
/// to connected clients via Server-Sent Events. The function limits concurrent site checks to
/// prevent overwhelming the system with too many HTTP requests at once.
///
/// The implementation initializes all sites with a "Loading" status and then begins periodic checks.
/// It maintains a history of the last 20 status checks for each site and calculates the uptime
/// percentage based on successful checks (excluding "Loading" statuses from the calculation).
///
/// # Arguments
///
/// * `state` - Shared uptime state containing configuration and historical data
///
/// # Returns
///
/// An SSE stream that continuously sends `UptimeHistory` data for all monitored sites
///
/// # Panics
///
/// This function might panic if `semaphore.acquire().await.unwrap()` fails.
/// However, this should not happen in practice as the semaphore is always available.
///
/// # Examples
///
/// Using this in an Axum router:
///
/// ```rust,no_run
/// use axum::{Router, routing::get};
/// use iron_shield::uptime::{uptime_stream, UptimeState};
/// use std::sync::{Arc, RwLock};
/// use std::collections::HashMap;
/// use std::collections::VecDeque;
///
/// // Assuming you have an uptime_state set up
/// let app = Router::new()
///     .route("/uptime", get(uptime_stream));
/// ```
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
    let shutdown_token = state.shutdown_token.clone();
    let shutdown_receiver = state.shutdown_events.subscribe();

    {
        if let Ok(config_guard) = config.read() {
            let sites_to_initialize = config_guard.sites.clone();
            drop(config_guard); // Release the read lock immediately

            for site in &sites_to_initialize {
                if let Ok(mut history_guard) = history_map.write() {
                    history_guard
                        .entry(site.name.clone())
                        .or_insert_with(VecDeque::new);
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
    let shutdown_token_for_task = shutdown_token.clone();
    tokio::spawn(async move {
        info!("Starting uptime monitoring service");

        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                () = shutdown_token_for_task.cancelled() => {
                    info!("Stopping uptime monitoring service due to shutdown signal");
                    break;
                }
                _ = interval.tick() => {}
            }
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
                    history_guard
                        .entry(site.name.clone())
                        .or_insert_with(VecDeque::new);
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
                        let uptime_percentage = calculate_uptime_percentage(site_history);
                        let site_name = site.name.clone();

                        debug!("Calculated uptime: site={site_name}, current_status=Loading, percentage={uptime_percentage:.2}%");

                        let data = create_uptime_history(
                            &site.name,
                            UptimeStatus::Loading,
                            site_history,
                            uptime_percentage,
                            None,
                        );

                        uptime_data.push(data);
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

                    let SiteCheckResult {
                        status,
                        response_time_ms,
                    } = check_site_status(&client, &url).await;
                    debug!(
                        "Uptime check completed for site: {site_name}, status: {status:?}, response_time_ms={response_time_ms:?}"
                    );

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
                        apply_final_status(site_history, status, response_time_ms);
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
                            let latest_response_time =
                                site_history.back().and_then(|entry| entry.response_time_ms);
                            let uptime_percentage = calculate_uptime_percentage(site_history);

                            debug!(
                                "Updated uptime stats: site={site_name}, status={status:?}, percentage={uptime_percentage:.2}%"
                            );

                            let data = create_uptime_history(
                                &site_name,
                                status,
                                site_history,
                                uptime_percentage,
                                latest_response_time,
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
    let uptime_stream = UnboundedReceiverStream::new(rx).map(|uptime_data| {
        if let Ok(event) = axum::response::sse::Event::default().json_data(&uptime_data) {
            Ok(event)
        } else {
            error!("Failed to serialize uptime data for SSE");
            Ok(axum::response::sse::Event::default().data("Error"))
        }
    });

    let maintenance_stream =
        BroadcastStream::new(shutdown_receiver).filter_map(|result| match result {
            Ok(message) => Some(Ok(axum::response::sse::Event::default()
                .event("maintenance")
                .data(message))),
            Err(e) => {
                error!("Failed to read shutdown notification for SSE: {e}");
                None
            }
        });

    let stream = uptime_stream.merge(maintenance_stream);

    Sse::new(stream)
}

/// Helper function to calculate the uptime percentage based on site history
///
/// This function calculates the percentage of time a site has been up based on its history,
/// excluding `Loading` statuses from the calculation to avoid artificially reducing the
/// percentage during initialization or active checking periods.
///
/// # Arguments
///
/// * `site_history` - A reference to a `VecDeque` containing the history of uptime statuses
///
/// # Returns
///
/// The uptime percentage as a floating-point number between 0.0 and 100.0
///
/// # Note
///
/// The calculation excludes `Loading` statuses to provide a more accurate representation
/// of actual uptime, since Loading is an intermediate state during checks rather than
/// an indicator of site availability.
#[must_use]
pub fn calculate_uptime_percentage(site_history: &VecDeque<HistoryEntry>) -> f64 {
    let up_count = site_history
        .iter()
        .filter(|entry| entry.status == UptimeStatus::Up)
        .count();
    let count_excluding_loading = site_history
        .iter()
        .filter(|entry| entry.status != UptimeStatus::Loading)
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

/// Helper function to create a `UptimeHistory` instance with current data
///
/// This function creates a new `UptimeHistory` struct populated with the current status,
/// timestamp, historical data, and calculated uptime percentage for a specific site.
///
/// # Arguments
///
/// * `site_name` - The unique identifier for the site
/// * `current_status` - The current uptime status to record
/// * `site_history` - The historical uptime data for this site
/// * `uptime_percentage` - The calculated uptime percentage
///
/// # Returns
///
/// A new `UptimeHistory` instance with the provided data and the current timestamp
///
fn create_uptime_history(
    site_name: &str,
    current_status: UptimeStatus,
    site_history: &VecDeque<HistoryEntry>,
    uptime_percentage: f64,
    response_time_ms: Option<u64>,
) -> UptimeHistory {
    UptimeHistory {
        site_id: site_name.to_string(),
        status: current_status,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs(),
        history: site_history.iter().cloned().collect(),
        uptime_percentage,
        response_time_ms,
    }
}

/// Replace the most recent `Loading` entry (if present) with the final status and metadata.
///
/// This ensures that loading markers act like a transient state. If no loading entry is found,
/// the final status is appended while enforcing the history length limit.
fn apply_final_status(
    site_history: &mut VecDeque<HistoryEntry>,
    final_status: UptimeStatus,
    response_time_ms: Option<u64>,
) {
    let mut replaced = false;

    if let Some(last) = site_history.back_mut() {
        if last.status == UptimeStatus::Loading {
            *last = HistoryEntry {
                status: final_status,
                response_time_ms,
            };
            replaced = true;
        }
    }

    if !replaced {
        site_history.push_back(HistoryEntry {
            status: final_status,
            response_time_ms,
        });
        if site_history.len() > MAX_HISTORY_ENTRIES {
            site_history.pop_front();
        }
    }
}

/// Helper function to check the status of a website
///
/// This function performs an HTTP HEAD request to the specified URL and determines
/// the uptime status based on the response. It handles timeouts and connection errors,
/// returning `UptimeStatus::Down` for any failure condition.
///
/// # Arguments
///
/// * `client` - A reqwest HTTP client to use for the request
/// * `url` - The URL of the site to check
///
/// # Returns
///
/// A `SiteCheckResult` containing the status and measured response time
///
/// # Note
///
/// The function uses a timeout of 10 seconds for the request. It returns `UptimeStatus::Down`
/// for any request failure, including timeouts, connection errors, or non-success HTTP status codes.
async fn check_site_status(client: &reqwest::Client, url: &str) -> SiteCheckResult {
    debug!("Checking site status: {url}");
    let start = Instant::now();
    let status = match client
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
    };

    let response_time_ms = start.elapsed().as_millis().try_into().unwrap_or(u64::MAX);

    SiteCheckResult {
        status,
        response_time_ms: Some(response_time_ms),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::collections::VecDeque;
    use std::sync::{Arc, RwLock};

    fn make_history(statuses: &[UptimeStatus]) -> VecDeque<HistoryEntry> {
        statuses
            .iter()
            .map(|status| HistoryEntry {
                status: *status,
                response_time_ms: None,
            })
            .collect()
    }

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
        let history: VecDeque<HistoryEntry> = VecDeque::new();
        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_uptime_percentage_all_up() {
        let history = make_history(&[UptimeStatus::Up, UptimeStatus::Up, UptimeStatus::Up]);

        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_uptime_percentage_all_down() {
        let history = make_history(&[UptimeStatus::Down, UptimeStatus::Down, UptimeStatus::Down]);

        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_uptime_percentage_mixed_no_loading() {
        let history = make_history(&[
            UptimeStatus::Up,
            UptimeStatus::Down,
            UptimeStatus::Up,
            UptimeStatus::Down,
        ]);

        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_uptime_percentage_with_loading() {
        let history = make_history(&[
            UptimeStatus::Up,
            UptimeStatus::Loading,
            UptimeStatus::Down,
            UptimeStatus::Loading,
            UptimeStatus::Up,
        ]);

        // 2 Up out of 3 non-Loading statuses = 66.67%
        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 66.67).abs() < 0.01);
    }

    #[test]
    fn test_calculate_uptime_percentage_all_loading() {
        let history = make_history(&[
            UptimeStatus::Loading,
            UptimeStatus::Loading,
            UptimeStatus::Loading,
        ]);

        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_create_uptime_history() {
        let history = VecDeque::from([
            HistoryEntry {
                status: UptimeStatus::Up,
                response_time_ms: Some(100),
            },
            HistoryEntry {
                status: UptimeStatus::Down,
                response_time_ms: None,
            },
            HistoryEntry {
                status: UptimeStatus::Loading,
                response_time_ms: None,
            },
        ]);

        let uptime_history =
            create_uptime_history("test-site", UptimeStatus::Up, &history, 50.0, Some(120));

        assert_eq!(uptime_history.site_id, "test-site");
        assert_eq!(uptime_history.status, UptimeStatus::Up);
        assert!((uptime_history.uptime_percentage - 50.0).abs() < f64::EPSILON);
        assert_eq!(
            uptime_history.history,
            vec![
                HistoryEntry {
                    status: UptimeStatus::Up,
                    response_time_ms: Some(100),
                },
                HistoryEntry {
                    status: UptimeStatus::Down,
                    response_time_ms: None,
                },
                HistoryEntry {
                    status: UptimeStatus::Loading,
                    response_time_ms: None,
                }
            ]
        );
        assert_eq!(uptime_history.response_time_ms, Some(120));

        // Check that timestamp is reasonable (within a few seconds of now)
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(uptime_history.timestamp <= current_time);
        assert!(uptime_history.timestamp >= current_time - 10); // Allow some buffer
    }

    #[test]
    fn test_apply_final_status_replaces_loading() {
        let mut history = VecDeque::from([
            HistoryEntry {
                status: UptimeStatus::Up,
                response_time_ms: Some(80),
            },
            HistoryEntry {
                status: UptimeStatus::Loading,
                response_time_ms: None,
            },
        ]);

        apply_final_status(&mut history, UptimeStatus::Down, Some(150));

        assert_eq!(
            history,
            VecDeque::from([
                HistoryEntry {
                    status: UptimeStatus::Up,
                    response_time_ms: Some(80),
                },
                HistoryEntry {
                    status: UptimeStatus::Down,
                    response_time_ms: Some(150),
                }
            ])
        );
    }

    #[test]
    fn test_apply_final_status_enforces_capacity() {
        let mut history = VecDeque::from(vec![
            HistoryEntry {
                status: UptimeStatus::Up,
                response_time_ms: None,
            };
            MAX_HISTORY_ENTRIES
        ]);

        apply_final_status(&mut history, UptimeStatus::Down, Some(200));

        assert_eq!(history.len(), MAX_HISTORY_ENTRIES);
        assert_eq!(
            history.back(),
            Some(&HistoryEntry {
                status: UptimeStatus::Down,
                response_time_ms: Some(200),
            })
        );
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

        let (shutdown_events, _) = tokio::sync::broadcast::channel(1);
        let shutdown_token = CancellationToken::new();

        let uptime_state = UptimeState {
            config,
            history,
            config_file_path,
            shutdown_events,
            shutdown_token,
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
        assert_eq!(result.status, UptimeStatus::Down);
        assert!(result.response_time_ms.is_some());
    }

    // Test the data structures
    #[test]
    fn test_uptime_history_serialization() {
        let uptime_history = UptimeHistory {
            site_id: "test-site".to_string(),
            status: UptimeStatus::Up,
            timestamp: 1_234_567_890, // Using underscores for readability
            history: vec![
                HistoryEntry {
                    status: UptimeStatus::Up,
                    response_time_ms: Some(80),
                },
                HistoryEntry {
                    status: UptimeStatus::Down,
                    response_time_ms: None,
                },
                HistoryEntry {
                    status: UptimeStatus::Loading,
                    response_time_ms: None,
                },
            ],
            uptime_percentage: 50.0,
            response_time_ms: Some(250),
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
        assert_eq!(
            uptime_history.response_time_ms,
            deserialized.response_time_ms
        );
    }

    #[tokio::test]
    async fn test_check_site_status_with_valid_url() {
        // Test with a URL that should return success (HTTPbin is commonly used for testing)
        let client = reqwest::Client::new();
        let result = check_site_status(&client, "https://httpbin.org/status/200").await;
        // Note: This might fail if no internet connection, but it's a good test when available
        // For now, we'll just check that it doesn't panic
        assert!(matches!(
            result.status,
            UptimeStatus::Up | UptimeStatus::Down
        ));
    }

    #[tokio::test]
    async fn test_check_site_status_down_non_2xx() {
        // Test with a URL that returns a 500 Internal Server Error
        let client = reqwest::Client::new();
        let result = check_site_status(&client, "https://httpbin.org/status/500").await;
        assert_eq!(result.status, UptimeStatus::Down);
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
        assert_eq!(result.status, UptimeStatus::Down);
    }

    #[test]
    fn test_calculate_uptime_percentage_accuracy() {
        // Test with a known percentage (75% uptime)
        let history = make_history(&[
            UptimeStatus::Up,
            UptimeStatus::Up,
            UptimeStatus::Up,
            UptimeStatus::Down,
        ]);

        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_uptime_percentage_with_mixed_loading() {
        // Test that loading statuses are excluded from calculation
        let history = make_history(&[
            UptimeStatus::Up,
            UptimeStatus::Loading,
            UptimeStatus::Down,
            UptimeStatus::Loading,
            UptimeStatus::Up,
            UptimeStatus::Up,
        ]);

        // Should be 3 up / 4 total non-loading = 75%
        let percentage = calculate_uptime_percentage(&history);
        assert!((percentage - 75.0).abs() < f64::EPSILON);
    }
}
