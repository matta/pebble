# Decisions

## Storage & Merge Strategy

**Status**: Decided

**Context**:
We need a storage format that supports:
1.  **Deterministic Merge**: Conflict resolution must be automated where possible, and deterministic where not.
2.  **MVP Scope**: We want to avoid excessive complexity (like full CRDT op-logs) for the initial version.
3.  **Git Integration**: The system is built on Git, so the format should play well with Git's versioning.

**Decision**:
We will use a **Single JSONL Snapshot + Custom Merge Driver** strategy.

-   **Storage**: All issues are stored in a single `issues.jsonl` file in the worktree.
    -   The file is sorted by Issue ID to minimize diffs for non-conflicting changes.
    -   Each line is a complete JSON object representing an Issue.
-   **Merge Strategy**:
    -   We will implement a custom Git merge driver (`pebble merge`).
    -   This driver will perform a 3-way merge at the logical `Issue` level (Base, Ours, Theirs).
    -   **Conflict Resolution**:
        -   **Last-Write-Wins (LWW)** based on `updated_at` timestamp for scalar fields.
        -   **Set Union** for set-like fields (e.g., labels).
        -   **Append-only** (or smart merge) for list-like fields (e.g., notes).
-   **Why**:
    -   Keeps the storage model simple (one file).
    -   Avoids the complexity of managing thousands of small files (inode exhaustion).
    -   Allows us to implement sophisticated merge logic (better than Git's text merge) without changing the storage format.

**Consequences**:
-   We need to implement the `pebble merge` command.
-   Users must configure `.gitattributes` to use this driver for `issues.jsonl`.
-   Parallel edits to the same issue will be resolved by the merge driver, avoiding manual conflict resolution in most cases.
