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
