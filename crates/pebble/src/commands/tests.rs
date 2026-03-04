#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]

use super::*;
use crate::models::{TaskFrontmatter, TaskStatus};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

fn make_test_node(id: &str, status: TaskStatus, needs: Vec<&str>) -> TaskNode {
    TaskNode {
        path: PathBuf::from(format!("{id}.md")),
        body: String::new(),
        frontmatter: TaskFrontmatter {
            id: id.to_string(),
            title: id.to_string(),
            status,
            priority: None,
            created_at: Some(
                DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
                    .expect("datetime should be valid ISO 8601")
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
fn test_blocking_list_excludes_terminal_dependents() {
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
        make_test_node("C", TaskStatus::done(), vec!["A"]),
    );

    let graph = TaskGraph::new(nodes);
    let node = graph.nodes.get("A").expect("test node 'A' should exist");
    let tasks_dir = PathBuf::from(".");
    let obj = TaskObject::from_node(node, &graph, tasks_dir.as_path());

    let mut blocking = obj.blocking.clone();
    blocking.sort();
    assert_eq!(blocking, vec!["B"]);
}

#[test]
fn test_config_values_map_matches_serialized_config_keys() {
    let config = Config::default();
    let extracted = config_values_map(&config).expect("Should extract config values");
    let serialized = serde_json::to_value(&config).expect("Should serialize config");
    let serialized_keys = serialized
        .as_object()
        .expect("Serialized config should be an object")
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let extracted_keys = extracted.keys().cloned().collect::<BTreeSet<_>>();

    assert_eq!(extracted_keys, serialized_keys);
}
