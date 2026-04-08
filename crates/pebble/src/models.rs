use chrono::{DateTime, Utc};
use clap::{ValueEnum, builder::PossibleValue};
use color_eyre::eyre::Result;
use serde::de::Error as SerdeDeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::error;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::result::Result as StdResult;
use std::str::FromStr;

/// The YAML frontmatter delimiter used in task files.
pub const FRONTMATTER_DELIMITER: &str = "---";

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

/// Error representing a search or retrieval failure when the item is not found.
/// Should result in exit code 1 without a stack trace.
#[derive(Debug)]
pub struct NotFoundError(pub String);
impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl error::Error for NotFoundError {}

/// Represents the actionable lifecycle states of a task.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum LiveStatus {
    /// Task has not been started yet.
    Todo,
    /// Task is actively being worked on.
    InProgress,
}

/// Represents the closed lifecycle states of a task.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ClosedStatus {
    /// Task has been completed successfully.
    Done,
    /// Task has been abandoned without completion.
    Canceled,
}

/// Represents the lifecycle state of a task.
///
/// # Examples
///
/// ```
/// # use pebble::models::TaskStatus;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let status = <TaskStatus as std::str::FromStr>::from_str("todo")?;
/// // Serializes to "todo"
/// assert_eq!(serde_json::to_string(&status)?, "\"todo\"");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TaskStatus {
    /// Task can still be worked on.
    Live(LiveStatus),
    /// Task is in a terminal state.
    Closed(ClosedStatus),
}

impl TaskStatus {
    pub const fn todo() -> Self {
        Self::Live(LiveStatus::Todo)
    }

    pub const fn in_progress() -> Self {
        Self::Live(LiveStatus::InProgress)
    }

    pub const fn done() -> Self {
        Self::Closed(ClosedStatus::Done)
    }

    pub const fn canceled() -> Self {
        Self::Closed(ClosedStatus::Canceled)
    }

    /// Checks if the status represents an open, workable state.
    ///
    /// Returns `true` if the status is one of the live statuses.
    pub fn is_actionable(&self) -> bool {
        matches!(self, Self::Live(_))
    }

    /// Returns `true` if the status represents a terminal state (`done` or `canceled`).
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Closed(_))
    }

    /// Returns the contained live status, if this task is actionable.
    pub fn as_live(&self) -> Option<LiveStatus> {
        match self {
            Self::Live(status) => Some(*status),
            Self::Closed(_) => None,
        }
    }

    /// Returns the contained closed status, if this task is terminal.
    pub fn as_closed(&self) -> Option<ClosedStatus> {
        match self {
            Self::Live(_) => None,
            Self::Closed(status) => Some(*status),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Live(LiveStatus::Todo) => "todo",
            Self::Live(LiveStatus::InProgress) => "in_progress",
            Self::Closed(ClosedStatus::Done) => "done",
            Self::Closed(ClosedStatus::Canceled) => "canceled",
        }
    }
}

impl Serialize for TaskStatus {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str((*self).as_str())
    }
}

impl<'de> Deserialize<'de> for TaskStatus {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        <Self as FromStr>::from_str(&raw).map_err(SerdeDeError::custom)
    }
}

impl FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        match s {
            "todo" => Ok(Self::Live(LiveStatus::Todo)),
            "in_progress" => Ok(Self::Live(LiveStatus::InProgress)),
            "done" => Ok(Self::Closed(ClosedStatus::Done)),
            "canceled" => Ok(Self::Closed(ClosedStatus::Canceled)),
            _ => Err(format!(
                "invalid status '{s}', expected one of: todo, in_progress, done, canceled"
            )),
        }
    }
}

impl ValueEnum for TaskStatus {
    fn value_variants<'a>() -> &'a [Self] {
        const VARIANTS: [TaskStatus; 4] = [
            TaskStatus::Live(LiveStatus::Todo),
            TaskStatus::Live(LiveStatus::InProgress),
            TaskStatus::Closed(ClosedStatus::Done),
            TaskStatus::Closed(ClosedStatus::Canceled),
        ];
        &VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new((*self).as_str()))
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            TaskStatus::Live(LiveStatus::Todo) => "Todo",
            TaskStatus::Live(LiveStatus::InProgress) => "InProgress",
            TaskStatus::Closed(ClosedStatus::Done) => "Done",
            TaskStatus::Closed(ClosedStatus::Canceled) => "Canceled",
        };
        write!(f, "{label}")
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

