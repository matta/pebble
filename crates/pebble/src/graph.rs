use crate::models::{TaskNode, default_datetime};
use crate::parser::parse_task_file;
use chrono::{DateTime, Utc};
use color_eyre::eyre::Result;
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::result;

mod ordering;

/// Composite sort key for ordering tasks in `pebble next` output.
///
/// Fields are compared lexicographically: effective priority first, then base
/// priority, then downstream blocking count, then creation time, then ID.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct NodeKey {
    /// Effective priority used by dynamic scoring.
    effective_priority: u32,
    /// Explicit task priority with a None-last sentinel value.
    base_priority: u32,
    /// Descending blocking count (wrapped in [`Reverse`] so `Ord` sorts highest first).
    blocking_count: Reverse<usize>,
    /// Creation timestamp, used as a tiebreaker after priority.
    created_at: DateTime<Utc>,
    /// Task ID, used as the final deterministic tiebreaker.
    id: String,
}

/// In-memory task graph with forward nodes and a reverse dependency index.
pub struct TaskGraph {
    pub nodes: HashMap<String, TaskNode>,
    /// Maps a task ID to the list of task IDs that depend on it.
    pub blocking: HashMap<String, Vec<String>>,
    /// IDs that appeared in multiple task files during load.
    pub duplicate_ids: HashSet<String>,
}

impl TaskGraph {
    const UNSET_PRIORITY_SENTINEL: u32 = 100;

