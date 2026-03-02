#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
pub mod support;

use serde_json::Value;
use std::fs;
use std::thread::sleep;
use std::time::Duration;
use support::{setup_test_env, write_task};

#[test]
fn test_add_terminal_status_sets_resolved_at() {
    let env = setup_test_env();

    let output = env
        .pebble()
        .args(["add", "Done Task", "--status", "done", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    assert!(
        json["resolved_at"].is_string(),
        "resolved_at should be set for a task added with 'done' status"
    );
}

#[test]
fn test_update_status_to_closed_sets_resolved_at() {
    let env = setup_test_env();
    write_task(&env.tasks_dir, "PROJ-1", "Task 1", "todo");

    let output = env
        .pebble()
        .args(["update", "PROJ-1", "--status", "done", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

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
    let content = r#"---
id: "PROJ-1"
title: "Task 1"
status: "done"
created_at: "2024-01-01T00:00:00Z"
resolved_at: "2024-01-01T12:00:00Z"
---
"#;
    fs::write(env.tasks_dir.join("PROJ-1.md"), content).expect("task file should be written");

    let output = env
        .pebble()
        .args(["update", "PROJ-1", "--status", "in_progress", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    assert!(
        json["resolved_at"].is_null(),
        "resolved_at should be cleared when status moves away from terminal"
    );
}

#[test]
fn test_update_always_sets_modified_at() {
    let env = setup_test_env();
    write_task(&env.tasks_dir, "PROJ-1", "Task 1", "todo");

    let output = env
        .pebble()
        .args([
            "update",
            "PROJ-1",
            "--append-body",
            "More content",
            "--json",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    assert!(
        json["modified_at"].is_string(),
        "modified_at should be set on any update"
    );
}

#[test]
fn test_archive_respects_threshold_and_handles_collisions() {
    let env = setup_test_env();

    // Create a task that is old enough to archive
    let old_content = r#"---
id: "PROJ-old"
title: "Old Task"
status: "done"
created_at: "2024-01-01T00:00:00Z"
resolved_at: "2024-01-01T12:00:00Z"
---
"#;
    fs::write(env.tasks_dir.join("PROJ-old.md"), old_content)
        .expect("old task file should be written");

    // Create a task that is NOT old enough
    let now_toml = chrono::Utc::now().to_rfc3339();
    let new_content = format!(
        r#"---
id: "PROJ-new"
title: "New Task"
status: "done"
created_at: "{now}"
resolved_at: "{now}"
---
"#,
        now = now_toml
    );
    fs::write(env.tasks_dir.join("PROJ-new.md"), new_content)
        .expect("new task file should be written");

    // Also pre-create a file in archive to trigger collision
    let archive_dir = env.tasks_dir.join("archive");
    fs::create_dir_all(&archive_dir).expect("archive directory should be created");
    fs::write(archive_dir.join("PROJ-old.md"), "already here")
        .expect("archive file should be written");

    let output = env
        .pebble()
        .args(["archive", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    let archived = json["archived"]
        .as_array()
        .expect("archived should be an array");
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
    let output = env
        .pebble()
        .args(["add", "Valid Priority", "--priority", "42"])
        .output()
        .expect("pebble command should execute successfully");
    assert!(output.status.success());

    // Invalid priority (too high)
    let output = env
        .pebble()
        .args(["add", "Invalid Priority", "--priority", "100"])
        .output()
        .expect("pebble command should execute successfully");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid value"));

    // Invalid priority (negative)
    let output = env
        .pebble()
        .args(["add", "Invalid Priority", "--priority", "-1"])
        .output()
        .expect("pebble command should execute successfully");
    assert!(!output.status.success());
}

#[test]
fn test_add_new_task_ends_with_trailing_newline() {
    let env = setup_test_env();

    let output = env
        .pebble()
        .args([
            "add",
            "New Task",
            "--body",
            "Body without newline",
            "--json",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    // Get the path from JSON
    let rel_path = json["path"].as_str().expect("path should be a string");
    let abs_path = env.root.join(rel_path);

    let content = fs::read_to_string(abs_path).expect("task file should be readable");
    assert!(
        content.ends_with('\n'),
        "Expected task file to end with a trailing newline even if body doesn't have one"
    );
}

#[test]
fn test_update_no_changes_does_not_modify_disk() {
    let env = setup_test_env();
    write_task(&env.tasks_dir, "X", "Task X", "todo");
    write_task(&env.tasks_dir, "Y", "Task Y", "todo");

    let path_x = env.tasks_dir.join("X.md");
    let path_y = env.tasks_dir.join("Y.md");
    let initial_mtime_x = fs::metadata(&path_x)
        .expect("mtime x")
        .modified()
        .expect("mtime x mod");
    let initial_mtime_y = fs::metadata(&path_y)
        .expect("mtime y")
        .modified()
        .expect("mtime y mod");

    // Wait for mtime resolution
    sleep(Duration::from_millis(100));

    // Update X with no changes (except --blocks Y, which only affects Y)
    let output = env
        .pebble()
        .args(["update", "X", "--blocks", "Y", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    // X should NOT have changed on disk
    let final_mtime_x = fs::metadata(&path_x)
        .expect("mtime x")
        .modified()
        .expect("mtime x mod");
    assert_eq!(
        initial_mtime_x, final_mtime_x,
        "Task X should not be modified on disk if no fields changed"
    );

    // X's JSON output should NOT have modified_at (since it wasn't modified)
    // Wait, let's check if the JSON output includes modified_at from the original file.
    // write_task doesn't set modified_at.
    assert!(
        json["modified_at"].is_null(),
        "modified_at should not be set if no change occurred"
    );

    // Y SHOULD have changed on disk
    let final_mtime_y = fs::metadata(&path_y)
        .expect("mtime y")
        .modified()
        .expect("mtime y mod");
    assert!(
        final_mtime_y > initial_mtime_y,
        "Task Y SHOULD be modified on disk because it now 'needs' X"
    );

    // Verify Y actually needs X
    let content_y = fs::read_to_string(&path_y).expect("read y");
    assert!(
        content_y.contains("needs:\n  - X")
            || content_y.contains("needs: [X]")
            || content_y.contains("needs: [\"X\"]")
    );
}

#[test]
fn test_update_body_and_append_body_together() {
    let env = setup_test_env();
    write_task(&env.tasks_dir, "PROJ-1", "Task 1", "todo");

    let output = env
        .pebble()
        .args([
            "update",
            "PROJ-1",
            "--body",
            "New Base Body",
            "--append-body",
            "Appended Part",
            "--json",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    assert_eq!(
        json["body"], "New Base Body\n\nAppended Part",
        "Both --body and --append-body should be applied if provided"
    );
}
