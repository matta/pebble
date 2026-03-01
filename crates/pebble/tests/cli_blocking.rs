#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod support;

use serde_json::Value;
use std::fs;
use std::path::Path;
use support::setup_test_env;

fn write_task_with_needs(tasks_dir: &Path, id: &str, title: &str, status: &str, needs: &[&str]) {
    let needs_str = needs
        .iter()
        .map(|n| format!("\"{n}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let content = format!(
        "---\nid: \"{id}\"\ntitle: \"{title}\"\nstatus: \"{status}\"\ncreated_at: \"2024-01-01T00:00:00Z\"\nneeds: [{needs_str}]\n---\nBody\n"
    );
    fs::write(tasks_dir.join(format!("{id}.md")), content).expect("task file should be written");
}

#[test]
fn test_show_json_blocking_field_contains_direct_non_terminal_dependents() {
    let env = setup_test_env();

    write_task_with_needs(&env.tasks_dir, "PROJ-A", "Task A", "todo", &[]);
    write_task_with_needs(&env.tasks_dir, "PROJ-B", "Task B", "todo", &["PROJ-A"]);
    write_task_with_needs(&env.tasks_dir, "PROJ-C", "Task C", "done", &["PROJ-A"]);

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
    write_task_with_needs(&env.tasks_dir, "PROJ-A", "Task A", "todo", &[]);
    write_task_with_needs(&env.tasks_dir, "PROJ-B", "Task B", "todo", &["PROJ-A"]);
    write_task_with_needs(&env.tasks_dir, "PROJ-C", "Task C", "todo", &["PROJ-B"]);

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
    write_task_with_needs(&env.tasks_dir, "PROJ-X", "Task X", "todo", &[]);
    write_task_with_needs(&env.tasks_dir, "PROJ-Y", "Task Y", "todo", &[]);
    write_task_with_needs(&env.tasks_dir, "PROJ-D", "Task D", "todo", &["PROJ-Y"]);

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
