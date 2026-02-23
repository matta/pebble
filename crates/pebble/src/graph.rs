use crate::models::TaskNode;
use crate::parser::parse_task_file;
use color_eyre::eyre::Result;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::path::Path;

mod ordering;

/// Composite sort key for ordering tasks in `pebble next` output.
///
/// Fields are compared lexicographically: tasks that block more downstream work
/// rank first; ties are broken by priority, then creation time, then ID.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct NodeKey {
    /// Descending blocking count (wrapped in [`Reverse`] so `Ord` sorts highest first).
    blocking_count: Reverse<usize>,
    /// Raw priority value; `u32::MAX` is used when priority is unset (sorts last).
    priority: u32,
    /// Creation timestamp, used as a tiebreaker after priority.
    created_at: toml_datetime::Datetime,
    /// Task ID, used as the final deterministic tiebreaker.
    id: String,
}

/// In-memory task graph with forward nodes and a reverse dependency index.
pub struct TaskGraph {
    pub nodes: HashMap<String, TaskNode>,
    /// Maps a task ID to the list of task IDs that depend on it.
    pub reverse_deps: HashMap<String, Vec<String>>,
}

impl TaskGraph {
    /// Builds a graph from a directory of task files.
    ///
    /// Scans the directory for Markdown (`.md`) files, parsing each as a [`TaskNode`].
    /// Files named `AGENTS.md` are explicitly ignored. Files that fail to parse
    /// (e.g., missing frontmatter) result in a warning printed to stderr but do not
    /// halt the loading process.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if the directory cannot be read or if any file read operation fails.
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
                    if !visited.insert(downstream_id.clone()) {
                        continue;
                    }

                    let Some(node) = self.nodes.get(downstream_id) else {
                        continue;
                    };

                    if node.frontmatter.status.is_actionable() {
                        count += 1;
                        stack.push(downstream_id.clone());
                    }
                }
            }
        }

        count
    }

    /// Builds the composite [`NodeKey`] for a task, used during sort comparisons.
    ///
    /// Looks up the pre-computed blocking count; falls back to 0 if the task is not
    /// present in `blocking_counts`. Unset priority is mapped to `u32::MAX`.
    fn next_task_key(&self, node: &TaskNode, blocking_counts: &HashMap<String, usize>) -> NodeKey {
        let blocking_count = *blocking_counts.get(&node.frontmatter.id).unwrap_or(&0);
        let priority = node.frontmatter.priority.map(u32::from).unwrap_or(u32::MAX);

        NodeKey {
            blocking_count: Reverse(blocking_count),
            priority,
            created_at: node.frontmatter.created_at,
            id: node.frontmatter.id.clone(),
        }
    }

    /// Order tasks by dependency-aware default sort (topology, blocking, priority, time, id).
    pub fn default_order<'a>(&'a self, nodes: Vec<&'a TaskNode>) -> Vec<&'a TaskNode> {
        ordering::default_order(self, nodes)
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

        let mut keys: HashMap<String, NodeKey> = HashMap::new();
        for node in &ready_tasks {
            keys.insert(
                node.frontmatter.id.clone(),
                self.next_task_key(node, &blocking_counts),
            );
        }

        ready_tasks.sort_by(|a, b| {
            let key_a = &keys[&a.frontmatter.id];
            let key_b = &keys[&b.frontmatter.id];
            key_a.cmp(key_b)
        });

        ready_tasks
    }
}

#[cfg(test)]
mod tests;
