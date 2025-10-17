use crate::config::Config;
use axum::{extract::State, response::Sse};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::sync::{Arc, RwLock};
use std::time::Duration;
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
    pub config: Config,
    pub history: Arc<RwLock<HashMap<String, VecDeque<UptimeStatus>>>>,
}

pub async fn uptime_stream(
    State(state): State<Arc<UptimeState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    // Clone the config and history for use in the stream
    let config = state.config.clone();
    let history_map = state.history.clone();
    let client = reqwest::Client::new();

    // Create a channel to send updates from the checker task
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn a task to periodically check sites and send updates
    tokio::spawn(async move {
        info!("Starting uptime monitoring service");

        let mut interval = tokio::time::interval(Duration::from_secs(5));

        // Initialize the history map with loading status for all sites
        {
            let mut history_guard = history_map.write().unwrap();
            for site in &config.sites {
                let mut history = VecDeque::new();
                history.push_back(UptimeStatus::Loading);
                history_guard.insert(site.name.clone(), history);
            }
        }

        loop {
            interval.tick().await;
            debug!("Starting new uptime check cycle");

            // Get a copy of the sites to check
            let sites_to_check = config.sites.clone();

            // Update history to loading state
            {
                let mut history_guard = history_map.write().unwrap();
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
                let history_guard = history_map.read().unwrap();
                let mut uptime_data = Vec::new();

                for site in &sites_to_check {
                    if let Some(site_history) = history_guard.get(&site.name) {
                        if let Some(&current_status) = site_history.back() {
                            // Calculate uptime percentage
                            let up_count = site_history
                                .iter()
                                .filter(|&&s| s == UptimeStatus::Up)
                                .count();
                            let uptime_percentage = if site_history.len() > 0 {
                                (up_count as f64) / (site_history.len() as f64) * 100.0
                            } else {
                                0.0
                            };

                            debug!("Calculated uptime: site={}, current_status={:?}, up_count={}, total_count={}, percentage={:.2}%",
                                site.name, current_status, up_count, site_history.len(), uptime_percentage);

                            let data = UptimeHistory {
                                site_id: site.name.clone(),
                                status: current_status,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                history: site_history.iter().cloned().collect(),
                                uptime_percentage,
                            };

                            uptime_data.push(data);
                        }
                    }
                }

                if tx.send(uptime_data).is_err() {
                    error!("Failed to send loading updates to SSE stream");
                    break; // Channel closed, exit the loop
                }
            }

            // Now check the actual status of each site
            for site in &sites_to_check {
                let client = client.clone();
                let url = site.url.clone();
                let tx = tx.clone();
                let history_map = history_map.clone();
                let site_name = site.name.clone();

                tokio::spawn(async move {
                    debug!("Starting uptime check for site: {}", site_name);

                    // Check the site status
                    let status = check_site_status(&client, &url).await;
                    debug!(
                        "Uptime check completed for site: {}, status: {:?}",
                        site_name, status
                    );

                    // Update the history with the actual status
                    {
                        let mut history_guard = history_map.write().unwrap();
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
                        let history_guard = history_map.read().unwrap();
                        if let Some(site_history) = history_guard.get(&site_name) {
                            // Calculate uptime percentage
                            let up_count = site_history
                                .iter()
                                .filter(|&&s| s == UptimeStatus::Up)
                                .count();
                            let total_count = site_history.len();
                            let uptime_percentage = if total_count > 0 {
                                (up_count as f64) / (total_count as f64) * 100.0
                            } else {
                                0.0
                            };

                            debug!("Updated uptime stats: site={}, status={:?}, up_count={}, total_count={}, percentage={:.2}%",
                                site_name, status, up_count, total_count, uptime_percentage);

                            let data = UptimeHistory {
                                site_id: site_name.clone(),
                                status,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                                history: site_history.iter().cloned().collect(),
                                uptime_percentage,
                            };

                            // Send only the update for this specific site
                            if tx.send(vec![data]).is_err() {
                                error!("Failed to send uptime update for site: {}", site_name);
                            }
                        }
                    }
                });
            }
        }
    });

    // Convert the receiving end of the channel into a stream
    let stream = UnboundedReceiverStream::new(rx).map(|uptime_data| {
        match axum::response::sse::Event::default().json_data(&uptime_data) {
            Ok(event) => Ok(event),
            Err(_) => {
                error!("Failed to serialize uptime data for SSE");
                Ok(axum::response::sse::Event::default().data("Error"))
            }
        }
    });

    Sse::new(stream)
}

// Helper function to check site status
async fn check_site_status(client: &reqwest::Client, url: &str) -> UptimeStatus {
    debug!("Checking site status: {}", url);
    match client
        .head(url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                debug!("Site {} is UP: status {}", url, status);
                UptimeStatus::Up
            } else {
                debug!("Site {} is DOWN: status {}", url, status);
                UptimeStatus::Down
            }
        }
        Err(e) => {
            debug!("Site {} is DOWN: error {}", url, e);
            UptimeStatus::Down // If there's an error, consider the site as down
        }
    }
}
