# Implementation Plan - Pebble (Spec Alignment)

This plan reorganizes work so Pebble can be trusted to manage a hierarchical, phase-based task set **as early as possible**. "Phase Zero" contains only the blockers to using Pebble itself for planning: representing nested tasks via `needs`, and trusting `pebble list` / `pebble next` to surface tasks in correct order.

TDD is mandatory: write a failing test before each behavior change.

## Hybrid Execution Model (Driver + Pebble Tasks)
This file remains the governance and process driver. Execution happens through Pebble tasks.

### Translation Rules
1. Create one root Pebble task for the full implementation program.
2. Create one Pebble task per phase (`P1`, `P2`, ...).
3. Keep actionable checklist items in the phase task body by default.
4. Promote checklist items into child Pebble tasks only when split criteria are met, and keep a `Child Tasks` mapping in the parent body.

### Adaptive Decomposition Rules
Use checklist-first decomposition and only promote checklist items into child Pebble tasks when complexity justifies it.

1. **Default behavior**:
    - Keep checklist items in the parent task body.
    - Promote to child Pebble tasks only when split criteria below are met.
2. **Split criteria**:
    - `MUST`: the item has independent `needs` or blocks other work.
    - `MUST`: the item requires independent status tracking (`todo`/`in_progress`/`done`) for planning value.
    - `MUST`: the item is expected to span multiple sessions or PRs.
    - `SHOULD`: the item touches multiple subsystems or high-risk surfaces.
    - `SHOULD`: the item requires a design decision, spike, or uncertainty reduction step.
    - `SHOULD`: estimated effort exceeds a single focused implementation session.
    - Promote when any `MUST` is true, or when at least two `SHOULD` criteria are true.
3. **Anti-explosion rule**:
    - Do not split work solely to mirror every checklist line.
    - Prefer narrative markdown checklists unless graph structure materially improves planning.
4. **Recursion rule**:
    - Re-evaluate split criteria after each child task completes.
    - Decompose further only where remaining checklist items meet split criteria.
5. **Traceability rule**:
    - Parent task body must include a `Child Tasks` section mapping checklist items to Pebble IDs when decomposition occurs.

### Dependency Rules
1. Root task depends on phase tasks.
2. Phase tasks depend on prior phases when strict phase sequencing is desired.
3. Child tasks (when created) are dependencies of their parent phase task.
4. Leaf tasks depend on predecessor leaves only when sequencing is required.
5. Process policies (TDD, gauntlets, gate checks) are enforced by Rules and AGENTS instructions, not by `needs`.

### Sync Rules
1. Keep this plan's checkboxes synchronized with Pebble task status.
2. For plan items represented as Pebble tasks, map status directly (`[ ]`/`[-]`/`[x]`).
3. For checklist-only plan items, update checkboxes in this file and in the parent phase task body.
4. When a task moves to in progress, mark the matching plan item `[-]`.
5. When a task is complete and verified, mark the matching plan item `[x]`.
6. If scope changes, update this plan first, then create/update Pebble tasks to match.