/// Represents the exact structure of task frontmatter.
///
/// This struct corresponds to the metadata block at the top of a task file.
/// It includes the task's unique ID, title, status, and other metadata.
///
/// # Examples
///
/// ```
/// # use pebble::models::{TaskFrontmatter, TaskStatus};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let yaml_str = "id: issue-123\ntitle: Test\nstatus: todo\ncreated_at: \"2023-01-01T00:00:00Z\"";
/// let fm: TaskFrontmatter = serde_saphyr::from_str(yaml_str)?;
/// assert_eq!(fm.title, "Test");
/// assert_eq!(fm.status, TaskStatus::todo());
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
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp of the last modification, if the task has been edited.
    pub modified_at: Option<DateTime<Utc>>,
    /// Timestamp when the task reached a terminal status, if applicable.
    pub resolved_at: Option<DateTime<Utc>>,
    /// IDs of tasks that must reach a terminal status before this task is ready.
    #[serde(default)]
    pub needs: Vec<String>,
    /// Arbitrary labels attached to the task.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Unknown keys preserved from parsing.
    #[serde(flatten)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
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
/// # use chrono::{DateTime, Utc};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let node = TaskNode {
///     path: PathBuf::from("tasks/issue-123.md"),
///     frontmatter: TaskFrontmatter {
///         id: "issue-123".into(),
///         title: "My Task".into(),
///         status: TaskStatus::todo(),
///         priority: None,
///         created_at: Some(DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")?.with_timezone(&Utc)),
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
    /// Parsed task frontmatter for this task.
    pub frontmatter: TaskFrontmatter,
    /// Raw Markdown content after the frontmatter delimiter. Free-form; no structural requirements.
    pub body: String,
}

#[derive(Serialize)]
struct YamlTaskFrontmatter {
    id: String,
    title: String,
    status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<Priority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_at: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    needs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl TaskNode {
    pub(crate) fn get_content_for_disk(&self) -> Result<String> {
        let yaml_frontmatter = YamlTaskFrontmatter {
            id: self.frontmatter.id.clone(),
            title: self.frontmatter.title.clone(),
            status: self.frontmatter.status,
            priority: self.frontmatter.priority,
            created_at: self.frontmatter.created_at.map(|dt| dt.to_rfc3339()),
            modified_at: self.frontmatter.modified_at.map(|dt| dt.to_rfc3339()),
            resolved_at: self.frontmatter.resolved_at.map(|dt| dt.to_rfc3339()),
            needs: self.frontmatter.needs.clone(),
            tags: self.frontmatter.tags.clone(),
            extra: self.frontmatter.extra.clone(),
        };

        let mut yaml_payload = serde_saphyr::to_string(&yaml_frontmatter)?;
        if let Some(stripped) = yaml_payload.strip_prefix(&format!("{}\n", FRONTMATTER_DELIMITER)) {
            yaml_payload = stripped.to_string();
        }
        if let Some(stripped) = yaml_payload.strip_suffix("...\n") {
            yaml_payload = stripped.to_string();
        }
        if !yaml_payload.ends_with('\n') {
            yaml_payload.push('\n');
        }

        let body = self.body.trim();
        if body.is_empty() {
            Ok(format!(
                "{}\n{}{}\n",
                FRONTMATTER_DELIMITER, yaml_payload, FRONTMATTER_DELIMITER
            ))
        } else {
            Ok(format!(
                "{}\n{}{}\n\n{}\n",
                FRONTMATTER_DELIMITER, yaml_payload, FRONTMATTER_DELIMITER, body
            ))
        }
    }

    /// Writes the task content to its file path on disk.
    ///
    /// # Errors
    ///
    /// Returns an error if generating the serialized content fails or if the file write operation fails.
    pub fn write_to_disk(&self) -> Result<()> {
        let content = self.get_content_for_disk()?;
        fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn create_new_to_disk(&self) -> Result<()> {
        let content = self.get_content_for_disk()?;

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}

pub fn default_datetime() -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod tests;
