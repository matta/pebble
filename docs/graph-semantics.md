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

## Permissive Writes, Strict Evaluation

Pebble embraces a "Permissive Writes, Strict Evaluation" philosophy for authoring the graph, treating the CLI not as a strict compiler, but as a fluid evaluation engine.
* **Dangling Pointers**: If a user references a dependency that does not exist, the task is strictly **NOT ready**. The read path simply views it as a missing prerequisite without panicking.
* **Cyclic Dependencies**: If a user authors a dependency cycle (e.g., A depends on B, B depends on A), neither node will ever satisfy the readiness rule. Therefore, neither will ever surface in the "ready" queue. It naturally and perfectly models a deadlock.
The write path allows these structures without failing. Preemptive validation and cycle breaking are intentionally omitted. `pebble doctor` is available as a diagnostic tool for human intervention, but the core graph behaves perfectly regardless of anomalies.

## Dynamic Scoring for Starvation Prevention

Because structural hierarchy has been flattened, the risk of priority inversion (where a low-priority task blocks a massive, high-priority epic) exists. Instead of calculating and storing transitive `effective_priority` up the chain, Pebble utilizes runtime dynamic scoring.

`pebble next` (or `pebble list --is-ready`) ranks the ready frontier using the following sort key tuple: **`(len(blocking) DESC, priority ASC, created_at ASC)`**.
1. **Count of Downstream Tasks Blocked (Primary Key)**: A task blocking a larger number of downstream tasks is forced to the top of the queue. This mathematically ensures that a critical bottleneck—even if its local priority is low—is surfaced over isolated tasks.
2. **Local Priority (Tiebreaker 1)**: The `priority` field defined in the task's frontmatter.
3. **Creation Time (Tiebreaker 2)**: The oldest task wins.

This algorithm provides starvation prevention natively, completely overriding and replacing the old transitive priority propagation model.
