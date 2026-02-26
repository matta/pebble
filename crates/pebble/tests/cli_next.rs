#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod support;

use assert_cmd::cargo_bin;
use std::fs;
use std::path::Path;
use std::process::Command;
use support::setup_test_env;

/// Helper to write a task that is blocked by a missing dependency.
fn write_blocked_task(dir: &Path, filename: &str, id: &str, status: &str) {
    let content = format!(
        r#"+++
id = "{id}"
title = "Blocked Task"
status = "{status}"
created_at = 2024-01-01T00:00:00Z
needs = ["MISSING-ID"]
+++
Body
"#
    );
    fs::write(dir.join(filename), content).expect("task file should be written");
}

#[test]
fn test_next_reports_blocked_tasks() {
    let env = setup_test_env();

    write_blocked_task(&env.tasks_dir, "blocked.md", "BLOCKED-1", "todo");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["next"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No ready tasks found. (1 task is blocked)"),
        "Stderr was: {}",
        stderr
    );
}

#[test]
fn test_next_with_no_tasks() {
    let env = setup_test_env();

    // No tasks created.

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["next"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No ready tasks found."),
        "Stderr was: {}",
        stderr
    );
    // Should NOT contain the blocked message
    assert!(
        !stderr.contains("task is blocked"),
        "Stderr was: {}",
        stderr
    );
}

#[test]
fn test_next_with_multiple_blocked_tasks() {
    let env = setup_test_env();

    write_blocked_task(&env.tasks_dir, "blocked1.md", "BLOCKED-1", "todo");
    write_blocked_task(&env.tasks_dir, "blocked2.md", "BLOCKED-2", "in_progress");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["next"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No ready tasks found. (2 tasks are blocked)"),
        "Stderr was: {}",
        stderr
    );
}
