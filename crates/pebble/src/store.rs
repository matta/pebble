use color_eyre::Result;
use color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};

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
///     dependencies: vec![],
///     extra: std::collections::HashMap::new(),
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
    #[serde(default)]
    pub dependencies: Vec<serde_json::Value>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// A persistent store for managing issues in a JSON Lines (JSONL) file.
///
/// This struct handles reading and writing [`Issue`] records to a file at a specified path.
/// Each line in the file corresponds to a single JSON object representing an issue.
pub struct JsonlStore {
    path: std::path::PathBuf,
}

impl JsonlStore {
    pub fn new<P: Into<std::path::PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
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
    /// let store = JsonlStore::new(file.path());
    /// let issues = store.read_issues()?;
    ///
    /// assert_eq!(issues.len(), 1);
    /// assert_eq!(issues[0].title, "Test");
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_issues(&self) -> Result<Vec<Issue>> {
        self.read_issues_inner()
            .with_context(|| format!("Failed to read issues from {}", self.path.display()))
    }

    fn read_issues_inner(&self) -> Result<Vec<Issue>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
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
            .with_context(|| format!("Failed to write issues to {}", self.path.display()))
    }

    fn write_issues_inner(&self, issues: &[Issue]) -> Result<()> {
        let file = File::create(&self.path)?;
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
    /// let store = JsonlStore::new(file.path());
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
    ///     dependencies: vec![],
    ///     extra: Default::default(),
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
            .with_context(|| format!("Failed to append issue to {}", self.path.display()))
    }

    fn append_issue_inner(&self, issue: &Issue) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&self.path)?;

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

        let store = JsonlStore::new(file.path());

        let issues = store.read_issues().expect("Failed to read issues");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].id, "mydoo-0kq");
        assert_eq!(issues[0].title, "Test Issue");
    }

    #[test]
    fn test_write_issues() {
        let file = NamedTempFile::new().unwrap();
        let store = JsonlStore::new(file.path());

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
            dependencies: vec![],
            extra: Default::default(),
        }];

        store.write_issues(&issues).expect("Failed to write issues");

        let read_back = store.read_issues().expect("Failed to read back issues");
        assert_eq!(read_back, issues);
    }

    #[test]
    fn test_append_issue() {
        let file = NamedTempFile::new().unwrap();
        let store = JsonlStore::new(file.path());

        let issue1 = Issue {
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
            dependencies: vec![],
            extra: Default::default(),
        };

        let issue2 = Issue {
            id: "test-2".to_string(),
            title: "Title 2".to_string(),
            description: "Desc 2".to_string(),
            status: "open".to_string(),
            priority: 2,
            issue_type: "task".to_string(),
            owner: "me".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            created_by: "Me".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            closed_at: None,
            close_reason: None,
            dependencies: vec![],
            extra: Default::default(),
        };

        store
            .append_issue(&issue1)
            .expect("Failed to append issue 1");
        store
            .append_issue(&issue2)
            .expect("Failed to append issue 2");

        let issues = store.read_issues().expect("Failed to read issues");
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0], issue1);
        assert_eq!(issues[1], issue2);
    }

    #[test]
    fn test_append_issue_newline_handling() {
        let file = NamedTempFile::new().unwrap();
        let store = JsonlStore::new(file.path());

        let issue = Issue {
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
            dependencies: vec![],
            extra: Default::default(),
        };

        // Case 1: File exists but doesn't end with newline
        {
            let mut f = std::fs::File::create(file.path()).unwrap();
            write!(f, r#"{"id":"0","title":"Existing"}"#).unwrap();
            f.flush().unwrap();
            // No trailing newline
        }

        store.append_issue(&issue).expect("Failed to append");

        let content = std::fs::read_to_string(file.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(
            lines.len(),
            2,
            "Should have exactly two lines when missing newline was fixed"
        );
        assert_eq!(lines[0], r#"{"id":"0","title":"Existing"}"#);
        assert!(lines[1].contains(r#""id":"test-1""#));
        assert!(content.ends_with('\n'));

        // Case 2: File already ends with newline
        {
            let mut f = std::fs::File::create(file.path()).unwrap();
            writeln!(f, r#"{"id":"0","title":"Existing"}"#).unwrap();
            f.flush().unwrap();
        }

        store.append_issue(&issue).expect("Failed to append");

        let content = std::fs::read_to_string(file.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2, "Should NOT have added an extra blank line");
        assert_eq!(lines[0], r#"{"id":"0","title":"Existing"}"#);
        assert!(lines[1].contains(r#""id":"test-1""#));
        assert!(content.ends_with('\n'));
        assert!(
            !content.contains("\n\n"),
            "Should not contain double newlines"
        );
    }
}
