# Architectural Decisions

## Storage Strategy

**Decision:** Pebble stores tasks as individual Markdown files with TOML frontmatter.

**Context:**
We considered three options for storage:
1.  **CRDT Op-Log:** Complex to implement, overkill for a single-user or small-team CLI tool.
2.  **JSONL Snapshot:** Single file, easy to parse, but merge conflicts are frequent and hard to resolve manually.
3.  **Per-Issue Files:** One file per task.

**Rationale:**
-   **Git Friendliness:** Per-file storage delegates merge conflict resolution to Git. Conflicts are rare (only when editing the same task) and easy to resolve (standard text merge).
-   **Human Readability:** Users can read and edit tasks with any text editor.
-   **Simplicity:** No need for a database engine or complex synchronization logic.
-   **Deterministic Merging:** Since each task is a separate file, "merging" datasets is primarily about file system operations. Deterministic ordering in `list` and `next` commands ensures consistent views across environments.

**Trade-offs:**
-   **Inode Usage:** Creates one file per task, which may be inefficient for projects with tens of thousands of tasks.
-   **Performance:** Listing tasks requires reading many small files, which can be slower than reading a single large file on some file systems. However, this is mitigated by the fact that typical project task counts (hundreds to low thousands) are negligible for modern file systems.

**Status:** Implemented.
