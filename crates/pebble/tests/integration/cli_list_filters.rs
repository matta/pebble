#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]

use crate::support::{TaskBuilder, setup_test_env, write_task};
use serde_json::Value;

#[test]
fn test_list_status_filter_includes_done_without_all() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-TODO", "Todo Task", "todo");
    write_task(&env.tasks_dir, "PROJ-DONE", "Done Task", "done");

    let output = env
        .pebble()
        .args(["list", "--json", "--dir", "tasks", "--status", "done"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let tasks = value["tasks"].as_array().expect("tasks should be an array");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"].as_str(), Some("PROJ-DONE"));
}

#[test]
fn test_list_tag_filter_requires_all_tags() {
    let env = setup_test_env();

    TaskBuilder::new("PROJ-BOTH")
        .title("Both Tags")
        .tags(&["backend", "urgent"])
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-BACKEND")
        .title("Backend Only")
        .tags(&["backend"])
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-URGENT")
        .title("Urgent Only")
        .tags(&["urgent"])
        .write(&env.tasks_dir);

    let output = env
        .pebble()
        .args([
            "list", "--json", "--dir", "tasks", "--tag", "backend", "--tag", "urgent",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let tasks = value["tasks"].as_array().expect("tasks should be an array");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"].as_str(), Some("PROJ-BOTH"));
}

#[test]
fn test_list_need_filter_matches_any_selected_need() {
    let env = setup_test_env();

    TaskBuilder::new("PROJ-NEEDS-A")
        .title("Needs A")
        .needs(&["DEP-A"])
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-NEEDS-B")
        .title("Needs B")
        .needs(&["DEP-B"])
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-NEEDS-C")
        .title("Needs C")
        .needs(&["DEP-C"])
        .write(&env.tasks_dir);

    let output = env
        .pebble()
        .args([
            "list", "--json", "--dir", "tasks", "--need", "DEP-A", "--need", "DEP-B",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let mut ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("tasks should be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();
    ids.sort();
    assert_eq!(ids, vec!["PROJ-NEEDS-A", "PROJ-NEEDS-B"]);
}

#[test]
fn test_list_priority_filter_matches_any_selected_priority() {
    let env = setup_test_env();

    TaskBuilder::new("PROJ-P1")
        .title("Priority 1")
        .priority(1)
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-P2")
        .title("Priority 2")
        .priority(2)
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-P3")
        .title("Priority 3")
        .priority(3)
        .write(&env.tasks_dir);

    let output = env
        .pebble()
        .args([
            "list",
            "--json",
            "--dir",
            "tasks",
            "--priority",
            "1",
            "--priority",
            "2",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let mut ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("tasks should be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();
    ids.sort();
    assert_eq!(ids, vec!["PROJ-P1", "PROJ-P2"]);
}

#[test]
fn test_list_all_includes_closed_tasks() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-TODO", "Todo Task", "todo");
    write_task(&env.tasks_dir, "PROJ-DONE", "Done Task", "done");
    write_task(&env.tasks_dir, "PROJ-CANCELED", "Canceled Task", "canceled");

    let output = env
        .pebble()
        .args(["list", "--json", "--dir", "tasks", "--all"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("tasks should be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    assert!(ids.contains(&"PROJ-TODO"));
    assert!(ids.contains(&"PROJ-DONE"));
    assert!(ids.contains(&"PROJ-CANCELED"));
}

#[test]
fn test_list_is_ready_filters_only_ready_tasks() {
    let env = setup_test_env();

    TaskBuilder::new("PROJ-DONE-DEP")
        .title("Done Dependency")
        .status("done")
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-READY")
        .title("Ready Task")
        .needs(&["PROJ-DONE-DEP"])
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-BLOCKED")
        .title("Blocked Task")
        .needs(&["PROJ-MISSING"])
        .write(&env.tasks_dir);

    let output = env
        .pebble()
        .args(["list", "--json", "--dir", "tasks", "--is-ready"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let tasks = value["tasks"].as_array().expect("tasks should be an array");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"].as_str(), Some("PROJ-READY"));
}

#[test]
fn test_list_limit_restricts_number_of_rows() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-A", "Task A", "todo");
    write_task(&env.tasks_dir, "PROJ-B", "Task B", "todo");
    write_task(&env.tasks_dir, "PROJ-C", "Task C", "todo");

    let output = env
        .pebble()
        .args(["list", "--json", "--dir", "tasks", "--limit", "2"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let tasks = value["tasks"].as_array().expect("tasks should be an array");
    assert_eq!(tasks.len(), 2);
}

#[test]
fn test_list_status_filter_uses_or_semantics() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-TODO", "Todo Task", "todo");
    write_task(&env.tasks_dir, "PROJ-DONE", "Done Task", "done");
    write_task(&env.tasks_dir, "PROJ-CANCELED", "Canceled Task", "canceled");

    let output = env
        .pebble()
        .args([
            "list", "--json", "--dir", "tasks", "--status", "todo", "--status", "done",
        ])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let mut ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("tasks should be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();
    ids.sort();

    assert_eq!(ids, vec!["PROJ-DONE", "PROJ-TODO"]);
}

#[test]
fn test_ls_alias_matches_list_output() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-A", "Task A", "todo");
    write_task(&env.tasks_dir, "PROJ-B", "Task B", "todo");

    let list_output = env
        .pebble()
        .args(["list", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");
    assert!(list_output.status.success());

    let ls_output = env
        .pebble()
        .args(["ls", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");
    assert!(ls_output.status.success());

    let list_value: Value =
        serde_json::from_slice(&list_output.stdout).expect("stdout should be valid JSON");
    let ls_value: Value =
        serde_json::from_slice(&ls_output.stdout).expect("stdout should be valid JSON");
    assert_eq!(ls_value, list_value);
}
