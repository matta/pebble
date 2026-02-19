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
fn test_search_issue() {
    let env = TestEnv::setup();
    let issue = create_test_issue("search-1", "Search Me Title");
    env.add_issue_to_worktree(&issue);
    let issue2 = create_test_issue("search-2", "Dont Find Me");
    env.add_issue_to_worktree(&issue2);

    env.pebble()
        .args(["search", "Search Me"])
        .assert()
        .success()
        .stdout(predicate::str::contains("search-1"))
        .stdout(predicate::str::contains("Search Me Title"))
        .stdout(predicate::str::contains("search-2").not());
}

#[test]
fn test_search_issue_json() {
    let env = TestEnv::setup();
    let issue = create_test_issue("search-json", "Search JSON Title");
    env.add_issue_to_worktree(&issue);

    let output = env
        .pebble()
        .args(["search", "Search JSON", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let issues: Vec<serde_json::Value> =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["id"], "search-json");
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
            "--issue-type",
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

#[test]
fn test_list_filters() {
    let env = TestEnv::setup();
    // Add multiple issues
    let issue1 = create_test_issue("list-1", "Open Issue");
    env.add_issue_to_worktree(&issue1); // status: open, priority: 0

    let mut issue2 = create_test_issue("list-2", "Closed Issue");
    issue2["status"] = serde_json::json!("closed");
    issue2["priority"] = serde_json::json!(1);
    env.add_issue_to_worktree(&issue2);

    // Filter by status
    env.pebble()
        .args(["list", "--status", "closed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list-2"))
        .stdout(predicate::str::contains("list-1").not());

    // Filter by priority
    env.pebble()
        .args(["list", "--priority", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list-2"))
        .stdout(predicate::str::contains("list-1").not());

    // Filter by owner (default is test@example.com)
    env.pebble()
        .args(["list", "--owner", "test@example.com"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list-1"))
        .stdout(predicate::str::contains("list-2"));

    env.pebble()
        .args(["list", "--owner", "other@example.com"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No issues found"));
}
