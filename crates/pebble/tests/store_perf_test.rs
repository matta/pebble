use pebble::store::{Issue, JsonlStore};
use tempfile::NamedTempFile;

#[test]
fn test_find_issue_performance_correctness() -> color_eyre::Result<()> {
    let file = NamedTempFile::new()?;
    let store = JsonlStore::new(file.path().to_str().unwrap());

    // Create 1000 issues
    let mut issues = Vec::new();
    for i in 0..1000 {
        issues.push(Issue {
            id: format!("ISSUE-{}", i),
            title: format!("Title {}", i),
            description: "Some description".to_string(),
            status: "open".to_string(),
            priority: 1,
            issue_type: "task".to_string(),
            owner: "me".to_string(),
            created_at: "2023-01-01".to_string(),
            created_by: "me".to_string(),
            updated_at: "2023-01-01".to_string(),
            closed_at: None,
            close_reason: None,
        });
    }

    store.write_issues(&issues)?;

    // Test find_issue (optimized)
    let found = store.find_issue("ISSUE-999")?;
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, "ISSUE-999");

    let not_found = store.find_issue("ISSUE-9999")?;
    assert!(not_found.is_none());

    // Test issue_exists (optimized)
    assert!(store.issue_exists("ISSUE-500")?);
    assert!(!store.issue_exists("ISSUE-5000")?);

    Ok(())
}
