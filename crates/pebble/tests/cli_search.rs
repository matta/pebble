#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod support;

use serde_json::Value;
use std::fs;
use std::path::Path;
use support::{setup_test_env, write_task};

fn write_task_with_body(tasks_dir: &Path, id: &str, title: &str, status: &str, body: &str) {
    let content = format!(
        "---\nid: \"{id}\"\ntitle: \"{title}\"\nstatus: \"{status}\"\ncreated_at: \"2024-01-01T00:00:00Z\"\n---\n{body}\n"
    );
    fs::write(tasks_dir.join(format!("{id}.md")), content).expect("task file should be written");
}

#[test]
fn test_search_matches_title_and_body_case_insensitive() {
    let env = setup_test_env();

    write_task_with_body(
        &env.tasks_dir,
        "PROJ-TITLE",
        "Implement Search",
        "todo",
        "no match here",
    );
    write_task_with_body(
        &env.tasks_dir,
        "PROJ-BODY",
        "Other Task",
        "todo",
        "Need to handle SEARCH parsing",
    );
    write_task_with_body(&env.tasks_dir, "PROJ-NOPE", "Unrelated", "todo", "random");

    let output = env
        .pebble()
        .args(["search", "search", "--json", "--dir", "tasks"])
        .output()
        .expect("pebble command should execute successfully");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let mut ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("tasks should be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();
    ids.sort();
    assert_eq!(ids, vec!["PROJ-BODY", "PROJ-TITLE"]);
}

#[test]
fn test_search_uses_default_list_ordering() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-A", "Task A", "todo");
    write_task_with_body(&env.tasks_dir, "PROJ-B", "Task B", "todo", "needs PROJ-A");

    // Make PROJ-B depend on PROJ-A so default ordering must return A before B.
    let b_path = env.tasks_dir.join("PROJ-B.md");
    let b_content = fs::read_to_string(&b_path)
        .expect("PROJ-B task file should be readable")
        .replace(
            "created_at: \"2024-01-01T00:00:00Z\"\n",
            "created_at: \"2024-01-01T00:00:00Z\"\nneeds: [\"PROJ-A\"]\n",
        );
    fs::write(&b_path, b_content).expect("task B file should be written");
    let output = env
        .pebble()
        .args(["search", "task", "--json", "--dir", "tasks"])
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
    assert_eq!(ids, vec!["PROJ-A", "PROJ-B"]);
}

fn assert_search_no_match(json: bool) {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-A", "Task A", "todo");

    let mut cmd = env.pebble();
    cmd.args(["search", "nonexistent-query", "--dir", "tasks"]);

    if json {
        cmd.arg("--json");
    }

    let output = cmd.output().expect("pebble command should execute");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No tasks found matching query 'nonexistent-query'"));
    assert!(output.stdout.is_empty());
}

#[test]
fn test_search_no_match() {
    assert_search_no_match(false);
}

#[test]
fn test_search_no_match_json() {
    assert_search_no_match(true);
}
