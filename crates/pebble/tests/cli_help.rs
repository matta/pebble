use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_output_contains_show() {
    let mut cmd = Command::cargo_bin("pebble").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("Output a specific task in various formats"));
}

#[test]
fn test_show_help_output_contains_path_only_flag() {
    let mut cmd = Command::cargo_bin("pebble").unwrap();
    cmd.arg("show")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--path-only"))
        .stdout(predicate::str::contains("Output just the raw filepath instead of the task entity"));
}
