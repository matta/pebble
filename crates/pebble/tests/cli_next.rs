use crate::support::{setup_test_env, write_task};
use serde_json::Value;

mod support;

#[test]
fn test_next_with_limit() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-1", "Task 1", "todo");
    write_task(&env.tasks_dir, "PROJ-2", "Task 2", "todo");
    write_task(&env.tasks_dir, "PROJ-3", "Task 3", "todo");

    // Default next should return a wrapped list of one task
    let output = env
        .pebble()
        .args(["next", "--json"])
        .output()
        .expect("next command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert!(value.is_object(), "Next should return a wrapped object");
    let tasks = value["tasks"].as_array().expect("tasks should be an array");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"].as_str(), Some("PROJ-1"));

    // Next with limit 2 should return two tasks (wrapped)
    let output = env
        .pebble()
        .args(["next", "--limit", "2", "--json"])
        .output()
        .expect("next command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert!(value.is_object(), "Next with limit > 1 should return a wrapped object");
    let tasks = value["tasks"].as_array().expect("tasks should be an array");
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["id"].as_str(), Some("PROJ-1"));
    assert_eq!(tasks[1]["id"].as_str(), Some("PROJ-2"));
}

#[test]
fn test_next_with_limit_human_readable() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-1", "Task 1", "todo");
    write_task(&env.tasks_dir, "PROJ-2", "Task 2", "todo");

    let output = env
        .pebble()
        .args(["next", "--limit", "2"])
        .output()
        .expect("next command should execute successfully");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PROJ-1 Task 1"));
    assert!(stdout.contains("PROJ-2 Task 2"));
}