### Task ID Index
- Root program task: `pebl-YNBL34`
- Root program task file: `docs/pebble/pebble-self-hosted-implementation-program.md`
- Phase task IDs:
- [x] `P1`: `pebl-cdIZGN`
- [x] `P2`: `pebl--yb8d4`
- [x] `P3`: `pebl-hRuKk1`
- [x] `P3.1`: `pebl-cug7mpg7cz` Inject current_dir into RunContext to improve testability of diagnostics
- [x] `P3.3`: `pebl-vnywlhmt3y` Implement pebble fix command
- [x] `P4`: `pebl-fFdi_z`
- [ ] `P5`: `pebl-pCyebx`
- Child task IDs:
- [x] `pebl-kntw9t388a`: P3.1 check --warn-only diagnostics: unknown frontmatter key warnings + schema/graph diagnostics
- Standalone task IDs:
- [x] `pebl-7Rnb6B`: ID generation uses nanoid SAFE alphabet instead of lowercase alphanumeric
- [ ] `pebl-Vs0xNh`: Investigate TestEnv dead_code allowances and test helper cleanup
- [ ] `pebl-buDx2q`: Sort order: blocking count overwhelms explicit priority
- [x] `pebl-itm1n1sj4n`: Config get unknown key should be usage error (exit 2)
- [x] `pebl-efz69mryyt`: Sync phase and plan checkboxes for completed help-json work
- [x] `pebl-5wuwlwxldk`: Add Tier-1 docs for public CLI types
- [x] `pebl-wy9nkoudjt`: Complete config get help text for KEY argument semantics
- [x] `pebl-ouajn82jhl`: Resolve init output contract inconsistency in cli-contract
- [x] `pebl-8kd42jnhv7`: Specify help-json discoverability guidance in docs (non-normative)
- [x] `pebl-exvts01y2i`: Harden help-json mapping to avoid panic on new commands
- [x] `pebl-4ssy3fsyds`: Feature request: add reverse dependency link at add time
- [ ] `pebl-p3k8qhfwqu`: Refactor commands_write_tests to avoid #[path] attribute
- [ ] `pebl-uj0hll5buc`: Fix stale reverse index in run_add JSON output
- [x] `pebl-czi45zargx`: Use lossy-safe path serialization in init JSON output
- [ ] `pebl-6pv06nwvpl`: Align help-json guidance text with actual emitted keys
- [x] `pebl-9fvf6xfco3`: Clarify global stream rules vs init human stderr output
- [x] `pebl-b4ei14wcbe`: Decide naming style in help-json guidance: exact keys vs conceptual labels
- [x] `pebl-FU-FSN`: Forbid clippy warning suppressions

## Phase Zero: Trustworthy Planning (Blockers Only)
Goal: confidently express phases as tasks with dependencies and trust `list`/`next` ordering, with reliable JSON output for agent use.

- [x] P0.1 Graph semantics for readiness and blocking.
- [x] P0.1.a `blocking` includes only **non-terminal** direct dependents.
- [x] P0.1.b Transitive blocking count excludes terminal tasks, excludes self, ignores missing IDs, and is cycle-safe.
- [x] P0.2 Deterministic default ordering for `list` and `next`.
- [x] P0.2.a Topological order respecting `needs` (missing needs ignored; cycles grouped, ordered by `created_at` then `id`).
- [x] P0.2.b Then transitive blocking count DESC.
- [x] P0.2.c Then priority ASC (None last).
- [x] P0.2.d Then `created_at` ASC.
- [x] P0.2.e Then `id` ASC.
- [x] P0.3 Tests for P0.1–P0.2 (TDD).
- [x] P0.4 JSON output is trustworthy for planning commands (`list`, `next`, `show`, `add`, `update`).
- [x] P0.4.a `--json` emits valid JSON to `stdout` and nothing else.
- [x] P0.4.b Errors and diagnostics go to `stderr` only; exit codes follow `0/1/2`.
- [x] P0.4.c Tests validating JSON purity and stdout/stderr separation for these commands.

## Phase 1: Core CLI Contract Coverage (Non-Blocking for Planning)
- [x] P1.0 Deferred scan/duplicate handling (immediately after Phase Zero).
- [x] P1.0.a Recursive scan of `tasks-dir` for all `*.md` files.
- [x] P1.0.b Duplicate ID handling (required for correct graph semantics).
- [x] P1.0.c Read commands warn to `stderr` and skip **all** files with duplicated IDs.
- [x] P1.0.d Write commands fail with a clear error if target ID is duplicated.
- [x] P1.1 `list` filters: `--status` (OR), `--tag` (AND), `--need` (OR), `--priority` (OR), `--is-ready`, `--all`, `--limit`.
- [x] P1.2 `list` alias `ls`.
- [x] P1.3 `--sort` for `list` with tie-breakers (`created_at`, then `id`).
- [x] P1.4 `search` command (case-insensitive substring over title + body; default list ordering).
- [x] P1.5 `config get <key>` command.
- [x] P1.6 `help-json` command output schema.
- [x] P1.7 Help text completeness and examples for every command.
- [x] P1.8 Extend `--json` purity and stdout/stderr separation across **all** commands.
- [x] P1.9 Exit code mapping: `0` success, `1` runtime error, `2` usage error (global).

