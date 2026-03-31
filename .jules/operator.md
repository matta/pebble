## 2024-05-22 - Stdout Pollution
**Friction:** Piping commands like `pebble next` or `pebble add` to other tools (like `jq` or `xargs`) was breaking because human-readable logs were mixed with data on stdout.
**Insight:** CLI tools should strictly separate machine-readable data (stdout) from human-readable logs/status (stderr).
**Standard:** All status messages ("Created task...", "No tasks found") must use `eprintln!`. Only requested data (task ID, JSON output) goes to `println!`.

## 2026-02-24 - Exit Code Lies in Search
**Friction:** `pebble next` returned exit code 0 even when no task was found, making it impossible to reliably script against (e.g. `while pebble next; do ... done`).
**Insight:** Commands that search for a specific item (like `next` or `grep`) should return non-zero exit codes when the item is not found, even if execution was technically "successful" (no crash).
**Standard:** Search/retrieval commands must return exit code 1 when the requested item is missing. Outputting "null" (JSON) or "Not found" (text) with exit code 0 is an anti-pattern for automation.

## 2024-10-24 - Stop Leaking Stack Traces in CLI Output
**Friction:** When a standard error occurred (like `pebble init` on an existing project), the CLI printed a scary stack trace and source code location (`Runtime error: ... Location: ...`), which is confusing for human users and pollutes agent/script logs with internal Rust details.
**Insight:** `color_eyre::eyre::Result` uses the `Debug` formatter (`{:?}`) by default to print its full report, including the backtrace. CLI boundaries need to use the `Display` formatter (`{}`) to show only the human-readable error message.
**Standard:** Use `{}` instead of `{:?}` when printing `eyre` errors to `stderr` at the top level, unless in verbose/debug mode.
