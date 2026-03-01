---
id: "pebl-vtz46af5jv"
title: "Mitigate CLI body string escaping pitfalls"
status: "todo"
created_at: "2026-02-24T05:37:28.694274+00:00"
needs: []
tags: []
---

### Problem Statement
When dealing with multiline strings in a CLI environment (particularly via shell scripts or automated agents), standard shell quoting rules can cause unintended escape sequence outcomes. For example, `pebble update --append-body "\n\nText"` often results in literal `\` and `n` characters being appended to a markdown task instead of true newline characters.

### Possible Mitigations

1. **Support Reading from Standard Input (`stdin`)**
   Modify the CLI to allow passing `-` to `--body` or `--append-body` so it reads content from `stdin`. This bypasses shell quoting and escape sequence behaviors completely `(e.g., echo -e "\n\nText" | pebble update PEBL-1 --append-body -)`.

2. **Auto-Unescape Literal Characters (`\n`, `\t`) within Strings**
   Intercept the string values for `--body` and `--append-body` inside the Rust arg parser and replace literal sequences like `\` followed by `n` with actual newline characters. This requires a robust unescaping implementation that still honors escaped slashes.

3. **Add `--body-file` and `--append-body-file` Arguments**
   Provide dedicated flags that accept file paths instead of string arguments, allowing scripts (or AI agents) to write content safely to a temporary disk location first, then target the file for appending/replacing.

### Recommendation
**Implement Option 1 (`--body -` and `--append-body -` to stream from `stdin`).**
Reading from standard input is the most idiomatic, robust, and POSIX-compliant approach for CLI utilities dealing with unknown multiline inputs. It completely circumvents complex quoting nuances across different shells.
