use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_next_stdout_is_clean_when_no_tasks() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create config
    let config_dir = root.join(".pebble");
    fs::create_dir(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
        issue_prefix = "PROJ"
        tasks_dir = "tasks"
        "#,
    )
    .unwrap();
    fs::create_dir(root.join("tasks")).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pebble"));
    cmd.current_dir(root)
        .arg("next")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("No ready tasks found."));
}

#[test]
fn test_add_stdout_is_clean_non_json() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create config
    let config_dir = root.join(".pebble");
    fs::create_dir(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.toml"),
        r#"
        issue_prefix = "PROJ"
        tasks_dir = "tasks"
        "#,
    )
    .unwrap();
    fs::create_dir(root.join("tasks")).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pebble"));
    cmd.current_dir(root)
        .arg("add")
        .arg("Clean Task")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Created task"));
}

#[test]
fn test_global_help_descriptions() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_pebble"));
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Change to the given directory"))
        .stdout(predicate::str::contains("Path to configuration file"))
        .stdout(predicate::str::contains("Output in JSON format"))
        .stdout(predicate::str::contains("Path to the tasks directory"));
}
