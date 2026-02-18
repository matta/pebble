use color_eyre::Result;
use color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
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
///     description: "Add doc comments to public API".to_string(),
///     status: "open".to_string(),
///     priority: 1,
///     issue_type: "task".to_string(),
///     owner: "alice@example.com".to_string(),
///     created_at: "2023-10-27T10:00:00Z".to_string(),
///     created_by: "Alice".to_string(),
///     updated_at: "2023-10-27T10:00:00Z".to_string(),
///     closed_at: None,
///     close_reason: None,
/// };
///
/// assert_eq!(issue.status, "open");
/// ```
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Issue {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub status: String,
    pub priority: i32,
    pub issue_type: String,
    #[serde(default)]
    pub owner: String,
    pub created_at: String,
    #[serde(default)]
    pub created_by: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub close_reason: Option<String>,
}

impl Issue {
    /// Merges another issue into this one.
    ///
    /// Updates all fields if the other issue has a more recent `updated_at` timestamp.
    /// This is used for syncing and importing data.
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
        }
    }
}

/// A persistent store for managing issues in a JSON Lines (JSONL) file.
///
/// This struct handles reading and writing [`Issue`] records to a file at a specified path.
/// Each line in the file corresponds to a single JSON object representing an issue.
pub struct JsonlStore {
    path: String,
}

impl JsonlStore {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
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

    fn read_issues_inner(&self) -> Result<Vec<Issue>> {
        let path = Path::new(&self.path);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut issues = Vec::new();

        // Optimization: Stream JSON objects directly from reader to avoid allocating String for each line
        let deserializer = serde_json::Deserializer::from_reader(reader);
        for issue in deserializer.into_iter::<Issue>() {
            let issue = issue?;
            issues.push(issue);
        }

        Ok(issues)
    }

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

