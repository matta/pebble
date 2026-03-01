---
id: "pebl-1cq47q454u"
title: "RFC005-8 Execute task-file conversion"
status: "done"
priority: 0
created_at: "2026-03-01T16:42:50.803369062+00:00"
modified_at: "2026-03-01T17:18:05.401965115+00:00"
resolved_at: "2026-03-01T17:18:05.401965115+00:00"
needs: ["pebl-41j62swwnm"]
tags: ["planning", "rfc005"]
---
Execution Note (Manual Tracking During YAML Migration):
- This task is the re-enable point for normal Pebble operations.
- Until this task completes, task status updates must be done by editing frontmatter directly in `docs/pebble/*.md`.
- After this task completes and all task files are YAML-frontmatter, resume normal `cargo pebble` command-based task tracking.

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

Conversion evidence:
- Ran write conversion: `scripts/rfc005_convert_frontmatter.py --glob 'docs/pebble/*.md' --write`
- Script summary: `mode=write matched=54 changed=54 skipped=0`
- Delimiter verification: all files in `docs/pebble/*.md` now begin with `---`.

Spot-check evidence (ID preserved + body unchanged):
- `docs/pebble/issue-dn9egB.md` (`issue-dn9egB`) body unchanged (hash match).
- `docs/pebble/phase-1-core-cli-contract-coverage.md` (`pebl-cdIZGN`) body unchanged (hash match).
- `docs/pebble/rfc005-1-baseline-parity-snapshot.md` (`pebl-urd2fpbmfk`) body unchanged (hash match).
- `docs/pebble/toctou-race-in-slug-collision-loop.md` (`pebl-GoOi96`) body unchanged (hash match).
- `docs/pebble/align-help-json-guidance-text-with-actual-emitted-keys.md` (`pebl-6pv06nwvpl`) body unchanged (hash match).
