use color_eyre::eyre::{Result, eyre};
use std::io::{self, Read};
use std::str::FromStr;

pub fn current_toml_time() -> Result<toml_datetime::Datetime> {
    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();
    toml_datetime::Datetime::from_str(&now_str)
        .map_err(|e| eyre!("Failed to parse datetime for TOML: {}", e))
}

pub fn read_opt_stdin(input: Option<String>) -> Result<Option<String>> {
    match input {
        Some(s) if s == "-" => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(Some(buffer))
        }
        other => Ok(other),
    }
}
