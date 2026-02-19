use pebble::store::{Issue, JsonlStore};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_read_issue_ids() {
    let mut file = NamedTempFile::new().unwrap();
    let issue_json = r#"{"id":"id-1","title":"Test Issue","description":"Desc","status":"open","priority":0,"issue_type":"task","owner":"me","created_at":"2026-01-01T00:00:00Z","created_by":"Me","updated_at":"2026-01-01T00:00:00Z","closed_at":null,"close_reason":null}"#;
    writeln!(file, "{}", issue_json).unwrap();
    let issue_json2 = r#"{"id":"id-2","title":"Test Issue 2","description":"Desc","status":"open","priority":0,"issue_type":"task","owner":"me","created_at":"2026-01-01T00:00:00Z","created_by":"Me","updated_at":"2026-01-01T00:00:00Z","closed_at":null,"close_reason":null}"#;
    writeln!(file, "{}", issue_json2).unwrap();

    let path = file.path().to_str().unwrap();
    let store = JsonlStore::new(path);

    let ids = store.read_issue_ids().expect("Failed to read issue IDs");

    assert_eq!(ids.len(), 2);
    assert!(ids.contains("id-1"));
    assert!(ids.contains("id-2"));
}

#[test]
fn test_read_issue_ids_empty_file() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_str().unwrap();
    let store = JsonlStore::new(path);

    let ids = store.read_issue_ids().expect("Failed to read issue IDs");
    assert_eq!(ids.len(), 0);
}

#[test]
fn test_find_issue() {
    let file = NamedTempFile::new().unwrap();
    let store = JsonlStore::new(file.path().to_str().unwrap());

    let issue = Issue {
        id: "find-me".to_string(),
        title: "Find Me".to_string(),
        description: Some("Desc".to_string()),
        status: "open".to_string(),
        priority: 1,
        issue_type: "task".to_string(),
        owner: Some("me".to_string()),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        created_by: Some("Me".to_string()),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        closed_at: None,
        close_reason: None,
        ..Default::default()
    };

    store.append_issue(&issue).expect("Failed to append issue");

    let found = store
        .find_issue("find-me")
        .expect("Failed to find issue")
        .expect("Issue not found");
    assert_eq!(found.title, "Find Me");

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
        description: Some("Desc".to_string()),
        status: "open".to_string(),
        priority: 1,
        issue_type: "task".to_string(),
        owner: Some("me".to_string()),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        created_by: Some("Me".to_string()),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        closed_at: None,
        close_reason: None,
        ..Default::default()
    };

    store.append_issue(&issue).expect("Failed to append issue");

    let exists = store
        .issue_exists("exists")
        .expect("Failed to check existence");
    assert!(exists);

    let exists = store
        .issue_exists("does-not-exist")
        .expect("Failed to check existence");
    assert!(!exists);
}
