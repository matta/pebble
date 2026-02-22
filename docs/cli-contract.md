# Pebble CLI Contract & Interface Layer

This document defines the strict specification for the Pebble Command Line Interface, detailing input arguments, flags, expected JSON output shapes, and configuration validation. This functions as the Interface Layer contract.

## Streams

- `stdout`: Primary command output only. Human-readable by default, machine-readable with `--json`.
- `stderr`: Diagnostics, warnings, progress logs, and error messages. Never emit JSON data to `stderr`.

## Exit Codes

- `0`: Success.
- `1`: Runtime error (I/O failure, config error, missing data).
- `2`: Usage error (invalid arguments or unsupported options).

## Command Surface Principles

- Every command has a **clear, distinct purpose**. Avoid redundant commands that do the same thing.
- All commands that produce output **must** support `--json`.
- No interactive prompts. If confirmation is needed, require `--yes` / `--force` or fail with a usage error.
- Use ubiquitous language: **one concept, one word** for both nouns and verbs. Do not abbreviate. Use the same term consistently (e.g., always `dependency`, never `dependencies` or `dep`; pick one verb such as `remove` or `delete` and use it everywhere).
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
* If a file contains unparseable TOML frontmatter, the CLI skips it with a warning to `stderr`.
* If multiple files share the same `id`, read commands skip all files with that ID (logging a warning to `stderr`). Write commands targeting a duplicated ID fail with a clear error.
* Unknown frontmatter keys are ignored by read commands (no warnings). `pebble doctor` and `pebble fix` emit warnings. `pebble check` treats them as errors. `pebble fix` does not remove unknown fields.
* Renaming or moving a file within `tasks-dir` does not change the `id` and does not break references — the frontmatter `id` is canonical; filenames are advisory.
* The CLI never rewrites the frontmatter `id` for an existing task file.

## Global Options

* `--json`: Outputs a single JSON value to `stdout` per invocation. On failure, no JSON is emitted; `stdout` is empty, an error message is written to `stderr`, and the exit code is non-zero.
* `--dir <PATH>`: Override the configured `tasks-dir`.
* `--help-json`: Emits a machine-readable JSON schema of commands, flags, and output shapes to `stdout`, then exits.

## JSON Mode

* `--json` outputs **valid JSON to stdout and nothing else**.
* JSON output is stable and schema-backed (see `--help-json` schemas).
* When `--json` is set, suppress color/formatting and any extra decorations.

## Help and Discoverability

* `--help` **must describe every option** for the command, including defaults, not just list the argument name.
* `--help` must include concrete usage examples for the common path.
* `--help-json` provides a machine-readable description of commands, flags, and output schemas.

## JSON Shape: `TaskObject`

Most commands emitting JSON will return either a single `TaskObject` or a list of them.
A `TaskObject` includes:
* **Basic Fields**: `id`, `title`, `status`, `priority` (optional), `created_at`, `modified_at` (optional), `resolved_at` (optional), `deps` (array), `tags` (array).
* **Computed Fields**: `is_ready` (boolean), `blocked_by` (array of ID strings), `blocking` (array of ID strings — direct non-terminal dependents whose `deps` include this task).
* **Content & Location**: `body` (raw Markdown content string), `path` (file path relative to `tasks-dir`).

## Timestamp Rules

* `created_at` is set to the current UTC time on `pebble add`.
* `modified_at` is automatically set to the current UTC time on every `pebble update` invocation.
* `resolved_at` is automatically set to the current UTC time when `pebble update` transitions a task's status to `done` or `canceled` (if not already set).
* `resolved_at` is automatically cleared when `pebble update` transitions a task's status away from `done` or `canceled`.
* `pebble fix` backfills a missing `created_at` with the current UTC time.

## Repository Management

### `pebble init`
Bootstraps the project environment.
* **Inputs**:
    * `--issue-prefix <PREFIX>` (sets initial prefix)
    * `--dir <PATH>` (sets initial tasks-dir; must be a relative path, otherwise fails)
* **Outputs**: None (stdout text on success). Sets up `.pebble/`, creates the `tasks-dir` if missing, and creates `.pebble/AGENTS.md`.

### `pebble config get <key>`
Reads an active configuration value.
* **Inputs**: `<key>` (e.g., `issue-prefix`, `tasks-dir`).
* **Output (`--json`)**: `{"key": "<key>", "value": "<value>"}`

## Query Commands

