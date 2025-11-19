use crate::error::Result;
use crate::uptime::{snapshot_current_histories, UptimeState};
use serde::Serialize;
use std::sync::Arc;
use tracing::debug;

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
            .filter(|history| history.status != crate::uptime::UptimeStatus::Loading)
            .map(|history| TelemetrySiteStatus {
                site_id: history.site_id,
                status: history.status,
                timestamp: history.timestamp,
                response_time_ms: history.response_time_ms,
            })
            .collect(),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&endpoint)
        .json(&payload)
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
