use predicates::prelude::*;

mod cli_support;
#[path = "cli_support/issues.rs"]
mod cli_support_issues;
mod common;
use cli_support::TestEnv;
use cli_support_issues::{TestEnvIssues, create_test_issue};

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
fn test_list_filters() {
    let env = TestEnv::setup();
    // Add multiple issues
    let issue1 = create_test_issue("list-1", "Open Issue");
    env.add_issue_to_worktree(&issue1); // status: open, priority: 0

    let mut issue2 = create_test_issue("list-2", "Closed Issue");
    issue2["status"] = serde_json::json!("closed");
    issue2["priority"] = serde_json::json!(1);
    issue2["issue_type"] = serde_json::json!("bug");
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

    // Filter by type
    env.pebble()
        .args(["list", "--type", "bug"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list-2"))
        .stdout(predicate::str::contains("list-1").not());
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
fn test_search_filters() {
    let env = TestEnv::setup();
    let issue = create_test_issue("search-filter-1", "Filter Me");
    env.add_issue_to_worktree(&issue);

    let mut issue2 = create_test_issue("search-filter-2", "Filter Me Too");
    issue2["status"] = serde_json::json!("closed");
    issue2["priority"] = serde_json::json!(2);
    issue2["issue_type"] = serde_json::json!("bug");
    issue2["owner"] = serde_json::json!("other@example.com");
    env.add_issue_to_worktree(&issue2);

    env.pebble()
        .args(["search", "Filter", "--status", "closed"])
        .assert()
        .success()
        .stdout(predicate::str::contains("search-filter-2"))
        .stdout(predicate::str::contains("search-filter-1").not());

    env.pebble()
        .args(["search", "Filter", "--owner", "test@example.com"])
        .assert()
        .success()
        .stdout(predicate::str::contains("search-filter-1"))
        .stdout(predicate::str::contains("search-filter-2").not());

    env.pebble()
        .args(["search", "Filter", "--priority", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("search-filter-2"))
        .stdout(predicate::str::contains("search-filter-1").not());

    env.pebble()
        .args(["search", "Filter", "--type", "task"])
        .assert()
        .success()
        .stdout(predicate::str::contains("search-filter-1"))
        .stdout(predicate::str::contains("search-filter-2").not());
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
