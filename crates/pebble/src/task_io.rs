use color_eyre::eyre::{Result, eyre};
use std::path::Path;
use std::str::FromStr;

pub fn current_toml_time() -> Result<toml_datetime::Datetime> {
    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();
    toml_datetime::Datetime::from_str(&now_str)
        .map_err(|e| eyre!("Failed to parse datetime for TOML: {}", e))
}

pub fn write_task_file(path: &Path, frontmatter: &impl serde::Serialize, body: &str) -> Result<()> {
    let fm_toml = toml::to_string(frontmatter)?;
    let content = format!("+++\n{}+++\n{}", fm_toml, body);
    std::fs::write(path, content)?;
    Ok(())
}
