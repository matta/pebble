use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: i32,
    pub issue_type: String,
    pub owner: String,
    pub created_at: String,
    pub created_by: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub close_reason: Option<String>,
}

pub struct JsonlStore {
    path: String,
}

impl JsonlStore {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    pub fn read_issues(&self) -> Result<Vec<Issue>> {
        let path = Path::new(&self.path);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file =
            File::open(path).with_context(|| format!("Failed to open file at {}", self.path))?;
        let reader = BufReader::new(file);
        let mut issues = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let issue: Issue = serde_json::from_str(&line)
                .with_context(|| format!("Failed to parse issue from line: {}", line))?;
            issues.push(issue);
        }

        Ok(issues)
    }

    pub fn write_issues(&self, issues: &[Issue]) -> Result<()> {
        let file = File::create(&self.path)
            .with_context(|| format!("Failed to create file at {}", self.path))?;
        let mut writer = std::io::BufWriter::new(file);

        for issue in issues {
            let json = serde_json::to_string(issue)
                .with_context(|| format!("Failed to serialize issue: {:?}", issue))?;
            writeln!(writer, "{}", json)?;
        }

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
}
