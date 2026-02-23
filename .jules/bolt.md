## 2026-02-21 - Premature Optimization of Task Parser
**Learning:** Replaced `lines().collect()` with manual string slicing in `parse_task_file` to save allocations. This only yielded a ~2% performance gain (6.87s -> 6.72s for 20k files) but significantly increased code complexity and was previously rejected.
**Action:** Avoid optimizing string parsing unless profiling shows it is a major bottleneck (>10% impact). Prioritize readability over micro-optimizations for <5% gains. Always check for previous similar rejected attempts.
