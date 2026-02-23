use super::*;
use crate::models::TaskFrontmatter;
use crate::models::TaskStatus;
use std::str::FromStr;

fn make_test_node(id: &str, status: TaskStatus, needs: Vec<&str>) -> TaskNode {
    TaskNode {
        path: Path::new("").to_path_buf(),
        body: "".to_string(),
        frontmatter: TaskFrontmatter {
            id: id.to_string(),
            title: id.to_string(),
            status,
            priority: None,
            created_at: toml_datetime::Datetime::from_str("2026-01-01T00:00:00Z").unwrap(),
            modified_at: None,
            resolved_at: None,
            needs: needs.into_iter().map(|s| s.to_string()).collect(),
            tags: vec![],
        },
    }
}

#[test]
fn test_absolute_readiness() {
    let mut nodes = HashMap::new();
    nodes.insert(
        "A".to_string(),
        make_test_node("A", TaskStatus::Todo, vec![]),
    );
    nodes.insert(
        "B".to_string(),
        make_test_node("B", TaskStatus::Todo, vec!["A"]),
    );
    nodes.insert(
        "C".to_string(),
        make_test_node("C", TaskStatus::Todo, vec!["X"]),
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

    let mut a = make_test_node("A", TaskStatus::Todo, vec![]);
    a.frontmatter.priority = Some(1);

    let mut b = make_test_node("B", TaskStatus::Todo, vec![]);
    b.frontmatter.priority = Some(5);

    let c = make_test_node("C", TaskStatus::Todo, vec!["B"]);
    let d = make_test_node("D", TaskStatus::Todo, vec!["B"]);

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

    let a = make_test_node("A", TaskStatus::Todo, vec![]);
    let b = make_test_node("B", TaskStatus::Todo, vec!["A"]);
    let c = make_test_node("C", TaskStatus::Done, vec!["B"]);
    let d = make_test_node("D", TaskStatus::Todo, vec!["C"]);
    let e = make_test_node("E", TaskStatus::Todo, vec!["B"]);

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
        make_test_node("A", TaskStatus::Todo, vec!["B"]),
    );
    nodes.insert(
        "B".to_string(),
        make_test_node("B", TaskStatus::Todo, vec!["A"]),
    );

    let graph = TaskGraph::new(nodes);

    // A should only count B (self excluded despite the cycle).
    assert_eq!(graph.count_blocking("A"), 1);
}

#[test]
fn test_default_order_respects_needs_and_priority() {
    let mut nodes = HashMap::new();

    let mut a = make_test_node("A", TaskStatus::Todo, vec![]);
    a.frontmatter.priority = Some(5);

    let mut b = make_test_node("B", TaskStatus::Todo, vec![]);
    b.frontmatter.priority = Some(1);

    let c = make_test_node("C", TaskStatus::Todo, vec!["A", "B"]);

    nodes.insert("A".to_string(), a);
    nodes.insert("B".to_string(), b);
    nodes.insert("C".to_string(), c);

    let graph = TaskGraph::new(nodes);
    let tasks: Vec<&TaskNode> = graph.nodes.values().collect();
    let ordered = graph.default_order(tasks);
    let ids: Vec<&str> = ordered
        .iter()
        .map(|node| node.frontmatter.id.as_str())
        .collect();

    assert_eq!(ids, vec!["B", "A", "C"]);
}

#[test]
fn test_default_order_cycle_grouping_created_at() {
    let mut nodes = HashMap::new();

    let mut a = make_test_node("A", TaskStatus::Todo, vec!["B"]);
    let mut b = make_test_node("B", TaskStatus::Todo, vec!["A"]);
    let c = make_test_node("C", TaskStatus::Todo, vec!["A"]);

    a.frontmatter.created_at = toml_datetime::Datetime::from_str("2026-01-02T00:00:00Z").unwrap();
    b.frontmatter.created_at = toml_datetime::Datetime::from_str("2026-01-01T00:00:00Z").unwrap();

    nodes.insert("A".to_string(), a);
    nodes.insert("B".to_string(), b);
    nodes.insert("C".to_string(), c);

    let graph = TaskGraph::new(nodes);
    let tasks: Vec<&TaskNode> = graph.nodes.values().collect();
    let ordered = graph.default_order(tasks);
    let ids: Vec<&str> = ordered
        .iter()
        .map(|node| node.frontmatter.id.as_str())
        .collect();

    assert_eq!(ids, vec!["B", "A", "C"]);
}

#[test]
fn test_default_order_id_tiebreaker() {
    let mut nodes = HashMap::new();

    let a = make_test_node("A", TaskStatus::Todo, vec![]);
    let b = make_test_node("B", TaskStatus::Todo, vec![]);

    nodes.insert("B".to_string(), b);
    nodes.insert("A".to_string(), a);

    let graph = TaskGraph::new(nodes);
    let tasks: Vec<&TaskNode> = graph.nodes.values().collect();
    let ordered = graph.default_order(tasks);
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
        make_test_node("X", TaskStatus::Todo, vec!["Y"]),
    );
    nodes.insert(
        "Y".to_string(),
        make_test_node("Y", TaskStatus::Todo, vec!["X"]),
    );

    let graph = TaskGraph::new(nodes);
    // Neither task ever surfaces as ready
    assert!(!graph.is_ready("X"));
    assert!(!graph.is_ready("Y"));
    assert_eq!(graph.get_next_tasks().len(), 0);
}
