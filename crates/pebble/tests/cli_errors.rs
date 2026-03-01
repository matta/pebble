#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod support;

use support::{setup_test_env, write_task_with_id};

#[test]
fn test_show_json_missing_id_reports_error_on_stderr() {
    let env = setup_test_env();

    write_task_with_id(&env.tasks_dir, "PROJ-1");

    let output = assert_cmd::Command::new(assert_cmd::cargo_bin!("pebble"))
        .args(["show", "PROJ-404", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_add_invalid_status_is_usage_error() {
    let output = assert_cmd::Command::new(assert_cmd::cargo_bin!("pebble"))
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
    let output = assert_cmd::Command::new(assert_cmd::cargo_bin!("pebble"))
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

    let output = assert_cmd::Command::new(assert_cmd::cargo_bin!("pebble"))
        .args(["update", "PROJ-1", "--priority", "100", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_list_fails_when_no_project_found() {
    let temp = tempfile::tempdir().expect("Failed to create temp dir");
    let root = temp.path();

    let output = assert_cmd::Command::new(assert_cmd::cargo_bin!("pebble"))
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

    let output = assert_cmd::Command::new(assert_cmd::cargo_bin!("pebble"))
        .current_dir(root)
        .args(["add", "New Task"])
        .output()
        .expect("Failed to execute pebble add");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No pebble project found"));
}