### `pebble list` (alias: `ls`)
Parses the directory, builds the DAG, and lists tasks. Defaults to omitting `done` and `canceled` statuses.
* **Inputs**:
    * `--status <status>`: Filters by status (OR'ed).
    * `--tag <tag>`: Filters by tag (AND'ed).
    * `--dep <id>`: Filters by dependency (OR'ed).
    * `--priority <N>`: Filters by priority (OR'ed). Valid range: `0..99` (lower number = higher priority).
    * `--is-ready`: Filters to tasks whose status is actionable (`todo` or `in_progress`), whose `deps` all exist, and whose `deps` are all `done` or `canceled`.
    * `--all`: Disables default omission of `done` and `canceled` tasks. (Note: explicitly requesting `--status done` or `--status canceled` also includes those tasks, even without `--all`.)
    * `--sort <field>`: Sort by a specific field. Valid fields: `priority`, `blocking`, `created_at`, `modified_at`, `status`, `title`. Defaults to ascending; prefix with `-` for descending (e.g., `--sort -created_at`). When sorting by `status`, the canonical order is: `todo`, `in_progress`, `done`, `canceled`. When sorting by `priority`, tasks with no `priority` sort after all prioritized tasks. When sorting by `blocking`, the key is the **transitive blocking count** (not `len(blocking)`).
    * `--limit <N>`: Limits returned rows.
* **Default sort order**: Deterministic and dependency-aware:
    1. **Topological order** (respecting `deps`): if B depends on A, A appears before B. Missing dependencies are ignored for ordering (only existing tasks participate). Cycles are grouped together; tasks inside a cycle are ordered by `created_at` then `id`.
    2. **Transitive blocking count** descending: the number of non-terminal tasks recursively reachable by traversing **reverse** `deps` edges (tasks that depend on this task, directly or indirectly), using unique task IDs and excluding self. Tasks blocking more downstream work appear first.
    3. **Priority** ascending (lower number = higher priority). Tasks with no `priority` sort after all prioritized tasks.
    4. **`created_at`** ascending (oldest first).
    5. **`id`** ascending (lexicographic) as the absolute tiebreaker, guaranteeing determinism.
* When `--sort` is specified, topological ordering is NOT applied — the results are sorted purely by the requested field. Ties are broken by `created_at` ascending, then `id` ascending.
* When `--is-ready` is active, all returned tasks are at the dependency frontier, so topological ordering has no practical effect and the order is effectively: blocking count → priority → created_at → id.
* **Output (`--json`)**: `{"tasks": [<TaskObject>, ...]}`

### `pebble next`
Returns the single highest-scoring ready task. Since `--is-ready` places all results at the dependency frontier, the effective sort is: `(transitive_blocking_count DESC, priority ASC, created_at ASC, id ASC)`. Equivalent to `pebble list --is-ready --limit 1` under the default sort order.
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
    * Flags: `--status <status>`, `--priority <N>` (valid range: `0..99`), `--body <text>`, `--dep <id>` (repeatable), `--tag <tag>` (repeatable).
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
    * Flags: `--title <text>`, `--status <status>`, `--priority <N>` (valid range: `0..99`), `--clear-priority`, `--body <text>` (replaces entire body), `--append-body <text>` (appends to body), `--add-tag <tag>`, `--remove-tag <tag>`, `--add-dep <id>`, `--remove-dep <id>`.
* **Output (`--json`)**: A single unwrapped `<TaskObject>` representing the modified task.

### `pebble archive`
Automated lifecycle manager that sweeps completed (`done`, `canceled`) tasks whose `resolved_at` timestamp is older than a configurable threshold (e.g., 30 days) into an `archive/` subdirectory under `tasks-dir`. If a filename collision occurs, a numeric suffix (`-2`, `-3`, etc.) is appended.
* **Inputs**: None (thresholds are configurable).
* **Output (`--json`)**: `{"archived": [{"id": "...", "moved_to": "..."}]}`

## Validation & Diagnostics

### `pebble doctor`
Performs a read-only health check on the graph. Does not rewrite state. Exits with status code `0`.
* **Inputs**: None.
* **Output (`--json`)**: `{"ok": bool, "errors": [{"file": "...", "line": N|null, "message": "...", "code": "<string>"?}]}`

### `pebble check`
Strict verification tool. Functions identically to `pebble doctor` but exits with a non-zero status code if graph or schema errors exist.
* **Inputs**: None.
* **Output (`--json`)**: Same shape as `pebble doctor`.

### `pebble fix`
Applies safe, deterministic repairs such as whitespace normalization or backfilling missing `created_at`. Does not rewrite dependency edges.
* **Inputs**: None.
* **Output (`--json`)**: Typically returns status of operations; follows the repair output format.

## Output Semantics

* Human output should be readable and may emit diagnostics to `stderr`.
* Structured output must never be mixed with diagnostics.
* Commands that return structured data:
    * `list`, `search` => `{"tasks": [<TaskObject>, ...]}`
    * `show`, `add`, `update` => single unwrapped `<TaskObject>`
    * `config get`, `init` => structured JSON response

## Idempotency and Safety

* Commands should be safe to re-run; `archive` is expected to be idempotent.
* When failing due to invalid usage, return exit code `2` with a clear error message on `stderr`.
