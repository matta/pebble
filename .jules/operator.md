## 2024-05-22 - Stdout Pollution
**Friction:** Piping commands like `pebble next` or `pebble add` to other tools (like `jq` or `xargs`) was breaking because human-readable logs were mixed with data on stdout.
**Insight:** CLI tools should strictly separate machine-readable data (stdout) from human-readable logs/status (stderr).
**Standard:** All status messages ("Created task...", "No tasks found") must use `eprintln!`. Only requested data (task ID, JSON output) goes to `println!`.

## 2026-02-24 - Exit Code Lies in Search
**Friction:** `pebble next` returned exit code 0 even when no task was found, making it impossible to reliably script against (e.g. `while pebble next; do ... done`).
**Insight:** Commands that search for a specific item (like `next` or `grep`) should return non-zero exit codes when the item is not found, even if execution was technically "successful" (no crash).
**Standard:** Search/retrieval commands must return exit code 1 when the requested item is missing. Outputting "null" (JSON) or "Not found" (text) with exit code 0 is an anti-pattern for automation.
## 2024-03-17 - Error Output and JSON Exit Code Lies
**Friction:** Runtime errors printed verbose Debug stack traces `Runtime error: {:?}` with ANSI color escapes and "Backtrace omitted" noise instead of clean text, frustrating agent parsers reading stderr. Additionally, `next --json` returned exit code 0 and `{"tasks":[]}` when no tasks existed, creating silent failures.
**Insight:** A lack of strict separation between human-friendly logs and machine-friendly errors caused noisy stderr. Checking `ctx.json` before `tasks.is_empty()` in the `next` command bypassed the `NotFoundError` meant to return a non-zero exit code.
**Standard:** Use `Display` formatting `{}` prefixed with `error: ` for all stderr errors. Return `NotFoundError` *before* checking `--json` for single-item or specific retrieval commands (`next`, `show`, `search`) to guarantee non-zero exit codes.
