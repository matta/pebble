+++
id = "pebl-ez0v1cj5ai"
title = "Plan RFC 005 YAML frontmatter migration"
status = "todo"
priority = 0
created_at = 2026-03-01T16:36:23.238794357+00:00
modified_at = 2026-03-01T17:16:07.128622177+00:00
needs = ["pebl-urd2fpbmfk", "pebl-zjpn6fbfmp", "pebl-rm0kn1fvli", "pebl-gr537c9can", "pebl-ombr9kv475", "pebl-jfi8jsyoai", "pebl-41j62swwnm", "pebl-1cq47q454u", "pebl-i0eszfcxas", "pebl-53ae3r5qrk"]
tags = ["planning", "rfc"]
+++
Execution Policy:
- Use Pebble subtasks only; do one subtask at a time in order.
- Keep each PR/change reviewable in a small chunk.
- Do not add code that explicitly rejects TOML frontmatter; files with TOML frontmatter are treated as missing YAML frontmatter.
- Store baseline/parity JSON evidence in `artifacts/rfc005-parity/` so it is durable and git-tracked.
- If parity evidence files are deleted later, remove them with `git rm artifacts/rfc005-parity/*.json`.

Temporary Tracking Mode (Required Until RFC005-8 Completes):
- Because read-path behavior is now YAML-only and repository task files are still largely TOML frontmatter, `cargo pebble` task queries/updates are temporarily non-functional for this repo.
- During this period, update task status directly in markdown frontmatter (`status`, `modified_at`, `resolved_at`) rather than using `cargo pebble update`.
- Keep frontmatter format unchanged per file until the dedicated conversion step (RFC005-8) runs.
- Immediately after RFC005-8 completes and all task files are YAML-frontmatter, return to normal `cargo pebble` task operations.

Child Tasks (Execution Order):
- [x] RFC005-1 Baseline parity snapshot -> `pebl-urd2fpbmfk`
- [x] RFC005-2 Spec-to-code inventory -> `pebl-zjpn6fbfmp`
- [x] RFC005-3 Read-path tests first (YAML-only detection) -> `pebl-rm0kn1fvli`
- [x] RFC005-4 Implement YAML read path -> `pebl-gr537c9can`
- [x] RFC005-5 Write-path tests first (YAML emit) -> `pebl-ombr9kv475`
- [x] RFC005-6 Implement YAML write path -> `pebl-jfi8jsyoai`
- [ ] RFC005-7 Migration script (dry run + backup plan) -> `pebl-41j62swwnm`
- [ ] RFC005-8 Execute task-file conversion -> `pebl-1cq47q454u`
- [ ] RFC005-9 Docs and contract sync -> `pebl-i0eszfcxas`
- [ ] RFC005-10 Parity verification and gauntlet -> `pebl-53ae3r5qrk`

Exit Criteria:
- Pebble uses YAML frontmatter per RFC 005.
- All task files in this repository are converted to YAML frontmatter.
- `pebble list` and `pebble next` report the same information as before conversion.
