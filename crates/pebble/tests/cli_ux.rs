mod support;

use assert_cmd::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use support::{setup_test_env, write_task};

#[test]
fn test_next_stdout_is_clean_when_no_tasks() {
    let env = setup_test_env();

    let mut cmd = Command::new(cargo_bin!());
    cmd.current_dir(&env.root)
        .arg("next")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("No ready tasks found."));
}

#[test]
fn test_add_stdout_is_clean_non_json() {
    let env = setup_test_env();

    let mut cmd = Command::new(cargo_bin!());
    cmd.current_dir(&env.root)
        .arg("add")
        .arg("Clean Task")
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Created task"));
}

#[test]
fn test_global_help_descriptions() {
    let mut cmd = Command::new(cargo_bin!());
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Change to the given directory"))
        .stdout(predicate::str::contains("Path to configuration file"))
        .stdout(predicate::str::contains("Output in JSON format"))
        .stdout(predicate::str::contains("Path to the tasks directory"));
}

#[test]
fn test_add_reads_body_from_stdin() {
    let env = setup_test_env();

    #[allow(deprecated)]
    #[expect(clippy::unwrap_used)]
    let mut cmd = assert_cmd::Command::cargo_bin("pebble").unwrap();
    cmd.current_dir(&env.root)
        .arg("add")
        .arg("Stdin Task")
        .arg("--body")
        .arg("-")
        .write_stdin("Body from pipe")
        .assert()
        .success();

    let task_path = env.tasks_dir.join("stdin-task.md");
    #[expect(clippy::expect_used)]
    let content = fs::read_to_string(&task_path).expect("Task file should exist");
    assert!(
        content.contains("Body from pipe"),
        "Task body missing content from stdin"
    );
    assert!(
        !content.contains("+++\n-"),
        "Task body should not be literal dash"
    );
}

#[test]
fn test_update_reads_body_from_stdin() {
    let env = setup_test_env();
    write_task(&env.tasks_dir, "pebl-1", "Update Target", "todo");

    #[allow(deprecated)]
    #[expect(clippy::unwrap_used)]
    let mut cmd = assert_cmd::Command::cargo_bin("pebble").unwrap();
    cmd.current_dir(&env.root)
        .arg("update")
        .arg("pebl-1")
        .arg("--body")
        .arg("-")
        .write_stdin("Updated body from pipe")
        .assert()
        .success();

    let task_path = env.tasks_dir.join("pebl-1.md");
    #[expect(clippy::expect_used)]
    let content = fs::read_to_string(&task_path).expect("Task file should exist");
    assert!(
        content.contains("Updated body from pipe"),
        "Task body missing updated content from stdin"
    );
}
