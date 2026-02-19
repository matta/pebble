mod cli_support;
mod common;

use cli_support::{TestEnv, create_test_issue};
use common::TEST_SYNC_BRANCH;
use tempfile::TempDir;

#[test]
fn test_list_issues_json() {
    let env = TestEnv::setup();
    let issue = create_test_issue("test-json", "JSON Issue");
    env.add_issue_to_worktree(&issue);

    let output = env
        .pebble()
        .arg("list")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let issues: Vec<serde_json::Value> =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0]["id"], "test-json");
}

#[test]
fn test_show_issue_json() {
    let env = TestEnv::setup();
    let issue = create_test_issue("test-json-show", "JSON Show Issue");
    env.add_issue_to_worktree(&issue);

    let output = env
        .pebble()
        .args(["show", "test-json-show", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let issue_out: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");
    assert_eq!(issue_out["id"], "test-json-show");
    assert_eq!(issue_out["title"], "JSON Show Issue");
}

#[test]
fn test_add_issue_json() {
    let env = TestEnv::setup();
    let output = env
        .pebble()
        .args([
            "add",
            "New JSON Issue",
            "--description",
            "JSON description",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let issue: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");
    assert_eq!(issue["title"], "New JSON Issue");
    assert_eq!(issue["description"], "JSON description");
    assert!(
        issue["id"]
            .as_str()
            .unwrap_or_default()
            .starts_with("issue-")
    );
}

#[test]
fn test_update_issue_json() {
    let env = TestEnv::setup();
    let output = env
        .pebble()
        .args(["add", "Edit JSON Issue"])
        .output()
        .expect("Failed to run add");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout
        .split_whitespace()
        .last()
        .expect("Failed to get ID from output");

    let output = env
        .pebble()
        .args(["update", id, "--title", "JSON Edited Title", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let issue: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");
    assert_eq!(issue["id"], id);
    assert_eq!(issue["title"], "JSON Edited Title");
}

#[test]
fn test_config_get_sync_branch_json() {
    let env = TestEnv::setup();
    let output = env
        .pebble()
        .args(["config", "get", "sync-branch", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let data: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");
    assert_eq!(data["key"], "sync-branch");
    assert_eq!(data["value"], TEST_SYNC_BRANCH);
}

#[test]
fn test_sync_json() {
    let env = TestEnv::setup();
    let origin_dir = TempDir::new().unwrap();

    std::process::Command::new("git")
        .args(["init", "--bare"])
        .current_dir(origin_dir.path())
        .status()
        .unwrap();

    std::process::Command::new("git")
        .args([
            "remote",
            "add",
            "origin",
            origin_dir.path().to_str().unwrap(),
        ])
        .current_dir(env.root())
        .status()
        .unwrap();

    std::process::Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(env.root())
        .status()
        .unwrap();

    std::process::Command::new("git")
        .args(["push", "-u", "origin", TEST_SYNC_BRANCH])
        .current_dir(env.root())
        .status()
        .unwrap();

    let output = env
        .pebble()
        .args(["sync", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let data: serde_json::Value =
        serde_json::from_str(&json_str).expect("Failed to parse JSON output");
    assert_eq!(data["status"], "ok");
}
