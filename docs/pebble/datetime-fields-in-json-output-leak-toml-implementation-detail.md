---
id: "pebl-549tht8znx"
title: "datetime fields in JSON output leak TOML implementation detail"
status: "todo"
created_at: "2026-02-25T03:43:10.719708+00:00"
needs: []
tags: []
---
Currently, `pebl` emits datetime fields in JSON output as TOML internal wrapper objects:

```json
"created_at": {"$__toml_private_datetime": "2026-02-24T05:22:50.792978+00:00"}
```

This leaks a TOML parser implementation detail (`toml_edit`'s `Datetime` type) into the public JSON contract. Consumers see an opaque object instead of a plain string.

**Expected output** (all datetime fields, all commands):

```json
"created_at": "2026-02-24T05:22:50.792978+00:00"
```

Fix: serialize datetime fields as plain RFC 3339 strings in JSON. The fix should apply to every field that currently emits a datetime (`created_at`, `modified_at`, `resolved_at`) across all commands that emit JSON (`add`, `list`, `get`, `update`, etc.).
