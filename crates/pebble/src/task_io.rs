use chrono::{DateTime, Utc};

pub fn current_task_time() -> DateTime<Utc> {
    Utc::now()
}
