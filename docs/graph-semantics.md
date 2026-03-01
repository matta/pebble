# Pebble Graph Semantics & Logic Layer

This document defines the core logic and graph traversal rules for Pebble. Instead of preemptive validation and complex domain logic, Pebble relies on pure, strict graph evaluations to determine task execution states.

## The One True Edge (`needs`)

Pebble models relationships using a pure directed graph.
* **Single Structural Edge**: There is exactly one structural edge: `needs` (dependencies).
* **Collapsing Hierarchy and Temporality**: Both hierarchical composition (Epics and subtasks) and temporal ordering (execution steps) are collapsed into this single edge.
    * If Task E is an Epic requiring tasks X and Y, Task E simply lists `needs: [X, Y]`.
    * If Task B must wait for Task A, Task B lists `needs: [A]`.
This severely simplifies traversal rules by completely eliminating the need for cascading effective statuses or mixed loop handling between different edge types.

## Absolute Readiness Rule

Readiness is computed strictly at read time, not stored or cascaded.
* **Rule definition**: A task is `ready` if and only if:
    1. Its local status is actionable (`todo` or `in_progress`).
    2. **EVERY** task listed in its `needs` array exists.
    3. **EVERY** task listed in its `needs` array has a terminal status (`done` or `canceled`).

## Computed Graph Fields

The following fields are derived at read time and included in every `TaskObject` emitted by the CLI. They are never stored in frontmatter.

* **`is_ready`** — `true` when the task satisfies the Absolute Readiness Rule above; `false` otherwise.
* **`blocked_by`** — The subset of this task's `needs` that are either missing (no file with that ID exists) or non-terminal (status is `todo` or `in_progress`). When `blocked_by` is empty and the task's own status is actionable, `is_ready` is `true`.
* **`blocking`** — The list of non-terminal (`todo` or `in_progress`) task IDs whose `needs` array directly includes this task's ID (the inverse edge of `needs`). Terminal dependents are excluded because concluded work cannot be blocked. For dynamic scoring, the sort key uses a **transitive blocking count** defined below.

## Permissive Writes, Strict Evaluation

Pebble embraces a "Permissive Writes, Strict Evaluation" philosophy for authoring the graph, treating the CLI not as a strict compiler, but as a fluid evaluation engine.
* **Dangling Pointers**: If a user references a dependency that does not exist, the task is strictly **NOT ready**. The read path simply views it as a missing prerequisite without panicking.
* **Cyclic Dependencies**: If a user authors a dependency cycle (e.g., A depends on B, B depends on A), neither node will ever satisfy the readiness rule. Therefore, neither will ever surface in the "ready" queue. It naturally and perfectly models a deadlock.
The write path allows these structures without failing. Preemptive validation and cycle breaking are intentionally omitted. `pebble check --warn-only` is available as a diagnostic tool for human intervention, and `pebble check --fix` can apply safe deterministic repairs (such as backfilling `created_at`) without rewriting dependency edges, but the core graph behaves perfectly regardless of anomalies.

## Dynamic Scoring for Starvation Prevention

Because hierarchy is flattened into `needs`, a low-priority prerequisite can block high-priority dependents. Rather than propagating priority up the graph, Pebble scores tasks dynamically at read time.

`pebble next` (and `pebble list --is-ready`) ranks the ready frontier by the sort key tuple **`(effective_priority ASC, base_priority ASC, transitive_blocking_count DESC, created_at ASC, id ASC)`**:

1. **Effective Priority** — `min(base_priority, downstream_min_priority)`, where:
   - `base_priority` is the task's own `priority` value, or sentinel `100` when unset.
   - `downstream_min_priority` is the minimum `base_priority` among actionable transitive downstream dependents (reachable by reverse `needs` traversal).
   This means a low/no-priority blocker of urgent work is treated with matching urgency.
2. **Base Priority** — The task's explicit `priority` from frontmatter (lower = higher priority), with unset priority sorting after all explicit values via sentinel `100`. This preserves user intent when `effective_priority` ties.
3. **Transitive Blocking Count** — The count of **unique, non-terminal** (`todo` or `in_progress`) tasks reachable by walking **reverse** `needs` edges from this task (i.e., direct and indirect dependents). The task itself is excluded. Missing IDs are ignored; cycles are handled via visited-set tracking. Traversal stops at terminal tasks (`done`/`canceled`) so completed work does not propagate blocking. Note: `len(blocking)` counts only *direct* non-terminal dependents; the transitive count walks the full downstream graph.
4. **Created At** — Oldest task wins.
5. **ID** — Lexicographic ascending. Guarantees determinism when all other keys are equal.

The general default sort for `pebble list` prepends a topological tier: (1) topological order respecting `needs` (cycles are grouped together and sorted internally by `created_at` then `id`), (2) effective priority ascending, (3) base priority ascending, (4) transitive blocking count descending, (5) `created_at` ascending, (6) `id` ascending. When `--is-ready` is active, all returned tasks are at the dependency frontier, so topological ordering has no practical effect and the sort reduces to the tuple above. See `cli-contract.md` for the full specification.
