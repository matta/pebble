use crate::graph::TaskGraph;
use crate::models::{TaskFrontmatter, TaskNode};
use serde::Serialize;

/// Serialized version of a TaskObject as expected by the JSON API contract.
#[derive(Serialize)]
pub struct TaskObject<'a> {
    #[serde(flatten)]
    pub frontmatter: &'a TaskFrontmatter,
    pub is_ready: bool,
    pub blocked_by: Vec<String>,
    pub blocking: Vec<String>,
    pub body: &'a str,
    pub path: String,
}

impl<'a> TaskObject<'a> {
    /// Build a serializable task view from a graph node and tasks directory.
    pub fn from_node(node: &'a TaskNode, graph: &TaskGraph, tasks_dir: &std::path::Path) -> Self {
        let is_ready = graph.is_ready(&node.frontmatter.id);

        let blocked_by = graph.get_blockers(&node.frontmatter.id);

        let blocking: Vec<String> = graph
            .blocking
            .get(&node.frontmatter.id)
            .into_iter()
            .flat_map(|ids| ids.iter())
            .filter(|dep_id| {
                graph
                    .nodes
                    .get(*dep_id)
                    .is_some_and(|dep_node| dep_node.frontmatter.status.is_actionable())
            })
            .cloned()
            .collect();

        let rel_path = node.path.strip_prefix(tasks_dir).unwrap_or(&node.path);

        TaskObject {
            frontmatter: &node.frontmatter,
            is_ready,
            blocked_by,
            blocking,
            body: &node.body,
            path: rel_path.display().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TaskStatus;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::str::FromStr;

    fn make_test_node(id: &str, status: TaskStatus, needs: Vec<&str>) -> TaskNode {
        TaskNode {
            path: PathBuf::from(format!("{id}.md")),
            body: String::new(),
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
    fn test_blocking_list_excludes_terminal_dependents() {
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
            make_test_node("C", TaskStatus::Done, vec!["A"]),
        );

        let graph = TaskGraph::new(nodes);
        let node = graph.nodes.get("A").unwrap();
        let tasks_dir = PathBuf::from(".");
        let obj = TaskObject::from_node(node, &graph, tasks_dir.as_path());

        let mut blocking = obj.blocking.clone();
        blocking.sort();
        assert_eq!(blocking, vec!["B"]);
    }
}
