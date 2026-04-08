## 2024-05-22 - Stdout Pollution
**Friction:** Piping commands like `pebble next` or `pebble add` to other tools (like `jq` or `xargs`) was breaking because human-readable logs were mixed with data on stdout.
**Insight:** CLI tools should strictly separate machine-readable data (stdout) from human-readable logs/status (stderr).
**Standard:** All status messages ("Created task...", "No tasks found") must use `eprintln!`. Only requested data (task ID, JSON output) goes to `println!`.

## 2026-02-24 - Exit Code Lies in Search
**Friction:** `pebble next` returned exit code 0 even when no task was found, making it impossible to reliably script against (e.g. `while pebble next; do ... done`).
**Insight:** Commands that search for a specific item (like `next` or `grep`) should return non-zero exit codes when the item is not found, even if execution was technically "successful" (no crash).
**Standard:** Search/retrieval commands must return exit code 1 when the requested item is missing. Outputting "null" (JSON) or "Not found" (text) with exit code 0 is an anti-pattern for automation.

## 2026-03-09 - JSON Output Centralization
**Friction:** JSON output handling (`println!("{}", serde_json::to_string(&data)?)`) was scattered across 10+ command modules, making it prone to inconsistencies, difficult to test, and easy to accidentally pipe to `stderr` or mix with human-readable logs.
**Insight:** A central `emit_json` function forces all commands to use the same serialization flow, ensuring deterministic, machine-readable output for agents while cleanly separating `stdout` data from `stderr` diagnostic logging.
**Standard:** All `--json` structured outputs must be emitted using the `pebble::commands::emit_json` function. Do not use manual `println!` macros with `serde_json::to_string` outside of tests.
