use crate::error::Result;
use crate::uptime::{snapshot_current_histories, UptimeState};
use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, warn};

#[derive(Deserialize, Serialize, Clone)]
pub struct SiteClickEvent {
    pub site_name: String,
    pub site_url: String,
}

#[derive(Serialize)]
struct HeaderRecord {
    name: String,
    value: String,
}

#[derive(Serialize)]
struct ClickTelemetryPayload {
    dashboard_name: String,
    generated_at: u64,
    site_name: String,
    site_url: String,
    headers: Vec<HeaderRecord>,
}

/// Payload delivered to the OpenTelemetry collector endpoint.
#[derive(Serialize)]
struct UptimeTelemetryPayload {
    /// Dashboard name to make it easier to identify the sender.
    dashboard_name: String,
    /// Unix timestamp indicating when the payload was generated.
    generated_at: u64,
    /// Latest status for each monitored site.
    uptime: Vec<TelemetrySiteStatus>,
}

#[derive(Serialize)]
struct TelemetrySiteStatus {
    site_id: String,
    status: crate::uptime::UptimeStatus,
    timestamp: u64,
    response_time_ms: Option<u64>,
}

/// Push the latest uptime snapshot to the configured endpoint.
///
/// This function is meant to be called on a best-effort basis, so callers should
/// handle errors gracefully and avoid blocking user flows when the endpoint is
/// unreachable.
///
/// # Errors
///
/// Returns an error if the HTTP request cannot be sent or if the endpoint responds
/// with a non-success status code.
pub async fn send_uptime_snapshot(
    state: Arc<UptimeState>,
    dashboard_name: String,
    endpoint: String,
) -> Result<()> {
    if endpoint.trim().is_empty() {
        return Ok(());
    }

    let snapshot = snapshot_current_histories(&state);

    if snapshot.is_empty() {
        debug!("No uptime history available; skipping telemetry payload");
    }

    let payload = UptimeTelemetryPayload {
        dashboard_name,
        generated_at: current_timestamp(),
        uptime: snapshot
            .into_iter()
            .filter(|history| {
                !matches!(
                    history.status,
                    crate::uptime::UptimeStatus::Loading | crate::uptime::UptimeStatus::Disabled
                )
            })
            .map(|history| TelemetrySiteStatus {
                site_id: history.site_id,
                status: history.status,
                timestamp: history.timestamp,
                response_time_ms: history.response_time_ms,
            })
            .collect(),
    };

    post_telemetry(&endpoint, &payload).await
}

/// Receive click tracking events from the frontend and forward them to the telemetry collector.
pub async fn track_site_click(
    State(state): State<Arc<UptimeState>>,
    headers: HeaderMap,
    Json(payload): Json<SiteClickEvent>,
) -> impl IntoResponse {
    if payload.site_name.trim().is_empty() || payload.site_url.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "site_name and site_url are required",
        )
            .into_response();
    }

    if let Some((endpoint, dashboard_name)) = telemetry_destination(&state) {
        let header_records = header_records(&headers);
        let click = payload.clone();
        tokio::spawn(async move {
            let telemetry_payload = ClickTelemetryPayload {
                dashboard_name,
                generated_at: current_timestamp(),
                site_name: click.site_name,
                site_url: click.site_url,
                headers: header_records,
            };

            if let Err(err) = post_telemetry(&endpoint, &telemetry_payload).await {
                warn!("Failed to send click telemetry: {err}");
            }
        });
    }

    StatusCode::ACCEPTED.into_response()
}

#[must_use]
pub fn telemetry_destination(state: &Arc<UptimeState>) -> Option<(String, String)> {
    let config_guard = state.config.read().ok()?;
    let endpoint = config_guard.opentelemetry_endpoint.clone()?;
    if endpoint.trim().is_empty() {
        return None;
    }
    Some((endpoint, config_guard.site_name.clone()))
}

fn header_records(headers: &HeaderMap) -> Vec<HeaderRecord> {
    headers
        .iter()
        .map(|(name, value)| HeaderRecord {
            name: name.as_str().to_string(),
            value: String::from_utf8_lossy(value.as_bytes()).to_string(),
        })
        .collect()
}

async fn post_telemetry(endpoint: &str, payload: &impl Serialize) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .post(endpoint)
        .json(payload)
        .send()
        .await
        .map_err(|err| {
            crate::error::IronShieldError::Generic(format!("Failed to send telemetry: {err}"))
        })?;

    response.error_for_status().map(|_| ()).map_err(|err| {
        crate::error::IronShieldError::Generic(format!(
            "Telemetry endpoint responded with error status: {err}"
        ))
    })
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
