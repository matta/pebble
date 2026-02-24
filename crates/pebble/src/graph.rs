use crate::models::TaskNode;
use crate::parser::parse_task_file;
use color_eyre::eyre::Result;
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

mod ordering;

/// A simple bitset implementation for tracking reachability.
#[derive(Clone)]
struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    fn new(size: usize) -> Self {
        let n_words = size.div_ceil(64);
        Self {
            words: vec![0; n_words],
        }
    }

    fn set(&mut self, idx: usize) {
        self.words[idx / 64] |= 1 << (idx % 64);
    }

    fn union_with(&mut self, other: &BitSet) {
        // Ensure same size or grow? We assume fixed size for this usage.
        if self.words.len() < other.words.len() {
            self.words.resize(other.words.len(), 0);
        }
        for (i, word) in other.words.iter().enumerate() {
            self.words[i] |= word;
        }
    }

    fn count_ones(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }
}

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
    pub blocking: HashMap<String, Vec<String>>,
    /// IDs that appeared in multiple task files during load.
    pub duplicate_ids: HashSet<String>,
}

impl TaskGraph {
    /// Recursively collect all Markdown files under `dir` in deterministic order.
    fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<std::result::Result<_, _>>()?;
        // Optimization: use `sort_by_cached_key` with `file_name` instead of `sort_by_key` with `path`.
        // `entry.path()` constructs a new PathBuf (allocation) on every comparison (O(N log N)).
        // `entry.file_name()` constructs a new OsString (allocation), but `sort_by_cached_key` calls it only once per element (O(N)).
        // Sorting by filename within a directory is equivalent to sorting by full path for deterministic traversal.
        entries.sort_by_cached_key(|entry| entry.file_name());

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
    /// Scans the directory for Markdown (`.md`) files, parsing each as a [`TaskNode`].
    /// Files named `AGENTS.md` are explicitly ignored. Files that start with `+++` but
    /// fail to parse result in a warning printed to stderr but do not halt the loading
    /// process. Files that do not start with `+++` are silently ignored.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if the directory cannot be read or if any file read operation fails.
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

                let content = std::fs::read_to_string(&path)?;
                // Skip files that don't start with +++
                if content.starts_with("+++") {
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

    /// Creates a TaskGraph from nodes and a set of duplicated IDs.
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

    /// Determines if a task is "ready" according to absolute readiness rules.
    /// Readiness rule:
    /// 1. Its local status is actionable (todo or in_progress).
    /// 2. EVERY task listed in its needs array exists.
    /// 3. EVERY task listed in its needs array has a terminal status (done or canceled).
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
        // Delegate to batch implementation for consistency, though less efficient for single calls.
        let counts = self.batch_count_blocking(&[task_id.to_string()]);
        *counts.get(task_id).unwrap_or(&0)
    }

