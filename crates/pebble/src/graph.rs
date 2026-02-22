use crate::models::TaskNode;
use crate::parser::parse_task_file;
use color_eyre::eyre::Result;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub struct TaskGraph {
    pub nodes: HashMap<String, TaskNode>,
    /// Maps a task ID to the list of task IDs that depend on it.
    pub reverse_deps: HashMap<String, Vec<String>>,
}

impl TaskGraph {
    /// Builds a graph from a directory of task files.
    pub fn load_from_dir(tasks_dir: &Path) -> Result<Self> {
        let mut nodes = HashMap::new();

        if tasks_dir.exists() {
            for entry in std::fs::read_dir(tasks_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                    // Ignore AGENTS.md or other known non-task files if they live here.
                    if path.file_name().and_then(|n| n.to_str()) == Some("AGENTS.md") {
                        continue;
                    }

                    let content = std::fs::read_to_string(&path)?;
                    // Skip files that don't start with +++
                    if content.starts_with("+++") {
                        match parse_task_file(&path, &content) {
                            Ok(node) => {
                                nodes.insert(node.frontmatter.id.clone(), node);
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        Ok(Self::new(nodes))
    }

    /// Creates a TaskGraph from an existing map of nodes, precomputing reverse indices.
    pub fn new(nodes: HashMap<String, TaskNode>) -> Self {
        let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();

        for (id, node) in &nodes {
            for dep_id in &node.frontmatter.deps {
                reverse_deps
                    .entry(dep_id.clone())
                    .or_default()
                    .push(id.clone());
            }
        }

        Self {
            nodes,
            reverse_deps,
        }
    }

    /// Determines if a task is "ready" according to absolute readiness rules.
    /// Readiness rule:
    /// 1. Its local status is actionable (todo or in_progress).
    /// 2. EVERY task listed in its deps array exists.
    /// 3. EVERY task listed in its deps array has a terminal status (done or canceled).
    pub fn is_ready(&self, task_id: &str) -> bool {
        let Some(node) = self.nodes.get(task_id) else {
            return false;
        };

        if !node.frontmatter.status.is_actionable() {
            return false;
        }

        for dep_id in &node.frontmatter.deps {
            if let Some(dep_node) = self.nodes.get(dep_id) {
                if !dep_node.frontmatter.status.is_closed() {
                    return false; // Dep is not in terminal state
                }
            } else {
                return false; // Dep is dangling (missing)
            }
        }

        true
    }

    /// Returns the number of downstream non-terminal tasks (transitively) blocked by the given task.
    /// Uses a DFS to count unique reachable tasks while excluding the task itself.
    pub fn count_blocking(&self, task_id: &str) -> usize {
        let mut visited = HashSet::new();
        let mut stack = vec![task_id.to_string()];
        let mut count = 0;

        visited.insert(task_id.to_string());

        while let Some(current) = stack.pop() {
            if let Some(downstream_ids) = self.reverse_deps.get(&current) {
                for downstream_id in downstream_ids {
                    if visited.insert(downstream_id.clone()) {
                        if let Some(node) = self.nodes.get(downstream_id)
                            && node.frontmatter.status.is_actionable()
                        {
                            count += 1;
                        }
                        stack.push(downstream_id.clone());
                    }
                }
            }
        }

        count
    }

    /// Returns a list of tasks that are ready, sorted by the Dynamic Scoring algorithm:
    /// (len(blocking) DESC, priority ASC, created_at ASC)
    pub fn get_next_tasks(&self) -> Vec<&TaskNode> {
        let mut ready_tasks: Vec<&TaskNode> = self
            .nodes
            .values()
            .filter(|n| self.is_ready(&n.frontmatter.id))
            .collect();

        // Compute `blocking` counts ahead of sorting to avoid O(N^2) sorting overhead.
        let blocking_counts: HashMap<String, usize> = ready_tasks
            .iter()
            .map(|n| {
                (
                    n.frontmatter.id.clone(),
                    self.count_blocking(&n.frontmatter.id),
                )
            })
            .collect();

        ready_tasks.sort_by(|a, b| {
            let count_a = blocking_counts[&a.frontmatter.id];
            let count_b = blocking_counts[&b.frontmatter.id];

            // 1. len(blocking) DESC
            let cmp = count_b.cmp(&count_a);
            if cmp != std::cmp::Ordering::Equal {
                return cmp;
            }

            // 2. priority ASC (treat None as highest numerical value = lowest priority)
            let prio_a = a.frontmatter.priority.unwrap_or(u8::MAX);
            let prio_b = b.frontmatter.priority.unwrap_or(u8::MAX);
            let cmp = prio_a.cmp(&prio_b);
            if cmp != std::cmp::Ordering::Equal {
                return cmp;
            }

            // 3. created_at ASC
            a.frontmatter.created_at.cmp(&b.frontmatter.created_at)
        });

        ready_tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TaskFrontmatter;
    use crate::models::TaskStatus;
    use std::str::FromStr;

    fn make_test_node(id: &str, status: TaskStatus, deps: Vec<&str>) -> TaskNode {
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
                deps: deps.into_iter().map(|s| s.to_string()).collect(),
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

        // Reachable non-terminal tasks from A are B, D, E. C is terminal, A is excluded.
        assert_eq!(graph.count_blocking("A"), 3);
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
}
