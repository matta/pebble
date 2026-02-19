# CLI I/O Contract

This document defines Pebble's observable CLI behavior for humans and agents.
It is aligned with `docs/ux-for-agents.md` and the current implementation plan.

## Streams
- `stdout`: Primary command output only. Human-readable by default, machine-readable with `--json`.
- `stderr`: Diagnostics, warnings, progress logs, and error messages. Never emit JSON data to `stderr`.

## Command Surface
- Every command has a **clear, distinct purpose**. Avoid redundant commands that do the same thing.
- All commands that produce output **must** support `--json`.
- No interactive prompts. If confirmation is needed, require `--yes` / `--force` or fail with a usage error.

## Exit Codes
- `0`: Success.
- `1`: Runtime error (I/O failure, config error, missing data).
- `2`: Usage error (invalid arguments or unsupported options).

## JSON Mode
- `--json` outputs **valid JSON to stdout and nothing else**.
- JSON output is stable and schema-backed (see `--help-json` schemas).
- When `--json` is set, suppress color/formatting and any extra decorations.

## Help and Discoverability
- `--help` **must describe every option** for the command, including defaults, not just list the argument name.
- `--help` must include concrete usage examples for the common path.
- `--help-json` provides a machine-readable description of commands, flags, and output schemas.

## Output Semantics
- Human output should be readable and may emit diagnostics to `stderr`.
- Structured output must never be mixed with diagnostics.
- Commands that return structured data:
  - `list` => IssueList JSON array
  - `show`, `add`, `update` => Issue JSON object
  - `config get`, `init`, `import`, `sync`, `search` => structured JSON response

## Idempotency and Safety
- Commands should be safe to re-run; `sync` and `import` are expected to be idempotent.
- When failing due to invalid usage, return exit code `2` with a clear error message on `stderr`.
