## 2026-02-21 - Premature Optimization of Task Parser
**Learning:** Replaced `lines().collect()` with manual string slicing in `parse_task_file` to save allocations. This only yielded a ~2% performance gain (6.87s -> 6.72s for 20k files) but significantly increased code complexity and was previously rejected.
**Action:** Avoid optimizing string parsing unless profiling shows it is a major bottleneck (>10% impact). Prioritize readability over micro-optimizations for <5% gains. Always check for previous similar rejected attempts.

## 2026-02-21 - Efficient Directory Sorting
**Learning:** `sort_by_key` with a key that allocates (e.g., `path()` or `file_name()`) is O(N log N) in allocations. `sort_by_cached_key` reduces this to O(N).
**Action:** Use `sort_by_cached_key` when sorting by derived properties that involve allocation, especially in hot paths like file system traversal.

## 2026-02-21 - Quadratic Graph Traversal in `count_blocking`
**Learning:** Naive DFS for "reachability count" called inside a loop over nodes (e.g., sorting tasks) results in $O(N^2)$ behavior in dense graphs or shared chains (e.g., N ready tasks blocking a chain of N tasks). This can be catastrophic (seconds vs ms).
**Action:** Use batched processing with transitive closure algorithms (like Tarjan's SCC + Bitsets) to reduce complexity to $O(N^2 / 64)$ or better when computing properties for many nodes simultaneously.
