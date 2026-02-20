use color_eyre::Result;
use color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Represents a single issue or task within the system.
///
/// This struct maps directly to the JSON object stored in `issues.jsonl`.
/// It contains all metadata related to an issue, including its lifecycle state,
/// ownership, and descriptive content.
///
/// # Examples
///
/// ```
/// use pebble::store::Issue;
///
/// let issue = Issue {
///     id: "PROJECT-123".to_string(),
///     title: "Implement documentation".to_string(),
///     description: Some("Add doc comments to public API".to_string()),
///     status: "open".to_string(),
///     priority: 1,
///     issue_type: "task".to_string(),
///     owner: Some("alice@example.com".to_string()),
///     created_at: "2023-10-27T10:00:00Z".to_string(),
///     created_by: Some("Alice".to_string()),
///     updated_at: "2023-10-27T10:00:00Z".to_string(),
///     closed_at: None,
///     close_reason: None,
///     ..Default::default()
/// };
///
/// assert_eq!(issue.status, "open");
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct IssueDependency {
    pub issue_id: String,
    pub depends_on_id: String,
    #[serde(rename = "type")]
    pub dependency_type: String,
    pub created_at: String,
    pub created_by: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Issue {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    pub priority: i32,
    pub issue_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_criteria: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub defer_until: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_by: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<IssueDependency>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_type: Option<String>,
}

impl Issue {
    /// Merges another issue into this one.
    ///
    /// This method updates mutable fields of the current issue if the `other` issue has
    /// a more recent `updated_at` timestamp. This is primarily used for syncing
    /// and importing data to resolve conflicts by taking the latest version.
    ///
    /// # Examples
    ///
    /// ```
    /// use pebble::store::Issue;
    ///
    /// let mut issue1 = Issue {
    ///     id: "1".to_string(),
    ///     title: "Old Title".to_string(),
    ///     description: Some("Old Desc".to_string()),
    ///     status: "open".to_string(),
    ///     priority: 1,
    ///     issue_type: "bug".to_string(),
    ///     owner: Some("me".to_string()),
    ///     created_at: "2023-01-01T00:00:00Z".to_string(),
    ///     created_by: Some("me".to_string()),
    ///     updated_at: "2023-01-01T00:00:00Z".to_string(),
    ///     closed_at: None,
    ///     close_reason: None,
    ///     ..Default::default()
    /// };
    ///
    /// let issue2 = Issue {
    ///     id: "1".to_string(),
    ///     title: "New Title".to_string(),
    ///     description: Some("New Desc".to_string()),
    ///     status: "in_progress".to_string(),
    ///     priority: 2,
    ///     issue_type: "bug".to_string(),
    ///     owner: Some("you".to_string()),
    ///     created_at: "2023-01-01T00:00:00Z".to_string(),
    ///     created_by: Some("me".to_string()),
    ///     updated_at: "2023-01-02T00:00:00Z".to_string(), // Newer
    ///     closed_at: None,
    ///     close_reason: None,
    ///     ..Default::default()
    /// };
    ///
    /// issue1.merge(issue2);
    ///
    /// assert_eq!(issue1.title, "New Title");
    /// assert_eq!(issue1.status, "in_progress");
    /// ```
    pub fn merge(&mut self, other: Issue) {
        if other.updated_at > self.updated_at {
            self.title = other.title;
            self.description = other.description;
            self.status = other.status;
            self.priority = other.priority;
            self.issue_type = other.issue_type;
            self.owner = other.owner;
            self.updated_at = other.updated_at;
            self.closed_at = other.closed_at;
            self.close_reason = other.close_reason;
            self.acceptance_criteria = other.acceptance_criteria;
            self.defer_until = other.defer_until;
            self.delete_reason = other.delete_reason;
            self.deleted_at = other.deleted_at;
            self.deleted_by = other.deleted_by;
            self.dependencies = other.dependencies;
            self.labels = other.labels;
            self.notes = other.notes;
            self.original_type = other.original_type;
        }
    }
}

/// Helper struct for partial deserialization of issue IDs.
#[derive(Deserialize)]
struct IdOnly {
    id: String,
}

/// Helper struct for zero-copy partial deserialization of issue IDs.
#[derive(Deserialize)]
struct IdOnlyBorrowed<'a> {
    id: &'a str,
}

