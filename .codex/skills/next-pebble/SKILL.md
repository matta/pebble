---
name: next-pebble
description: Find the next pebble work to do
---

Find the next pebble work to do.

# SELECTION CRITERIA
- Pass the --json option to all `pebble` commands.
- Honor the needs/blocking status of pebble tasks (see 'pebble show <id> --json').
- **Micro-Chunking Policy**: Favor the smallest possible meaningful change. Each pass should ideally address only **one** (or a subset of one) checklist item.
- **Planning-Only Passes**: A valid "work product" for a session is simply the re-evaluation of task feasibility or the creation of sub-tasks using the Adaptive Decomposition Rules. If the state is complex, **stop** after planning to allow for review before implementation.
- If the next work item is large, unclear, or spans multiple subsystems, use Adaptive Decomposition Rules to break it down into granular sub-tasks BEFORE writing code.

## Adaptive Decomposition Rules
Use checklist-first decomposition and only promote checklist items into child Pebble tasks when complexity justifies it.

1. **Default behavior**:
    - Keep checklist items in the parent task body.
    - Promote to child Pebble tasks only when split criteria below are met.
2. **Split criteria**:
    - `MUST`: the item has independent `needs` or blocks other work.
    - `MUST`: the item requires independent status tracking (`todo`/`in_progress`/`done`) for planning value.
    - `MUST`: the item is expected to span multiple sessions or PRs.
    - `SHOULD`: the item touches multiple subsystems or high-risk surfaces.
    - `SHOULD`: the item requires a design decision, spike, or uncertainty reduction step.
    - `SHOULD`: estimated effort exceeds a single focused implementation session.
    - Promote when any `MUST` is true, or when at least two `SHOULD` criteria are true.
3. **Anti-explosion rule**:
    - Do not split work solely to mirror every checklist line.
    - Prefer narrative markdown checklists unless graph structure materially improves planning.
4. **Recursion rule**:
    - Re-evaluate split criteria after each child task completes.
    - Decompose further only where remaining checklist items meet split criteria.
5. **Traceability rule**:
    - Parent task body must include a `Child Tasks` section mapping checklist items to Pebble IDs when decomposition occurs.


# PROCESS DISCIPLINE
- **Granularity is Safety**: If you find yourself editing multiple files or addressing multiple checklist points, you have likely "over-reached". Back up, split the task, and submit a smaller PR/diff.
- **Reviewability first**: Your goal is to produce changes that can be reviewed in under 2 minutes.

# Completion Gate
[ ] Run the just gauntlet, fix issues until green.
[ ] Synchronize implementation_plan.md and task bodies.
[ ] Report work done.
[ ] Stop and wait.
