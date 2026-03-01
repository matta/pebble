use clap::ValueEnum;
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use toml_datetime::Datetime;

/// Error representing an invalid user invocation or configuration value.
/// Should results in exit code 2.
#[derive(Debug)]
pub struct UsageError(pub String);
impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl error::Error for UsageError {}

/// Represents the lifecycle state of a task.
///
/// This enum defines the possible states a task can be in.
/// The variants are serialized to snake_case strings in the TOML frontmatter.
///
/// # Examples
///
/// ```
/// # use pebble::models::TaskStatus;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let status = TaskStatus::Todo;
/// // Serializes to "todo"
/// assert_eq!(toml::to_string(&status)?.trim(), "\"todo\"");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Copy, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task has not been started yet.
    Todo,
    /// Task is actively being worked on.
    InProgress,
    /// Task has been completed successfully.
    Done,
    /// Task has been abandoned without completion.
    Canceled,
}

impl TaskStatus {
    /// Returns `true` if the status represents an open, workable state (`todo` or `in_progress`).
    pub fn is_actionable(&self) -> bool {
        matches!(self, Self::Todo | Self::InProgress)
    }

    /// Returns `true` if the status represents a terminal state (`done` or `canceled`).
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Done | Self::Canceled)
    }
}

bounded_integer::bounded_integer! {
    /// Priority newtype constrained to the inclusive range `0..=99`.
    ///
    /// This enforces Pebble's domain invariant at the type level so invalid priorities
    /// cannot be represented in parsed task data or command mutation inputs.
    #[repr(u32)]
    pub struct Priority(0, 99);
}

/// Represents the exact structure of the TOML front matter.
///
/// This struct corresponds to the metadata block at the top of a task file.
/// It includes the task's unique ID, title, status, and other metadata.
///
/// # Examples
///
/// ```
/// # use pebble::models::{TaskFrontmatter, TaskStatus};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let toml_str = "id = \"issue-123\"\ntitle = \"Test\"\nstatus = \"todo\"\ncreated_at = 2023-01-01T00:00:00Z";
/// let fm: TaskFrontmatter = toml::from_str(toml_str)?;
/// assert_eq!(fm.title, "Test");
/// assert_eq!(fm.status, TaskStatus::Todo);
/// # Ok(())
/// # }
/// ```
// TODO: Implement unknown-key handling: reads ignore; check/check --warn-only report; check --fix preserves.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TaskFrontmatter {
    /// Unique, immutable task identifier (e.g. `"issue-abc123"`).
    pub id: String,
    /// Human-readable title of the task.
    pub title: String,
    /// Lifecycle state; strictly validated against the [`TaskStatus`] enum.
    pub status: TaskStatus,
    /// Optional priority for ordering (lower value = higher priority). Range 0–99.
    pub priority: Option<Priority>,
    /// Timestamp when the task was created.
    pub created_at: Option<Datetime>,
    /// Timestamp of the last modification, if the task has been edited.
    pub modified_at: Option<Datetime>,
    /// Timestamp when the task reached a terminal status, if applicable.
    pub resolved_at: Option<Datetime>,
    /// IDs of tasks that must reach a terminal status before this task is ready.
    #[serde(default)]
    pub needs: Vec<String>,
    /// Arbitrary labels attached to the task.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Unknown keys preserved from parsing.
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, toml::Value>,
}

/// The in-memory representation of a task.
///
/// This struct combines the task's metadata ([`TaskFrontmatter`]), its file path,
/// and its raw markdown body. It serves as the primary node in the task graph.
///
/// # Examples
///
/// ```
/// # use std::path::PathBuf;
/// # use std::str::FromStr;
/// # use pebble::models::{TaskNode, TaskFrontmatter, TaskStatus};
/// # use toml_datetime::Datetime;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let node = TaskNode {
///     path: PathBuf::from("tasks/issue-123.md"),
///     frontmatter: TaskFrontmatter {
///         id: "issue-123".into(),
///         title: "My Task".into(),
///         status: TaskStatus::Todo,
///         priority: None,
///         created_at: Datetime::from_str("2023-01-01T00:00:00Z")?,
///         modified_at: None,
///         resolved_at: None,
///         needs: vec![],
///         tags: vec![],
///         extra: Default::default(),
///     },
///     body: "Detailed description".into(),
/// };
/// assert_eq!(node.frontmatter.title, "My Task");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TaskNode {
    /// Absolute path to the Markdown file on disk.
    pub path: PathBuf,
    /// Parsed TOML frontmatter for this task.
    pub frontmatter: TaskFrontmatter,
    /// Raw Markdown content after the frontmatter delimiter. Free-form; no structural requirements.
    pub body: String,
}

