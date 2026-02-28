#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod support;

use serde_json::Value;
use support::setup_test_env;

#[test]
fn test_cli_add_need_renaming() {
    let env = setup_test_env();

    let output = env
        .pebble()
        .args([
            "add",
            "Child Task",
            "--need",
            "parent",
            "--json",
            "--dir",
            "tasks",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(
        output.status.success(),
        "pebble add --need failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let needs = value
        .get("needs")
        .expect("needs should be present")
        .as_array()
        .expect("needs should be an array");
    assert_eq!(needs[0], "parent");
}

#[test]
fn test_cli_update_add_need_renaming() {
    let env = setup_test_env();

    // Create a task to update
    let output = env
        .pebble()
        .args(["add", "Task", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let task_id = value["id"].as_str().expect("id should be a string");

    let output = env
        .pebble()
        .args([
            "update",
            task_id,
            "--add-need",
            "another-parent",
            "--json",
            "--dir",
            "tasks",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let needs = value["needs"].as_array().expect("needs should be an array");
    assert!(needs.iter().any(|v| v.as_str() == Some("another-parent")));
}

#[test]
fn test_cli_update_remove_need_renaming() {
    let env = setup_test_env();

    // Create a task with a need
    let output = env
        .pebble()
        .args([
            "add", "Task", "--need", "parent", "--json", "--dir", "tasks",
        ])
        .output()
        .expect("pebble command should execute successfully");
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let task_id = value["id"].as_str().expect("id should be a string");

    let output = env
        .pebble()
        .args([
            "update",
            task_id,
            "--remove-need",
            "parent",
            "--json",
            "--dir",
            "tasks",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let needs = value["needs"].as_array().expect("needs should be an array");
    assert!(needs.is_empty());
}

#[test]
fn test_cli_computed_blocking_fields() {
    let env = setup_test_env();

    // Create a child task
    let output = env
        .pebble()
        .args(["add", "Child Task", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let child_id = value
        .get("id")
        .expect("id should be present")
        .as_str()
        .expect("id should be a string")
        .to_string();

    // Create a parent task
    let output = env
        .pebble()
        .args(["add", "Parent Task", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let parent_id = value
        .get("id")
        .expect("id should be present")
        .as_str()
        .expect("id should be a string")
        .to_string();

    // Now update 'child' to need this new parent
    env.pebble()
        .args([
            "update",
            &child_id,
            "--add-need",
            &parent_id,
            "--dir",
            "tasks",
        ])
        .assert()
        .success();

    // 'child' needs parent_id. parent_id is todo.
    // So 'child' should be blocked_by parent_id
    let output = env
        .pebble()
        .args(["show", &child_id, "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let blocked_by = value
        .get("blocked_by")
        .expect("blocked_by should be present")
        .as_array()
        .expect("blocked_by should be an array");
    assert!(blocked_by.iter().any(|v| v.as_str() == Some(&parent_id)));

    // parent_id should be blocking 'child'
    let output = env
        .pebble()
        .args(["show", &parent_id, "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let blocking = value
        .get("blocking")
        .expect("blocking should be present")
        .as_array()
        .expect("blocking should be an array");
    assert!(blocking.iter().any(|v| v.as_str() == Some(&child_id)));
}

#[test]
fn test_update_blocks_roundtrip() {
    let env = setup_test_env();

    let source_id = add_task(&env, "Source Task");
    let target_id = add_task(&env, "Target Task");

    let output = env
        .pebble()
        .args([
            "update", &source_id, "--blocks", &target_id, "--json", "--dir", "tasks",
        ])
        .output()
        .expect("pebble command should execute successfully");
    assert!(output.status.success());

    let output = env
        .pebble()
        .args(["show", &target_id, "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");
    assert!(output.status.success());
    let target_after_add: Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let needs = target_after_add["needs"]
        .as_array()
        .expect("needs should be an array");
    assert!(
        needs.iter().any(|v| v.as_str() == Some(&source_id)),
        "Expected target needs to include source ID after --blocks"
    );
}

#[test]
fn test_update_remove_blocks_roundtrip() {
    let env = setup_test_env();

    let source_id = add_task(&env, "Source Task");
    let target_id = add_task(&env, "Target Task");

    // Setup: source blocks target
    env.pebble()
        .args([
            "update", &source_id, "--blocks", &target_id, "--dir", "tasks",
        ])
        .assert()
        .success();

    let output = env
        .pebble()
        .args([
            "update",
            &source_id,
            "--remove-blocks",
            &target_id,
            "--json",
            "--dir",
            "tasks",
        ])
        .output()
        .expect("pebble command should execute successfully");
    assert!(output.status.success());

    let output = env
        .pebble()
        .args(["show", &target_id, "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");
    assert!(output.status.success());
    let target_after_remove: Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let needs = target_after_remove["needs"]
        .as_array()
        .expect("needs should be an array");
    assert!(
        !needs.iter().any(|v| v.as_str() == Some(&source_id)),
        "Expected target needs to exclude source ID after --remove-blocks"
    );
}

fn add_task(env: &support::TestEnv, title: &str) -> String {
    let output = env
        .pebble()
        .args(["add", title, "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");
    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    json["id"]
        .as_str()
        .expect("id should be a string")
        .to_string()
}

#[test]
fn test_update_blocks_fails_for_unknown_target_id() {
    let env = setup_test_env();

    let output = env
        .pebble()
        .args(["add", "Source Task", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");
    assert!(output.status.success());
    let source_json: Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let source_id = source_json["id"]
        .as_str()
        .expect("id should be a string")
        .to_string();

    let output = env
        .pebble()
        .args([
            "update",
            &source_id,
            "--blocks",
            "PROJ-missing",
            "--dir",
            "tasks",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(
        !output.status.success(),
        "Expected update --blocks with missing target to fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found"),
        "Expected missing-target error, got: {stderr}"
    );
}
