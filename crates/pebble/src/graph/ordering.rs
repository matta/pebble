use super::{NodeKey, TaskGraph};
use crate::models::TaskNode;
use std::cmp::{Ordering, Reverse};
use std::collections::{HashMap, HashSet};

/// Adjacency list mapping each task ID to the IDs of tasks that depend on it.
type Adjacency = HashMap<String, Vec<String>>;

/// Output of Tarjan's SCC algorithm: groups of tasks and a reverse-lookup index.
#[derive(Clone, Debug)]
struct SccData {
    /// Strongly connected components; each inner `Vec` is one SCC (may be a cycle).
    sccs: Vec<Vec<String>>,
    /// Maps each task ID to the index of its SCC in `sccs`.
    index: HashMap<String, usize>,
}

/// Order tasks using the dependency-aware default ordering rules.
pub(super) fn default_order<'a>(
    graph: &'a TaskGraph,
    nodes: Vec<&'a TaskNode>,
) -> Vec<&'a TaskNode> {
    if nodes.len() <= 1 {
        return nodes;
    }

    let ids = collect_ids(&nodes);
    let id_to_node = build_id_to_node(&nodes);
    let adjacency = build_adjacency(&ids, &id_to_node);
    let blocking_counts = compute_blocking_counts(graph, &ids);

    let scc_data = compute_sccs(&ids, &adjacency);
    let scc_keys = scc_keys(&scc_data.sccs, &id_to_node, &blocking_counts);
    let ordered_sccs = topo_order_sccs(&scc_data.sccs, &scc_data.index, &adjacency, &scc_keys);

    let mut ordered_nodes: Vec<&TaskNode> = Vec::with_capacity(ids.len());
    for scc_idx in ordered_sccs {
        let scc = &scc_data.sccs[scc_idx];
        ordered_nodes.extend(order_scc_nodes(scc, &id_to_node, &adjacency));
    }

    ordered_nodes
}

/// Builds a [`NodeKey`] for a single task, using the pre-computed blocking count.
fn node_key(node: &TaskNode, blocking_counts: &HashMap<String, usize>) -> NodeKey {
    let blocking_count = *blocking_counts.get(&node.frontmatter.id).unwrap_or(&0);
    let priority = node.frontmatter.priority.map(u32::from).unwrap_or(u32::MAX);

    NodeKey {
        blocking_count: Reverse(blocking_count),
        priority,
        created_at: node.frontmatter.created_at,
        id: node.frontmatter.id.clone(),
    }
}

/// Extracts task IDs from a node slice, sorted and deduplicated.
fn collect_ids(nodes: &[&TaskNode]) -> Vec<String> {
    let mut ids: Vec<String> = nodes
        .iter()
        .map(|node| node.frontmatter.id.clone())
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// Builds a map from task ID to node reference for fast lookup.
fn build_id_to_node<'a>(nodes: &[&'a TaskNode]) -> HashMap<String, &'a TaskNode> {
    let mut map = HashMap::new();
    for node in nodes {
        map.insert(node.frontmatter.id.clone(), *node);
    }
    map
}

/// Builds the forward adjacency list restricted to the given task IDs.
///
/// Each entry maps a task ID to the IDs of tasks within `ids` that list it as a need
/// (i.e. "who depends on me among the given set"). Edges to IDs outside `ids` are ignored.
fn build_adjacency(ids: &[String], id_to_node: &HashMap<String, &TaskNode>) -> Adjacency {
    let mut adjacency: Adjacency = HashMap::new();
    let included: HashSet<String> = ids.iter().cloned().collect();

    for id in ids {
        adjacency.entry(id.clone()).or_default();
    }

    for node in id_to_node.values() {
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

/// Computes the graph-wide transitive blocking count for each ID in `ids`.
fn compute_blocking_counts(graph: &TaskGraph, ids: &[String]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for id in ids {
        counts.insert(id.clone(), graph.count_blocking(id));
    }
    counts
}

/// Returns the representative [`NodeKey`] for each SCC (the minimum key among its members).
fn scc_keys(
    sccs: &[Vec<String>],
    id_to_node: &HashMap<String, &TaskNode>,
    blocking_counts: &HashMap<String, usize>,
) -> Vec<NodeKey> {
    sccs.iter()
        .map(|scc| {
            scc.iter()
                .filter_map(|id| id_to_node.get(id))
                .map(|node| node_key(node, blocking_counts))
                .min()
                .expect("SCC must contain at least one node")
        })
        .collect()
}

/// Produces a topological ordering of SCCs, breaking ties using their representative keys.
///
/// Runs a modified Kahn's algorithm on the SCC DAG: at each step the available
/// (zero in-degree) SCC with the smallest key is emitted, so the overall order
/// is both topologically valid and deterministic.
fn topo_order_sccs(
    sccs: &[Vec<String>],
    scc_index: &HashMap<String, usize>,
    adjacency: &Adjacency,
    scc_keys: &[NodeKey],
) -> Vec<usize> {
    let scc_count = sccs.len();
    let mut scc_edges: Vec<Vec<usize>> = vec![Vec::new(); scc_count];
    let mut indegree = vec![0usize; scc_count];
    let mut scc_edge_sets: Vec<HashSet<usize>> = vec![HashSet::new(); scc_count];

    for (from, tos) in adjacency {
        let from_idx = scc_index[from];
        for to in tos {
            let to_idx = scc_index[to];
            if from_idx == to_idx {
                continue;
            }
            if scc_edge_sets[from_idx].insert(to_idx) {
                indegree[to_idx] += 1;
            }
        }
    }

    for (idx, set) in scc_edge_sets.into_iter().enumerate() {
        let mut edges: Vec<usize> = set.into_iter().collect();
        edges.sort();
        scc_edges[idx] = edges;
    }

    let mut available: Vec<usize> = (0..scc_count).filter(|i| indegree[*i] == 0).collect();
    let mut ordered: Vec<usize> = Vec::with_capacity(scc_count);

    while !available.is_empty() {
        available.sort_by(|a, b| scc_keys[*a].cmp(&scc_keys[*b]));
        let scc_idx = available.remove(0);
        ordered.push(scc_idx);

        for &neighbor in &scc_edges[scc_idx] {
            indegree[neighbor] = indegree[neighbor].saturating_sub(1);
            if indegree[neighbor] == 0 {
                available.push(neighbor);
            }
        }
    }

    ordered
}

/// Returns `true` if the given SCC represents a dependency cycle.
///
/// An SCC is a cycle when it contains more than one node, or when a single node
/// lists itself as a need (self-loop).
fn is_cycle(scc: &[String], adjacency: &Adjacency) -> bool {
    scc.len() > 1
        || scc
            .first()
            .map(|id| adjacency.get(id).is_some_and(|edges| edges.contains(id)))
            .unwrap_or(false)
}

/// Orders the nodes within a single SCC for output.
///
/// For acyclic SCCs (single nodes with no self-loop) the original order is
/// preserved. For cycles, nodes are sorted by `created_at` then ID.
fn order_scc_nodes<'a>(
    scc: &[String],
    id_to_node: &HashMap<String, &'a TaskNode>,
    adjacency: &Adjacency,
) -> Vec<&'a TaskNode> {
    let mut scc_nodes: Vec<&TaskNode> = scc
        .iter()
        .filter_map(|id| id_to_node.get(id).copied())
        .collect();

    if is_cycle(scc, adjacency) {
        scc_nodes.sort_by(|a, b| {
            let cmp = a.frontmatter.created_at.cmp(&b.frontmatter.created_at);
            if cmp != Ordering::Equal {
                return cmp;
            }
            a.frontmatter.id.cmp(&b.frontmatter.id)
        });
    }

    scc_nodes
}

