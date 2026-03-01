---
id: "pebl-urd2fpbmfk"
title: "RFC005-1 Baseline parity snapshot"
status: "done"
priority: 0
created_at: "2026-03-01T16:38:55.000718409+00:00"
modified_at: "2026-03-01T16:50:53.440346091+00:00"
resolved_at: "2026-03-01T16:50:53.440335476+00:00"
needs: []
tags: ["planning", "rfc005"]
---
Goal:
Capture pre-change command output so post-migration parity can be checked mechanically.

Do exactly this:
1. Run `mkdir -p artifacts/rfc005-parity`.
2. Run `cargo pebble list --json > artifacts/rfc005-parity/before-list.json`.
3. Run `cargo pebble next --json > artifacts/rfc005-parity/before-next.json`.
4. Run `cargo pebble list --is-ready --json > artifacts/rfc005-parity/before-list-ready.json`.
5. Run `git add artifacts/rfc005-parity/before-list.json artifacts/rfc005-parity/before-next.json artifacts/rfc005-parity/before-list-ready.json`.
6. Record task-file count with `find docs/pebble -name "*.md" | wc -l` in this task body.
7. Do not edit Rust code in this step.

Acceptance Criteria:
- Three baseline JSON files exist in `artifacts/rfc005-parity/` with the exact names above.
- Baseline JSON files are git-tracked (staged or committed).
- This task body includes the observed task-file count.
- A later agent can use these files for before/after parity checks.



Observed task-file count (docs/pebble/*.md): 54
