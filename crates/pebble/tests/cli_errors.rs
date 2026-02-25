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
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn test_list_fails_when_no_project_found() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    let output = Command::new(cargo_bin!())
        .current_dir(root)
        .args(["list"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No pebble project found"));
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
        .unwrap();

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
        .unwrap();

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
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}
