# Pebble CLI Contract & Interface Layer

This document defines the strict specification for the Pebble Command Line Interface, detailing input arguments, flags, expected JSON output shapes, and configuration validation. This functions as the Interface Layer contract.

## Streams

- `stdout`: Primary command output only. Human-readable by default, machine-readable with `--json`, except `pebble init` in human mode (see [`pebble init`](#pebble-init)).
- `stderr`: Diagnostics, warnings, progress logs, and error messages. Never emit JSON data to `stderr`.

## Exit Codes

- `0`: Success.
- `1`: Runtime error (I/O failure, config error, missing data).
- `2`: Usage error (invalid arguments or unsupported options).

## Command Surface Principles

- Every command has a **clear, distinct purpose**. Avoid redundant commands that do the same thing.
- All commands that produce output **must** support `--json`.
- No interactive prompts. If confirmation is needed, require `--yes` / `--force` or fail with a usage error.
- Use ubiquitous language: **one concept, one word** for both nouns and verbs. Do not abbreviate. Use the same term consistently (e.g., always `need`, never `needs` or `dep`; pick one verb such as `remove` or `delete` and use it everywhere).
- For list/set fields, the `update` command must use **consistent incremental flags** across all such fields:
    - Add items with `--add-<field> <value>` (repeatable).
    - Remove items with `--remove-<field> <value>` (repeatable).
    - If whole-list replacement is supported, it must be explicit via `--set-<field> <value>` (repeatable) and must not share flags with incremental operations.

## Configuration & Path Resolution

Pebble locates its configuration and task files using strict path resolution rules:
* **Project Root**: Defined by the directory containing the `.pebble/` configuration folder. The CLI locates it by walking up from the current working directory to the nearest parent containing `.pebble/`. It is not required to be the Git repository root.
* **Configuration File**: Lives at `.pebble/config.toml` (relative to the project root).
    * Supported keys:
        * `issue-prefix` (string): prefix for new IDs (default: `issue`).
        * `tasks-dir` (string): path to task files (default: `docs/pebble/`).
* **`tasks-dir` Resolution**: The `tasks-dir` defined in `.pebble/config.toml` **MUST** be a relative path and is always resolved relative to the **project root**. If an absolute path is found in the config file, the CLI fails with a clear error.
* **`--dir` Flag Resolution**: The `--dir <PATH>` flag can be used on any command to override `tasks-dir`.
    * If an absolute path is provided, it is used as-is.
    * If a relative path is provided, it is strictly resolved relative to the **user's current working directory (cwd)**.
* **Precedence**: `--dir` flag > `tasks-dir` in config > default `docs/pebble/`.

> [!WARNING]
> **Path-resolution example.** Suppose the project root is `/home/user/myproject` and the config sets `tasks-dir = "docs/pebble/"`. A user in `/home/user/myproject/src` runs:
> ```
> pebble list --dir ../notes
> ```
> * `tasks-dir` resolves to `/home/user/myproject/docs/pebble/` (relative to project root).
> * `--dir ../notes` resolves to `/home/user/myproject/notes` (relative to cwd `/home/user/myproject/src`).
> * Because `--dir` takes precedence, the CLI scans `/home/user/myproject/notes`.

## File Scanning & Error Handling

* The CLI **recursively** treats every `*.md` file under `tasks-dir` as a potential task file.
* If a file contains unparseable YAML frontmatter, the CLI skips it with a warning to `stderr`.
* If multiple files share the same `id`, read commands skip all files with that ID (logging a warning to `stderr`). Write commands targeting a duplicated ID fail with a clear error.
* Tasks missing required schema keys (for example `created_at`) may still be loaded for repair workflows; `pebble check` reports them as schema issues and `pebble check --fix` can backfill supported fields.
* Unknown frontmatter keys are ignored by read commands (no warnings). `pebble check` and `pebble check --warn-only` report them as findings. `pebble check --fix` reports them as findings but does not remove unknown fields.
* Renaming or moving a file within `tasks-dir` does not change the `id` and does not break references — the frontmatter `id` is canonical; filenames are advisory.
* The CLI never rewrites the frontmatter `id` for an existing task file.

## Global Options

* `--json`: Outputs a single JSON value to `stdout` per invocation for commands with structured output. For diagnostic commands (`pebble check`, including `pebble check --fix`), JSON is still emitted when findings are present even if the command exits non-zero.
* `--dir <PATH>`: Override the configured `tasks-dir`.

## JSON Mode

* `--json` outputs **valid JSON to stdout and nothing else**.
* JSON output for operational commands is stable and contract-backed by this document.
* `pebble help-json` is for discovery and may change over time; consumers should parse it defensively.
* When `--json` is set, suppress color/formatting and any extra decorations.

## Help and Discoverability

* `--help` **must describe every option** for the command, including defaults, not just list the argument name.
* `--help` must include concrete usage examples for the common path.
* `pebble help-json` gives a machine-readable overview of commands and options.
* Command-level help text is a normative interface contract and must be kept in sync with behavior.

### Required `--help` Content (Must-Have)

For every command, `--help` output **MUST** include all of the following:

* A one-line purpose statement that explains what the command does at a high level.
* A default behavior summary (for example, what is included/excluded when no flags are supplied).
* A complete argument/flag table where each entry includes:
    * semantic behavior (not just type/name),
    * repeatability and combination semantics (OR/AND/replace),
    * default values or omission behavior,
    * valid ranges/enums and validation rules when applicable.
* Interaction semantics for commonly combined flags (for example, how explicit filters interact with default omission behavior).
* At least one concrete usage example for the default path and one example that combines non-trivial flags.

If any of the required content above is missing for a command, that command's help is incomplete and non-conformant with this contract.

### Command-Specific Help Minimums (All Commands)

In addition to the general requirements above, each command's `--help` **MUST** include the following command-specific semantics:

* `pebble init`
    * What project initialization creates (`.pebble/`, config, tasks dir, `AGENTS.md`).
    * `--dir` relative-path requirement and failure behavior for absolute paths.
    * `--issue-prefix` meaning and default behavior.
* `pebble config get <key>`
    * Supported keys and behavior for unknown keys.
    * Output semantics in human mode vs `--json`.
* `pebble help-json`
    * Gives a machine-readable overview of commands, flags, and output metadata.
    * Writes valid JSON to `stdout` only, with no diagnostics mixed into `stdout`.
* `pebble list` / `pebble ls`
    * Full filter semantics and combined-flag behavior as defined below in `pebble list --help` must-haves.
* `pebble next`
    * That it returns one highest-ranked ready task.
    * Ranking tuple and equivalence to `pebble list --is-ready --limit 1`.
    * Behavior when no ready tasks exist.
* `pebble show <id>`
    * Full-object vs `--path-only` behavior.
    * Not-found behavior and stream/exit semantics.
* `pebble search <query>`
    * Search surface (title + body), case-insensitive substring semantics, and non-regex behavior.
    * Result ordering (default list ordering).
* `pebble add <title>`
    * Generated fields and defaults (`id`, timestamps, status defaults).
    * Repeatable flags (`--need`, `--tag`, `--blocks`) and semantics.
    * Priority validation range (`0..99`).
* `pebble update <id>`
    * Mutability surface (what can be changed and what cannot, especially immutable `id`).
    * Incremental list operations semantics (`--add-*`, `--remove-*`, and any clear/set behavior).
    * Reverse-link operations semantics (`--blocks`, `--remove-blocks`) and target-ID behavior.
    * Timestamp transition semantics (`modified_at`, `resolved_at` transitions).
* `pebble archive`
    * Selection criteria (terminal status + age threshold).
    * Destination path behavior and filename collision handling.
* `pebble check`
    * Read-only diagnostics scope and non-mutating behavior.
    * Default strict behavior (non-zero exit on issues).
    * `--warn-only` behavior (same diagnostics, exit code `0`).
    * `--fix` behavior: what repairs are allowed, what is explicitly not rewritten, and that findings are reported to `stderr`.
    * `--fix` exit behavior: non-zero if any findings remain after attempted repairs.

If any command omits its command-specific semantics above, its `--help` output is incomplete.

## JSON Shape: `TaskObject`

Most commands emitting JSON will return either a single `TaskObject` or a list of them.
A `TaskObject` includes:
* **Basic Fields**: `id`, `title`, `status`, `priority` (optional), `created_at`, `modified_at` (optional), `resolved_at` (optional), `needs` (array), `tags` (array).
* **Computed Fields**: `is_ready` (boolean), `blocked_by` (array of ID strings), `blocking` (array of ID strings — direct non-terminal dependents whose `needs` include this task).
* **Content & Location**: `body` (raw Markdown content string), `path` (file path relative to `tasks-dir`).

## `help-json` Output (Guidance, Not Contract)

The `pebble help-json` command returns machine-readable CLI metadata intended for **AI-agent discovery and tool integration**, not for strict schema validation or programmatic logic.

### What it returns

When this document discusses `help-json` fields, it uses exact emitted key names (for example, `global_options`, not paraphrases like "global flags list"). These names improve discoverability, but the output remains guidance rather than a versioned contract.

The output is a JSON object that generally includes:
* Root keys:
    * `name` — the CLI binary name.
    * `global_options` — a list of global option objects.
    * `commands` — a list of command objects.
* Option object keys:
    * `name` — option identifier (for example, `--json`).
    * `description` — one-line option help text.
* Command object keys:
    * `name` — canonical command name.
    * `description` — one-line purpose statement.
    * `options` — list of option objects (`name`, `description`).
    * `output` — hint for `--json` output shape on leaf commands (for example, `TaskObject`, `{"tasks": ["TaskObject"]}`).
    * `subcommands` — list of nested command objects for command groups (for example, `config get`).

### How AI agents should use it

* Use `help-json` to **discover what commands exist** and what flags they accept, especially before composing a `pebble` invocation for the first time.
* Use `output` hints to understand what JSON structure to expect before parsing `--json` output.
* **Do not** embed `help-json` structure in hard-coded logic that will break if fields are renamed or reorganized. Always parse defensively.
* For any field that matters to downstream logic, prefer the contract-backed `--json` output from the actual command over `help-json` metadata.

### Stability disclaimer

Field shape, presence, ordering, and naming in `help-json` output **may change between Pebble versions**. New fields may be added; existing fields may be renamed or removed. Consumers must not assume that all fields will be present.

## Timestamp Rules

* `created_at` is set to the current UTC time on `pebble add`.
* Missing `created_at` is a schema issue reported by `pebble check` and `pebble check --warn-only`.
* `modified_at` is automatically set to the current UTC time on every `pebble update` invocation.
* `resolved_at` is automatically set to the current UTC time when `pebble update` transitions a task's status to `done` or `canceled` (if not already set).
* `resolved_at` is automatically cleared when `pebble update` transitions a task's status away from `done` or `canceled`.
* `pebble check --fix` backfills a missing `created_at` with the current UTC time.

## Repository Management

### `pebble init`
Bootstraps the project environment.
* **Inputs**:
    * `--issue-prefix <PREFIX>` (sets initial prefix)
    * `--dir <PATH>` (sets initial tasks-dir; must be a relative path, otherwise fails)
* **Outputs**:
    * **Human**: Success message to `stderr`.
    * **JSON (`--json`)**: `{"status": "success", "project_root": "...", "tasks_dir": "...", "issue_prefix": "..."}` to `stdout`.

### `pebble config get <key>`
Reads an active configuration value.
* **Inputs**: `<key>` (e.g., `issue-prefix`, `tasks-dir`).
* **Output (`--json`)**: `{"key": "<key>", "value": "<value>"}`

### `pebble help-json`
Returns machine-readable CLI metadata for AI-agent discovery, including commands, flags, and output metadata. This output is **guidance, not a versioned contract** — see the [`help-json` Output section](#help-json-output-guidance-not-contract) for full semantics and stability caveats.
* **Inputs**: None.
* **Output**: JSON object on `stdout` only. No diagnostics are mixed into `stdout`.

## Query Commands

### `pebble list` (alias: `ls`)
Parses the directory, builds the DAG, and lists tasks. Defaults to omitting `done` and `canceled` statuses.
* **Inputs**:
    * `--status <status>`: Filters by status (OR'ed).
    * `--tag <tag>`: Filters by tag (AND'ed).
    * `--need <id>`: Filters by need (OR'ed).
    * `--priority <N>`: Filters by priority (OR'ed). Valid range: `0..99` (lower number = higher priority).
    * `--is-ready`: Filters to tasks whose status is actionable (`todo` or `in_progress`), whose `needs` all exist, and whose `needs` are all `done` or `canceled`.
    * `--all`: Disables default omission of `done` and `canceled` tasks. (Note: explicitly requesting `--status done` or `--status canceled` also includes those tasks, even without `--all`.)
    * `--sort <field>`: Sort by a specific field. Valid fields: `priority`, `blocking`, `created_at`, `modified_at`, `status`, `title`. Defaults to ascending; prefix with `-` for descending (e.g., `--sort -created_at`). When sorting by `status`, the canonical order is: `todo`, `in_progress`, `done`, `canceled`. When sorting by `priority`, tasks with no `priority` sort after all prioritized tasks. When sorting by `blocking`, the key is the **transitive blocking count** (not `len(blocking)`).
    * `--limit <N>`: Limits returned rows.
* **Default sort order**: Deterministic and dependency-aware:
    1. **Topological order** (respecting `needs`): if B depends on A, A appears before B. Missing needs are ignored for ordering (only existing tasks participate). Cycles are grouped together; tasks inside a cycle are ordered by `created_at` then `id`.
    2. **Effective priority** ascending: `min(base_priority, downstream_min_priority)`, where `base_priority` is the task's own priority (with unset values sorted after all explicit priorities) and `downstream_min_priority` is the minimum base priority among actionable transitive downstream dependents.
    3. **Base priority** ascending (lower number = higher priority). Tasks with no `priority` sort after all prioritized tasks.
    4. **Transitive blocking count** descending: the number of non-terminal tasks recursively reachable by traversing **reverse** `needs` edges (tasks that depend on this task, directly or indirectly), using unique task IDs and excluding self. Traversal stops at terminal tasks (`done`/`canceled`) so completed work does not propagate blocking. Tasks blocking more downstream work appear first.
    5. **`created_at`** ascending (oldest first).
    6. **`id`** ascending (lexicographic) as the absolute tiebreaker, guaranteeing determinism.
* When `--sort` is specified, topological ordering is NOT applied — the results are sorted purely by the requested field. Ties are broken by `created_at` ascending, then `id` ascending.
* When `--is-ready` is active, all returned tasks are at the dependency frontier, so topological ordering has no practical effect and the order is effectively: effective priority → base priority → blocking count → created_at → id.
* **Output (`--json`)**: `{"tasks": [<TaskObject>, ...]}`

#### `pebble list --help` Must-Have Semantics

The `pebble list --help` output **MUST** explicitly document:

* What `list` does in general (scan, graph build, and task listing behavior).
* The default omission rule for `done`/`canceled`.
* `--status` OR semantics, including that explicit closed statuses include closed tasks even without `--all`.
* `--tag` AND semantics.
* `--need` OR semantics.
* `--priority` OR semantics and valid range `0..99`.
* `--is-ready` absolute readiness criteria summary.
* `--all` interaction with default omission behavior.
* `--limit` semantics (applied after ordering/filtering).
* `--sort` field set and tie-breaker behavior.

### `pebble next`
Returns the single highest-scoring ready task. Since `--is-ready` places all results at the dependency frontier, the effective sort is: `(effective_priority ASC, base_priority ASC, transitive_blocking_count DESC, created_at ASC, id ASC)`, where `effective_priority = min(base_priority, downstream_min_priority)`. Equivalent to `pebble list --is-ready --limit 1` under the default sort order.
* **Inputs**: None.
* **Output (`--json`)**: A single unwrapped `<TaskObject>`, or `null` if no ready tasks exist.

### `pebble show <id>`
Outputs full details of a specific task.
* **Inputs**:
    * `<id>`: The ID of the task to view.
    * `--path-only`: Strips all object data, returning only the file path. (Without `--json`, prints a raw string filepath. With `--json`, prints `{"path": "..."}`).
* **Output (`--json`)**: A single unwrapped `<TaskObject>`.

### `pebble search <query>`
Case-insensitive substring search against task `title` (frontmatter) and raw Markdown `body` (frontmatter excluded). No regex, stemming, or tokenization.
* **Inputs**: `<query>` (string).
* **Results order**: Default list order (see `pebble list` default sort).
* **Output (`--json`)**: `{"tasks": [<TaskObject>, ...]}`

## Mutation Commands

### `pebble add <title>`
Creates a new task file with generated boilerplate.
* **Inputs**: `<title>` (string).
    * Flags: `--status <status>`, `--priority <N>` (valid range: `0..99`), `--body <text>`, `--need <id>` (repeatable), `--tag <tag>` (repeatable), `--blocks <id>` (repeatable).
    * `--blocks <id>` semantics: after creating the new task, append the new task ID to `needs` of each referenced task ID (deduplicated). Missing or duplicate target IDs cause the command to fail.
* **ID Generation**: The generated ID follows the pattern `<issue-prefix>-<suffix>`, where `issue-prefix` comes from the `issue-prefix` config key (default: `issue`). The suffix uses the alphabet `a-z0-9` (36 characters). The suffix length is computed from the current issue count to keep collision probability under 1e-12 (birthday paradox sizing).
* **Filename Generation**: The filename is derived from the `<title>` using a deterministic slug. To ensure maximum reach and cross-platform safety, slugs are strictly restricted to lowercase alphanumeric characters, dashes, and underscores:
    * lowercase
    * ASCII only (strictly `a-z`, `0-9`, `-`, `_`)
    * whitespace and other non-alphanumeric characters → `-`
    * collapse repeated `-`
    * trim leading/trailing `-`
    * truncate to 80 characters (then trim trailing `-` again)
    * If the result is empty, use `task`.
    * If the filename already exists, append `-2`, `-3`, etc.
* **Output (`--json`)**: A single unwrapped `<TaskObject>` representing the newly created task.

### `pebble update <id>`
Safely modifies existing frontmatter properties or appends body content.
* **Inputs**: `<id>` (string).
    * Flags: `--title <text>`, `--status <status>`, `--priority <N>` (valid range: `0..99`), `--clear-priority`, `--body <text>` (replaces entire body), `--append-body <text>` (appends to body), `--add-tag <tag>`, `--remove-tag <tag>`, `--add-need <id>`, `--remove-need <id>`, `--blocks <id>`, `--remove-blocks <id>`.
    * `--blocks <id>` semantics: add this task's ID to `needs` of each referenced task ID.
    * `--remove-blocks <id>` semantics: remove this task's ID from `needs` of each referenced task ID.
    * Reverse-link flag behavior: referenced IDs are deduplicated; missing or duplicate target IDs cause failure.
* **Output (`--json`)**: A single unwrapped `<TaskObject>` representing the modified task.

### `pebble archive`
Automated lifecycle manager that sweeps completed (`done`, `canceled`) tasks whose `resolved_at` timestamp is older than a configurable threshold (e.g., 30 days) into an `archive/` subdirectory under `tasks-dir`. If a filename collision occurs, a numeric suffix (`-2`, `-3`, etc.) is appended.
* **Inputs**: None (thresholds are configurable).
* **Output (`--json`)**: `{"archived": [{"id": "...", "moved_to": "..."}]}`

## Validation & Diagnostics

### `pebble check`
Read-only graph verification tool by default. With `--fix`, applies safe deterministic repairs before reporting remaining findings.
* **Inputs**:
    * `--warn-only`: report issues but always exit with status code `0`.
    * `--fix`: apply safe deterministic repairs (for example, backfilling `created_at`) before reporting remaining findings.
    * `--warn-only` and `--fix` are mutually exclusive.
* **Default behavior** (without `--warn-only`): exits with a non-zero status code if graph or schema errors exist.
* **Checks Performed**:
    * **Missing Required Keys**: reports tasks missing required schema keys such as `created_at`.
    * **Unknown Frontmatter Keys**: reports keys not recognized by the schema.
    * **Duplicate Task IDs**: reports when multiple files declare the same `id`.
    * **Dangling References**: reports when a task lists a `needs` ID that does not exist in the graph.
    * **Dependency Cycles**: reports circular dependencies (for example, A needs B, B needs A).
* **Output (`--json`)**:
    * `pebble check` / `pebble check --warn-only`: `{"ok": bool, "errors": [{"file": "...", "line": N|null, "message": "...", "code": "<string>"?}]}`
    * `pebble check --fix`: `{"ok": bool, "fixed_tasks": ["<id>", ...], "errors": [{"file": "...", "line": N|null, "message": "...", "code": "<string>"?}]}`
* **`--fix` semantics**:
    * Repairs currently include backfilling missing `created_at`.
    * Unknown keys are reported as findings and preserved.
    * Dependency edges are never rewritten.
    * Human mode writes repair summary to `stdout` and findings to `stderr`.
    * Exit code is `0` only if no findings remain after repairs.

## Output Semantics

* Human output should be readable and may emit diagnostics to `stderr`.
* Structured output must never be mixed with diagnostics.
* Commands that return structured data:
    * `list`, `search` => `{"tasks": [<TaskObject>, ...]}`
    * `show`, `add`, `update` => single unwrapped `<TaskObject>`
    * `config get`, `init`, `archive` => structured JSON response

## Idempotency and Safety

* Commands should be safe to re-run; `archive` is expected to be idempotent.
* When failing due to invalid usage, return exit code `2` with a clear error message on `stderr`.
