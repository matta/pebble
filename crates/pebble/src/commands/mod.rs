use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use pebble::command::CommandExt;
use pebble::config::Config;
use std::path::Path;

pub mod add;
pub mod config_cmd;
pub mod edit;
pub mod import;
pub mod init;
pub mod list;
pub mod show;
pub mod sync;

pub fn load_config(path: Option<&Path>) -> Result<Config> {
    let config_path = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => Config::default_path(&std::env::current_dir()?),
    };
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file at {}", config_path.display()))?;
    let config: Config = toml::from_str(&content).context("Failed to parse config")?;
    config.validate()?;
    Ok(config)
}

pub fn get_worktree_manager(config: &Config) -> Result<pebble::worktree::WorktreeManager> {
    let sync_branch = config
        .sync_branch
        .as_deref()
        .ok_or_else(|| eyre!("sync-branch not configured"))?;

    let repo_root = std::env::current_dir()?;
    Ok(pebble::worktree::WorktreeManager::new(
        repo_root,
        sync_branch.to_string(),
    ))
}

pub fn get_store(
    config: &Config,
) -> Result<(
    pebble::store::JsonlStore,
    pebble::worktree::WorktreeManager,
    std::path::PathBuf,
)> {
    let manager = get_worktree_manager(config)?;
    let jsonl_path = manager.get_absolute_jsonl_path()?;
    let store = pebble::store::JsonlStore::new(
        jsonl_path
            .to_str()
            .ok_or_else(|| eyre!("Path contains invalid UTF-8 characters"))?,
    );
    Ok((store, manager, jsonl_path))
}

pub fn get_git_config(key: &str) -> Result<String> {
    std::process::Command::new("git")
        .args(["config", key])
        .check_output()
        .map(|s| s.trim().to_string())
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_worktree_manager_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let config = Config {
            sync_branch: Some("my-sync-branch".to_string()),
            ..Default::default()
        };

        let result = get_worktree_manager(&config);
        assert!(result.is_ok());

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_get_worktree_manager_missing_sync_branch() {
        let config = Config {
            sync_branch: None,
            ..Default::default()
        };
        let result = get_worktree_manager(&config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "sync-branch not configured");
    }
}
