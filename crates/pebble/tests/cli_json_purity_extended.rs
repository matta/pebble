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
        .expect("Failed to run search command");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("Invalid search JSON output");
    assert!(value.get("tasks").is_some());
}

#[test]
fn test_config_get_json_stdout_only() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["config", "get", "issue-prefix", "--json"])
        .output()
        .expect("Failed to run config get");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("Invalid config JSON output");
    assert_eq!(value["key"].as_str(), Some("issue-prefix"));
}

#[test]
fn test_config_get_json_error_keeps_stdout_empty() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["config", "get", "invalid-key", "--json"])
        .output()
        .expect("Failed to run config get");

    assert_eq!(output.status.code(), Some(1));
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
        .expect("Failed to run archive");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("Invalid archive JSON output");
    assert!(value.get("archived").is_some());
}

#[test]
fn test_help_json_stdout_only() {
    let output = Command::new(cargo_bin!())
        .args(["help-json"])
        .output()
        .expect("Failed to run help-json");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: Value =
        serde_json::from_slice(&output.stdout).expect("Invalid help-json command output");
    assert_eq!(value["name"].as_str(), Some("pebble"));
}

#[test]
fn test_init_json_stays_quiet() {
    let dir = tempfile::tempdir().expect("Failed to create tempdir");
    let output = Command::new(cargo_bin!())
        .current_dir(dir.path())
        .args(["init", "--json"])
        .output()
        .expect("Failed to run init --json");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(output.stdout.is_empty());
}
