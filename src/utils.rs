use chrono::{DateTime, Utc};

/// Get the current UTC time formatted as a string with "UTC" suffix
///
/// This function returns the current time in UTC timezone formatted as "HH:MM:SS UTC".
/// The format is consistent and follows the 24-hour format for hours, minutes, and seconds.
///
/// # Returns
///
/// A `String` containing the current UTC time in the format "HH:MM:SS UTC".
///
/// # Examples
///
/// ```
/// use iron_shield::utils::get_current_time_string;
///
/// let time_string = get_current_time_string();
/// println!("Current time: {}", time_string);
/// assert!(time_string.contains("UTC"));
/// ```
#[must_use]
pub fn get_current_time_string() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%H:%M UTC").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_time_string_format() {
        let time_string = get_current_time_string();

        // Check that it contains "UTC" suffix
        assert!(time_string.ends_with(" UTC"));

        // The time part should be at least 7 characters (H:MM:SS) to 8 characters (HH:MM)
        // plus 4 for " UTC", so total length should be between 11 and 12
        assert!(time_string.len() >= 8);
        assert!(time_string.len() <= 9);

        // Extract the time part (before " UTC")
        let time_part = &time_string[..time_string.len() - 4]; // Remove " UTC"

        // Split by ':'
        let parts: Vec<&str> = time_part.split(':').collect();
        assert_eq!(parts.len(), 2); // hours:minutes

        // Check that each part contains only digits
        for part in parts {
            assert!(part.chars().all(|c| c.is_ascii_digit()));
            // Each part should be either 1 or 2 digits
            assert!(part.len() == 1 || part.len() == 2);
        }

        // Check that there are exactly 1 colons in the time part
        assert_eq!(time_part.chars().filter(|&c| c == ':').count(), 1);
    }

    #[test]
    fn test_get_current_time_string_contains_utc() {
        let time_string = get_current_time_string();
        assert!(time_string.contains("UTC"));
    }

    #[test]
    fn test_get_current_time_string_not_empty() {
        let time_string = get_current_time_string();
        assert!(!time_string.is_empty());
    }
}
