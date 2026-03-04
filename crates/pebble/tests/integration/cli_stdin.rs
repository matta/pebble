#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]

use super::support::setup_test_env;
use serde_json::Value;

#[test]
fn test_add_body_from_stdin() {
    let env = setup_test_env();

    let output = env
        .pebble()
        .args(["add", "Task from Stdin", "--body", "-", "--json"])
        .write_stdin("This is body content from stdin")
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    assert_eq!(
        json["body"], "This is body content from stdin",
        "Body should be read from stdin when '-' is provided"
    );
}

#[test]
fn test_update_append_body_from_stdin() {
    let env = setup_test_env();

    // Create initial task
    let output = env
        .pebble()
        .args(["add", "Initial Task", "--body", "Initial body", "--json"])
        .output()
        .expect("pebble add should succeed");
    let json: Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("valid json");
    let id = json["id"].as_str().expect("id exists");

    // Update with append-body from stdin
    let output = env
        .pebble()
        .args(["update", id, "--append-body", "-", "--json"])
        .write_stdin("Appended from stdin")
        .output()
        .expect("pebble update should succeed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    assert_eq!(
        json["body"], "Initial body\n\nAppended from stdin",
        "Body should have content appended from stdin when '-' is provided"
    );
}

#[test]
fn test_update_body_and_append_body_both_stdin() {
    let env = setup_test_env();

    // Create initial task
    let output = env
        .pebble()
        .args(["add", "Initial Task", "--body", "Initial body", "--json"])
        .output()
        .expect("pebble add should succeed");
    let json: Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).expect("valid json");
    let id = json["id"].as_str().expect("id exists");

    // Update with both from stdin. body takes precedence in logic,
    // and stdin can only be read once.
    let output = env
        .pebble()
        .args(["update", id, "--body", "-", "--append-body", "-", "--json"])
        .write_stdin("New body from stdin")
        .output()
        .expect("pebble update should succeed");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let json: Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    // mutations.body should win
    assert_eq!(
        json["body"], "New body from stdin",
        "Body from stdin should win and consume stdin"
    );
}
