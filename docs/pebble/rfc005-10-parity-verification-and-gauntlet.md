+++
id = "pebl-53ae3r5qrk"
title = "RFC005-10 Parity verification and gauntlet"
status = "todo"
priority = 0
created_at = 2026-03-01T16:43:25.890222517+00:00
needs = ["pebl-i0eszfcxas"]
tags = ["planning", "rfc005"]
+++
Goal:
Prove the migration preserved command-visible task information while switching to YAML frontmatter.

Do exactly this:
1. Re-run post-change snapshots:
   - `mkdir -p artifacts/rfc005-parity`
   - `cargo pebble list --json > artifacts/rfc005-parity/after-list.json`
   - `cargo pebble next --json > artifacts/rfc005-parity/after-next.json`
   - `cargo pebble list --is-ready --json > artifacts/rfc005-parity/after-list-ready.json`
   - `git add artifacts/rfc005-parity/after-list.json artifacts/rfc005-parity/after-next.json artifacts/rfc005-parity/after-list-ready.json`
2. Compare before/after snapshots from RFC005-1 and document differences.
3. Confirm differences are only expected formatting/storage-level changes and not task-information regressions.
4. If snapshot files are removed after parity sign-off, delete them with `git rm artifacts/rfc005-parity/*.json` (not plain `rm`).
5. Run `just check` and `just test`.
6. Document command exit codes and key evidence in this task body.

Acceptance Criteria:
- Before/after parity evidence is documented.
- Before/after snapshot JSON files are git-tracked while in use.
- If evidence files are deleted, the deletion is done via `git rm`.
- `just check` and `just test` pass.
- This task can be marked done only when migration exit criteria are demonstrably met.