/// A persistent store for managing issues in a JSON Lines (JSONL) file.
///
/// This struct handles reading and writing [`Issue`] records to a file at a specified path.
/// Each line in the file corresponds to a single JSON object representing an issue.
pub struct JsonlStore {
    path: String,
}

impl JsonlStore {
    /// Creates a new `JsonlStore` instance.
    ///
    /// The store will use the specified file path for reading and writing issues.
    /// The file does not need to exist when the store is created, but the parent
    /// directory should be writable if new issues are to be added.
    ///
    /// # Examples
    ///
    /// ```
    /// use pebble::store::JsonlStore;
    ///
    /// let store = JsonlStore::new("issues.jsonl");
    /// ```
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    /// Helper to open a buffered reader for the store file.
    /// Returns `Ok(None)` if the file does not exist.
    fn open_reader(&self) -> Result<Option<BufReader<File>>> {
        let path = Path::new(&self.path);
        if !path.exists() {
            return Ok(None);
        }
        let file = File::open(path)?;
        Ok(Some(BufReader::new(file)))
    }

    /// Reads and deserializes all issues from the store file.
    ///
    /// This method opens the file at the configured path and processes it line by line.
    /// Empty lines or lines containing only whitespace are skipped. If the file does not exist,
    /// an empty vector is returned successfully.
    ///
    /// # Errors
    ///
    /// Returns `Err` if a file I/O error occurs (e.g., permission denied, read failure)
    /// or if a line cannot be parsed as a valid [`Issue`] JSON object.
    ///
    /// # Examples
    ///
    /// ```
    /// use pebble::store::{JsonlStore, Issue};
    /// use std::io::Write;
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() -> color_eyre::Result<()> {
    /// let mut file = NamedTempFile::new()?;
    /// let json = r#"{"id":"1","title":"Test","status":"open","priority":1,"issue_type":"bug","created_at":"2023-01-01","updated_at":"2023-01-01","closed_at":null,"close_reason":null}"#;
    /// writeln!(file, "{}", json)?;
    ///
    /// let store = JsonlStore::new(file.path().to_str().unwrap());
    /// let issues = store.read_issues()?;
    ///
    /// assert_eq!(issues.len(), 1);
    /// assert_eq!(issues[0].title, "Test");
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_issues(&self) -> Result<Vec<Issue>> {
        self.read_issues_inner()
            .with_context(|| format!("Failed to read issues from {}", self.path))
    }

    /// Reads and returns only the IDs of all issues from the store.
    ///
    /// This method is optimized to avoid full deserialization of issues when only
    /// the set of existing IDs is needed (e.g., for ID generation).
    pub fn read_issue_ids(&self) -> Result<HashSet<String>> {
        self.read_issue_ids_inner()
            .with_context(|| format!("Failed to read issue IDs from {}", self.path))
    }

    fn read_issues_inner(&self) -> Result<Vec<Issue>> {
        let Some(reader) = self.open_reader()? else {
            return Ok(Vec::new());
        };

        let mut issues = Vec::new();

        // Optimization: Stream JSON objects directly from reader to avoid allocating String for each line
        let deserializer = serde_json::Deserializer::from_reader(reader);
        for issue in deserializer.into_iter::<Issue>() {
            let issue = issue?;
            issues.push(issue);
        }

        Ok(issues)
    }

    fn read_issue_ids_inner(&self) -> Result<HashSet<String>> {
        let Some(reader) = self.open_reader()? else {
            return Ok(HashSet::new());
        };

        let mut ids = HashSet::new();

        // Optimization: Stream JSON objects directly but only parse the ID field
        let deserializer = serde_json::Deserializer::from_reader(reader);
        for item in deserializer.into_iter::<IdOnly>() {
            let item = item?;
            ids.insert(item.id);
        }

        Ok(ids)
    }

    /// Overwrites the store file with the provided list of issues.
    ///
    /// This method replaces the entire content of the file with the serialized JSON
    /// representation of the given issues. If the file or its parent directories
    /// do not exist, they will be created.
    ///
    /// # Errors
    ///
    /// Returns `Err` if a file I/O error occurs (e.g., permission denied, write failure)
    /// or if serialization of any issue fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use pebble::store::{JsonlStore, Issue};
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() -> color_eyre::Result<()> {
    /// let file = NamedTempFile::new()?;
    /// let store = JsonlStore::new(file.path().to_str().unwrap());
    ///
    /// let issue = Issue {
    ///     id: "1".to_string(),
    ///     title: "Test".to_string(),
    ///     description: Some("Desc".to_string()),
    ///     status: "open".to_string(),
    ///     priority: 1,
    ///     issue_type: "task".to_string(),
    ///     owner: Some("me".to_string()),
    ///     created_at: "2023-01-01T00:00:00Z".to_string(),
    ///     created_by: Some("me".to_string()),
    ///     updated_at: "2023-01-01T00:00:00Z".to_string(),
    ///     closed_at: None,
    ///     close_reason: None,
    ///     ..Default::default()
    /// };
    ///
    /// store.write_issues(&[issue])?;
    ///
    /// let read_back = store.read_issues()?;
    /// assert_eq!(read_back.len(), 1);
    /// assert_eq!(read_back[0].title, "Test");
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_issues(&self, issues: &[Issue]) -> Result<()> {
        self.write_issues_inner(issues)
            .with_context(|| format!("Failed to write issues to {}", self.path))
    }

    fn write_issues_inner(&self, issues: &[Issue]) -> Result<()> {
        let path = Path::new(&self.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);

        // Sort issues by ID to ensure deterministic output
        let mut sorted_issues: Vec<&Issue> = issues.iter().collect();
        sorted_issues.sort_by_key(|issue| &issue.id);

        for issue in sorted_issues {
            serde_json::to_writer(&mut writer, issue)?;
            writeln!(writer)?;
        }
        writer.flush()?;

        Ok(())
    }

    /// Appends a new issue to the end of the store file.
    ///
    /// If the file does not exist, it is created. If the file exists but does not end with a
    /// newline character, one is inserted before the new record to ensure valid JSONL format.
    ///
    /// # Errors
    ///
    /// Returns `Err` if a file I/O error occurs (e.g., permission denied, seek failure, write failure)
    /// or if the issue cannot be serialized to JSON.
    ///
    /// # Examples
    ///
    /// ```
    /// use pebble::store::{JsonlStore, Issue};
    /// use tempfile::NamedTempFile;
    ///
    /// # fn main() -> color_eyre::Result<()> {
    /// let file = NamedTempFile::new()?;
    /// let store = JsonlStore::new(file.path().to_str().unwrap());
    ///
    /// let issue = Issue {
    ///     id: "2".to_string(),
    ///     title: "New Issue".to_string(),
    ///     description: Some("Description".to_string()),
    ///     status: "open".to_string(),
    ///     priority: 1,
    ///     issue_type: "bug".to_string(),
    ///     owner: Some("me".to_string()),
    ///     created_at: "2023-01-01".to_string(),
    ///     created_by: Some("me".to_string()),
    ///     updated_at: "2023-01-01".to_string(),
    ///     closed_at: None,
    ///     close_reason: None,
    ///     ..Default::default()
    /// };
    ///
    /// store.append_issue(&issue)?;
    ///
    /// let issues = store.read_issues()?;
    /// assert_eq!(issues.len(), 1);
    /// assert_eq!(issues[0].id, "2");
    /// # Ok(())
    /// # }
    /// ```
    pub fn append_issue(&self, issue: &Issue) -> Result<()> {
        self.append_issue_inner(issue)
            .with_context(|| format!("Failed to append issue to {}", self.path))
    }

    fn append_issue_inner(&self, issue: &Issue) -> Result<()> {
        let path = Path::new(&self.path);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)?;

        // If file is not empty and doesn't end with newline, add one
        let mut needs_newline = false;
        let metadata = file.metadata()?;
        if metadata.len() > 0 {
            file.seek(SeekFrom::End(-1))?;
            let mut last_byte = [0u8; 1];
            file.read_exact(&mut last_byte)?;
            if last_byte[0] != b'\n' {
                needs_newline = true;
            }
        }

        let mut writer = std::io::BufWriter::new(file);
        if needs_newline {
            writeln!(writer)?;
        }

        serde_json::to_writer(&mut writer, issue)?;
        writeln!(writer)?;
        writer.flush()?;

        Ok(())
    }

    /// Finds a single issue by its ID without loading the entire file into memory.
    ///
    /// This method is optimized for performance by reading the file line-by-line and
    /// partially deserializing only the `id` field first. It avoids full deserialization
    /// and allocation for non-matching records.
    ///
    /// # Errors
    ///
    /// Returns `Err` if a file I/O error occurs (e.g., permission denied, read failure)
    /// or if a matching line cannot be fully deserialized as an [`Issue`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pebble::store::{JsonlStore, Issue};
    /// use std::io::Write;
    ///
    /// # fn main() -> color_eyre::Result<()> {
    /// let dir = tempfile::tempdir()?;
    /// let file_path = dir.path().join("issues.jsonl");
    /// let mut file = std::fs::File::create(&file_path)?;
    /// let json = r#"{"id":"1","title":"Test","status":"open","priority":1,"issue_type":"bug","created_at":"2023-01-01","updated_at":"2023-01-01","closed_at":null,"close_reason":null}"#;
    /// writeln!(file, "{}", json)?;
    ///
    /// let store = JsonlStore::new(file_path.to_str().unwrap());
    /// let issue = store.find_issue("1")?;
    ///
    /// assert!(issue.is_some());
    /// assert_eq!(issue.unwrap().title, "Test");
    ///
    /// let not_found = store.find_issue("999")?;
    /// assert!(not_found.is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_issue(&self, id: &str) -> Result<Option<Issue>> {
        if let Some(line) = self
            .find_line_by_id(id)
            .with_context(|| format!("Failed to find issue {} in {}", id, self.path))?
        {
            let issue: Issue = serde_json::from_str(&line)?;
            Ok(Some(issue))
        } else {
            Ok(None)
        }
    }

    /// Checks if an issue exists by its ID without loading the entire file into memory.
    ///
    /// This method is optimized for existence checks (e.g., during ID generation)
    /// and avoids allocating full Issue structs.
    pub fn issue_exists(&self, id: &str) -> Result<bool> {
        Ok(self
            .find_line_by_id(id)
            .with_context(|| format!("Failed to check existence of issue {} in {}", id, self.path))?
            .is_some())
    }

    fn find_line_by_id(&self, id: &str) -> Result<Option<String>> {
        let Some(mut reader) = self.open_reader()? else {
            return Ok(None);
        };

        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            if !line.trim().is_empty() {
                // Optimization: Parse only ID first to avoid full deserialization overhead
                // Use IdOnlyBorrowed to avoid allocating String for the ID
                if serde_json::from_str::<IdOnlyBorrowed>(&line).is_ok_and(|item| item.id == id) {
                    return Ok(Some(line));
                }
            }
            line.clear();
        }

        Ok(None)
    }
}
