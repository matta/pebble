#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod support;

use assert_cmd::cargo_bin;
use std::fs;
use std::process::Command;
use support::setup_test_env;

#[test]
fn test_next_reports_blocked_tasks() {
    let env = setup_test_env();

    // Create a task that depends on a missing ID, so it's blocked.
    // write_task helper doesn't support needs, so we write manually.
    let task_content = r#"+++
id = "BLOCKED-1"
title = "Blocked Task"
status = "todo"
created_at = 2024-01-01T00:00:00Z
needs = ["MISSING-ID"]
+++
Body
"#;
    fs::write(env.tasks_dir.join("blocked.md"), task_content).expect("task file should be written");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["next"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    // This assertion is expected to fail initially.
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

    let task1 = r#"+++
id = "BLOCKED-1"
title = "Blocked Task 1"
status = "todo"
created_at = 2024-01-01T00:00:00Z
needs = ["MISSING-ID"]
+++
"#;
    fs::write(env.tasks_dir.join("blocked1.md"), task1).expect("task file should be written");

    let task2 = r#"+++
id = "BLOCKED-2"
title = "Blocked Task 2"
status = "in_progress"
created_at = 2024-01-01T00:00:00Z
needs = ["MISSING-ID"]
+++
"#;
    fs::write(env.tasks_dir.join("blocked2.md"), task2).expect("task file should be written");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["next"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    // This assertion is expected to fail initially.
    assert!(
        stderr.contains("No ready tasks found. (2 tasks are blocked)"),
        "Stderr was: {}",
        stderr
    );
}
