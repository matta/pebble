mod support;

use assert_cmd::cargo_bin;
use serde_json::Value;
use std::fs;
use std::process::Command;
use support::{setup_test_env, write_task};

#[test]
fn test_add_terminal_status_sets_resolved_at() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "Done Task", "--status", "done", "--json"])
        .output()
        .expect("Failed to execute pebble add");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(
        json["resolved_at"].is_string(),
        "resolved_at should be set for a task added with 'done' status"
    );
}

#[test]
fn test_update_status_to_closed_sets_resolved_at() {
    let env = setup_test_env();
    write_task(&env.tasks_dir, "PROJ-1", "Task 1", "todo");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["update", "PROJ-1", "--status", "done", "--json"])
        .output()
        .expect("Failed to execute pebble update");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(
        json["resolved_at"].is_string(),
        "resolved_at should be set when status moves to 'done', got: {:?}",
        json["resolved_at"]
    );
}

#[test]
fn test_update_status_away_from_closed_clears_resolved_at() {
    let env = setup_test_env();
    // Start with a done task (we mock it with resolved_at)
    let content = r#"+++
id = "PROJ-1"
title = "Task 1"
status = "done"
created_at = 2024-01-01T00:00:00Z
resolved_at = 2024-01-01T12:00:00Z
+++
"#;
    fs::write(env.tasks_dir.join("PROJ-1.md"), content).unwrap();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["update", "PROJ-1", "--status", "in_progress", "--json"])
        .output()
        .expect("Failed to execute pebble update");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(
        json["resolved_at"].is_null(),
        "resolved_at should be cleared when status moves away from terminal"
    );
}

#[test]
fn test_update_always_sets_modified_at() {
    let env = setup_test_env();
    write_task(&env.tasks_dir, "PROJ-1", "Task 1", "todo");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args([
            "update",
            "PROJ-1",
            "--append-body",
            "More content",
            "--json",
        ])
        .output()
        .expect("Failed to execute pebble update");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    assert!(
        json["modified_at"].is_string(),
        "modified_at should be set on any update"
    );
}

#[test]
fn test_archive_respects_threshold_and_handles_collisions() {
    let env = setup_test_env();

    // Create a task that is old enough to archive
    let old_content = r#"+++
id = "PROJ-old"
title = "Old Task"
status = "done"
created_at = 2024-01-01T00:00:00Z
resolved_at = 2024-01-01T12:00:00Z
+++
"#;
    fs::write(env.tasks_dir.join("PROJ-old.md"), old_content).unwrap();

    // Create a task that is NOT old enough
    let now_toml = chrono::Utc::now().to_rfc3339();
    let new_content = format!(
        r#"+++
id = "PROJ-new"
title = "New Task"
status = "done"
created_at = {now}
resolved_at = {now}
+++
"#,
        now = now_toml
    );
    fs::write(env.tasks_dir.join("PROJ-new.md"), new_content).unwrap();

    // Also pre-create a file in archive to trigger collision
    let archive_dir = env.tasks_dir.join("archive");
    fs::create_dir_all(&archive_dir).unwrap();
    fs::write(archive_dir.join("PROJ-old.md"), "already here").unwrap();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["archive", "--json"])
        .output()
        .expect("Failed to execute pebble archive");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    let archived = json["archived"].as_array().unwrap();
    assert_eq!(archived.len(), 1, "Expected 1 task to be archived");
    assert_eq!(archived[0]["id"], "PROJ-old");
    assert_eq!(archived[0]["moved_to"], "archive/PROJ-old-2.md");

    assert!(!env.tasks_dir.join("PROJ-old.md").exists());
    assert!(env.tasks_dir.join("PROJ-new.md").exists());
    assert!(archive_dir.join("PROJ-old-2.md").exists());
}

#[test]
fn test_priority_validation_enforces_range() {
    let env = setup_test_env();

    // Valid priority
    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "Valid Priority", "--priority", "42"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Invalid priority (too high)
    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "Invalid Priority", "--priority", "100"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid value"));

    // Invalid priority (negative)
    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "Invalid Priority", "--priority", "-1"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_add_new_task_ends_with_trailing_newline() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args([
            "add",
            "New Task",
            "--body",
            "Body without newline",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&stdout).unwrap();

    // Get the path from JSON
    let rel_path = json["path"].as_str().unwrap();
    let abs_path = env.root.join(rel_path);

    let content = fs::read_to_string(abs_path).unwrap();
    assert!(
        content.ends_with('\n'),
        "Expected task file to end with a trailing newline even if body doesn't have one"
    );
}
