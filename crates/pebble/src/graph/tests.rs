#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
use super::*;
use crate::models::TaskStatus;
use crate::models::test_utils::TaskNodeBuilder;
use std::collections::HashMap;

#[test]
fn test_absolute_readiness() {
    let mut nodes = HashMap::new();
    nodes.insert(
        "A".to_string(),
        TaskNodeBuilder::new("A").status(TaskStatus::Todo).build(),
    );
    nodes.insert(
        "B".to_string(),
        TaskNodeBuilder::new("B")
            .status(TaskStatus::Todo)
            .needs(vec!["A"])
            .build(),
    );
    nodes.insert(
        "C".to_string(),
        TaskNodeBuilder::new("C")
            .status(TaskStatus::Todo)
            .needs(vec!["X"])
            .build(),
    ); // Dangling pointer

    let graph = TaskGraph::new(nodes);
    assert!(graph.is_ready("A"));
    assert!(!graph.is_ready("B")); // A is not terminal
    assert!(!graph.is_ready("C")); // X does not exist
}

#[test]
fn test_dynamic_scoring() {
    // A is priority 1, blocks nothing
    // B is priority 5, blocks C and D
    // C and D depend on B
    let mut nodes = HashMap::new();

    let a = TaskNodeBuilder::new("A")
        .status(TaskStatus::Todo)
        .priority(1)
        .build();

    let b = TaskNodeBuilder::new("B")
        .status(TaskStatus::Todo)
        .priority(5)
        .build();

    let c = TaskNodeBuilder::new("C")
        .status(TaskStatus::Todo)
        .needs(vec!["B"])
        .build();
    let d = TaskNodeBuilder::new("D")
        .status(TaskStatus::Todo)
        .needs(vec!["B"])
        .build();

    nodes.insert("A".to_string(), a);
    nodes.insert("B".to_string(), b);
    nodes.insert("C".to_string(), c);
    nodes.insert("D".to_string(), d);

    let graph = TaskGraph::new(nodes);
    let next_tasks = graph.get_next_tasks();

    assert_eq!(next_tasks.len(), 2); // C and D are not ready

    // B should be first because it blocks 2 things, even though its priority is 5.
    // A blocks 0 things.
    assert_eq!(next_tasks[0].frontmatter.id, "B");
    assert_eq!(next_tasks[1].frontmatter.id, "A");
}

#[test]
fn test_count_blocking_excludes_terminal_and_self() {
    let mut nodes = HashMap::new();

    let a = TaskNodeBuilder::new("A").status(TaskStatus::Todo).build();
    let b = TaskNodeBuilder::new("B")
        .status(TaskStatus::Todo)
        .needs(vec!["A"])
        .build();
    let c = TaskNodeBuilder::new("C")
        .status(TaskStatus::Done)
        .needs(vec!["B"])
        .build();
    let d = TaskNodeBuilder::new("D")
        .status(TaskStatus::Todo)
        .needs(vec!["C"])
        .build();
    let e = TaskNodeBuilder::new("E")
        .status(TaskStatus::Todo)
        .needs(vec!["B"])
        .build();

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
        TaskNodeBuilder::new("A")
            .status(TaskStatus::Todo)
            .needs(vec!["B"])
            .build(),
    );
    nodes.insert(
        "B".to_string(),
        TaskNodeBuilder::new("B")
            .status(TaskStatus::Todo)
            .needs(vec!["A"])
            .build(),
    );

    let graph = TaskGraph::new(nodes);

    // A should only count B (self excluded despite the cycle).
    assert_eq!(graph.count_blocking("A"), 1);
}

#[test]
fn test_default_order_respects_needs_and_priority() {
    let mut nodes = HashMap::new();

    let a = TaskNodeBuilder::new("A")
        .status(TaskStatus::Todo)
        .priority(5)
        .build();

    let b = TaskNodeBuilder::new("B")
        .status(TaskStatus::Todo)
        .priority(1)
        .build();

    let c = TaskNodeBuilder::new("C")
        .status(TaskStatus::Todo)
        .needs(vec!["A", "B"])
        .build();

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

    let a = TaskNodeBuilder::new("A")
        .status(TaskStatus::Todo)
        .needs(vec!["B"])
        .created_at("2026-01-02T00:00:00Z")
        .build();
    let b = TaskNodeBuilder::new("B")
        .status(TaskStatus::Todo)
        .needs(vec!["A"])
        .created_at("2026-01-01T00:00:00Z")
        .build();
    let c = TaskNodeBuilder::new("C")
        .status(TaskStatus::Todo)
        .needs(vec!["A"])
        .build();

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

    let a = TaskNodeBuilder::new("A").status(TaskStatus::Todo).build();
    let b = TaskNodeBuilder::new("B").status(TaskStatus::Todo).build();

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
        TaskNodeBuilder::new("X")
            .status(TaskStatus::Todo)
            .needs(vec!["Y"])
            .build(),
    );
    nodes.insert(
        "Y".to_string(),
        TaskNodeBuilder::new("Y")
            .status(TaskStatus::Todo)
            .needs(vec!["X"])
            .build(),
    );

    let graph = TaskGraph::new(nodes);
    // Neither task ever surfaces as ready
    assert!(!graph.is_ready("X"));
    assert!(!graph.is_ready("Y"));
    assert_eq!(graph.get_next_tasks().len(), 0);
}
