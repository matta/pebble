use predicates::prelude::*;

#[test]
fn test_help_output_contains_show() {
    let mut cmd = assert_cmd::Command::new(assert_cmd::cargo_bin!("pebble"));
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("Show one task by ID"));
}

#[test]
fn test_show_help_output_contains_path_only_flag() {
    let mut cmd = assert_cmd::Command::new(assert_cmd::cargo_bin!("pebble"));
    cmd.arg("show")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--path-only"))
        .stdout(predicate::str::contains(
            "Output only the file path relative to tasks-dir",
        ));
}

#[test]
fn test_check_help_includes_warn_only_flag() {
    let mut cmd = assert_cmd::Command::new(assert_cmd::cargo_bin!("pebble"));
    cmd.arg("check")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--warn-only"))
        .stdout(predicate::str::contains("--fix"))
        .stdout(predicate::str::contains("exit with status code 0"));
}
