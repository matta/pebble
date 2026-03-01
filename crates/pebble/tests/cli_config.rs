#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod support;

use serde_json::Value;
use support::setup_test_env;

#[test]
fn test_config_get_issue_prefix_human_output() {
    let env = setup_test_env();

    let output = env
        .pebble()
        .args(["config", "get", "issue-prefix"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "PROJ\n");
}

#[test]
fn test_config_get_tasks_dir_json_shape() {
    let env = setup_test_env();

    let output = env
        .pebble()
        .args(["config", "get", "tasks-dir", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(value["key"].as_str(), Some("tasks-dir"));
    assert_eq!(value["value"].as_str(), Some("tasks"));
}

#[test]
fn test_config_get_unknown_key_is_usage_error() {
    let env = setup_test_env();

    let output = env
        .pebble()
        .args(["config", "get", "not-a-key", "--json"])
        .output()
        .expect("pebble command should execute successfully");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown config key"));
    assert!(stderr.contains("not-a-key"));
}
