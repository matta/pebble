+++
id = "pebl-41j62swwnm"
title = "RFC005-7 Migration script (dry run + backup plan)"
status = "todo"
priority = 0
created_at = 2026-03-01T16:42:26.492454988+00:00
needs = ["pebl-jfi8jsyoai"]
tags = ["planning", "rfc005"]
+++
Execution Note (Manual Tracking During YAML Migration):
- `cargo pebble` task reads are temporarily non-functional until RFC005-8 converts all `docs/pebble/*.md` files to YAML frontmatter.
- During this window, mark task state changes by directly editing frontmatter fields in task files (`status`, `modified_at`, `resolved_at`) instead of using `cargo pebble update`.

Goal:
Prepare a one-off repository migration script to convert existing task files from TOML frontmatter to YAML frontmatter.

Do exactly this:
1. Add a script (Python is allowed) that targets `docs/pebble/*.md`.
2. Script must support a dry-run mode that reports planned file changes without writing.
3. Script must preserve task body text unchanged.
4. Script must preserve frontmatter field values and IDs exactly.
5. Document exact invocation commands in this task body.

Acceptance Criteria:
- Script exists in-repo and can run in dry-run mode.
- Dry-run output clearly lists files that would change.
- No task files are modified in this step.
