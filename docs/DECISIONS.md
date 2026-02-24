# Architectural Decisions

## Storage Strategy

**Decision:** Pebble uses a "Per-issue Markdown files with TOML frontmatter" storage strategy.

**Rationale:**
*   **Human-Readable & Editable:** Markdown is the standard for documentation. Users can easily read and edit tasks using any text editor, even without the CLI.
*   **Git-Friendly:** Storing each task as a separate file minimizes merge conflicts compared to a single large JSON/YAML file. Git handles file-level operations efficiently.
*   **Graph Semantics:** Dependencies and relationships are stored in the TOML frontmatter (`needs`, `blocking`, etc.), allowing the CLI to build an in-memory graph for advanced queries (e.g., `pebble next`, `pebble list --is-ready`).
*   **No Database Required:** The filesystem is the database. This simplifies deployment and usage (no server to run).
*   **Performance:** While scanning thousands of files can be slower than a database, it is sufficient for the scale of personal or small-team task management (thousands of tasks).

**Format:**
Each task is a file named `<slug>.md` (or `<slug>-<n>.md` for collisions) in the configured `tasks-dir`.
The content structure is:
```markdown
+++
id = "issue-<unique-suffix>"
title = "Task Title"
status = "todo"
created_at = 2023-10-27T10:00:00Z
needs = ["issue-123"]
tags = ["bug", "ui"]
+++

Markdown body content here...
```

## Merge Strategy

**Context:** When multiple users edit tasks concurrently, merge conflicts can occur in the frontmatter or body.

**Decision:**
*   **Frontmatter:** Merge logic should treat TOML frontmatter as structured data. A deterministic 3-way merge strategy can be applied to fields like `needs`, `tags`, and `status`. (Future Work: Implement custom merge driver or CLI command to handle this deterministically).
*   **Body:** Git's standard text merge is usually sufficient for the Markdown body.

## CLI & JSON Output

**Decision:**
*   **--json:** All commands that output data MUST support `--json` to facilitate integration with other tools (e.g., scripts, IDEs, agents).
*   **Stdout/Stderr Separation:**
    *   **Stdout:** reserved for the requested data (JSON or human-readable list).
    *   **Stderr:** reserved for logs, status messages, errors, and diagnostics.
    *   This ensures that `pebble command --json | jq .` always works without interference from log messages.
*   **Exit Codes:**
    *   `0`: Success.
    *   `1`: Runtime error (e.g., file not found, permission denied).
    *   `2`: Usage error (e.g., invalid arguments, unknown command).
