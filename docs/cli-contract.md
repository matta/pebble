# Pebble CLI Contract & Interface Layer

This document defines the strict specification for the Pebble Command Line Interface, detailing input arguments, flags, expected JSON output shapes, and configuration validation. This functions as the Interface Layer contract.

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

## Global Options

* `--json`: Outputs a single JSON value to `stdout` per invocation. On failure, no JSON is emitted; `stdout` is empty, an error message is written to `stderr`, and the exit code is non-zero.
* `--dir <PATH>`: Override the configured `tasks-dir`.
* `--help-json`: Emits a machine-readable JSON schema of commands, flags, and output shapes to `stdout`, then exits.

## JSON Shape: `TaskObject`

Most commands emitting JSON will return either a single `TaskObject` or a list of them.
A `TaskObject` includes:
* **Basic Fields**: `id`, `title`, `status`, `priority` (optional), `created_at`, `modified_at` (optional), `resolved_at` (optional), `deps` (array), `tags` (array).
* **Computed Fields**: `is_ready` (boolean), `blocked_by` (array of ID strings), `blocking` (array of ID strings).
* **Content & Location**: `body` (raw Markdown content string), `path` (file path relative to `tasks-dir`).

## Repository Management

### `pebble init`
Bootstraps the project environment.
* **Inputs**:
    * `--issue-prefix <PREFIX>` (sets initial prefix)
    * `--dir <PATH>` (sets initial tasks-dir; must be a relative path, otherwise fails)
* **Outputs**: None (stdout text on success). Sets up `.pebble/` and creates `.pebble/AGENTS.md`.

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
    * `--priority <N>`: Filters by priority (OR'ed).
    * `--is-ready`: Filters to tasks where all `deps` are `done` or `canceled`.
    * `--all`: Disables default omission of `done` and `canceled` tasks.
    * `--sort <field>`: Sort by a specific field. Valid fields: `priority`, `created_at`, `modified_at`, `status`, `title`. Prefix with `-` for descending. Ex: `--sort -created_at`.
    * `--limit <N>`: Limits returned rows.
* **Output (`--json`)**: `{"tasks": [<TaskObject>, ...]}`

### `pebble next`
Returns the single highest-scoring ready task based on the dynamic scoring algorithm. Equivalent to `pebble list --is-ready --limit 1` under default sorting.
* **Inputs**: None.
* **Output (`--json`)**: A single unwrapped `<TaskObject>`, or `null` if no ready tasks exist.

### `pebble show <id>`
Outputs full details of a specific task.
* **Inputs**:
    * `<id>`: The ID of the task to view.
    * `--path-only`: Strips all object data, returning only the file path. (Without `--json`, prints a raw string filepath. With `--json`, prints `{"path": "..."}`).
* **Output (`--json`)**: A single unwrapped `<TaskObject>`.

### `pebble search <query>`
Full-text substring search across titles and Markdown bodies.
* **Inputs**: `<query>` (string).
* **Output (`--json`)**: `{"tasks": [<TaskObject>, ...]}`

## Mutation Commands

### `pebble add <title>`
Creates a new task file with generated boilerplate.
* **Inputs**: `<title>` (string).
    * Flags: `--status <status>`, `--priority <N>`, `--body <text>`, `--dep <id>` (repeatable), `--tag <tag>` (repeatable).
* **Output (`--json`)**: A single unwrapped `<TaskObject>` representing the newly created task.

### `pebble update <id>`
Safely modifies existing frontmatter properties or appends body content.
* **Inputs**: `<id>` (string).
    * Flags: `--title <text>`, `--status <status>`, `--priority <N>`, `--clear-priority`, `--body <text>` (replaces entire body), `--append-body <text>` (appends to body), `--add-tag <tag>`, `--remove-tag <tag>`, `--add-dep <id>`, `--remove-dep <id>`.
* **Output (`--json`)**: A single unwrapped `<TaskObject>` representing the modified task.

### `pebble archive`
Automated lifecycle manager that sweeps completed (`done`, `canceled`) tasks beyond a `resolved_at` age threshold to an `archive/` subdirectory.
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
Applies safe, deterministic repairs such as whitespace normalization or timestamp backfilling. Does not rewrite dependency edges.
* **Inputs**: None.
* **Output (`--json`)**: Typically returns status of operations; follows the repair output format.
