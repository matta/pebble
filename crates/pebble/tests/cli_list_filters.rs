mod support;

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use support::{setup_test_env, write_task};

struct CustomTask<'a> {
    id: &'a str,
    title: &'a str,
    status: &'a str,
    priority: Option<u8>,
    needs: &'a [&'a str],
    tags: &'a [&'a str],
}

fn write_task_custom(tasks_dir: &std::path::Path, task: CustomTask<'_>) {
    let mut frontmatter = format!(
        "id = \"{}\"\ntitle = \"{}\"\nstatus = \"{}\"\ncreated_at = 2024-01-01T00:00:00Z\n",
        task.id, task.title, task.status
    );

    if let Some(value) = task.priority {
        frontmatter.push_str(&format!("priority = {value}\n"));
    }

    if !task.needs.is_empty() {
        let values = task
            .needs
            .iter()
            .map(|v| format!("\"{v}\""))
            .collect::<Vec<_>>()
            .join(", ");
        frontmatter.push_str(&format!("needs = [{values}]\n"));
    }

    if !task.tags.is_empty() {
        let values = task
            .tags
            .iter()
            .map(|v| format!("\"{v}\""))
            .collect::<Vec<_>>()
            .join(", ");
        frontmatter.push_str(&format!("tags = [{values}]\n"));
    }

    let content = format!("+++\n{frontmatter}+++\nBody\n");
    fs::write(tasks_dir.join(format!("{}.md", task.id)), content).unwrap();
}

#[test]
fn test_list_status_filter_includes_done_without_all() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-TODO", "Todo Task", "todo");
    write_task(&env.tasks_dir, "PROJ-DONE", "Done Task", "done");

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["list", "--json", "--dir", "tasks", "--status", "done"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let tasks = value["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"].as_str(), Some("PROJ-DONE"));
}

#[test]
fn test_list_tag_filter_requires_all_tags() {
    let env = setup_test_env();

    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-BOTH",
            title: "Both Tags",
            status: "todo",
            priority: None,
            needs: &[],
            tags: &["backend", "urgent"],
        },
    );
    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-BACKEND",
            title: "Backend Only",
            status: "todo",
            priority: None,
            needs: &[],
            tags: &["backend"],
        },
    );
    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-URGENT",
            title: "Urgent Only",
            status: "todo",
            priority: None,
            needs: &[],
            tags: &["urgent"],
        },
    );

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args([
            "list", "--json", "--dir", "tasks", "--tag", "backend", "--tag", "urgent",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let tasks = value["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"].as_str(), Some("PROJ-BOTH"));
}

#[test]
fn test_list_need_filter_matches_any_selected_need() {
    let env = setup_test_env();

    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-NEEDS-A",
            title: "Needs A",
            status: "todo",
            priority: None,
            needs: &["DEP-A"],
            tags: &[],
        },
    );
    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-NEEDS-B",
            title: "Needs B",
            status: "todo",
            priority: None,
            needs: &["DEP-B"],
            tags: &[],
        },
    );
    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-NEEDS-C",
            title: "Needs C",
            status: "todo",
            priority: None,
            needs: &["DEP-C"],
            tags: &[],
        },
    );

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args([
            "list", "--json", "--dir", "tasks", "--need", "DEP-A", "--need", "DEP-B",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut ids: Vec<&str> = value["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();
    ids.sort();
    assert_eq!(ids, vec!["PROJ-NEEDS-A", "PROJ-NEEDS-B"]);
}

#[test]
fn test_list_priority_filter_matches_any_selected_priority() {
    let env = setup_test_env();

    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-P1",
            title: "Priority 1",
            status: "todo",
            priority: Some(1),
            needs: &[],
            tags: &[],
        },
    );
    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-P2",
            title: "Priority 2",
            status: "todo",
            priority: Some(2),
            needs: &[],
            tags: &[],
        },
    );
    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-P3",
            title: "Priority 3",
            status: "todo",
            priority: Some(3),
            needs: &[],
            tags: &[],
        },
    );

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
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
        .unwrap();

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut ids: Vec<&str> = value["tasks"]
        .as_array()
        .unwrap()
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

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["list", "--json", "--dir", "tasks", "--all"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let ids: Vec<&str> = value["tasks"]
        .as_array()
        .unwrap()
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

    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-DONE-DEP",
            title: "Done Dependency",
            status: "done",
            priority: None,
            needs: &[],
            tags: &[],
        },
    );
    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-READY",
            title: "Ready Task",
            status: "todo",
            priority: None,
            needs: &["PROJ-DONE-DEP"],
            tags: &[],
        },
    );
    write_task_custom(
        &env.tasks_dir,
        CustomTask {
            id: "PROJ-BLOCKED",
            title: "Blocked Task",
            status: "todo",
            priority: None,
            needs: &["PROJ-MISSING"],
            tags: &[],
        },
    );

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["list", "--json", "--dir", "tasks", "--is-ready"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let tasks = value["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"].as_str(), Some("PROJ-READY"));
}

#[test]
fn test_list_limit_restricts_number_of_rows() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-A", "Task A", "todo");
    write_task(&env.tasks_dir, "PROJ-B", "Task B", "todo");
    write_task(&env.tasks_dir, "PROJ-C", "Task C", "todo");

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args(["list", "--json", "--dir", "tasks", "--limit", "2"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let tasks = value["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 2);
}

#[test]
fn test_list_status_filter_uses_or_semantics() {
    let env = setup_test_env();

    write_task(&env.tasks_dir, "PROJ-TODO", "Todo Task", "todo");
    write_task(&env.tasks_dir, "PROJ-DONE", "Done Task", "done");
    write_task(&env.tasks_dir, "PROJ-CANCELED", "Canceled Task", "canceled");

    let output = Command::new(env!("CARGO_BIN_EXE_pebble"))
        .current_dir(&env.root)
        .args([
            "list", "--json", "--dir", "tasks", "--status", "todo", "--status", "done",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut ids: Vec<&str> = value["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|task| task["id"].as_str())
        .collect();
    ids.sort();

    assert_eq!(ids, vec!["PROJ-DONE", "PROJ-TODO"]);
}
