use pebble::store::{Issue, JsonlStore};
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

#[test]
fn test_append_issue_newline_handling() {
    let file = NamedTempFile::new().unwrap();
    let store = JsonlStore::new(file.path().to_str().unwrap());

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
    };

    // Case 1: File exists but doesn't end with newline
    {
        let mut f = std::fs::File::create(file.path()).unwrap();
        write!(f, r#"{{"id":"0","title":"Existing"}}"#).unwrap();
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
        writeln!(f, r#"{{"id":"0","title":"Existing"}}"#).unwrap();
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

#[test]
fn test_find_issue() {
    let file = NamedTempFile::new().unwrap();
    let store = JsonlStore::new(file.path().to_str().unwrap());

    let issue = Issue {
        id: "find-me".to_string(),
        title: "Find Me".to_string(),
        description: "Desc".to_string(),
        status: "open".to_string(),
        priority: 1,
        issue_type: "task".to_string(),
        owner: "me".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        created_by: "Me".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        closed_at: None,
        close_reason: None,
    };

    store.append_issue(&issue).expect("Failed to append issue");

    // Test finding existing issue
    let found = store
        .find_issue("find-me")
        .expect("Failed to find issue")
        .expect("Issue not found");
    assert_eq!(found.title, "Find Me");

    // Test finding non-existing issue
    let not_found = store
        .find_issue("does-not-exist")
        .expect("Failed to search issue");
    assert!(not_found.is_none());
}

#[test]
fn test_issue_exists() {
    let file = NamedTempFile::new().unwrap();
    let store = JsonlStore::new(file.path().to_str().unwrap());

    let issue = Issue {
        id: "exists".to_string(),
        title: "Exists".to_string(),
        description: "Desc".to_string(),
        status: "open".to_string(),
        priority: 1,
        issue_type: "task".to_string(),
        owner: "me".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        created_by: "Me".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        closed_at: None,
        close_reason: None,
    };

    store.append_issue(&issue).expect("Failed to append issue");

    // Test existing issue
    let exists = store
        .issue_exists("exists")
        .expect("Failed to check existence");
    assert!(exists);

    // Test non-existing issue
    let exists = store
        .issue_exists("does-not-exist")
        .expect("Failed to check existence");
    assert!(!exists);
}
