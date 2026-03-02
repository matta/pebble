---
id: pebl-pCyebx
title: Phase 5 Test Coverage for Gaps
status: done
created_at: 2026-02-23T01:36:06.795969+00:00
modified_at: 2026-03-02T01:29:36.135944231+00:00
needs:
  - pebl-fFdi_z
  - pebl-z5v17c3486
  - pebl-zlijanufss
  - pebl-n6xopu3hq3
tags:
  - bootstrap
  - self_hosted
---

Checklist:
- [x] P5.1 Recursive scan and duplicate ID behavior tests (read + write)
- [x] P5.2 Blocking list and transitive blocking count tests
- [x] P5.3 Default list ordering and `--sort` tie-breaker tests
- [x] P5.4 `list` filter and limit tests
- [x] P5.5 `search` behavior tests
- [x] P5.6 `config get` and `help-json` output shape tests
- [x] P5.7 `--json` purity and stdout/stderr separation tests
- [x] P5.8 Exit code mapping tests (runtime vs usage errors)
- [x] P5.9 `archive` threshold and collision behavior tests
- [x] P5.10 Slug transliteration + relative output + trailing newline tests

## Coverage Analysis (completed investigation)

All integration tests live in `crates/pebble/tests/cli_*.rs` and use `support::setup_test_env()`.
Unit tests live inline in the source modules (`#[cfg(test)] mod tests`).

### P5.1 — ALREADY COVERED ✓
File: `tests/cli_scan_duplicates.rs` (7 tests)
- Recursive nested `.md` discovery (list + show)
- Duplicate ID skip on read (list, show), runtime error on write (update)
- Unique ID unaffected by other duplicates
- Warning stderr mentions every file path for each duplicate ID

### P5.2 — ALREADY COVERED ✓
File: `tests/cli_blocking.rs` (3 tests)
- `show --json` `blocking` includes direct non-terminal dependents only
- `list --sort -blocking` uses transitive blocking counts
- default list ordering uses blocking count to break ties

### P5.3 — ALREADY COVERED ✓
Unit tests in `src/graph/tests.rs`: `test_default_order_respects_needs_and_priority`,
`test_default_order_cycle_grouping_created_at`, `test_default_order_id_tiebreaker`.
Integration tests in `tests/cli_list_sort.rs`: title descending, priority tie-breakers.

### P5.4 — ALREADY COVERED ✓
File: `tests/cli_list_filters.rs` (8 tests)
- `--status` OR semantics (including done without `--all`)
- `--tag` AND semantics
- `--need` OR semantics
- `--priority` OR semantics
- `--all` includes closed tasks
- `--is-ready` absolute readiness
- `--limit` row restriction
- `ls` alias equivalence

### P5.5 — ALREADY COVERED ✓
File: `tests/cli_search.rs` (4 tests)
- Title + body case-insensitive matching
- Default list ordering preserved
- No match → exit code 1 + error on stderr (both human and JSON modes)

### P5.6 — ALREADY COVERED ✓
Files: `tests/cli_config.rs` (3 tests), `tests/cli_help_json.rs` (6 tests)
- `config get` human + JSON + unknown key (exit 2)
- `help-json` valid schema, core commands listed, command descriptions non-empty,
  add command options, check command flags/output, help-json not a global flag

### P5.7 — ALREADY COVERED ✓
Files: `tests/cli_json.rs` (4 tests), `tests/cli_json_purity_extended.rs` (6 tests)
- All commands tested for JSON-only stdout + empty stderr on success:
  list, next, show, add, update, search, config get, archive, help-json, init
- Error paths verified: stdout empty + stderr non-empty

### P5.8 — ALREADY COVERED ✓
File: `tests/cli_errors.rs` (12 tests)
- runtime errors: missing task ID, no-project cases
- usage errors: invalid status, invalid priority, missing required args, absolute `init --dir`
- invalid `list --sort` field behavior is covered with asserted error text and current exit code mapping

### P5.9 — ALREADY COVERED ✓
File: `tests/cli_archive.rs` (4 tests)
- archives old resolved tasks and confirms file move to `tasks/archive/`
- skips recently resolved tasks
- skips non-terminal tasks
- appends numeric suffix on archive path collisions

### P5.10 — ALREADY COVERED ✓
Unit tests in `src/commands_add.rs` (13 tests): basic slugify, mixed separators, empty fallback,
reserved chars, transliteration (café, über, naïve, résumé, Æneid, 日本語),
newline-producing Unicode, unknown chars, empty transliterations, multi-char transliterations,
safety invariant checks.
Integration tests in `tests/cli_add.rs`: JSON path relative to cwd, human output relative path.

Child Tasks:
- [x] P5.2: pebl-z5v17c3486
- [x] P5.3: pebl-zlijanufss
- [x] P5.4: pebl-n6xopu3hq3
- None others currently. Promote only when Adaptive Decomposition criteria are met.
