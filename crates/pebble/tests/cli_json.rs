use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn setup_test_env() -> (TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();

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

    let tasks_dir = root.join("tasks");
    fs::create_dir(&tasks_dir).unwrap();

    (dir, tasks_dir)
}

fn write_task(tasks_dir: &std::path::Path, id: &str, title: &str, status: &str) {
    let content = format!(
        r#"+++
id = "{id}"
title = "{title}"
status = "{status}"
created_at = 2024-01-01T00:00:00Z
+++
Body
"#,
        id = id,
        title = title,
        status = status
    );
    fs::write(tasks_dir.join(format!("{id}.md")), content).unwrap();
}

#[test]
fn test_list_json_stdout_only() {
    let (dir, tasks_dir) = setup_test_env();
    let root = dir.path();

    write_task(&tasks_dir, "PROJ-1", "First Task", "todo");

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(root)
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
    let (dir, _tasks_dir) = setup_test_env();
    let root = dir.path();

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(root)
        .args(["next", "--json", "--dir", "tasks"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value.is_null());
}

#[test]
fn test_show_json_stdout_only() {
    let (dir, tasks_dir) = setup_test_env();
    let root = dir.path();

    write_task(&tasks_dir, "PROJ-2", "Show Task", "todo");

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(root)
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
    let (dir, _tasks_dir) = setup_test_env();
    let root = dir.path();

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(root)
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
    let (dir, tasks_dir) = setup_test_env();
    let root = dir.path();

    write_task(&tasks_dir, "PROJ-3", "Update Task", "todo");

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(root)
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
