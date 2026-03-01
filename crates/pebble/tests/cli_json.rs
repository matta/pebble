#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod support;

use assert_cmd::cargo_bin;
use serde_json::Value;
use std::process::Command;
use support::{setup_test_env, write_task};

#[test]
fn test_list_json_stdout_only() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-1", "First Task", "todo");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["list", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert!(value.get("tasks").is_some());
}

#[test]
fn test_next_json_stdout_only() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["next", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(!output.status.success());
    assert!(!output.stderr.is_empty());
    assert!(output.stdout.is_empty());
}

#[test]
fn test_show_json_stdout_only() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-2", "Show Task", "todo");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["show", "PROJ-2", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(value.get("id").and_then(|v| v.as_str()), Some("PROJ-2"));

    // Verify renamed fields exist in JSON
    assert!(value.get("needs").is_some(), "JSON missing 'needs' field");
    assert!(
        value.get("blocked_by").is_some(),
        "JSON missing 'blocked_by' field"
    );
    assert!(
        value.get("blocking").is_some(),
        "JSON missing 'blocking' field"
    );
    assert!(
        value.get("deps").is_none(),
        "JSON still contains legacy 'deps' field"
    );
}

#[test]
fn test_add_json_stdout_only() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "New Task", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(
        value.get("title").and_then(|v| v.as_str()),
        Some("New Task")
    );
}

#[test]
fn test_update_json_stdout_only() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-3", "Update Task", "todo");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args([
            "update", "PROJ-3", "--status", "done", "--json", "--dir", "tasks",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(value.get("status").and_then(|v| v.as_str()), Some("done"));
}
