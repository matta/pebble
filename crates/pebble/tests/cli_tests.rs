use assert_cmd::Command;
use assert_cmd::cargo_bin;
use predicates::prelude::*;
use tempfile::TempDir;

mod common;
use common::TEST_SYNC_BRANCH;
mod cli_support;
use cli_support::TestEnv;

#[test]
fn test_config_get_unknown_key() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["config", "get", "unknown-key"])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Unknown config key 'unknown-key'"));
}

#[test]
fn test_config_get_unset_key() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["config", "get", "issue-prefix"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Config key 'issue-prefix' not set",
        ));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("pebble 0.1.0"));
}

#[test]
fn test_config_get_sync_branch() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["config", "get", "sync-branch"])
        .assert()
        .success()
        .stdout(predicate::str::contains(TEST_SYNC_BRANCH));
}

#[test]
fn test_sync_fail_no_config() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(temp_dir.path())
        .arg("sync")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: Pebble is not initialized in this repository. Run 'pebble init' to get started."));
}

#[test]
fn test_add_issue() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["add", "New Test Issue", "--description", "This is a test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added issue issue-"));
}

#[test]
fn test_add_issue_json() {
    let env = TestEnv::setup();
    let output = env
        .pebble()
        .args(["add", "New JSON Issue", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let issue: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");
    assert_eq!(issue["title"], "New JSON Issue");
    assert!(issue["id"].as_str().unwrap().starts_with("issue-"));
}

#[test]
fn test_add_and_show_issue() {
    let env = TestEnv::setup();
    // 1. Add issue
    let output = env
        .pebble()
        .args(["add", "Show Test Issue", "--description", "Showing this"])
        .output()
        .expect("Failed to run add");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .split_whitespace()
        .last()
        .expect("Failed to get ID from output");

    // 2. Show issue
    env.pebble()
        .args(["show", id])
        .assert()
        .success()
        .stdout(predicate::str::contains("Show Test Issue"))
        .stdout(predicate::str::contains("Showing this"));
}

#[test]
fn test_show_non_existent_issue() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["show", "non-existent-id"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Issue non-existent-id not found"));
}

#[test]
fn test_update_issue() {
    let env = TestEnv::setup();
    // 1. Add issue
    let output = env
        .pebble()
        .args(["add", "Edit Test Issue"])
        .output()
        .expect("Failed to run add");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .split_whitespace()
        .last()
        .expect("Failed to get ID from output");

    // 2. Update issue
    env.pebble()
        .args(["update", id, "--title", "New Updated Title"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated issue"));

    // 3. Verify update
    env.pebble()
        .args(["show", id])
        .assert()
        .success()
        .stdout(predicate::str::contains("New Updated Title"));
}

#[test]
fn test_update_non_existent_issue() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["update", "non-existent-id", "--title", "New Title"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Issue non-existent-id not found"));
}

#[test]
fn test_directory_flag() {
    let env = TestEnv::setup();
    let temp_dir = TempDir::new().unwrap();

    // Run pebble from a completely different directory, pointing to env.root with -C
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.current_dir(temp_dir.path())
        .arg("-C")
        .arg(env.root())
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("Using database:"))
        .stderr(predicate::str::contains("No issues found."));
}

#[test]
fn test_no_args_fails() {
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "A distributed issue tracking system built on Git.",
        ))
        .stderr(predicate::str::contains(
            "Usage: pebble [OPTIONS] <COMMAND>",
        ));
}

#[test]
fn test_help_includes_examples() {
    let mut cmd = Command::new(cargo_bin!("pebble"));
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("pebble init"));
}

#[test]
fn test_update_issue_json() {
    let env = TestEnv::setup();
    // 1. Add issue
    let output = env
        .pebble()
        .args(["add", "Update JSON Test"])
        .output()
        .expect("Failed to run add");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .split_whitespace()
        .last()
        .expect("Failed to get ID from output");

    // 2. Update issue with --json
    let output = env
        .pebble()
        .args(["update", id, "--title", "Updated JSON Title", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let issue: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");
    assert_eq!(issue["id"], id);
    assert_eq!(issue["title"], "Updated JSON Title");
}

#[test]
fn test_update_status_priority_owner() {
    let env = TestEnv::setup();
    // 1. Add issue
    let output = env
        .pebble()
        .args(["add", "Update Fields Test"])
        .output()
        .expect("Failed to run add");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .split_whitespace()
        .last()
        .expect("Failed to get ID from output");

    // 2. Update fields
    env.pebble()
        .args([
            "update",
            id,
            "--status",
            "closed",
            "--priority",
            "5",
            "--owner",
            "new@example.com",
            "--type",
            "bug",
        ])
        .assert()
        .success();

    // 3. Verify with json show
    let output = env
        .pebble()
        .args(["show", id, "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let issue: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");

    assert_eq!(issue["status"], "closed");
    assert_eq!(issue["priority"], 5);
    assert_eq!(issue["owner"], "new@example.com");
    assert_eq!(issue["issue_type"], "bug");
}
