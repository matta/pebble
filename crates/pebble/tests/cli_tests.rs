use assert_cmd::Command;
use assert_cmd::cargo_bin;
use predicates::prelude::*;
use tempfile::TempDir;

mod common;
use common::TEST_SYNC_BRANCH;
mod cli_support;
use cli_support::{TestEnv, create_test_issue};

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
fn test_list_issues_empty() {
    let env = TestEnv::setup();
    env.pebble()
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("Using database:"))
        .stderr(predicate::str::contains("No issues found."));
}

#[test]
fn test_list_issues_with_data() {
    let env = TestEnv::setup();
    let issue = create_test_issue("test-123", "Fixture Issue");
    env.add_issue_to_worktree(&issue);

    env.pebble()
        .arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("Using database:"))
        .stdout(predicate::str::contains("test-123 [open] Fixture Issue"));
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
fn test_edit_issue() {
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

    // 2. Edit issue
    env.pebble()
        .args(["edit", id, "--title", "New Edited Title"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated issue"));

    // 3. Verify edit
    env.pebble()
        .args(["show", id])
        .assert()
        .success()
        .stdout(predicate::str::contains("New Edited Title"));
}

#[test]
fn test_edit_non_existent_issue() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["edit", "non-existent-id", "--title", "New Title"])
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