    /// Recursively collect all Markdown files under `dir` in deterministic order.
    fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<result::Result<_, _>>()?;
        entries.sort_by_key(|entry| entry.path());

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                Self::collect_markdown_files(&path, out)?;
            } else if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                out.push(path);
            }
        }

        Ok(())
    }

    /// Builds a graph from a directory of task files.
    ///
    /// Scans the directory and all subdirectories for Markdown (`.md`) files, parsing each
    /// as a [`TaskNode`].
    ///
    /// The loading process handles the following cases:
    /// * Valid Tasks: Files starting with `---` and containing valid YAML frontmatter are loaded.
    /// * Duplicates: If multiple files declare the same task ID, a warning is printed to stderr,
    ///   and all occurrences of that ID are excluded from the graph to prevent ambiguity.
    /// * Ignored Files: Files named `AGENTS.md` or those not starting with `---` are silently skipped.
    /// * Parse Errors: Files starting with `---` but containing invalid frontmatter result in a
    ///   warning to stderr but do not halt the loading process.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory walk fails or if any file read operation fails (e.g. permission denied).
    pub fn load_from_dir(tasks_dir: &Path) -> Result<Self> {
        let mut parsed_nodes = Vec::new();

        if tasks_dir.exists() {
            let mut markdown_files = Vec::new();
            Self::collect_markdown_files(tasks_dir, &mut markdown_files)?;

            for path in markdown_files {
                // Ignore AGENTS.md or other known non-task files if they live here.
                if path.file_name().and_then(|n| n.to_str()) == Some("AGENTS.md") {
                    continue;
                }

                let content = fs::read_to_string(&path)?;
                // Skip files that don't start with YAML frontmatter.
                if content.starts_with("---") {
                    match parse_task_file(&path, &content) {
                        Ok(node) => {
                            parsed_nodes.push(node);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        let mut grouped: BTreeMap<String, Vec<TaskNode>> = BTreeMap::new();
        for node in parsed_nodes {
            grouped
                .entry(node.frontmatter.id.clone())
                .or_default()
                .push(node);
        }

        let mut nodes = HashMap::new();
        let mut duplicate_ids = HashSet::new();

        for (id, mut id_nodes) in grouped {
            if id_nodes.len() > 1 {
                id_nodes.sort_by(|a, b| a.path.cmp(&b.path));
                let paths: Vec<String> = id_nodes
                    .iter()
                    .map(|node| node.path.display().to_string())
                    .collect();
                eprintln!(
                    "Warning: Duplicate task ID '{}' found in files: {}. Skipping all files with this ID.",
                    id,
                    paths.join(", ")
                );
                duplicate_ids.insert(id);
            } else if let Some(node) = id_nodes.pop() {
                nodes.insert(node.frontmatter.id.clone(), node);
            }
        }

        Ok(Self::new_with_duplicates(nodes, duplicate_ids))
    }

    /// Creates a TaskGraph from an existing map of nodes, precomputing reverse indices.
    pub fn new(nodes: HashMap<String, TaskNode>) -> Self {
        Self::new_with_duplicates(nodes, HashSet::new())
    }

    /// Creates a new [`TaskGraph`] from a map of nodes, computing the reverse dependency index.
    ///
    /// This constructor performs the following:
    /// 1. Stores the provided `nodes` map.
    /// 2. Iterates over all nodes to build the `blocking` index (a map from dependency ID to
    ///    dependent task IDs).
    /// 3. Stores the set of `duplicate_ids` for later validation or reporting.
    ///
    /// # Arguments
    ///
    /// * `nodes` - A map where keys are task IDs and values are the parsed [`TaskNode`]s.
    /// * `duplicate_ids` - A set of task IDs that were found in multiple files during loading.
    pub fn new_with_duplicates(
        nodes: HashMap<String, TaskNode>,
        duplicate_ids: HashSet<String>,
    ) -> Self {
        let mut blocking: HashMap<String, Vec<String>> = HashMap::new();

        for (id, node) in &nodes {
            for dep_id in &node.frontmatter.needs {
                blocking.entry(dep_id.clone()).or_default().push(id.clone());
            }
        }

        Self {
            nodes,
            blocking,
            duplicate_ids,
        }
    }

    /// Returns true if the given ID was found in multiple files.
    pub fn is_duplicate_id(&self, task_id: &str) -> bool {
        self.duplicate_ids.contains(task_id)
    }

    /// Determines if a task is "ready" to be worked on.
    ///
    /// A task is considered ready if it satisfies all of the following conditions:
    /// 1. Its status is actionable (i.e., [`crate::models::TaskStatus::todo()`] or [`crate::models::TaskStatus::in_progress()`]).
    /// 2. It has no missing dependencies (all tasks in `needs` exist in the graph).
    /// 3. All its dependencies are in a terminal state (i.e., [`crate::models::TaskStatus::done()`] or [`crate::models::TaskStatus::canceled()`]).
    pub fn is_ready(&self, task_id: &str) -> bool {
        let Some(node) = self.nodes.get(task_id) else {
            return false;
        };

        if !node.frontmatter.status.is_actionable() {
            return false;
        }

        for dep_id in &node.frontmatter.needs {
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
            if let Some(downstream_ids) = self.blocking.get(&current) {
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

    /// Returns the base priority for a task node, using a sentinel for unset priority.
    fn base_priority(node: &TaskNode) -> u32 {
        node.frontmatter
            .priority
            .map(u32::from)
            .unwrap_or(Self::UNSET_PRIORITY_SENTINEL)
    }

    /// Returns the minimum base priority among actionable transitive downstream dependents.
    ///
    /// Traversal follows reverse `needs` edges, excludes the task itself, and only
    /// includes actionable (`todo` / `in_progress`) tasks. Terminal dependents stop
    /// traversal for that branch, matching transitive blocking semantics.
    fn downstream_min_priority(&self, task_id: &str) -> u32 {
        let mut visited = HashSet::new();
        let mut stack = vec![task_id.to_string()];
        let mut min_priority = Self::UNSET_PRIORITY_SENTINEL;

        visited.insert(task_id.to_string());

        while let Some(current) = stack.pop() {
            if let Some(downstream_ids) = self.blocking.get(&current) {
                for downstream_id in downstream_ids {
                    if !visited.insert(downstream_id.clone()) {
                        continue;
                    }

                    let Some(node) = self.nodes.get(downstream_id) else {
                        continue;
                    };

                    if node.frontmatter.status.is_actionable() {
                        min_priority = min_priority.min(Self::base_priority(node));
                        stack.push(downstream_id.clone());
                    }
                }
            }
        }

        min_priority
    }

    /// Returns effective priority for a task ID.
    pub(crate) fn effective_priority_for_task(&self, task_id: &str) -> u32 {
        let Some(node) = self.nodes.get(task_id) else {
            return Self::UNSET_PRIORITY_SENTINEL;
        };

        let base = Self::base_priority(node);
        let downstream_min = self.downstream_min_priority(task_id);
        base.min(downstream_min)
    }

    /// Builds the composite [`NodeKey`] for a task, used during sort comparisons.
    ///
    /// Looks up the pre-computed blocking count; falls back to 0 if the task is not
    /// present in `blocking_counts`.
    fn next_task_key(
        &self,
        node: &TaskNode,
        blocking_counts: &HashMap<String, usize>,
        effective_priorities: &HashMap<String, u32>,
    ) -> NodeKey {
        let blocking_count = *blocking_counts.get(&node.frontmatter.id).unwrap_or(&0);
        let base_priority = Self::base_priority(node);
        let effective_priority = *effective_priorities
            .get(&node.frontmatter.id)
            .unwrap_or(&base_priority);
        let created_at = node.frontmatter.created_at.unwrap_or_else(default_datetime);

        NodeKey {
            effective_priority,
            base_priority,
            blocking_count: Reverse(blocking_count),
            created_at,
            id: node.frontmatter.id.clone(),
        }
    }

    /// Order tasks by dependency-aware default sort
    /// (topology, effective priority, base priority, blocking, time, id).
    pub fn default_order<'a>(&'a self, nodes: Vec<&'a TaskNode>) -> Result<Vec<&'a TaskNode>> {
        ordering::default_order(self, nodes)
    }

    /// Returns a list of tasks that are ready to be worked on.
    ///
    /// The returned tasks are those that satisfy [`TaskGraph::is_ready`] and are sorted
    /// using Dynamic Scoring. The sort order is determined by:
    /// 1. Effective priority (ascending).
    /// 2. Base priority (ascending, with unset priority sorting last).
    /// 3. Number of downstream blocked tasks (descending).
    /// 4. Creation time (ascending).
    /// 5. Task ID (lexicographical tie-breaker).
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

        let effective_priorities: HashMap<String, u32> = ready_tasks
            .iter()
            .map(|n| {
                (
                    n.frontmatter.id.clone(),
                    self.effective_priority_for_task(&n.frontmatter.id),
                )
            })
            .collect();

        let mut keys: HashMap<String, NodeKey> = HashMap::new();
        for node in &ready_tasks {
            keys.insert(
                node.frontmatter.id.clone(),
                self.next_task_key(node, &blocking_counts, &effective_priorities),
            );
        }

        ready_tasks.sort_by(|a, b| {
            let key_a = &keys[&a.frontmatter.id];
            let key_b = &keys[&b.frontmatter.id];
            key_a.cmp(key_b)
        });

        ready_tasks
    }

    /// Computes strongly connected components (SCCs) for all tasks in the graph.
    pub(crate) fn compute_sccs(&self) -> ordering::SccData {
        let ids: Vec<String> = self.nodes.keys().cloned().collect();
        let adjacency = self.build_adjacency(&ids);
        ordering::compute_sccs(&ids, adjacency)
    }

    /// Builds an adjacency list for the provided IDs based on graph edges.
    fn build_adjacency(&self, ids: &[String]) -> ordering::Adjacency {
        let mut adjacency: ordering::Adjacency = HashMap::new();
        let included: HashSet<String> = ids.iter().cloned().collect();

        for id in ids {
            adjacency.entry(id.clone()).or_default();
        }

        for node in self.nodes.values() {
            if !included.contains(&node.frontmatter.id) {
                continue;
            }
            for dep in &node.frontmatter.needs {
                if included.contains(dep) {
                    adjacency
                        .entry(dep.clone())
                        .or_default()
                        .push(node.frontmatter.id.clone());
                }
            }
        }

        for edges in adjacency.values_mut() {
            edges.sort();
            edges.dedup();
        }

        adjacency
    }
}

#[cfg(test)]
mod tests;
