# Architectural Decisions

## Storage Strategy: Per-Issue Markdown Files

**Status:** Accepted

**Context:**
We need a storage mechanism for tasks that supports distributed teams, offline access, and conflict resolution without relying on a central database server. The system must be usable by both humans (editing files directly) and machines (CLI/API).

**Decision:**
We will store each task as an individual Markdown file with TOML frontmatter.

**Rationale:**
1.  **Git-Native Workflow:** By using individual files, we leverage Git's existing merge capabilities. Merge conflicts are scoped to specific files (tasks), reducing the blast radius of concurrent edits.
2.  **Human Readability:** Markdown + TOML is easily readable and editable in any text editor. This allows users to fix data issues manually if needed.
3.  **Simplicity:** The "database" is just a directory scan. No complex indexing or query engine is required for the MVP scale (thousands of tasks).
4.  **Portability:** The data is just files. It can be synced via Dropbox, rsync, or any file syncing tool if Git is not used.

**Consequences:**
1.  **ID Management:** We cannot rely on auto-incrementing integers from a central DB. We use a combination of a user-configurable prefix and a random suffix (e.g., `issue-abc123`) to ensure global uniqueness and minimize collision probability.
2.  **Performance:** Listing tasks requires reading and parsing many small files. This is acceptable for typical project sizes but may require optimization (caching) for very large repositories (10k+ tasks).
3.  **Refactoring:** Renaming a file or ID requires updating all references to it. The CLI must handle this gracefully.

## Merge Strategy: Git-Native

**Status:** Accepted

**Context:**
When multiple users edit the task graph simultaneously, we need a deterministic way to resolve conflicts.

**Decision:**
We rely on standard Git merge drivers. We do not implement a custom CRDT or operational log.

**Rationale:**
1.  **Standard Tooling:** Developers are already familiar with resolving Git conflicts.
2.  **Reduced Complexity:** Implementing a robust CRDT for a graph structure is non-trivial and overkill for the intended use case (async collaboration).
3.  **Deterministic Formatting:** The CLI enforces deterministic formatting of the TOML frontmatter (sorted keys, consistent serialization) to minimize "noisy" diffs that could lead to false conflicts.

**Consequences:**
1.  **Conflict Resolution:** Users must manually resolve merge conflicts if two people edit the same task file simultaneously in conflicting ways.
2.  **Graph Integrity:** A merge could theoretically leave the graph in an inconsistent state (e.g., a task depends on a deleted ID). The `pebble check` command provides diagnostics to identify and fix these issues.
