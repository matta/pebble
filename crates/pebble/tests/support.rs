#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
//! Shared test helpers for CLI integration tests.

use assert_cmd::{Command, cargo_bin};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Temp Pebble workspace with an initialized config and tasks directory.
pub struct TestEnv {
    _dir: TempDir,
    /// Root directory of the temporary test workspace.
    pub root: PathBuf,
    /// Tasks directory within the temporary workspace.
    pub tasks_dir: PathBuf,
}

impl TestEnv {
    /// Returns a pre-configured `assert_cmd::Command` for the pebble CLI.
    pub fn pebble(&self) -> Command {
        let mut cmd = Command::new(cargo_bin!());
        cmd.current_dir(&self.root);
        cmd
    }
}

/// Create a temp Pebble project with a config and empty tasks dir.
pub fn setup_test_env() -> TestEnv {
    let dir = tempfile::tempdir().expect("temp directory should be created");
    let root = dir.path().to_path_buf();

    let config_dir = root.join(".pebble");
    fs::create_dir(&config_dir).expect("config directory should be created");
    fs::write(
        config_dir.join("config.toml"),
        r#"
        issue-prefix = "PROJ"
        tasks-dir = "tasks"
        "#,
    )
    .expect("config file should be written");

    let tasks_dir = root.join("tasks");
    fs::create_dir(&tasks_dir).expect("tasks directory should be created");

    TestEnv {
        _dir: dir,
        root,
        tasks_dir,
    }
}

/// Write a simple task file into the tasks directory.
pub fn write_task(tasks_dir: &Path, id: &str, title: &str, status: &str) {
    let content = format!(
        r#"---
id: "{id}"
title: "{title}"
status: "{status}"
created_at: "2024-01-01T00:00:00Z"
---
Body
"#,
        id = id,
        title = title,
        status = status
    );
    fs::write(tasks_dir.join(format!("{id}.md")), content).expect("task file should be written");
}

/// Write a simple task file with title equal to the id.
pub fn write_task_with_id(tasks_dir: &Path, id: &str) {
    write_task(tasks_dir, id, id, "todo");
}