/// Runs Tarjan's SCC algorithm on the given IDs and adjacency list.
fn compute_sccs(ids: &[String], adjacency: &Adjacency) -> SccData {
    Tarjan::new(adjacency).run(ids)
}

/// Tarjan's strongly connected components (SCC) walker for dependency graphs.
///
/// This is the classic Tarjan algorithm for SCC detection, used here to group
/// dependency cycles so the default ordering can treat cycles as a single unit.
struct Tarjan<'a> {
    index: usize,
    indices: HashMap<String, usize>,
    lowlink: HashMap<String, usize>,
    stack: Vec<String>,
    on_stack: HashSet<String>,
    adjacency: &'a Adjacency,
    sccs: Vec<Vec<String>>,
}

impl<'a> Tarjan<'a> {
    /// Creates a new Tarjan walker over the given adjacency list.
    fn new(adjacency: &'a Adjacency) -> Self {
        Self {
            index: 0,
            indices: HashMap::new(),
            lowlink: HashMap::new(),
            stack: Vec::new(),
            on_stack: HashSet::new(),
            adjacency,
            sccs: Vec::new(),
        }
    }

    /// Executes the algorithm over all provided IDs and returns the grouped [`SccData`].
    fn run(mut self, ids: &[String]) -> SccData {
        for id in ids {
            if !self.indices.contains_key(id) {
                self.strongconnect(id);
            }
        }

        let mut scc_index: HashMap<String, usize> = HashMap::new();
        for (idx, scc) in self.sccs.iter().enumerate() {
            for id in scc {
                scc_index.insert(id.clone(), idx);
            }
        }

        SccData {
            sccs: self.sccs,
            index: scc_index,
        }
    }

    /// Visits a single node, recursively visiting unvisited neighbors and
    /// emitting an SCC when a root is detected.
    fn strongconnect(&mut self, v: &str) {
        self.indices.insert(v.to_string(), self.index);
        self.lowlink.insert(v.to_string(), self.index);
        self.index += 1;
        self.stack.push(v.to_string());
        self.on_stack.insert(v.to_string());

        if let Some(neighbors) = self.adjacency.get(v) {
            for w in neighbors {
                if !self.indices.contains_key(w) {
                    self.strongconnect(w);
                    if let (Some(low_v), Some(low_w)) =
                        (self.lowlink.get(v).copied(), self.lowlink.get(w).copied())
                        && low_w < low_v
                    {
                        self.lowlink.insert(v.to_string(), low_w);
                    }
                } else if self.on_stack.contains(w)
                    && let (Some(low_v), Some(index_w)) =
                        (self.lowlink.get(v).copied(), self.indices.get(w).copied())
                    && index_w < low_v
                {
                    self.lowlink.insert(v.to_string(), index_w);
                }
            }
        }

        let is_root = self.indices.get(v) == self.lowlink.get(v);
        if is_root {
            let mut scc: Vec<String> = Vec::new();
            while let Some(w) = self.stack.pop() {
                self.on_stack.remove(&w);
                scc.push(w.clone());
                if w == v {
                    break;
                }
            }
            self.sccs.push(scc);
        }
    }
}
