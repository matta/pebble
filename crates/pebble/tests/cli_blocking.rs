#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
pub mod support;

use serde_json::Value;
use support::{setup_test_env, TaskBuilder};

#[test]
fn test_show_json_blocking_field_contains_direct_non_terminal_dependents() {
    let env = setup_test_env();

    TaskBuilder::new("PROJ-A").title("Task A").status("todo").write(&env.tasks_dir);
    TaskBuilder::new("PROJ-B").title("Task B").status("todo").needs(&["PROJ-A"]).write(&env.tasks_dir);
    TaskBuilder::new("PROJ-C").title("Task C").status("done").needs(&["PROJ-A"]).write(&env.tasks_dir);

    let output = env
        .pebble()
        .args(["show", "PROJ-A", "--json", "--dir", "tasks"])
        .output()
        .expect("show command should execute successfully");

    assert!(
        output.status.success(),
        "show failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let blocking: Vec<&str> = value["blocking"]
        .as_array()
        .expect("blocking should be an array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    assert!(
        blocking.contains(&"PROJ-B"),
        "blocking should contain PROJ-B (non-terminal dependent), got: {blocking:?}"
    );
    assert!(
        !blocking.contains(&"PROJ-C"),
        "blocking should not contain PROJ-C (terminal/done dependent), got: {blocking:?}"
    );
}

#[test]
fn test_list_sort_blocking_uses_transitive_count() {
    let env = setup_test_env();

    // A blocks B, B blocks C → A has transitive blocking count 2, B has 1, C has 0
    TaskBuilder::new("PROJ-A").title("Task A").status("todo").write(&env.tasks_dir);
    TaskBuilder::new("PROJ-B").title("Task B").status("todo").needs(&["PROJ-A"]).write(&env.tasks_dir);
    TaskBuilder::new("PROJ-C").title("Task C").status("todo").needs(&["PROJ-B"]).write(&env.tasks_dir);

    let output = env
        .pebble()
        .args(["list", "--json", "--dir", "tasks", "--sort", "-blocking"])
        .output()
        .expect("list command should execute successfully");

    assert!(
        output.status.success(),
        "list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("tasks should be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    assert_eq!(ids, vec!["PROJ-A", "PROJ-B", "PROJ-C"]);
}

#[test]
fn test_list_default_order_blocking_count_breaks_ties() {
    let env = setup_test_env();

    // X blocks nothing (count 0), Y blocks D (count 1)
    TaskBuilder::new("PROJ-X").title("Task X").status("todo").write(&env.tasks_dir);
    TaskBuilder::new("PROJ-Y").title("Task Y").status("todo").write(&env.tasks_dir);
    TaskBuilder::new("PROJ-D").title("Task D").status("todo").needs(&["PROJ-Y"]).write(&env.tasks_dir);

    let output = env
        .pebble()
        .args(["list", "--json", "--dir", "tasks"])
        .output()
        .expect("list command should execute successfully");

    assert!(
        output.status.success(),
        "list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("tasks should be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    let y_pos = ids
        .iter()
        .position(|&id| id == "PROJ-Y")
        .expect("PROJ-Y should be in the list");
    let x_pos = ids
        .iter()
        .position(|&id| id == "PROJ-X")
        .expect("PROJ-X should be in the list");

    assert!(
        y_pos < x_pos,
        "PROJ-Y (blocking count 1) should appear before PROJ-X (blocking count 0), got order: {ids:?}"
    );
}

#[test]
fn test_next_promotes_blocker_of_higher_priority_downstream_work() {
    let env = setup_test_env();

    TaskBuilder::new("PROJ-BLOCKER")
        .title("Blocker")
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-URGENT")
        .title("Urgent blocked task")
        .needs(&["PROJ-BLOCKER"])
        .priority(0)
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-OTHER")
        .title("Other ready task")
        .priority(1)
        .write(&env.tasks_dir);

    let output = env
        .pebble()
        .args(["next", "--json", "--dir", "tasks"])
        .output()
        .expect("next command should execute successfully");

    assert!(
        output.status.success(),
        "next failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(value["id"].as_str(), Some("PROJ-BLOCKER"));
}

#[test]
fn test_list_is_ready_uses_effective_priority_then_base_priority() {
    let env = setup_test_env();

    TaskBuilder::new("PROJ-BLOCKER")
        .title("Blocker")
        .priority(5)
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-DIRECT")
        .title("Direct P2 task")
        .priority(2)
        .write(&env.tasks_dir);

    TaskBuilder::new("PROJ-DOWNSTREAM")
        .title("Downstream P2 task")
        .needs(&["PROJ-BLOCKER"])
        .priority(2)
        .write(&env.tasks_dir);

    let output = env
        .pebble()
        .args(["list", "--json", "--dir", "tasks", "--is-ready"])
        .output()
        .expect("list command should execute successfully");

    assert!(
        output.status.success(),
        "list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("tasks should be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();

    let direct_pos = ids
        .iter()
        .position(|&id| id == "PROJ-DIRECT")
        .expect("PROJ-DIRECT should be present");
    let blocker_pos = ids
        .iter()
        .position(|&id| id == "PROJ-BLOCKER")
        .expect("PROJ-BLOCKER should be present");

    assert!(
        direct_pos < blocker_pos,
        "When effective priority ties at P2, base priority should break ties: {ids:?}"
    );
}
