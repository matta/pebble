#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
use super::*;
use crate::models::{Priority, TaskFrontmatter, TaskStatus};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use tempfile::tempdir;

fn make_test_node(id: &str, status: TaskStatus, needs: Vec<&str>) -> TaskNode {
    TaskNode {
        path: Path::new("").to_path_buf(),
        body: "".to_string(),
        frontmatter: TaskFrontmatter {
            id: id.to_string(),
            title: id.to_string(),
            status,
            priority: None,
            created_at: Some(
                DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
                    .expect("Failed to parse datetime")
                    .with_timezone(&Utc),
            ),
            modified_at: None,
            resolved_at: None,
            needs: needs.into_iter().map(|s| s.to_string()).collect(),
            tags: vec![],
            extra: BTreeMap::new(),
        },
    }
}

#[test]
fn test_absolute_readiness() {
    let mut nodes = HashMap::new();
    nodes.insert(
        "A".to_string(),
        make_test_node("A", TaskStatus::todo(), vec![]),
    );
    nodes.insert(
        "B".to_string(),
        make_test_node("B", TaskStatus::todo(), vec!["A"]),
    );
    nodes.insert(
        "C".to_string(),
        make_test_node("C", TaskStatus::todo(), vec!["X"]),
    ); // Dangling pointer

    let graph = TaskGraph::new(nodes);
    assert!(graph.is_ready("A"));
    assert!(!graph.is_ready("B")); // A is not terminal
    assert!(!graph.is_ready("C")); // X does not exist
}

#[test]
fn test_dynamic_scoring() {
    // A is priority 1, blocks nothing.
    // B is priority 5 and blocks C and D, but downstream priorities are unset.
    // Explicit priority should remain primary in this case.
    let mut nodes = HashMap::new();

    let mut a = make_test_node("A", TaskStatus::todo(), vec![]);
    a.frontmatter.priority =
        Some(Priority::try_from(1).expect("Priority value should be within the valid range 0-99"));

    let mut b = make_test_node("B", TaskStatus::todo(), vec![]);
    b.frontmatter.priority =
        Some(Priority::try_from(5).expect("Priority value should be within the valid range 0-99"));

    let c = make_test_node("C", TaskStatus::todo(), vec!["B"]);
    let d = make_test_node("D", TaskStatus::todo(), vec!["B"]);

    nodes.insert("A".to_string(), a);
    nodes.insert("B".to_string(), b);
    nodes.insert("C".to_string(), c);
    nodes.insert("D".to_string(), d);

    let graph = TaskGraph::new(nodes);
    let next_tasks = graph.get_next_tasks();

    assert_eq!(next_tasks.len(), 2); // C and D are not ready

    assert_eq!(next_tasks[0].frontmatter.id, "A");
    assert_eq!(next_tasks[1].frontmatter.id, "B");
}

#[test]
fn test_dynamic_scoring_promotes_blocker_of_higher_priority_downstream_work() {
    // BLOCKER has no explicit priority, but it blocks P0-TASK.
    // OTHER has explicit priority 1.
    // BLOCKER should be treated as effective P0 and surface first.
    let mut nodes = HashMap::new();

    let blocker = make_test_node("BLOCKER", TaskStatus::todo(), vec![]);

    let mut p0_task = make_test_node("P0-TASK", TaskStatus::todo(), vec!["BLOCKER"]);
    p0_task.frontmatter.priority =
        Some(Priority::try_from(0).expect("Priority value should be within the valid range 0-99"));

    let mut other = make_test_node("OTHER", TaskStatus::todo(), vec![]);
    other.frontmatter.priority =
        Some(Priority::try_from(1).expect("Priority value should be within the valid range 0-99"));

    nodes.insert("BLOCKER".to_string(), blocker);
    nodes.insert("P0-TASK".to_string(), p0_task);
    nodes.insert("OTHER".to_string(), other);

    let graph = TaskGraph::new(nodes);
    let next_tasks = graph.get_next_tasks();

    assert_eq!(next_tasks.len(), 2); // P0-TASK is not ready
    assert_eq!(next_tasks[0].frontmatter.id, "BLOCKER");
    assert_eq!(next_tasks[1].frontmatter.id, "OTHER");
}

