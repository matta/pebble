use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct Config {
    #[serde(rename = "sync-branch", skip_serializing_if = "Option::is_none")]
    pub sync_branch: Option<String>,

    #[serde(rename = "issue-prefix", skip_serializing_if = "Option::is_none")]
    pub issue_prefix: Option<String>,

    #[serde(rename = "no-db", skip_serializing_if = "Option::is_none")]
    pub no_db: Option<bool>,

    #[serde(rename = "no-daemon", skip_serializing_if = "Option::is_none")]
    pub no_daemon: Option<bool>,

    #[serde(rename = "auto-start-daemon", skip_serializing_if = "Option::is_none")]
    pub auto_start_daemon: Option<bool>,
}

impl Config {
    pub fn is_initialized(root: &Path) -> bool {
        root.join(".pebble").is_dir() || root.join(".beads").is_dir()
    }

    pub fn save(&self, path: &Path) -> color_eyre::Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
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

    #[test]
    fn test_config_all_fields() {
        let content = "
sync-branch: pebble-sync
issue-prefix: pebble
no-db: true
no-daemon: true
auto-start-daemon: false
";
        let config: Config = serde_yaml::from_str(content).expect("Failed to parse config");

        assert_eq!(config.sync_branch, Some("pebble-sync".to_string()));
        assert_eq!(config.issue_prefix, Some("pebble".to_string()));
        assert_eq!(config.no_db, Some(true));
        assert_eq!(config.no_daemon, Some(true));
        assert_eq!(config.auto_start_daemon, Some(false));
    }

    #[test]
    fn test_config_empty() {
        let content = "{}";
        let config: Config = serde_yaml::from_str(content).expect("Failed to parse config");

        assert_eq!(config.sync_branch, None);
        assert_eq!(config.issue_prefix, None);
        assert_eq!(config.no_db, None);
        assert_eq!(config.no_daemon, None);
        assert_eq!(config.auto_start_daemon, None);
    }

    #[test]
    fn test_config_invalid_yaml() {
        let content = "sync-branch: pebble-sync\n: invalid\n";
        let result: Result<Config, _> = serde_yaml::from_str(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_invalid_types() {
        let content = "no-db: 'not-a-boolean'\n";
        let result: Result<Config, _> = serde_yaml::from_str(content);
        assert!(result.is_err());
    }
}
