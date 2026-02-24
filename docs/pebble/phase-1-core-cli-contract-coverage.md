+++
id = "pebl-cdIZGN"
title = "Phase 1 Core CLI Contract Coverage"
status = "todo"
created_at = 2026-02-23T01:36:05.771442+00:00
needs = []
tags = ["bootstrap", "self_hosted"]
+++
Checklist:
- [x] P1.0 Deferred scan/duplicate handling (immediately after Phase Zero)
- [x] P1.0.a Recursive scan of `tasks-dir` for all `*.md` files
- [x] P1.0.b Duplicate ID handling (required for correct graph semantics)
- [x] P1.0.c Read commands warn to `stderr` and skip all files with duplicated IDs
- [x] P1.0.d Write commands fail with a clear error if target ID is duplicated
- [x] P1.1 `list` filters: `--status`, `--tag`, `--need`, `--priority`, `--is-ready`, `--all`, `--limit`
- [x] P1.2 `list` alias `ls`
- [x] P1.3 `--sort` for `list` with tie-breakers (`created_at`, then `id`)
- [x] P1.4 `search` command over title + body (case-insensitive substring; default list ordering)
- [ ] P1.5 `config get <key>` command
- [ ] P1.6 `help-json` command output schema
- [ ] P1.7 Help text completeness and examples for every command
- [ ] P1.8 Extend `--json` purity and stdout/stderr separation across all commands
- [ ] P1.9 Exit code mapping: `0` success, `1` runtime error, `2` usage error

Child Tasks:
- None currently. Promote only when Adaptive Decomposition criteria are met.
