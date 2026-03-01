use color_eyre::eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// Resolved project configuration loaded from `.pebble/config.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Config {
    /// Prefix prepended to generated task IDs (e.g. `"issue"` produces `"issue-abc123"`).
    #[serde(default = "default_issue_prefix")]
    #[serde(rename = "issue-prefix")]
    pub issue_prefix: String,

    /// Path to the tasks directory, relative to the project root.
    #[serde(default = "default_tasks_dir")]
    #[serde(rename = "tasks-dir")]
    pub tasks_dir: PathBuf,

    /// Number of days a terminal task must be resolved before it is eligible for archiving.
    #[serde(default = "default_archive_threshold_days")]
    #[serde(rename = "archive-threshold-days")]
    pub archive_threshold_days: i64,
}

fn default_issue_prefix() -> String {
    "issue".to_string()
}

fn default_tasks_dir() -> PathBuf {
    PathBuf::from("docs/pebble/")
}

fn default_archive_threshold_days() -> i64 {
    30
}

impl Default for Config {
    fn default() -> Self {
        Self {
            issue_prefix: default_issue_prefix(),
            tasks_dir: default_tasks_dir(),
            archive_threshold_days: default_archive_threshold_days(),
        }
    }
}

/// Resolves the project root by walking up from the given directory.
///
/// It searches until it finds a `.pebble` directory and returns `None` if it hits the filesystem root.
pub fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
    for ancestor in start_dir.ancestors() {
        if ancestor.join(".pebble").is_dir() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

/// Validates that the tasks directory path is safe.
///
/// Ensures the path is relative and does not contain `..` components.
///
/// # Errors
///
/// Returns an error if:
/// * `path` is absolute.
/// * `path` contains parent directory components (`..`).
///
/// # Examples
///
/// ```
/// # use std::path::Path;
/// # use pebble::config::validate_tasks_dir;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// assert!(validate_tasks_dir(Path::new("docs/tasks")).is_ok());
/// assert!(validate_tasks_dir(Path::new("/absolute/path")).is_err());
/// assert!(validate_tasks_dir(Path::new("docs/../tasks")).is_err());
/// # Ok(())
/// # }
/// ```
pub fn validate_tasks_dir(path: &Path) -> Result<()> {
    if path.is_absolute() {
        return Err(eyre!(
            "Configuration error: 'tasks-dir' must be a relative path to the project root. Found: {}",
            path.display()
        ));
    }

    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(eyre!(
            "Configuration error: 'tasks-dir' must not contain parent directory components ('..'). Found: {}",
            path.display()
        ));
    }

    Ok(())
}

/// Parses configuration from a TOML string, validating path constraints.
///
/// Ensures that `tasks-dir` is a relative path and does not contain parent directory
/// components (`..`) to prevent path traversal issues.
///
/// # Errors
///
/// Returns an error if:
/// * The TOML string cannot be parsed.
/// * `tasks-dir` is an absolute path.
/// * `tasks-dir` contains `..` components.
///
/// # Examples
///
/// ```
/// # use pebble::config::parse_config;
/// # use std::path::PathBuf;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let toml = r#"
/// issue-prefix = "ticket"
/// tasks-dir = "docs/tasks/"
/// "#;
///
/// let config = parse_config(toml)?;
/// assert_eq!(config.issue_prefix, "ticket");
/// assert_eq!(config.tasks_dir, PathBuf::from("docs/tasks/"));
/// # Ok(())
/// # }
/// ```
pub fn parse_config(toml_str: &str) -> Result<Config> {
    let config: Config = if toml_str.trim().is_empty() {
        Config::default()
    } else {
        toml::from_str(toml_str)?
    };

    validate_tasks_dir(&config.tasks_dir)?;

    Ok(config)
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_default_config() {
        let toml = "";
        let config = parse_config(toml).expect("Valid config");
        assert_eq!(config.issue_prefix, "issue");
        assert_eq!(config.tasks_dir, PathBuf::from("docs/pebble/"));
    }

    #[test]
    fn test_parse_custom_config() {
        let toml = r#"
        issue-prefix = "TKT"
        tasks-dir = "my-tasks/"
        "#;
        let config = parse_config(toml).expect("Valid config");
        assert_eq!(config.issue_prefix, "TKT");
        assert_eq!(config.tasks_dir, PathBuf::from("my-tasks/"));
    }

    #[test]
    fn test_parse_config_rejects_absolute_tasks_dir() {
        let toml = r#"
        tasks-dir = "/absolute/path/to/tasks"
        "#;
        let err = parse_config(toml).expect_err("Should reject absolute tasks path");
        assert!(
            err.to_string().contains("must be a relative path"),
            "Error was: {}",
            err
        );
    }

    #[test]
    fn test_parse_config_rejects_parent_dir_components() {
        let cases = [
            r#"tasks-dir = "../parent/dir""#,
            r#"tasks-dir = "nested/../parent""#,
        ];

        for toml in cases {
            let err = parse_config(toml).expect_err("Should reject parent directory components");
            assert!(
                err.to_string()
                    .contains("must not contain parent directory components"),
                "Failed to reject invalid config: {}. Error was: {}",
                toml,
                err
            );
        }
    }

    #[test]
    fn test_find_project_root() {
        let temp = tempfile::tempdir().expect("Failed to create temp dir");
        let root = temp.path();

        let pebble_dir = root.join(".pebble");
        fs::create_dir(&pebble_dir).expect("Failed to create .pebble dir");

        let deeply_nested = root.join("some").join("deep").join("path");
        fs::create_dir_all(&deeply_nested).expect("Failed to create deeply nested path");

        // Should find the root when starting from the nested directory
        let found = find_project_root(&deeply_nested).expect("Should find root");
        assert_eq!(found, root);

        // Should find the root when starting from the root itself
        let found2 = find_project_root(root).expect("Should find root");
        assert_eq!(found2, root);

        // Should return None when there is no .pebble dir
        let empty_temp = tempfile::tempdir().expect("Failed to create second temp dir");
        assert_eq!(find_project_root(empty_temp.path()), None);
    }
}
