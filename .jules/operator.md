## 2024-05-22 - Stdout Pollution
**Friction:** Piping commands like `pebble next` or `pebble add` to other tools (like `jq` or `xargs`) was breaking because human-readable logs were mixed with data on stdout.
**Insight:** CLI tools should strictly separate machine-readable data (stdout) from human-readable logs/status (stderr).
**Standard:** All status messages ("Created task...", "No tasks found") must use `eprintln!`. Only requested data (task ID, JSON output) goes to `println!`.

## 2026-02-24 - Exit Code Lies in Search
**Friction:** `pebble next` returned exit code 0 even when no task was found, making it impossible to reliably script against (e.g. `while pebble next; do ... done`).
**Insight:** Commands that search for a specific item (like `next` or `grep`) should return non-zero exit codes when the item is not found, even if execution was technically "successful" (no crash).
**Standard:** Search/retrieval commands must return exit code 1 when the requested item is missing. Outputting "null" (JSON) or "Not found" (text) with exit code 0 is an anti-pattern for automation.

## 2025-02-24 - Exit Code Lies in Search and Retrieval Commands
**Friction:** `pebble next --json` returned exit code 0 and an empty JSON array `{"tasks":[]}` when no tasks were found, whereas the human-readable output correctly returned a non-zero exit code.
**Insight:** Single-item search or retrieval commands (like `next` or `show`) must consistently return non-zero exit codes when the requested item is missing, regardless of the output format. Returning "null" or an empty array with exit code 0 breaks CI/CD and shell script automation that relies on exit codes for control flow.
**Standard:** Search/retrieval commands must return exit code 1 when the requested item is missing. Always check for empty results and return `Err(NotFoundError(...).into())` *before* attempting to serialize or format the output for `--json`.
