# Pebble Data Schema

Pebble uses a Markdown-native storage model. Each task is represented as a single Markdown file located within the configured tasks directory. 

The file consists of two parts:
1. **YAML Frontmatter**: Contains all structured metadata and graph edges. Delimited by `---`.
2. **Markdown Body**: Contains free-form description, notes, conversational elements, and checklists, separated from the frontmatter.

## Rust Schema Definitions

The data layer is strictly defined by the following Rust structures. AI agents and CLI tooling must adhere to these definitions when parsing or generating task files.

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "snake_case")] // Ensures YAML matches exactly "todo", "in_progress", "done", "canceled".
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Canceled,
}

/// Represents the exact structure of the YAML front matter.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaskFrontmatter {
    pub id: String,
    pub title: String,
    // Status strictly validated against the enum.
    pub status: TaskStatus,
    // Valid range: 0..99. Lower number = higher priority.
    pub priority: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub deps: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// The in-memory representation.
/// This is what the CLI stores in its graph topology.
#[derive(Debug, Clone)]
pub struct TaskNode {
    pub path: PathBuf,
    pub frontmatter: TaskFrontmatter,
    /// Raw Markdown content after the frontmatter delimiter. Free-form; no structural requirements.
    pub body: String,
}
```

## Omitted Audit Fields

Pebble intentionally omits traditional issue tracker audit fields from the schema:
* `owner`
* `created_by`
* `updated_at` (replaced by strict `modified_at`)
* `closed_at` (replaced by strict `resolved_at`)
* `close_reason`

**Rationale**: Audit trails are delegated to the underlying Git history. This minimizes write contention, metadata bloat, and manual synchronization effort. 

## Strict Timestamp Management

Instead of vague update markers, Pebble relies on specific timestamps for operations:
* **`modified_at`**: Used to indicate when the task was last modified. This provides a clear, deterministic indicator of stale or neglected tasks.
* **`resolved_at`**: Used purely for archival purposes. Tasks in a terminal state (`done`, `canceled`) whose `resolved_at` passes a certain age threshold can be easily archived. Relying on an explicit frontmatter field rather than file system `mtime` makes archiving deterministic and independent of Git cloning behavior.

## Priority Range

Valid values for `priority` are `0..99` (lower number = higher priority). Values outside this range are rejected as schema errors by the CLI. Tasks with no `priority` sort after all prioritized tasks in sort operations.

## Unknown Frontmatter Fields

Unknown frontmatter keys are **not** fatal for normal reads and are never removed by `pebble fix`:

* **Read commands** ignore unknown keys without warning.
* **`pebble doctor`** reports unknown fields as warnings.
* **`pebble fix`** emits warnings for unknown fields but does **not** remove them.
* **`pebble check`** treats unknown fields as errors.
