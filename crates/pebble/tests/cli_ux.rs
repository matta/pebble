mod support;

use assert_cmd::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use support::setup_test_env;

#[test]
fn test_next_stdout_is_clean_when_no_tasks() {
    let env = setup_test_env();

    env.pebble()
        .arg("next")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("No ready tasks found."));
}

#[test]
fn test_add_stdout_is_clean_non_json() {
    let env = setup_test_env();

    env.pebble()
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
