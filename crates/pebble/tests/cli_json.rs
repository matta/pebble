mod support;

use assert_cmd::Command;
use serde_json::Value;
use support::{setup_test_env, write_task};

#[test]
fn test_list_json_stdout_only() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-1", "First Task", "todo");

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["list", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value.get("tasks").is_some());
}

#[test]
fn test_next_json_stdout_only() {
    let env = setup_test_env();

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["next", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(!output.stderr.is_empty());
    assert!(output.stdout.is_empty());
}

#[test]
fn test_show_json_stdout_only() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-2", "Show Task", "todo");

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["show", "PROJ-2", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value.get("id").and_then(|v| v.as_str()), Some("PROJ-2"));
}

#[test]
fn test_add_json_stdout_only() {
    let env = setup_test_env();

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["add", "New Task", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value.get("title").and_then(|v| v.as_str()),
        Some("New Task")
    );
}

#[test]
fn test_update_json_stdout_only() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-3", "Update Task", "todo");

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args([
            "update", "PROJ-3", "--status", "done", "--json", "--dir", "tasks",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value.get("status").and_then(|v| v.as_str()), Some("done"));
}
