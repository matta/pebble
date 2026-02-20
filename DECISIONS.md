# Decisions

## Storage Strategy

**Decision:** Use a **JSONL Snapshot** approach for storing issues.

**Rationale:**
- **Simplicity:** JSONL (JSON Lines) is a simple, human-readable format. Each line is a valid JSON object.
- **Git Friendliness:** Text-based format works well with Git diffs and merges.
- **Performance:** Appending new issues is O(1). Reading can be streamed.
- **Deterministic Output:** The file is rewritten with issues sorted by ID to minimize merge conflicts due to ordering changes.

**Implementation Details:**
- File: `issues.jsonl` located in the root of the data worktree.
- Schema: Strict adherence to the `Issue` struct defined in `crates/pebble/src/store.rs`.
- Persistence: `JsonlStore::write_issues` overwrites the file with sorted issues.

## Merge Strategy

**Decision:** Use a **Last-Write-Wins (LWW)** strategy based on the `updated_at` timestamp.

**Rationale:**
- **Conflict Resolution:** Simple and effective for most issue tracking scenarios where concurrent edits to the same field are rare.
- **Granularity:** Merging happens at the **Issue** level. If two branches modify different fields of the same issue, the LWW rule applies to the *entire issue object* based on the latest `updated_at`.
- **Determinism:** Given the same set of inputs (local and remote issues), the merge result is predictable.

**Implementation Details:**
- Logic: Encapsulated in `Issue::merge` method.
- Trigger: During `pebble sync`, a custom merge driver (to be implemented) or manual resolution tool should apply this logic.
- Timestamp: `updated_at` must be updated on every modification to ensure LWW works correctly.

## MVP Command Surface

**Decision:** The following commands constitute the Minimum Viable Product (MVP) for daily use:

1.  **Core Operations:**
    - `add`: Create new issues.
    - `list`: List issues with filters (`--status`, `--owner`, `--priority`).
    - `show`: Display details of a single issue.
    - `update`: Modify issue fields (`title`, `description`, `status`, `priority`, `owner`, `issue_type`).
    - `search`: fast substring search in title/description.

2.  **System Operations:**
    - `init`: Initialize repository and worktree.
    - `sync`: Synchronize with remote via worktree.
    - `import`: Import issues from existing JSONL files.
    - `config`: Manage configuration.

3.  **Output Formats:**
    - All commands support `--json` for machine readability.
    - The CLI supports a global `--help-json` flag to dump the entire CLI schema for integration with tools/agents.
