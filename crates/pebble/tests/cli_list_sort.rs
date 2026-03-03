#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
pub mod support;

use serde_json::Value;
use support::{TaskBuilder, setup_test_env};

#[test]
fn test_list_sort_title_descending() {
    let env = setup_test_env();

    TaskBuilder::new("PROJ-A")
        .title("Alpha")
        .status("todo")
        .created_at("2024-01-01T00:00:00Z")
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-B")
        .title("Beta")
        .status("todo")
        .created_at("2024-01-01T00:00:00Z")
        .write(&env.tasks_dir);

    let output = env
        .pebble()
        .args(["list", "--json", "--dir", "tasks", "--sort", "-title"])
        .output()
        .expect("list command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("tasks should be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();
    assert_eq!(ids, vec!["PROJ-B", "PROJ-A"]);
}

#[test]
fn test_list_sort_priority_uses_created_at_then_id_tiebreakers() {
    let env = setup_test_env();

    TaskBuilder::new("PROJ-B")
        .title("Task B")
        .status("todo")
        .created_at("2024-01-02T00:00:00Z")
        .priority(5)
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-A")
        .title("Task A")
        .status("todo")
        .created_at("2024-01-01T00:00:00Z")
        .priority(5)
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-C")
        .title("Task C")
        .status("todo")
        .created_at("2024-01-01T00:00:00Z")
        .priority(5)
        .write(&env.tasks_dir);

    let output = env
        .pebble()
        .args(["list", "--json", "--dir", "tasks", "--sort", "priority"])
        .output()
        .expect("list command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("tasks should be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();
    assert_eq!(ids, vec!["PROJ-A", "PROJ-C", "PROJ-B"]);
}