impl TaskNode {
    pub fn write_to_disk(&self) -> Result<()> {
        let fm_toml = toml::to_string(&self.frontmatter)?;
        let mut content = format!("+++\n{}+++\n{}", fm_toml, self.body);
        if !content.ends_with('\n') {
            content.push('\n');
        }
        fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn create_new_to_disk(&self) -> Result<()> {
        let fm_toml = toml::to_string(&self.frontmatter)?;
        let mut content = format!("+++\n{}+++\n{}", fm_toml, self.body);
        if !content.ends_with('\n') {
            content.push('\n');
        }

        use std::fs::OpenOptions;
        use std::io::Write;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

pub fn default_datetime() -> Datetime {
    Datetime {
        date: Some(toml_datetime::Date {
            year: 1970,
            month: 1,
            day: 1,
        }),
        time: Some(toml_datetime::Time {
            hour: 0,
            minute: 0,
            second: 0,
            nanosecond: 0,
        }),
        offset: Some(toml_datetime::Offset::Z),
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::mem;

    #[test]
    fn test_task_status_deserialization() {
        assert_eq!(
            serde_json::from_str::<TaskStatus>("\"todo\"").expect("Should deserialize todo"),
            TaskStatus::Todo
        );
        assert_eq!(
            serde_json::from_str::<TaskStatus>("\"in_progress\"")
                .expect("Should deserialize in_progress"),
            TaskStatus::InProgress
        );
        assert_eq!(
            serde_json::from_str::<TaskStatus>("\"done\"").expect("Should deserialize done"),
            TaskStatus::Done
        );
        assert_eq!(
            serde_json::from_str::<TaskStatus>("\"canceled\"")
                .expect("Should deserialize canceled"),
            TaskStatus::Canceled
        );

        let err = serde_json::from_str::<TaskStatus>("\"invalid_status\"")
            .expect_err("Should fail to deserialize invalid status");
        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn test_task_status_helpers() {
        assert!(TaskStatus::Todo.is_actionable());
        assert!(TaskStatus::InProgress.is_actionable());
        assert!(!TaskStatus::Done.is_actionable());
        assert!(!TaskStatus::Canceled.is_actionable());

        assert!(TaskStatus::Done.is_closed());
        assert!(TaskStatus::Canceled.is_closed());
        assert!(!TaskStatus::Todo.is_closed());
        assert!(!TaskStatus::InProgress.is_closed());
    }

    #[test]
    fn test_task_frontmatter_deserialization() {
        let toml_str = r#"
id = "issue-123"
title = "Implement Task Node"
status = "todo"
priority = 1
created_at = 2026-02-21T17:00:00Z
needs = ["issue-122"]
"#;
        let fm: TaskFrontmatter = toml::from_str(toml_str).expect("Valid task frontmatter");
        assert_eq!(fm.id, "issue-123");
        assert_eq!(fm.title, "Implement Task Node");
        assert_eq!(fm.status, TaskStatus::Todo);
        assert_eq!(
            fm.priority,
            Some(Priority::try_from(1).expect("Priority 1 is valid"))
        );
        assert_eq!(fm.needs, vec!["issue-122"]);
        assert!(
            fm.tags.is_empty(),
            "Tags should default to empty vec if omitted"
        );
    }

    #[test]
    fn test_priority_try_from_u8_enforces_range() {
        assert_eq!(
            Priority::try_from(0u8).expect("Priority 0 is valid").get(),
            0
        );
        assert_eq!(
            Priority::try_from(99u8)
                .expect("Priority 99 is valid")
                .get(),
            99
        );
        assert!(Priority::try_from(100u8).is_err());
    }

    #[test]
    fn test_task_frontmatter_rejects_out_of_range_priority() {
        let toml_str = r#"
id = "issue-123"
title = "Implement Task Node"
status = "todo"
priority = 100
created_at = 2026-02-21T17:00:00Z
"#;

        let err = toml::from_str::<TaskFrontmatter>(toml_str)
            .expect_err("Should reject out-of-range priority");
        assert!(
            err.to_string().contains("priority"),
            "Expected priority validation error, got: {err}"
        );
    }

    #[test]
    fn test_priority_toml_serializes_as_integer() {
        #[derive(Serialize)]
        struct Wrapper {
            priority: Priority,
        }

        let wrapper = Wrapper {
            priority: Priority::try_from(5).expect("Priority 5 is valid"),
        };
        let toml = toml::to_string(&wrapper).expect("Should serialize Wrapper");
        assert_eq!(toml.trim(), "priority = 5");
    }

    #[test]
    fn test_priority_uses_u32_representation_size() {
        assert_eq!(
            mem::size_of::<Priority>(),
            mem::size_of::<u32>(),
            "Priority should use u32 representation"
        );
    }

    #[test]
    fn test_priority_into_u32() {
        let p = Priority::new(42).expect("Priority 42 is valid");
        let v: u32 = p.into();
        assert_eq!(v, 42);
    }
}
