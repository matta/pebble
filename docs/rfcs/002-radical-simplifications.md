[STATUS: FROZEN]

# Radical Simplifications for Pebble (The "Permissive Writes, Strict Evaluation" Design)

This document preserves the core design philosophies that pivoted Pebble from a strict, normalized relational-database-like system to a highly fluid, simple, graph-based tracker following the "Permissive Writes, Strict Evaluation" principle heavily inspired by Git's content-addressable DAG.

## 1. The One True Edge (`deps`)
Instead of dividing structural relationships into `parent/child` (hierarchy) and `after` (temporal prerequisites) which requires complex graph traversal and mixed-loop handling, Pebble uses exactly one structural edge: `deps` (dependencies).
* If Task A cannot be started because Task B must happen first ➔ Task A has a `dep` on Task B.
* If Task E is an "Epic" made of Tasks X, Y, and Z ➔ Task E simply has a `dep` on X, Y, and Z.

Mathematically and practically, an Epic is just a task that requires other tasks to finish before it can be closed. By collapsing hierarchy and temporality into a single directed edge, we dramatically reduce graph traversal rules.

## 2. Permissive Writes, Strict Evaluation (Cycles & Dangling Pointers)
Instead of proactively "handling" invalid data, enforcing "fail-safe" vs "fail-open", or blocking writes to prevent cycles, Pebble simply defines what it means to be ready:
**Rule: A task is "ready" if and only if all of its `deps` exist and are closed (`done` or `canceled`).**
* **Dangling Pointers:** If A depends on Z, and Z doesn't exist, is Z closed? No. Therefore, A is blocked.
* **Cyclical Dependencies:** If A depends on B, and B depends on A, neither will ever close. Therefore, neither will ever emerge in the "ready" queue. It perfectly models a deadlock.

There are no strict cycle-breaker algorithms or preemptive write validation panics. The data layer accepts whatever the user authors. We rely on a `pebble doctor` command to inform the user: "Hey, you have a cycle here," and the human edits the file to resolve it.

## 3. The Death of `paused` and `effective_status`
To simplify state management, the `paused` state and cascading `effective_status` are removed. Waiting *is* a task. If you are waiting on a vendor, you do not pause an Epic—you create a task named "Wait for Vendor" and depend on it. This keeps state local, absolute, and immutable from the outside. A task's status is either actionable (`todo`, `in_progress`) or terminal (`done`, `canceled`).

## 4. Directional Backlinks over Symmetrical `related`
Enforcing symmetry on `related` links requires self-healing and bi-directional write locks. Instead, Pebble drops `related` from the schema entirely. If tasks overlap or are related, users write standard Markdown links (e.g., `[See also: proj-234](proj-234.md)`) in the body. When `pebble show` is run, it dynamically queries the filesystem for "who links to me?" (like Roam or Obsidian). No enforcement, no self-healing, zero write complexity.

## 5. Dynamic PageRank Scoring over Transitive Priority
Instead of baking "starvation prevention" into the data schema (pushing `effective_priority` up the chain), priority inherits a simple local integer `priority` in the frontmatter. Starvation prevention becomes a runtime ranking algorithm for the `pebble next` command. 
`pebble next` finds the "ready frontier" and scores it dynamically using: `Local Priority + Count of Downstream Tasks Blocked`. A small task blocking a massive Epic naturally bubbles to the top of the queue seamlessly.