        for issue in issues {
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
    ///     description: "Description".to_string(),
    ///     status: "open".to_string(),
    ///     priority: 1,
    ///     issue_type: "bug".to_string(),
    ///     owner: "me".to_string(),
    ///     created_at: "2023-01-01".to_string(),
    ///     created_by: "me".to_string(),
    ///     updated_at: "2023-01-01".to_string(),
    ///     closed_at: None,
    ///     close_reason: None,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_issues() {
        let mut file = NamedTempFile::new().unwrap();
        let issue_json = r#"{"id":"mydoo-0kq","title":"Test Issue","description":"Desc","status":"open","priority":0,"issue_type":"task","owner":"me","created_at":"2026-01-01T00:00:00Z","created_by":"Me","updated_at":"2026-01-01T00:00:00Z","closed_at":null,"close_reason":null}"#;
        writeln!(file, "{}", issue_json).unwrap();

        let path = file.path().to_str().unwrap();
        let store = JsonlStore::new(path);

        let issues = store.read_issues().expect("Failed to read issues");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, "mydoo-0kq");
        assert_eq!(issues[0].title, "Test Issue");
    }

    #[test]
    fn test_write_issues() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap().to_string();
        let store = JsonlStore::new(&path);

        let issues = vec![Issue {
            id: "test-1".to_string(),
            title: "Title 1".to_string(),
            description: "Desc 1".to_string(),
            status: "open".to_string(),
            priority: 1,
            issue_type: "task".to_string(),
            owner: "me".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            created_by: "Me".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            closed_at: None,
            close_reason: None,
        }];

        store.write_issues(&issues).expect("Failed to write issues");

        let read_back = store.read_issues().expect("Failed to read back issues");
        assert_eq!(read_back, issues);
    }

    #[test]
    fn test_issue_merge() {
        let mut base = Issue {
            id: "test-1".to_string(),
            title: "Original Title".to_string(),
            description: "Original Desc".to_string(),
            status: "open".to_string(),
            priority: 1,
            issue_type: "task".to_string(),
            owner: "me".to_string(),
            created_at: "2026-01-01T10:00:00Z".to_string(),
            created_by: "Me".to_string(),
            updated_at: "2026-01-01T10:00:00Z".to_string(),
            closed_at: None,
            close_reason: None,
        };

        let incoming = Issue {
            id: "test-1".to_string(),
            title: "New Title".to_string(),
            description: "New Desc".to_string(),
            status: "closed".to_string(),
            priority: 2,
            issue_type: "bug".to_string(),
            owner: "you".to_string(),
            created_at: "2026-01-01T10:00:00Z".to_string(),
            created_by: "Me".to_string(),
            updated_at: "2026-01-01T11:00:00Z".to_string(), // Newer
            closed_at: Some("2026-01-01T11:00:00Z".to_string()),
            close_reason: Some("fixed".to_string()),
        };

        base.merge(incoming);

        assert_eq!(base.title, "New Title");
        assert_eq!(base.status, "closed");
        assert_eq!(base.updated_at, "2026-01-01T11:00:00Z");
        assert_eq!(base.closed_at, Some("2026-01-01T11:00:00Z".to_string()));
    }

    #[test]
    fn test_issue_merge_older_ignored() {
        let mut base = Issue {
            id: "test-1".to_string(),
            title: "Newer Title".to_string(),
            description: "Newer Desc".to_string(),
            status: "open".to_string(),
            priority: 1,
            issue_type: "task".to_string(),
            owner: "me".to_string(),
            created_at: "2026-01-01T10:00:00Z".to_string(),
            created_by: "Me".to_string(),
            updated_at: "2026-01-01T12:00:00Z".to_string(), // Newer
            closed_at: None,
            close_reason: None,
        };

        let incoming = Issue {
            id: "test-1".to_string(),
            title: "Older Title".to_string(),
            description: "Older Desc".to_string(),
            status: "closed".to_string(),
            priority: 2,
            issue_type: "bug".to_string(),
            owner: "you".to_string(),
            created_at: "2026-01-01T10:00:00Z".to_string(),
            created_by: "Me".to_string(),
            updated_at: "2026-01-01T11:00:00Z".to_string(), // Older
            closed_at: Some("2026-01-01T11:00:00Z".to_string()),
            close_reason: Some("fixed".to_string()),
        };

        base.merge(incoming);

        assert_eq!(base.title, "Newer Title");
        assert_eq!(base.status, "open");
        assert_eq!(base.updated_at, "2026-01-01T12:00:00Z");
    }

    #[test]
    fn test_read_issues_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();
        let store = JsonlStore::new(path);

        let issues = store.read_issues().expect("Failed to read issues");
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_read_issues_corrupted_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"id": "1", ... "#).unwrap();
        let path = file.path().to_str().unwrap();
        let store = JsonlStore::new(path);

        let result = store.read_issues();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_issues_mixed_valid_and_invalid() {
        let mut file = NamedTempFile::new().unwrap();
        let valid_issue = Issue {
            id: "1".to_string(),
            title: "Valid".to_string(),
            description: String::new(),
            status: "open".to_string(),
            priority: 1,
            issue_type: "task".to_string(),
            owner: "me".to_string(),
            created_at: "2023-01-01".to_string(),
            created_by: String::new(),
            updated_at: "2023-01-01".to_string(),
            closed_at: None,
            close_reason: None,
        };
        let valid_json = serde_json::to_string(&valid_issue).unwrap();
        writeln!(file, "{}", valid_json).unwrap();
        writeln!(file, "invalid json").unwrap();

        let path = file.path().to_str().unwrap();
        let store = JsonlStore::new(path);

        let result = store.read_issues();
        assert!(result.is_err());
    }

    #[test]
    fn test_read_issues_nonexistent_file() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap().to_string();
        file.close().unwrap(); // Delete the file

        let store = JsonlStore::new(&path);
        let issues = store.read_issues().expect("Failed to read issues");

        assert_eq!(issues.len(), 0);
    }
}
