mod support;

use assert_cmd::cargo_bin;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use support::setup_test_env;

struct SortTask<'a> {
    id: &'a str,
    title: &'a str,
    status: &'a str,
    created_at: &'a str,
    priority: Option<u8>,
}

fn write_task_with_created_at(tasks_dir: &Path, task: SortTask<'_>) {
    let mut frontmatter = format!(
        "id = \"{}\"\ntitle = \"{}\"\nstatus = \"{}\"\ncreated_at = {}\n",
        task.id, task.title, task.status, task.created_at
    );
    if let Some(value) = task.priority {
        frontmatter.push_str(&format!("priority = {value}\n"));
    }
    let content = format!("+++\n{frontmatter}+++\nBody\n");
    fs::write(tasks_dir.join(format!("{}.md", task.id)), content)
        .expect("Failed to write task file");
}

#[test]
fn test_list_sort_title_descending() {
    let env = setup_test_env();

    write_task_with_created_at(
        &env.tasks_dir,
        SortTask {
            id: "PROJ-A",
            title: "Alpha",
            status: "todo",
            created_at: "2024-01-01T00:00:00Z",
            priority: None,
        },
    );
    write_task_with_created_at(
        &env.tasks_dir,
        SortTask {
            id: "PROJ-B",
            title: "Beta",
            status: "todo",
            created_at: "2024-01-01T00:00:00Z",
            priority: None,
        },
    );

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["list", "--json", "--dir", "tasks", "--sort", "-title"])
        .output()
        .expect("Failed to execute list command with descending title sort");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("Failed to parse JSON output");
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("Expected 'tasks' to be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();
    assert_eq!(ids, vec!["PROJ-B", "PROJ-A"]);
}

#[test]
fn test_list_sort_priority_uses_created_at_then_id_tiebreakers() {
    let env = setup_test_env();

    write_task_with_created_at(
        &env.tasks_dir,
        SortTask {
            id: "PROJ-B",
            title: "Task B",
            status: "todo",
            created_at: "2024-01-02T00:00:00Z",
            priority: Some(5),
        },
    );
    write_task_with_created_at(
        &env.tasks_dir,
        SortTask {
            id: "PROJ-A",
            title: "Task A",
            status: "todo",
            created_at: "2024-01-01T00:00:00Z",
            priority: Some(5),
        },
    );
    write_task_with_created_at(
        &env.tasks_dir,
        SortTask {
            id: "PROJ-C",
            title: "Task C",
            status: "todo",
            created_at: "2024-01-01T00:00:00Z",
            priority: Some(5),
        },
    );

    let output = Command::new(cargo_bin!())
        .current_dir(&env.root)
        .args(["list", "--json", "--dir", "tasks", "--sort", "priority"])
        .output()
        .expect("Failed to execute list command with priority sort");

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("Failed to parse JSON output");
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .expect("Expected 'tasks' to be an array")
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();
    assert_eq!(ids, vec!["PROJ-A", "PROJ-C", "PROJ-B"]);
}
