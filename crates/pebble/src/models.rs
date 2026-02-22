use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Canceled,
}

/// Represents the exact structure of the YAML front matter.
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
            serde_yaml::from_str::<TaskStatus>("todo").unwrap(),
            TaskStatus::Todo
        );
        assert_eq!(
            serde_yaml::from_str::<TaskStatus>("in_progress").unwrap(),
            TaskStatus::InProgress
        );
        assert_eq!(
            serde_yaml::from_str::<TaskStatus>("done").unwrap(),
            TaskStatus::Done
        );
        assert_eq!(
            serde_yaml::from_str::<TaskStatus>("canceled").unwrap(),
            TaskStatus::Canceled
        );

        let err = serde_yaml::from_str::<TaskStatus>("invalid_status").unwrap_err();
        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn test_task_frontmatter_deserialization() {
        let yaml = r#"
id: issue-123
title: Implement Task Node
status: todo
priority: 1
created_at: 2026-02-21T17:00:00Z
deps: [issue-122]
"#;
        let fm: TaskFrontmatter = serde_yaml::from_str(yaml).unwrap();
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
