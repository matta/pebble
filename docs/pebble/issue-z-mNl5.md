---
id: issue-z-mNl5
title: carbo pebble prints help, but not all commands have help text
status: done
created_at: 2026-02-22T20:22:11.233211+00:00
modified_at: 2026-03-01T22:20:54.980076+00:00
resolved_at: 2026-03-01T22:20:54.980075+00:00
---
run 'cargo pebble' and observe that not all commands have single-line help text. fix this TDD style, with a test that actually runs the command and verifies help output

\n\nImplemented: added integration tests that run real --help output and assert all listed commands/subcommands include one-line summaries.
