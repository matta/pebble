## 2024-05-22 - Stdout Pollution
**Friction:** Piping commands like `pebble next` or `pebble add` to other tools (like `jq` or `xargs`) was breaking because human-readable logs were mixed with data on stdout.
**Insight:** CLI tools should strictly separate machine-readable data (stdout) from human-readable logs/status (stderr).
**Standard:** All status messages ("Created task...", "No tasks found") must use `eprintln!`. Only requested data (task ID, JSON output) goes to `println!`.

## 2026-02-24 - Exit Code Lies in Search
**Friction:** `pebble next` returned exit code 0 even when no task was found, making it impossible to reliably script against (e.g. `while pebble next; do ... done`).
**Insight:** Commands that search for a specific item (like `next` or `grep`) should return non-zero exit codes when the item is not found, even if execution was technically "successful" (no crash).
**Standard:** Search/retrieval commands must return exit code 1 when the requested item is missing. Outputting "null" (JSON) or "Not found" (text) with exit code 0 is an anti-pattern for automation.

## 2026-03-01 - Error output includes stack traces
**Friction:** Unhandled errors, even simple ones like "Project already initialized", leaked giant stack traces to users, causing confusion and making agent parsing harder.
**Insight:** color_eyre formatters default to including backtraces on `Debug` format (`{:?}`). Using `Display` (`{}`) prints just the error string.
**Standard:** Always format eyre/anyhow errors using `{}` for user-facing output unless explicitly building debug logs.
