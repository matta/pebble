# CLI I/O Contract

## Streams
- `stdout`: Primary command output (human-readable by default, machine-readable with `--json`).
- `stderr`: Diagnostics, warnings, progress logs, and error messages.

## Exit Codes
- `0`: Success.
- `1`: Runtime error (I/O failure, config error, missing data).
- `2`: Usage error (invalid arguments or unsupported options).
