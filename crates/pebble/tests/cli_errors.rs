#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod support;

use assert_cmd::cargo_bin;
use std::process::Command;
use support::{setup_test_env, write_task_with_id};

#[test]
fn test_show_json_missing_id_reports_error_on_stderr() {
    let env = setup_test_env();

    write_task_with_id(&env.tasks_dir, "PROJ-1");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["show", "PROJ-404", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_add_invalid_status_is_usage_error() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args([
            "add",
            "Bad Status",
            "--status",
            "not-a-status",
            "--json",
            "--dir",
            "tasks",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_add_priority_above_99_is_usage_error() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["add", "Bad Priority", "--priority", "100", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_update_priority_above_99_is_usage_error() {
    let env = setup_test_env();
    write_task_with_id(&env.tasks_dir, "PROJ-1");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["update", "PROJ-1", "--priority", "100", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

// The following tests intentionally use bare tempdirs (not `TestEnv`) because
// they verify behavior when no pebble project exists.

#[test]
fn test_list_fails_when_no_project_found() {
    let temp = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp.path();

    let output = Command::new(cargo_bin!())
        .current_dir(root)
        .args(["list"])
        .output()
        .expect("Failed to execute pebble list");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No pebble project found"));
}

#[test]
fn test_add_fails_when_no_project_found() {
    let temp = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp.path();

    let output = Command::new(cargo_bin!())
        .current_dir(root)
        .args(["add", "New Task"])
        .output()
        .expect("Failed to execute pebble add");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No pebble project found"));
}

#[test]
fn test_update_invalid_status_is_usage_error() {
    let env = setup_test_env();
    write_task_with_id(&env.tasks_dir, "PROJ-1");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args([
            "update",
            "PROJ-1",
            "--status",
            "not-a-status",
            "--json",
            "--dir",
            "tasks",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_list_invalid_sort_field_is_usage_error() {
    let env = setup_test_env();

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["list", "--sort", "not-a-field", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    // SortSpec::parse returns an eyre error (not UsageError), so exit code is 1.
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid sort field"));
}

#[test]
fn test_update_missing_id_is_runtime_error() {
    let env = setup_test_env();
    write_task_with_id(&env.tasks_dir, "PROJ-1");

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args([
            "update", "PROJ-404", "--status", "done", "--json", "--dir", "tasks",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_search_missing_query_is_usage_error() {
    let output = Command::new(cargo_bin!())
        .args(["search"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_show_missing_id_arg_is_usage_error() {
    let output = Command::new(cargo_bin!())
        .args(["show"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_init_absolute_dir_is_usage_error() {
    let temp = tempfile::tempdir().expect("Failed to create temp dir");

    let output = Command::new(cargo_bin!())
        .current_dir(temp.path())
        .args(["init", "--dir", "/absolute/path"])
        .output()
        .expect("pebble command should execute successfully");

    // validate_tasks_dir wraps in UsageError, so exit code is 2.
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}
