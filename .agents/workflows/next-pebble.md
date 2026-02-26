---
name: next-pebble
description: Find the next pebble work to do
---

Find the next pebble work to do.

# SELECTION CRITERIA
- Run pebble with `cargo pebble <ARGS>`.
- Pass the --json opion to all `pebble` commands.
- Honor the needs/blocking status of pebble tasks (see 'pebble show <id> --json').
- Keep work focused and manageable. A single pebble may contain many checklist items; you need not complete them all.
- If the next work item is large or unclear or needs planning, use Adaptive Decomposition Rules to break it down.

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


# Completion Gate
[ ] Run the just gauntlet, fix issues until green.
[ ] Report work done.
[ ] Stop and wait.
