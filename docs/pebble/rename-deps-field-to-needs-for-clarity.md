---
id: pebl-rWaJHG
title: Rename deps field to needs for clarity
status: done
priority: 0
created_at: 2026-02-23T05:41:07.031257+00:00
modified_at: 2026-02-23T17:11:11.810272+00:00
resolved_at: 2026-02-23T17:11:11.810267+00:00
tags:
  - rename
  - breaking
---
The word "dependency" is a relational noun with no inherent direction. "A depends on
B" is clear, but "A's dependencies" is ambiguous — it could mean "things A needs" or
"things that need A." This causes persistent confusion in documentation and
conversation.

Rename the `deps` frontmatter field to `needs`. The semantics stay identical: the
array lists tasks that must reach a terminal status before this task is considered
ready.

This pairs cleanly with the existing computed fields:
- `needs` (declared): "I need these tasks done first"
- `blocked_by` (computed): "...and these aren't done yet"
- `blocking` (computed): "these other tasks need me"

## Scope
This is a repo-wide rename touching:
- TOML frontmatter field name in all `docs/pebble/*.md` task files
- `models.rs` struct field + serde annotation
- `graph.rs` / `ordering.rs` references
- `commands.rs` / `commands_write.rs` CLI flags and output
- All test files
- All documentation (`schema.md`, `cli-contract.md`, `graph-semantics.md`, RFCs)
- `AGENTS.md` invariants section
- `implementation_plan.md` dependency rules

No behavioral change. Pure rename. No migration needed since only this repo uses
Pebble.
