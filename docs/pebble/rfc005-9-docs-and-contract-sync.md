---
id: "pebl-i0eszfcxas"
title: "RFC005-9 Docs and contract sync"
status: "todo"
priority: 0
created_at: "2026-03-01T16:43:09.597888821+00:00"
needs: ["pebl-1cq47q454u"]
tags: ["planning", "rfc005"]
---
Execution Note (Manual Tracking During YAML Migration):
- If this task is started before RFC005-8 completion, continue manual frontmatter state edits (`status`, `modified_at`, `resolved_at`) in task files.
- Once RFC005-8 completes successfully, switch back to normal `cargo pebble` task updates.

Goal:
Update all normative docs to reflect YAML frontmatter and remove TOML-frontmatter language.

Do exactly this:
1. Update `docs/schema.md` frontmatter format and schema examples to YAML.
2. Update `docs/cli-contract.md` scanning/error wording for YAML frontmatter.
3. Update root `AGENTS.md` and any other indexed docs that currently claim TOML frontmatter for tasks.
4. Ensure docs keep current graph invariants and semantics unchanged (`needs`, readiness, immutable `id`).

Acceptance Criteria:
- No normative docs still describe task frontmatter as TOML.
- Documentation remains internally consistent with RFC005 and current CLI behavior.