## Phase 2: Mutation Semantics & Validation
- [x] P2.1 ID generation: `<issue-prefix>-<suffix>` with alphabet `a-z0-9`, suffix length based on issue count to keep collision probability < 1e-12.
- [x] P2.2 Priority validation: enforce `0..99` in `add` and `update`.
- [x] P2.3 Status transitions:
- [x] P2.3.a `resolved_at` auto-set when moving to `done`/`canceled`.
- [x] P2.3.b `resolved_at` cleared when leaving terminal states.
- [x] P2.3.c `modified_at` always set on `update`.
- [x] P2.4 `add` output prints relative path from **current working directory**.
- [x] P2.5 Ensure new task files end with a trailing newline.
- [x] P2.6 `archive` behavior per contract: configurable threshold; collision suffixes; JSON output relative to `tasks-dir`.

## Phase 3: Diagnostics & Repairs
- [x] P3.1 `check --warn-only` diagnostics: warnings for unknown frontmatter keys; schema/graph diagnostics.
- [x] P3.2 `check` command: unknown frontmatter keys are errors; exit code non-zero on issues.
- [ ] P3.3 `fix` command: backfill missing `created_at`, warn on unknown keys, do not remove or rewrite dependencies.

## Phase 4: Spec-Driven UX Issues (`docs/pebble/*.md`)
- [x] P4.1 Improve `.pebble/AGENTS.md` content generated by `init`.
- [x] P4.2 Fix TOCTOU slug collision by using atomic create (`OpenOptions::create_new(true)` + retry).
- [x] P4.3 Transliterate non-ASCII in `slugify` (e.g., `deunicode`).
- [x] P4.4 Help text completeness test that exercises every subcommand.
- [x] P4.5 Clippy warning suppression forbiddance in `just` checks (operator approval received).

## Phase 5: Test Coverage (TDD for Each Gap)
- [x] P5.1 Recursive scan and duplicate ID behavior (read + write). *Already covered: `tests/cli_scan_duplicates.rs` (7 tests).*
- [ ] P5.2 Blocking list and transitive blocking count integration tests. *Unit tests exist; need CLI integration tests for `blocking` JSON field and `--sort blocking`.*
- [x] P5.3 Default list ordering and explicit `--sort` tie-breakers. *Covered: unit tests in `graph/tests.rs` + integration in `cli_list_sort.rs`.*
- [x] P5.4 Filters and limits for `list`. *Covered: `tests/cli_list_filters.rs` (8 tests).*
- [x] P5.5 `search` query behavior. *Covered: `tests/cli_search.rs` (4 tests).*
- [x] P5.6 `config get` and `help-json` output shapes. *Covered: `tests/cli_config.rs` (3) + `tests/cli_help_json.rs` (6).*
- [x] P5.7 `--json` purity and stdout/stderr separation across commands. *Covered: `tests/cli_json.rs` (4) + `tests/cli_json_purity_extended.rs` (6).*
- [ ] P5.8 Exit code mapping for runtime vs usage errors. *Partial: 6 tests exist. Need tests for invalid sort field, update missing ID, missing clap args, init --dir absolute.*
- [ ] P5.9 `archive` threshold and collision behavior. *Unit test for path collision exists. Need integration tests for actual archiving with old `resolved_at`, skip logic, collision suffix.*
- [x] P5.10 `add` slug transliteration + relative path output + newline termination. *Covered: 13 unit tests + 2 integration tests.*

## Rules
0. **Keep checkmarks up to date. Use [-] for in-progress, [ ] for not started, and [x] for done.**
1. **One failing test at a time.**
2. **Refactor only after green.**
3. **No preemptive cycle prevention in the write path.**
4. **Run `just check` and `just test` regularly.**
