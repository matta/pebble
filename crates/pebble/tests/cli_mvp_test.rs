use assert_cmd::Command;
use assert_cmd::cargo_bin;

mod cli_support;
mod common;
use cli_support::TestEnv;

#[test]
fn test_add_full_fields_json() {
    run_add_full_fields_json();
}

#[allow(clippy::cognitive_complexity)]
fn run_add_full_fields_json() {
    let env = TestEnv::setup();
    let output = env
        .pebble()
        .args([
            "add",
            "Full Issue",
            "--description",
            "Has everything",
            "--status",
            "in_progress",
            "--priority",
            "1",
            "--type",
            "bug",
            "--owner",
            "me@example.com",
            "--acceptance-criteria",
            "Must work",
            "--defer-until",
            "2024-01-01",
            "--label",
            "frontend",
            "--label",
            "urgent",
            "--note",
            "First note",
            "--note",
            "Second note",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let issue: serde_json::Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");

    assert_eq!(issue["title"], "Full Issue");
    assert_eq!(issue["status"], "in_progress");
    assert_eq!(issue["priority"], 1);
    assert_eq!(issue["issue_type"], "bug");
    assert_eq!(issue["owner"], "me@example.com");
    assert_eq!(issue["acceptance_criteria"], "Must work");
    assert_eq!(issue["defer_until"], "2024-01-01");

    let labels = issue["labels"].as_array().expect("labels should be array");
    assert_eq!(labels.len(), 2);
    // Labels might be sorted or not, depending on implementation. `add` doesn't sort explicitly?
    // `store` serialization might not sort, but `update` sorts.
    // Let's check containment.
    let labels_vec: Vec<&str> = labels.iter().map(|v| v.as_str().unwrap()).collect();
    assert!(labels_vec.contains(&"frontend"));
    assert!(labels_vec.contains(&"urgent"));

    let notes = issue["notes"].as_array().expect("notes should be array");
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0], "First note");
    assert_eq!(notes[1], "Second note");
}

#[test]
fn test_update_incremental_fields() {
    let env = TestEnv::setup();
    // 1. Add issue
    let output = env
        .pebble()
        .args([
            "add",
            "Update Test",
            "--label",
            "old",
            "--note",
            "old note",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let issue: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();
    let id = issue["id"].as_str().unwrap();

    // 2. Update
    let output = env
        .pebble()
        .args([
            "update",
            id,
            "--add-label",
            "new",
            "--remove-label",
            "old",
            "--add-note",
            "new note",
            "--acceptance-criteria",
            "New AC",
            "--defer-until",
            "2025-01-01",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let issue: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();

    assert_eq!(issue["acceptance_criteria"], "New AC");
    assert_eq!(issue["defer_until"], "2025-01-01");

    let labels = issue["labels"].as_array().unwrap();
    let labels_vec: Vec<&str> = labels.iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(labels_vec.len(), 1);
    assert_eq!(labels_vec[0], "new"); // "old" removed, "new" added

    let notes = issue["notes"].as_array().unwrap();
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0], "old note");
    assert_eq!(notes[1], "new note");
}

#[test]
fn test_list_json() {
    let env = TestEnv::setup();
    env.pebble()
        .args(["add", "Issue 1", "--status", "open"])
        .assert()
        .success();

    // Add Issue 2 then close it
    let output = env
        .pebble()
        .args(["add", "Issue 2"])
        .output()
        .expect("add failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Output format: "Added issue issue-..."
    let id = stdout.split_whitespace().last().unwrap();

    env.pebble()
        .args(["update", id, "--status", "closed", "--close-reason", "done"])
        .assert()
        .success();

    let output = env
        .pebble()
        .args(["list", "--status", "open", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let issues: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();
    let arr = issues.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["title"], "Issue 1");
}

#[test]
fn test_search_json() {
    let env = TestEnv::setup();
    env.pebble().args(["add", "Find Me"]).assert().success();
    env.pebble().args(["add", "Hide Me"]).assert().success();

    let output = env
        .pebble()
        .args(["search", "Find", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let issues: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output).unwrap()).unwrap();
    let arr = issues.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["title"], "Find Me");
}

#[test]
fn test_help_json() {
    let mut cmd = Command::new(cargo_bin!("pebble"));
    let output = cmd
        .arg("--help-json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8(output).unwrap()).expect("Valid JSON");
    assert_eq!(json["name"], "pebble");
    assert!(!json["commands"].as_array().unwrap().is_empty());
}
