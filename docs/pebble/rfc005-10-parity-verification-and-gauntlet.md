---
id: pebl-53ae3r5qrk
title: RFC005-10 Parity verification and gauntlet
status: done
priority: 0
created_at: 2026-03-01T16:43:25.890222517+00:00
modified_at: 2026-03-01T17:36:44.426656883+00:00
resolved_at: 2026-03-01T17:33:03.445059895+00:00
needs:
  - pebl-i0eszfcxas
tags:
  - planning
  - rfc005
---

Execution Note (Manual Tracking During YAML Migration):
- If RFC005-8 is not yet complete, update this task status manually in frontmatter.
- After RFC005-8 conversion completes, use `cargo pebble` commands normally for parity verification steps and final status updates.

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

Execution Evidence (current pass):
- Snapshot commands:
  - `cargo pebble list --json > artifacts/rfc005-parity/after-list.json` (exit `0`)
  - `cargo pebble next --json > artifacts/rfc005-parity/after-next.json` (exit `0`)
  - `cargo pebble list --is-ready --json > artifacts/rfc005-parity/after-list-ready.json` (exit `0`)
  - `git add artifacts/rfc005-parity/after-list.json artifacts/rfc005-parity/after-next.json artifacts/rfc005-parity/after-list-ready.json` (success)
- Snapshot comparison summary vs RFC005-1 baseline:
  - `list` task count changed `29 -> 20` (9 fewer in post snapshot).
  - `list --is-ready` task count stayed `16 -> 16`.
  - `next` changed from `pebl-urd2fpbmfk` to `pebl-53ae3r5qrk`.
  - IDs missing from post `list` are exactly RFC005 tasks completed during migration (`pebl-urd2fpbmfk`, `pebl-zjpn6fbfmp`, `pebl-rm0kn1fvli`, `pebl-gr537c9can`, `pebl-ombr9kv475`, `pebl-jfi8jsyoai`, `pebl-41j62swwnm`, `pebl-1cq47q454u`, `pebl-i0eszfcxas`), which is expected because default `list` omits terminal states.
- Gauntlet status:
  - `just check` passed (exit `0`).
  - `just test` passed (exit `0`).
  - All workspace tests passed after converting remaining integration fixtures to YAML and fixing clippy/formatting issues.
  - Final scrub verification for TOML usage in source code:
    - Runtime task parsing/writing code no longer uses TOML-specific types (`toml_datetime::Datetime`, `toml::Value` removed).
    - Task timestamps now use `chrono::DateTime<Utc>`, and task unknown fields use `serde_json::Value`.
    - Remaining runtime TOML parse usage is config-only: `crates/pebble/src/config.rs` (`toml::from_str` for `.pebble/config.toml`).

Cleanup after parity sign-off:
- Removed parity evidence files via `git rm artifacts/rfc005-parity/*.json` (all six before/after JSON snapshots).

Conclusion:
- Migration parity evidence is documented, required gauntlet commands pass, cleanup was completed with `git rm`, and exit criteria are met.
