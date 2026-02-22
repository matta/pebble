# Pebble Graph Semantics & Logic Layer

This document defines the core logic and graph traversal rules for Pebble. Instead of preemptive validation and complex domain logic, Pebble relies on pure, strict graph evaluations to determine task execution states.

## The One True Edge (`deps`)

Pebble models relationships using a pure directed graph.
* **Single Structural Edge**: There is exactly one structural edge: `deps` (dependencies).
* **Collapsing Hierarchy and Temporality**: Both hierarchical composition (Epics and subtasks) and temporal ordering (execution steps) are collapsed into this single edge.
    * If Task E is an Epic requiring tasks X and Y, Task E simply lists `deps: [X, Y]`.
    * If Task B must wait for Task A, Task B lists `deps: [A]`.
This severely simplifies traversal rules by completely eliminating the need for cascading effective statuses or mixed loop handling between different edge types.

## Absolute Readiness Rule

Readiness is computed strictly at read time, not stored or cascaded.
* **Rule definition**: A task is `ready` if and only if:
    1. Its local status is actionable (`todo` or `in_progress`).
    2. **EVERY** task listed in its `deps` array exists.
    3. **EVERY** task listed in its `deps` array has a terminal status (`done` or `canceled`).

## Computed Graph Fields

The following fields are derived at read time and included in every `TaskObject` emitted by the CLI. They are never stored in frontmatter.

* **`is_ready`** — `true` when the task satisfies the Absolute Readiness Rule above; `false` otherwise.
* **`blocked_by`** — The subset of this task's `deps` that are either missing (no file with that ID exists) or non-terminal (status is `todo` or `in_progress`). When `blocked_by` is empty and the task's own status is actionable, `is_ready` is `true`.
* **`blocking`** — The list of task IDs whose `deps` array directly includes this task's ID (the inverse edge of `deps`). For the purpose of dynamic scoring (`len(blocking)` in the sort key), the count is **transitive**: it includes all tasks recursively reachable downstream through dependency edges, not just direct dependents.

## Permissive Writes, Strict Evaluation

Pebble embraces a "Permissive Writes, Strict Evaluation" philosophy for authoring the graph, treating the CLI not as a strict compiler, but as a fluid evaluation engine.
* **Dangling Pointers**: If a user references a dependency that does not exist, the task is strictly **NOT ready**. The read path simply views it as a missing prerequisite without panicking.
* **Cyclic Dependencies**: If a user authors a dependency cycle (e.g., A depends on B, B depends on A), neither node will ever satisfy the readiness rule. Therefore, neither will ever surface in the "ready" queue. It naturally and perfectly models a deadlock.
The write path allows these structures without failing. Preemptive validation and cycle breaking are intentionally omitted. `pebble doctor` is available as a diagnostic tool for human intervention, but the core graph behaves perfectly regardless of anomalies.

## Dynamic Scoring for Starvation Prevention

Because structural hierarchy has been flattened, the risk of priority inversion (where a low-priority task blocks a massive, high-priority epic) exists. Instead of calculating and storing transitive `effective_priority` up the chain, Pebble utilizes runtime dynamic scoring.

`pebble next` (or `pebble list --is-ready`) ranks the ready frontier using the following sort key tuple: **`(len(blocking) DESC, priority ASC, created_at ASC)`**.

The general default sort for `pebble list` prepends a topological tier: (1) topological order respecting `deps`, (2) blocking count descending, (3) priority ascending, (4) `created_at` ascending. When `--is-ready` is active, all returned tasks are at the dependency frontier, so topological ordering has no practical effect and the sort reduces to the tuple above. See `cli-contract.md` for the full specification.
1. **Count of Downstream Tasks Blocked (Primary Key)**: `blocking` counts all tasks **transitively** reachable downstream through dependency edges (PageRank-inspired). If A is depended on by B, and B is depended on by C, then A's blocking count is 2. A task blocking a larger number of downstream tasks is forced to the top of the queue. This mathematically ensures that a critical bottleneck—even if its local priority is low—is surfaced over isolated tasks.
2. **Local Priority (Tiebreaker 1)**: The `priority` field defined in the task's frontmatter. Tasks with no `priority` sort after all prioritized tasks.
3. **Creation Time (Tiebreaker 2)**: The oldest task wins.

This algorithm provides starvation prevention natively, completely overriding and replacing the old transitive priority propagation model.
