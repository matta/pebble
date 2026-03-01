---
id: "pebl-pCyebx"
title: "Phase 5 Test Coverage for Gaps"
status: "todo"
created_at: "2026-02-23T01:36:06.795969+00:00"
needs: ["pebl-fFdi_z"]
tags: ["bootstrap", "self_hosted"]
---
Checklist:
- [x] P5.1 Recursive scan and duplicate ID behavior tests (read + write)
- [ ] P5.2 Blocking list and transitive blocking count tests
- [x] P5.3 Default list ordering and `--sort` tie-breaker tests
- [x] P5.4 `list` filter and limit tests
- [x] P5.5 `search` behavior tests
- [x] P5.6 `config get` and `help-json` output shape tests
- [x] P5.7 `--json` purity and stdout/stderr separation tests
- [ ] P5.8 Exit code mapping tests (runtime vs usage errors)
- [ ] P5.9 `archive` threshold and collision behavior tests
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

### P5.2 — GAPS REMAIN
Existing unit tests in `src/graph/tests.rs`: `test_count_blocking_excludes_terminal_and_self`,
`test_count_blocking_cycle_excludes_self`, `test_dynamic_scoring`.
Existing unit test in `src/commands.rs`: `test_blocking_list_excludes_terminal_dependents`.

**Missing integration tests** (add to `tests/cli_list_sort.rs` or a new `tests/cli_blocking.rs`):
- `test_list_json_blocking_field_contains_direct_non_terminal_dependents`: Create A (todo), B needs A (todo), C needs A (done). Verify `show A --json` → `blocking: ["B"]` (C excluded because done).
- `test_list_sort_blocking_uses_transitive_count`: Create A (todo), B needs A (todo), C needs B (todo). A blocks B+C transitively (count=2), B blocks C (count=1). `list --json --sort -blocking` should return A before B.
- `test_list_default_order_blocking_count_breaks_ties`: Two independent ready tasks where one blocks more downstream work; the one blocking more should appear first in default order.

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

### P5.8 — GAPS REMAIN
File: `tests/cli_errors.rs` (6 tests) covers:
- show missing ID → exit 1
- add invalid status → exit 2
- add/update priority > 99 → exit 2
- list/add with no project → exit 1 (runtime)

**Missing integration tests** (add to `tests/cli_errors.rs`):
- `test_update_invalid_status_is_usage_error`: `update PROJ-1 --status not-a-status` → exit 2, empty stdout.
- `test_list_invalid_sort_field_is_usage_error`: `list --sort not-a-field` → exit 2, empty stdout. Error message should mention valid fields.
- `test_update_missing_id_is_runtime_error`: `update PROJ-404 --status done` → exit 1, empty stdout.
- `test_search_missing_query_is_usage_error`: `search` (no query arg) → exit 2 (clap usage error).
- `test_show_missing_id_arg_is_usage_error`: `show` (no id arg) → exit 2 (clap usage error).
- `test_init_absolute_dir_is_usage_error`: `init --dir /absolute/path` → exit 2 per cli-contract.md.

### P5.9 — GAPS REMAIN
Existing unit test in `src/commands_archive.rs`: `test_get_archive_path` (collision logic via mock).
Existing integration test in `tests/cli_json_purity_extended.rs`: `test_archive_json_stdout_only` (empty archive).

**Missing integration tests** (add to a new `tests/cli_archive.rs`):
- `test_archive_moves_old_resolved_task`: Create a task with status=done and `resolved_at` 60 days ago. Run `archive --json`. Verify the task appears in `archived` array with correct `id` and `moved_to`. Verify the file was actually moved to `tasks/archive/`.
- `test_archive_skips_recently_resolved_task`: Create a done task with `resolved_at` = now. Run `archive --json`. Verify `archived` is empty and the file remains in place.
- `test_archive_skips_non_terminal_task`: Create a todo task. Run `archive --json`. Verify `archived` is empty.
- `test_archive_collision_appends_numeric_suffix`: Create a done task with old `resolved_at`. Manually create `tasks/archive/<filename>.md`. Run `archive --json`. Verify `moved_to` ends with `-2.md`.

**Implementation notes for archive tests**: The `resolved_at` field uses `chrono::DateTime<Utc>`. The archive threshold comes from `ctx.config.archive_threshold_days` (check `Config` struct for default). Task files need a `resolved_at` value old enough to exceed the threshold — format as RFC3339 in YAML frontmatter like `resolved_at: "2025-01-01T00:00:00Z"`.

### P5.10 — ALREADY COVERED ✓
Unit tests in `src/commands_add.rs` (13 tests): basic slugify, mixed separators, empty fallback,
reserved chars, transliteration (café, über, naïve, résumé, Æneid, 日本語),
newline-producing Unicode, unknown chars, empty transliterations, multi-char transliterations,
safety invariant checks.
Integration tests in `tests/cli_add.rs`: JSON path relative to cwd, human output relative path.

Child Tasks:
- None currently. Promote only when Adaptive Decomposition criteria are met.
