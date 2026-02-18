use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    #[serde(rename = "sync-branch")]
    pub sync_branch: Option<String>,

    #[serde(rename = "issue-prefix")]
    pub issue_prefix: Option<String>,

    #[serde(rename = "no-db")]
    pub no_db: Option<bool>,

    #[serde(rename = "no-daemon")]
    pub no_daemon: Option<bool>,

    #[serde(rename = "auto-start-daemon")]
    pub auto_start_daemon: Option<bool>,
}

impl Config {
    pub fn is_initialized(root: &Path) -> bool {
        root.join(".pebble").is_dir() || root.join(".beads").is_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let content = "sync-branch: pebble-sync\nissue-prefix: pebble\n";
        let config: Config = serde_yaml::from_str(content).expect("Failed to parse config");

        assert_eq!(config.sync_branch, Some("pebble-sync".to_string()));
        assert_eq!(config.issue_prefix, Some("pebble".to_string()));
    }
}
