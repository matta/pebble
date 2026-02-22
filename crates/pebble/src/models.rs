use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use toml_datetime::Datetime;

/// Represents the lifecycle state of a task.
///
/// This enum defines the possible states a task can be in.
/// The variants are serialized to snake_case strings in the TOML frontmatter.
///
/// # Examples
///
/// ```
/// use pebble::models::TaskStatus;
///
/// let status = TaskStatus::Todo;
/// // Serializes to "todo"
/// assert_eq!(toml::to_string(&status).unwrap().trim(), "\"todo\"");
/// ```
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Canceled,
}

/// Represents the exact structure of the TOML front matter.
///
/// This struct corresponds to the metadata block at the top of a task file.
/// It includes the task's unique ID, title, status, and other metadata.
///
/// # Examples
///
/// ```
/// use pebble::models::{TaskFrontmatter, TaskStatus};
///
/// let toml_str = "id = \"123\"\ntitle = \"Test\"\nstatus = \"todo\"\ncreated_at = 2023-01-01T00:00:00Z";
/// let fm: TaskFrontmatter = toml::from_str(toml_str).unwrap();
/// assert_eq!(fm.title, "Test");
/// assert_eq!(fm.status, TaskStatus::Todo);
/// ```
// TODO: Widen `priority` from Option<u8> to Option<u32> with 0..99 range validation.
//   Also update corresponding Option<u8> in: main.rs (Add, Update), commands_write.rs (run_add, run_update).
// TODO: Implement unknown-key handling: reads ignore; doctor/fix warn; check errors; fix preserves.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TaskFrontmatter {
    pub id: String,
    pub title: String,
    // Status strictly validated against the enum.
    pub status: TaskStatus,
    // Optional priority for ordering.
    pub priority: Option<u8>,
    pub created_at: Datetime,
    pub modified_at: Option<Datetime>,
    pub resolved_at: Option<Datetime>,
    #[serde(default)]
    pub deps: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// The in-memory representation of a task.
///
/// This struct combines the task's metadata ([`TaskFrontmatter`]), its file path,
/// and its raw markdown body. It serves as the primary node in the task graph.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use std::str::FromStr;
/// use pebble::models::{TaskNode, TaskFrontmatter, TaskStatus};
/// use toml_datetime::Datetime;
///
/// let node = TaskNode {
///     path: PathBuf::from("tasks/123.md"),
///     frontmatter: TaskFrontmatter {
///         id: "123".into(),
///         title: "My Task".into(),
///         status: TaskStatus::Todo,
///         priority: None,
///         created_at: Datetime::from_str("2023-01-01T00:00:00Z").unwrap(),
///         modified_at: None,
///         resolved_at: None,
///         deps: vec![],
///         tags: vec![],
///     },
///     body: "Detailed description".into(),
/// };
/// assert_eq!(node.frontmatter.title, "My Task");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TaskNode {
    pub path: PathBuf,
    pub frontmatter: TaskFrontmatter,
    /// Raw Markdown content after the frontmatter delimiter. Free-form; no structural requirements.
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_deserialization() {
        assert_eq!(
            serde_json::from_str::<TaskStatus>("\"todo\"").unwrap(),
            TaskStatus::Todo
        );
        assert_eq!(
            serde_json::from_str::<TaskStatus>("\"in_progress\"").unwrap(),
            TaskStatus::InProgress
        );
        assert_eq!(
            serde_json::from_str::<TaskStatus>("\"done\"").unwrap(),
            TaskStatus::Done
        );
        assert_eq!(
            serde_json::from_str::<TaskStatus>("\"canceled\"").unwrap(),
            TaskStatus::Canceled
        );

        let err = serde_json::from_str::<TaskStatus>("\"invalid_status\"").unwrap_err();
        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn test_task_frontmatter_deserialization() {
        let toml_str = r#"
id = "issue-123"
title = "Implement Task Node"
status = "todo"
priority = 1
created_at = 2026-02-21T17:00:00Z
deps = ["issue-122"]
"#;
        let fm: TaskFrontmatter = toml::from_str(toml_str).unwrap();
        assert_eq!(fm.id, "issue-123");
        assert_eq!(fm.title, "Implement Task Node");
        assert_eq!(fm.status, TaskStatus::Todo);
        assert_eq!(fm.priority, Some(1));
        assert_eq!(fm.deps, vec!["issue-122"]);
        assert!(
            fm.tags.is_empty(),
            "Tags should default to empty vec if omitted"
        );
    }
}
