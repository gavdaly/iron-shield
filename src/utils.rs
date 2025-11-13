use chrono::{DateTime, Utc};

/// Get the current UTC time formatted as a string with "UTC" suffix
#[must_use]
pub fn get_current_time_string() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%H:%M:%S UTC").to_string()
}
