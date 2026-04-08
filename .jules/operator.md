## 2024-05-22 - Stdout Pollution
**Friction:** Piping commands like `pebble next` or `pebble add` to other tools (like `jq` or `xargs`) was breaking because human-readable logs were mixed with data on stdout.
**Insight:** CLI tools should strictly separate machine-readable data (stdout) from human-readable logs/status (stderr).
**Standard:** All status messages ("Created task...", "No tasks found") must use `eprintln!`. Only requested data (task ID, JSON output) goes to `println!`.

## 2026-02-24 - Exit Code Lies in Search
**Friction:** `pebble next` returned exit code 0 even when no task was found, making it impossible to reliably script against (e.g. `while pebble next; do ... done`).
**Insight:** Commands that search for a specific item (like `next` or `grep`) should return non-zero exit codes when the item is not found, even if execution was technically "successful" (no crash).
**Standard:** Search/retrieval commands must return exit code 1 when the requested item is missing. Outputting "null" (JSON) or "Not found" (text) with exit code 0 is an anti-pattern for automation.

## 2026-03-05 - Silent Failures in JSON Output
**Friction:** The `pebble next --json` command returned `{"tasks":[]}` and exited with code `0` when no tasks were found, despite returning code `1` and an error message when `--json` was omitted. This caused silent failures for AI agents relying on the error exit code to react.
**Insight:** A failure condition is inherently a failure regardless of output format. Altering behavior and returning success just to emit an empty JSON object breaks expectations for agents reading exit codes.
**Standard:** When a search or retrieval fails, commands must return a `NotFoundError` (exit code `1`) and a clean message to `stderr`, preserving this behavior uniformly even in `--json` mode to prevent silent automation failures.
