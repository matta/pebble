use chrono::{DateTime, Utc};

/// Returns the current time in UTC.
///
/// This is a simple wrapper around `Utc::now()`, used for standardizing timestamps
/// across the pebble tasks ecosystem.
///
/// # Examples
///
/// ```
/// use pebble::task_io::current_task_time;
///
/// let now = current_task_time();
/// assert!(now.timestamp() > 0);
/// ```
pub fn current_task_time() -> DateTime<Utc> {
    Utc::now()
}
