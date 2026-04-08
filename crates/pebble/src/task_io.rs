use chrono::{DateTime, Utc};

/// Retrieves the current time in UTC for timestamping tasks.
///
/// This function centralizes the acquisition of the current timestamp to ensure
/// all generated dates in the application are consistently recorded in UTC.
///
/// # Examples
///
/// ```
/// use chrono::Utc;
/// use pebble::task_io::current_task_time;
///
/// let now = current_task_time();
/// assert!(now <= Utc::now());
/// ```
pub fn current_task_time() -> DateTime<Utc> {
    Utc::now()
}
