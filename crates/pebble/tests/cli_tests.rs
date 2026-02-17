#![allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("pebble").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("pebble 0.1.0"));
}

#[test]
fn test_config_get_sync_branch() {
    let mut cmd = Command::cargo_bin("pebble").unwrap();

    cmd.current_dir("../../../mydoo") // Run in mydoo so it finds .beads/config.yaml
        .args(["config", "get", "sync-branch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("beads-sync"));
}

#[test]
fn test_sync_fail_no_config() {
    // Create a temp dir to run in, ensuring no config
    let temp_dir = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("pebble").unwrap();

    cmd.current_dir(&temp_dir)
        .arg("sync")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read config file"));
}
