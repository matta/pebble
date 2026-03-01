+++
id = "pebl-1cq47q454u"
title = "RFC005-8 Execute task-file conversion"
status = "todo"
priority = 0
created_at = 2026-03-01T16:42:50.803369062+00:00
needs = ["pebl-41j62swwnm"]
tags = ["planning", "rfc005"]
+++
Goal:
Run the one-off conversion on all existing task files and validate the result set.

Do exactly this:
1. Run the migration script in write mode for `docs/pebble/*.md`.
2. Verify every task file now starts with YAML frontmatter delimiter `---`.
3. Spot-check at least 5 files across different task ages for unchanged body content and preserved IDs.
4. Record conversion counts and spot-check file names in this task body.

Acceptance Criteria:
- All task files under `docs/pebble/` are converted to YAML frontmatter.
- Spot-check evidence is documented in this task body.