#[test]
fn test_dynamic_scoring_effective_priority_tie_breaks_by_base_priority() {
    // DIRECT has base priority 2 and no downstream dependents.
    // BLOCKER has base priority 5 but blocks DOWNSTREAM-P2, so effective priority is 2.
    // With equal effective priority, lower base priority should win.
    let mut nodes = HashMap::new();

    let mut direct = make_test_node("DIRECT", TaskStatus::todo(), vec![]);
    direct.frontmatter.priority =
        Some(Priority::try_from(2).expect("Priority value should be within the valid range 0-99"));

    let mut blocker = make_test_node("BLOCKER", TaskStatus::todo(), vec![]);
    blocker.frontmatter.priority =
        Some(Priority::try_from(5).expect("Priority value should be within the valid range 0-99"));

    let mut downstream_p2 = make_test_node("DOWNSTREAM-P2", TaskStatus::todo(), vec!["BLOCKER"]);
    downstream_p2.frontmatter.priority =
        Some(Priority::try_from(2).expect("Priority value should be within the valid range 0-99"));

    nodes.insert("DIRECT".to_string(), direct);
    nodes.insert("BLOCKER".to_string(), blocker);
    nodes.insert("DOWNSTREAM-P2".to_string(), downstream_p2);

    let graph = TaskGraph::new(nodes);
    let next_tasks = graph.get_next_tasks();

    assert_eq!(next_tasks.len(), 2); // DOWNSTREAM-P2 is not ready
    assert_eq!(next_tasks[0].frontmatter.id, "DIRECT");
    assert_eq!(next_tasks[1].frontmatter.id, "BLOCKER");
}

#[test]
fn test_count_blocking_excludes_terminal_and_self() {
    let mut nodes = HashMap::new();

    let a = make_test_node("A", TaskStatus::todo(), vec![]);
    let b = make_test_node("B", TaskStatus::todo(), vec!["A"]);
    let c = make_test_node("C", TaskStatus::done(), vec!["B"]);
    let d = make_test_node("D", TaskStatus::todo(), vec!["C"]);
    let e = make_test_node("E", TaskStatus::todo(), vec!["B"]);

    nodes.insert("A".to_string(), a);
    nodes.insert("B".to_string(), b);
    nodes.insert("C".to_string(), c);
    nodes.insert("D".to_string(), d);
    nodes.insert("E".to_string(), e);

    let graph = TaskGraph::new(nodes);

    // Reachable non-terminal tasks from A are B and E.
    // C is terminal and stops traversal, so D is not counted.
    assert_eq!(graph.count_blocking("A"), 2);
}

#[test]
fn test_count_blocking_cycle_excludes_self() {
    let mut nodes = HashMap::new();
    nodes.insert(
        "A".to_string(),
        make_test_node("A", TaskStatus::todo(), vec!["B"]),
    );
    nodes.insert(
        "B".to_string(),
        make_test_node("B", TaskStatus::todo(), vec!["A"]),
    );

    let graph = TaskGraph::new(nodes);

    // A should only count B (self excluded despite the cycle).
    assert_eq!(graph.count_blocking("A"), 1);
}

#[test]
fn test_default_order_respects_needs_and_priority() {
    let mut nodes = HashMap::new();

    let mut a = make_test_node("A", TaskStatus::todo(), vec![]);
    a.frontmatter.priority = Some(Priority::try_from(5).expect("Valid priority"));

    let mut b = make_test_node("B", TaskStatus::todo(), vec![]);
    b.frontmatter.priority = Some(Priority::try_from(1).expect("Valid priority"));

    let c = make_test_node("C", TaskStatus::todo(), vec!["A", "B"]);

    nodes.insert("A".to_string(), a);
    nodes.insert("B".to_string(), b);
    nodes.insert("C".to_string(), c);

    let graph = TaskGraph::new(nodes);
    let tasks: Vec<&TaskNode> = graph.nodes.values().collect();
    let ordered = graph
        .default_order(tasks)
        .expect("default_order should succeed");
    let ids: Vec<&str> = ordered
        .iter()
        .map(|node| node.frontmatter.id.as_str())
        .collect();

    assert_eq!(ids, vec!["B", "A", "C"]);
}

