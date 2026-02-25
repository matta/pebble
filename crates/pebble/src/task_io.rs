use color_eyre::eyre::{Result, eyre};
use std::str::FromStr;

pub fn current_toml_time() -> Result<toml_datetime::Datetime> {
    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();
    toml_datetime::Datetime::from_str(&now_str)
        .map_err(|e| eyre!("Failed to parse datetime for TOML: {}", e))
}