    /// Computes blocking counts for a batch of tasks efficiently using bitsets.
    /// This avoids O(N^2) behavior when many tasks share the same downstream dependencies.
    pub fn batch_count_blocking(&self, ids: &[String]) -> HashMap<String, usize> {
        // 1. Build the subgraph of reachable ACTIONABLE nodes.
        let mut subgraph: HashMap<String, Vec<String>> = HashMap::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();

        // Initialize with input IDs
        for id in ids {
            if visited.insert(id.clone()) {
                queue.push_back(id.clone());
            }
        }

        while let Some(u) = queue.pop_front() {
            if let Some(children) = self.blocking.get(&u) {
                let mut actionable_children = Vec::new();
                for v in children {
                    if let Some(node) = self.nodes.get(v)
                        && node.frontmatter.status.is_actionable()
                    {
                        actionable_children.push(v.clone());
                        if visited.insert(v.clone()) {
                            queue.push_back(v.clone());
                        }
                    }
                }
                subgraph.insert(u, actionable_children);
            }
        }

        // 2. Run Tarjan's algorithm to find SCCs and Topological Order (Reverse Topo of SCCs).
        // Note: ordering::Tarjan returns SCCs in reverse topological order (children first).
        let scc_data = ordering::Tarjan::new(&subgraph).run(ids);

        // 3. Map all visited nodes to integers for BitSet.
        let mut node_to_int: HashMap<String, usize> = HashMap::new();
        for (i, id) in visited.iter().enumerate() {
            node_to_int.insert(id.clone(), i);
        }
        let total_nodes = visited.len();

        // 4. Compute reachability using BitSets on the Condensed Graph.
        // scc_data.sccs is in children-first order.
        let mut scc_reachability: Vec<BitSet> = vec![BitSet::new(total_nodes); scc_data.sccs.len()];

        for (scc_idx, scc) in scc_data.sccs.iter().enumerate() {
            let mut reach = BitSet::new(total_nodes);

            // Add nodes in this SCC
            for node in scc {
                if let Some(&idx) = node_to_int.get(node) {
                    reach.set(idx);
                }

                // Union with children SCCs
                if let Some(children) = subgraph.get(node) {
                    for child in children {
                        if let Some(&child_scc_idx) = scc_data.index.get(child)
                            && child_scc_idx != scc_idx
                        {
                            // Since we iterate in reverse topo order, child_scc_idx must have been processed?
                            // Wait, Tarjan output order:
                                // If A -> B. SCC(B) comes before SCC(A).
                                // So yes, child_scc_idx is already processed (smaller index? No, purely order in vec).
                                // But index in vec determines order.
                                // We iterate `scc_data.sccs` from 0..len.
                                // 0 is a sink component.
                                // So we can look up `scc_reachability[child_scc_idx]`.
                                // BUT `child_scc_idx` is an index into `scc_data.sccs`.
                                // We need to be sure `child_scc_idx` < `scc_idx`.
                                // Tarjan guarantees reverse topological order.
                                // So sinks come first.
                                // So children are processed before parents.
                                // So `scc_reachability[child_scc_idx]` is ready.
                                reach.union_with(&scc_reachability[child_scc_idx]);
                            }
                        }
                    }
                }
            }
            // For nodes within the same SCC, they all reach each other + descendants.
            // Since we unioned all `reach` into one local `reach`, it effectively merges them.
            // Now store it.
            scc_reachability[scc_idx] = reach;
        }

        // 5. Extract counts.
        let mut results = HashMap::new();
        for id in ids {
            if let Some(&scc_idx) = scc_data.index.get(id) {
                let count = scc_reachability[scc_idx].count_ones();
                // Exclude self.
                results.insert(id.clone(), count.saturating_sub(1));
            } else {
                // Should not happen if id was in ids passed to subgraph
                results.insert(id.clone(), 0);
            }
        }
        results
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
        // Use batch implementation to avoid redundant traversals.
        let ready_ids: Vec<String> = ready_tasks
            .iter()
            .map(|n| n.frontmatter.id.clone())
            .collect();
        let blocking_counts = self.batch_count_blocking(&ready_ids);

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

#[cfg(test)]
mod perf_tests {
    use super::*;
    use crate::models::{TaskFrontmatter, TaskStatus};
    use std::time::Instant;
    use toml_datetime::Datetime;

    fn make_node(id: &str, needs: Vec<&str>) -> TaskNode {
        TaskNode {
            path: std::path::PathBuf::from(format!("{}.md", id)),
            frontmatter: TaskFrontmatter {
                id: id.to_string(),
                title: id.to_string(),
                status: TaskStatus::Todo,
                priority: None,
                created_at: Datetime {
                    date: None,
                    time: None,
                    offset: None,
                },
                modified_at: None,
                resolved_at: None,
                needs: needs.iter().map(|s| s.to_string()).collect(),
                tags: vec![],
            },
            body: String::new(),
        }
    }

