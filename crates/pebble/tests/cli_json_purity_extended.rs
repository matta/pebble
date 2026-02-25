mod support;

use assert_cmd::cargo_bin;
use serde_json::Value;
use std::process::Command;
use support::{setup_test_env, write_task};

#[test]
fn test_search_json_stdout_only() {
    let env = setup_test_env();
    write_task(&env.tasks_dir, "PROJ-1", "Search Task", "todo");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["search", "search", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert!(value.get("tasks").is_some());
}

#[test]
fn test_config_get_json_stdout_only() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["config", "get", "issue-prefix", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(value["key"].as_str(), Some("issue-prefix"));
}

#[test]
fn test_config_get_json_error_keeps_stdout_empty() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["config", "get", "invalid-key", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_archive_json_stdout_only() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["archive", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert!(value.get("archived").is_some());
}

#[test]
fn test_help_json_stdout_only() {
    let output = Command::new(cargo_bin!())
        .args(["help-json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(value["name"].as_str(), Some("pebble"));
}

#[test]
fn test_init_json_stdout_only() {
    let dir = tempfile::tempdir().expect("temp directory should be created");
    let output = Command::new(cargo_bin!())
        .current_dir(dir.path())
        .args(["init", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(value["status"].as_str(), Some("success"));
}
