use super::*;
use chrono::{DateTime, Utc};
use std::convert::TryFrom;
use std::mem;

#[test]
fn test_task_status_deserialization() {
    assert_eq!(
        serde_json::from_str::<TaskStatus>("\"todo\"").expect("Should deserialize todo"),
        TaskStatus::todo()
    );
    assert_eq!(
        serde_json::from_str::<TaskStatus>("\"in_progress\"")
            .expect("Should deserialize in_progress"),
        TaskStatus::in_progress()
    );
    assert_eq!(
        serde_json::from_str::<TaskStatus>("\"done\"").expect("Should deserialize done"),
        TaskStatus::done()
    );
    assert_eq!(
        serde_json::from_str::<TaskStatus>("\"canceled\"").expect("Should deserialize canceled"),
        TaskStatus::canceled()
    );

    let err = serde_json::from_str::<TaskStatus>("\"invalid_status\"")
        .expect_err("Should fail to deserialize invalid status");
    assert!(err.to_string().contains("invalid status"));
}

#[test]
fn test_task_status_helpers() {
    assert!(TaskStatus::todo().is_actionable());
    assert!(TaskStatus::in_progress().is_actionable());
    assert!(!TaskStatus::done().is_actionable());
    assert!(!TaskStatus::canceled().is_actionable());

    assert!(TaskStatus::done().is_closed());
    assert!(TaskStatus::canceled().is_closed());
    assert!(!TaskStatus::todo().is_closed());
    assert!(!TaskStatus::in_progress().is_closed());

    assert_eq!(TaskStatus::todo().as_live(), Some(LiveStatus::Todo));
    assert_eq!(
        TaskStatus::in_progress().as_live(),
        Some(LiveStatus::InProgress)
    );
    assert_eq!(TaskStatus::done().as_live(), None);
    assert_eq!(TaskStatus::canceled().as_live(), None);

    assert_eq!(TaskStatus::done().as_closed(), Some(ClosedStatus::Done));
    assert_eq!(
        TaskStatus::canceled().as_closed(),
        Some(ClosedStatus::Canceled)
    );
    assert_eq!(TaskStatus::todo().as_closed(), None);
    assert_eq!(TaskStatus::in_progress().as_closed(), None);
}

#[test]
fn test_task_frontmatter_deserialization() {
    let yaml_str = r#"
id: issue-123
title: Implement Task Node
status: todo
priority: 1
created_at: "2026-02-21T17:00:00Z"
needs: ["issue-122"]
"#;
    let fm: TaskFrontmatter = serde_saphyr::from_str(yaml_str).expect("Valid task frontmatter");
    assert_eq!(fm.id, "issue-123");
    assert_eq!(fm.title, "Implement Task Node");
    assert_eq!(fm.status, TaskStatus::todo());
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
    let yaml_str = r#"
id: issue-123
title: Implement Task Node
status: todo
priority: 100
created_at: "2026-02-21T17:00:00Z"
"#;

    let err = serde_saphyr::from_str::<TaskFrontmatter>(yaml_str)
        .expect_err("Should reject out-of-range priority");
    assert!(
        err.to_string().contains("priority"),
        "Expected priority validation error, got: {err}"
    );
}

#[test]
fn test_priority_json_serializes_as_integer() {
    #[derive(Serialize)]
    struct Wrapper {
        priority: Priority,
    }

    let wrapper = Wrapper {
        priority: Priority::try_from(5).expect("Priority 5 is valid"),
    };
    let json = serde_json::to_value(&wrapper).expect("Should serialize Wrapper");
    assert_eq!(json["priority"].as_u64(), Some(5));
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

#[test]
fn test_task_node_disk_content_uses_yaml_frontmatter() {
    let node = TaskNode {
        path: PathBuf::from("docs/pebble/task.md"),
        frontmatter: TaskFrontmatter {
            id: "issue-123".to_string(),
            title: "YAML writer".to_string(),
            status: TaskStatus::todo(),
            priority: None,
            created_at: Some(
                DateTime::parse_from_rfc3339("2026-03-01T00:00:00Z")
                    .expect("created_at should parse")
                    .with_timezone(&Utc),
            ),
            modified_at: None,
            resolved_at: None,
            needs: vec![],
            tags: vec![],
            extra: HashMap::new(),
        },
        body: "Body\n".to_string(),
    };

    let content = node
        .get_content_for_disk()
        .expect("task content should render");
    assert!(
        content.starts_with("---\n"),
        "frontmatter should start with YAML delimiter"
    );
    assert!(
        content.contains("\n---\n"),
        "frontmatter should include YAML closing delimiter"
    );
    assert!(
        content.contains("\nid:"),
        "frontmatter should use YAML key syntax"
    );
    assert!(
        !content.contains("+++"),
        "frontmatter should not use TOML delimiters"
    );
    assert!(
        !content.contains("id = "),
        "frontmatter should not use TOML assignment syntax"
    );
}