    #[test]
    fn test_count_blocking_quadratic_behavior() {
        let mut nodes = HashMap::new();
        let n = 1000; // Adjust for measurable impact

        // Create a long chain: 0 <- 1 <- ... <- n-1
        // Node 0 is the root (blocked by nothing). Node n-1 blocks everything.
        // Wait, dependency direction:
        // A needs B. B blocks A.
        // Chain: A -> B -> C.
        // A needs B. B needs C.
        // If C is Todo. B is blocked. A is blocked.
        // count_blocking(C): Returns count of blocked tasks (B, A).

        // We want many ready tasks pointing to the same subgraph.
        // Let's have C_0 ... C_M as a chain.
        // And R_0 ... R_K as ready tasks.
        // Each R_i needs C_0.
        // If C_0 is Todo.
        // R_i is NOT ready.
        // Wait, get_next_tasks calls count_blocking on READY tasks.
        // A ready task has NO open dependencies.
        // So R_i must not need anything open.
        // But R_i can BLOCK things.
        // So R_i -> B_1 -> B_2 ...
        // R_i blocks B_1. B_1 blocks B_2.
        // So if we have R_0 ... R_K ready.
        // And they all point to the SAME chain B_1 -> B_2 ... ?
        // R_i blocks B_1.
        // count_blocking(R_i) = {B_1, B_2, ...}.
        // This is the case.

        // Create chain B_0 <- B_1 <- ... <- B_M.
        // B_0 needs B_1. (B_1 blocks B_0).
        // Wait, "A needs B". B blocks A.
        // Chain: B_M -> ... -> B_1 -> B_0.
        // B_M needs B_{M-1}.
        // B_0 is the "root" blocker?
        // No.
        // If A needs B.
        // B blocks A.
        // If we want R_i to block a chain.
        // R_i <- C_0 <- C_1 ...
        // C_0 needs R_i.
        // C_1 needs C_0.
        // R_i is ready (needs nothing).
        // C_0 is blocked by R_i.
        // C_1 is blocked by C_0.
        // count_blocking(R_i) should include C_0, C_1 ...

        // Shared chain:
        // C_0 needs R_0, R_1, ... R_K.
        // C_1 needs C_0.
        // ...

        // If C_0 needs R_0. R_0 blocks C_0.
        // C_0 also needs R_1. R_1 blocks C_0.
        // count_blocking(R_0) -> {C_0, C_1...}
        // count_blocking(R_1) -> {C_0, C_1...}

        // Construct graph:
        // R_0...R_999 (Ready).
        // C_0...C_999 (Chain).
        // C_0 needs all R_i.
        // C_{i+1} needs C_i.

        for i in 0..n {
            nodes.insert(format!("r{}", i), make_node(&format!("r{}", i), vec![]));
        }

        let mut c0_needs = Vec::new();
        for i in 0..n {
            c0_needs.push(format!("r{}", i));
        }
        nodes.insert(
            "c0".to_string(),
            make_node("c0", c0_needs.iter().map(|s| s.as_str()).collect()),
        );

        for i in 1..n {
            nodes.insert(
                format!("c{}", i),
                make_node(&format!("c{}", i), vec![&format!("c{}", i - 1)]),
            );
        }

        let graph = TaskGraph::new(nodes);

        let _start = Instant::now();
        // Mimic get_next_tasks logic for blocking counts
        let ready_tasks: Vec<&TaskNode> = graph
            .nodes
            .values()
            .filter(|n| graph.is_ready(&n.frontmatter.id))
            .collect();
        assert_eq!(ready_tasks.len(), n);

        let ready_ids: Vec<String> = ready_tasks
            .iter()
            .map(|n| n.frontmatter.id.clone())
            .collect();
        let counts = graph.batch_count_blocking(&ready_ids);

        let mut total_blocking = 0;
        for count in counts.values() {
            total_blocking += count;
        }

        // n=1000. 1000 * 1000 = 1M traversals.
        // With optimization (batching + bitsets), it should be fast (< 100ms).
        // Correctness check: each R_i blocks C_0...C_{n-1} (n tasks).
        // total_blocking should be n * n.
        assert_eq!(total_blocking, n * n);
    }
}
