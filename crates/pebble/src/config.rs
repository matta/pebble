use color_eyre::eyre::{Result, eyre};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    #[serde(rename = "sync-branch")]
    pub sync_branch: Option<String>,

    /// Used in `main.rs` to generate issue IDs (e.g., `issue-123`).
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
    pub fn validate(&self) -> Result<()> {
        if self.no_daemon == Some(false) {
            return Err(eyre!("Configuration 'no-daemon: false' is invalid as daemon mode is not supported."));
        }
        if self.auto_start_daemon == Some(true) {
            return Err(eyre!("Configuration 'auto-start-daemon: true' is invalid as daemon mode is not supported."));
        }
        if self.no_db == Some(false) {
            return Err(eyre!("Configuration 'no-db: false' is invalid as database mode is not supported."));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let content = "sync-branch: beads-sync\nissue-prefix: pebble\n";
        let config: Config = serde_yaml::from_str(content).expect("Failed to parse config");
        config.validate().expect("Validation failed");

        assert_eq!(config.sync_branch, Some("beads-sync".to_string()));
        assert_eq!(config.issue_prefix, Some("pebble".to_string()));
    }

    #[test]
    fn test_validation_no_daemon() {
        let config = Config {
            sync_branch: None,
            issue_prefix: None,
            no_db: None,
            no_daemon: Some(false),
            auto_start_daemon: None,
        };
        let err = config.validate().unwrap_err();
        assert_eq!(err.to_string(), "Configuration 'no-daemon: false' is invalid as daemon mode is not supported.");
    }

    #[test]
    fn test_validation_auto_start_daemon() {
        let config = Config {
            sync_branch: None,
            issue_prefix: None,
            no_db: None,
            no_daemon: None,
            auto_start_daemon: Some(true),
        };
        let err = config.validate().unwrap_err();
        assert_eq!(err.to_string(), "Configuration 'auto-start-daemon: true' is invalid as daemon mode is not supported.");
    }

    #[test]
    fn test_validation_no_db() {
        let config = Config {
            sync_branch: None,
            issue_prefix: None,
            no_db: Some(false),
            no_daemon: None,
            auto_start_daemon: None,
        };
        let err = config.validate().unwrap_err();
        assert_eq!(err.to_string(), "Configuration 'no-db: false' is invalid as database mode is not supported.");
    }
}
