---
id: pebl-GoOi96
title: TOCTOU race in slug collision loop
status: done
created_at: 2026-02-22T22:32:50.259702+00:00
resolved_at: 2026-03-01T16:40:00+00:00
tags:
  - defect
---
There is a TOCTOU race condition in run_add: between checking filepath.exists() and calling fs::write(), another process could create the same file. Acceptable for single-user CLI but worth noting.

Common strategies:

- Atomic creation: Use OS flags like O_CREAT | O_EXCL (or Rust's OpenOptions::new().create_new(true)) which atomically fail if the file already exists — no gap between check and create.
- File locking: Acquire a lockfile (e.g., flock) before the check-and-write sequence so concurrent processes serialize access.
- Unique filenames: Use UUIDs or mkstemp-style temp files so collisions are effectively impossible, then rename atomically into place.

For a single-user CLI like pebble, option 1 (create_new) is the simplest and most appropriate fix.
