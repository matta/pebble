use color_eyre::eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub fn validate_branch_name(branch: &str) -> Result<()> {
    if branch.starts_with('-') {
        return Err(color_eyre::eyre::eyre!("sync-branch cannot start with '-'"));
    }
    Ok(())
}

/// Represents the configuration for the Pebble application.
///
/// This struct maps to the TOML configuration file, typically located at `.pebble/config.toml`.
/// It holds settings that control the behavior of the application, such as the branch used for
/// synchronization and ID generation prefixes.
///
/// # Examples
///
/// ```
/// use pebble_legacy::config::Config;
///
/// let config = Config {
///     sync_branch: Some("pebble-sync".to_string()),
///     issue_prefix: Some("issue".to_string()),
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(rename = "sync-branch", skip_serializing_if = "Option::is_none")]
    pub sync_branch: Option<String>,

    /// Used in `main.rs` to generate issue IDs (e.g., `issue-123`).
    #[serde(rename = "issue-prefix", skip_serializing_if = "Option::is_none")]
    pub issue_prefix: Option<String>,
}

impl Config {
    pub fn default_path(root: &Path) -> PathBuf {
        root.join(crate::CONFIG_DIR).join(crate::CONFIG_FILE)
    }

    pub fn is_initialized(root: &Path) -> bool {
        root.join(crate::CONFIG_DIR).is_dir()
    }

    pub fn save(&self, path: &Path) -> color_eyre::Result<()> {
        let content = toml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(branch) = &self.sync_branch {
            validate_branch_name(branch)?;
        } else {
            return Err(eyre!("sync-branch is required in configuration"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_sync_branch_starts_with_dash() {
        let content = "sync-branch = \"-bad\"\n";
        let config: Config = toml::from_str(content).expect("Failed to parse config");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_parsing() {
        let content = "sync-branch = \"pebble-sync\"\nissue-prefix = \"pebble\"\n";
        let config: Config = toml::from_str(content).expect("Failed to parse config");
        config.validate().expect("Validation failed");

        assert_eq!(config.sync_branch, Some("pebble-sync".to_string()));
        assert_eq!(config.issue_prefix, Some("pebble".to_string()));
    }

    #[test]
    fn test_config_all_fields() {
        let content = r#"
sync-branch = "pebble-sync"
issue-prefix = "pebble"
"#;
        let config: Config = toml::from_str(content).expect("Failed to parse config");
        config.validate().expect("Validation failed");

        assert_eq!(config.sync_branch, Some("pebble-sync".to_string()));
        assert_eq!(config.issue_prefix, Some("pebble".to_string()));
    }

    #[test]
    fn test_config_empty() {
        let content = "";
        let config: Config = toml::from_str(content).expect("Failed to parse config");
        assert!(config.validate().is_err()); // Validation should fail without sync-branch

        assert_eq!(config.sync_branch, None);
        assert_eq!(config.issue_prefix, None);
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            sync_branch: None,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        let config = Config {
            sync_branch: Some("main".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_invalid_toml() {
        let content = "sync-branch = pebble-sync\n: invalid\n";
        let result: Result<Config, _> = toml::from_str(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_invalid_types() {
        let content = "sync-branch = 123\n";
        let result: Result<Config, _> = toml::from_str(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_unknown_field() {
        let content = "unknown-field = \"some-value\"\n";
        let result: Result<Config, _> = toml::from_str(content);
        assert!(result.is_err());
    }
}
