#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod support;

use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_show_path_only_relative() {
    let dir = tempdir().expect("temp directory should be created");
    let root = dir.path();

    // Create config
    let config_dir = root.join(".pebble");
    fs::create_dir(&config_dir).expect("config directory should be created");
    fs::write(
        config_dir.join("config.toml"),
        r#"
        issue_prefix = "PROJ"
        tasks_dir = "tasks"
        "#,
    )
    .expect("config file should be written");

    let tasks_dir = root.join("tasks");
    fs::create_dir(&tasks_dir).expect("tasks directory should be created");
    let task_path = tasks_dir.join("PROJ-1.md");
    fs::write(
        &task_path,
        r#"+++
id = "PROJ-1"
title = "First Task"
status = "todo"
needs = []
created_at = 2024-01-01T00:00:00Z
+++
Body text
"#,
    )
    .expect("config file should be written");

    let expected_rel_path = "PROJ-1.md\n";

    support::pebble(root)
        .arg("show")
        .arg("PROJ-1")
        .arg("--path-only")
        .arg("--dir")
        .arg("tasks")
        .assert()
        .success()
        .stdout(predicate::eq(expected_rel_path));
}

#[test]
fn test_show_formatted_output() {
    let dir = tempdir().expect("temp directory should be created");
    let root = dir.path();

    // Create config
    let config_dir = root.join(".pebble");
    fs::create_dir(&config_dir).expect("config directory should be created");
    fs::write(
        config_dir.join("config.toml"),
        r#"
        issue_prefix = "PROJ"
        tasks_dir = "tasks"
        "#,
    )
    .expect("config file should be written");

    let tasks_dir = root.join("tasks");
    fs::create_dir(&tasks_dir).expect("tasks directory should be created");
    let task_path = tasks_dir.join("PROJ-2.md");
    fs::write(
        &task_path,
        r#"+++
id = "PROJ-2"
title = "A Formatted Task"
status = "todo"
needs = []
tags = ["frontend"]
created_at = 2024-01-01T00:00:00Z
+++
Body text explaining the task.
"#,
    )
    .expect("config file should be written");

    support::pebble(root)
        .arg("show")
        .arg("PROJ-2")
        .arg("--dir")
        .arg("tasks")
        .assert()
        .success()
        // Wait till we get full formatted output to match
        .stdout(predicate::str::contains("Task: A Formatted Task (PROJ-2)"))
        .stdout(predicate::str::contains("Status: Todo"))
        .stdout(predicate::str::contains("Tags: [\"frontend\"]"))
        .stdout(predicate::str::contains("Body text explaining the task."));
}
