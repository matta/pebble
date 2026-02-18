pub mod config;
pub mod command;
pub mod store;
pub mod worktree;

pub const PEBBLE_DIR: &str = ".pebble";
pub const CONFIG_FILE: &str = "config.toml";
pub const ISSUES_FILE: &str = "issues.jsonl";
pub const WORKTREE_DIR: &str = ".git/x-pebble";