#[test]
fn test_default_order_cycle_grouping_created_at() {
    let mut nodes = HashMap::new();

    let mut a = make_test_node("A", TaskStatus::todo(), vec!["B"]);
    let mut b = make_test_node("B", TaskStatus::todo(), vec!["A"]);
    let c = make_test_node("C", TaskStatus::todo(), vec!["A"]);

    a.frontmatter.created_at = Some(
        DateTime::parse_from_rfc3339("2026-01-02T00:00:00Z")
            .expect("Failed to parse datetime")
            .with_timezone(&Utc),
    );
    b.frontmatter.created_at = Some(
        DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .expect("Failed to parse datetime")
            .with_timezone(&Utc),
    );

    nodes.insert("A".to_string(), a);
    nodes.insert("B".to_string(), b);
    nodes.insert("C".to_string(), c);

    let graph = TaskGraph::new(nodes);
    let tasks: Vec<&TaskNode> = graph.nodes.values().collect();
    let ordered = graph
        .default_order(tasks)
        .expect("default_order should succeed");
    let ids: Vec<&str> = ordered
        .iter()
        .map(|node| node.frontmatter.id.as_str())
        .collect();

    assert_eq!(ids, vec!["B", "A", "C"]);
}

#[test]
fn test_default_order_id_tiebreaker() {
    let mut nodes = HashMap::new();

    let a = make_test_node("A", TaskStatus::todo(), vec![]);
    let b = make_test_node("B", TaskStatus::todo(), vec![]);

    nodes.insert("B".to_string(), b);
    nodes.insert("A".to_string(), a);

    let graph = TaskGraph::new(nodes);
    let tasks: Vec<&TaskNode> = graph.nodes.values().collect();
    let ordered = graph
        .default_order(tasks)
        .expect("default_order should succeed");
    let ids: Vec<&str> = ordered
        .iter()
        .map(|node| node.frontmatter.id.as_str())
        .collect();

    assert_eq!(ids, vec!["A", "B"]);
}

#[test]
fn test_cycle_readiness() {
    let mut nodes = HashMap::new();
    nodes.insert(
        "X".to_string(),
        make_test_node("X", TaskStatus::todo(), vec!["Y"]),
    );
    nodes.insert(
        "Y".to_string(),
        make_test_node("Y", TaskStatus::todo(), vec!["X"]),
    );

    let graph = TaskGraph::new(nodes);
    // Neither task ever surfaces as ready
    assert!(!graph.is_ready("X"));
    assert!(!graph.is_ready("Y"));
    assert_eq!(graph.get_next_tasks().len(), 0);
}

#[test]
fn test_load_from_dir_prefers_yaml_frontmatter_and_ignores_non_yaml() {
    let temp = tempdir().expect("tempdir should be created");

    let yaml_task = r#"---
id: YAML-1
title: YAML task
status: todo
created_at: "2026-02-21T17:00:00Z"
---
Body
"#;
    fs::write(temp.path().join("yaml.md"), yaml_task).expect("yaml task should be written");

    let legacy_toml_task = r#"+++
id = "TOML-1"
title = "Legacy TOML task"
status = "todo"
created_at = 2026-02-21T17:00:00Z
+++
Body
"#;
    fs::write(temp.path().join("legacy.md"), legacy_toml_task)
        .expect("legacy task should be written");

    fs::write(temp.path().join("notes.md"), "# Plain markdown\n")
        .expect("plain markdown file should be written");

    let graph = TaskGraph::load_from_dir(temp.path()).expect("graph should load");

    assert!(
        graph.nodes.contains_key("YAML-1"),
        "valid YAML frontmatter should be loaded"
    );
    assert!(
        !graph.nodes.contains_key("TOML-1"),
        "legacy TOML frontmatter should be treated as missing YAML frontmatter"
    );
    assert_eq!(
        graph.nodes.len(),
        1,
        "non-YAML files should be skipped as missing metadata"
    );
}
