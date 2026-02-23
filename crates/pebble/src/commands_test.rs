use crate::commands::RunContext;
use crate::commands_write::{run_add, run_update};
use crate::config::Config;
use crate::graph::TaskGraph;
use crate::models::TaskStatus;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
#[allow(clippy::cognitive_complexity)]
fn test_init_and_add() {
    let dir = tempdir().unwrap();
    let current_dir = dir.path();

    let ctx = RunContext {
        project_root: Some(current_dir.to_path_buf()),
        config: Config {
            issue_prefix: "TEST".to_string(),
            tasks_dir: PathBuf::from("docs/pebble"),
        },
        tasks_dir: current_dir.join("docs/pebble"),
        json: false,
    };

    run_add(
        &ctx,
        "My First Task".to_string(),
        None,
        Some(5),
        Some("Body text".to_string()),
        vec![],
        vec!["urgent".to_string()],
    )
    .unwrap();

    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir).unwrap();
    assert_eq!(graph.nodes.len(), 1);
    let node = graph.nodes.values().next().unwrap();

    assert_eq!(node.frontmatter.title, "My First Task");
    assert_eq!(node.frontmatter.status, TaskStatus::Todo);
    assert_eq!(node.frontmatter.priority, Some(5));
    assert_eq!(node.frontmatter.tags, vec!["urgent".to_string()]);
    assert_eq!(node.body, "Body text");

    let id = node.frontmatter.id.clone();

    run_update(
        &ctx,
        id.clone(),
        Some("Updated Title".to_string()),
        Some("in_progress".to_string()),
        None,
        true, // clear_priority
        None,
        Some("Appended body".to_string()),
        vec!["new_tag".to_string()],
        vec!["urgent".to_string()],
        vec![],
        vec![],
    )
    .unwrap();

    let graph2 = TaskGraph::load_from_dir(&ctx.tasks_dir).unwrap();
    let updated_node = graph2.nodes.get(&id).unwrap();

    assert_eq!(updated_node.frontmatter.title, "Updated Title");
    assert_eq!(updated_node.frontmatter.status, TaskStatus::InProgress);
    assert_eq!(updated_node.frontmatter.priority, None);
    assert_eq!(updated_node.frontmatter.tags, vec!["new_tag".to_string()]);
    assert_eq!(updated_node.body, "Body text\n\nAppended body");
}

#[test]
fn test_add_slug_filename() {
    let dir = tempdir().unwrap();
    let current_dir = dir.path();
    let tasks_dir = current_dir.join("docs/pebble");

    let ctx = RunContext {
        project_root: Some(current_dir.to_path_buf()),
        config: Config {
            issue_prefix: "TEST".to_string(),
            tasks_dir: PathBuf::from("docs/pebble"),
        },
        tasks_dir: tasks_dir.clone(),
        json: false,
    };

    // 1. Basic slugification
    run_add(
        &ctx,
        "Implement Task Node".to_string(),
        None,
        None,
        None,
        vec![],
        vec![],
    )
    .unwrap();
    assert!(tasks_dir.join("implement-task-node.md").exists());

    // 2. Collision handling
    run_add(
        &ctx,
        "Implement Task Node".to_string(),
        None,
        None,
        None,
        vec![],
        vec![],
    )
    .unwrap();
    assert!(tasks_dir.join("implement-task-node-2.md").exists());

    // 3. Punctuation and spaces
    run_add(
        &ctx,
        "Fix: Bug #123! (now)".to_string(),
        None,
        None,
        None,
        vec![],
        vec![],
    )
    .unwrap();
    assert!(tasks_dir.join("fix-bug-123-now.md").exists());

    // 4. Empty slug fallback
    run_add(&ctx, "!!!".to_string(), None, None, None, vec![], vec![]).unwrap();
    assert!(tasks_dir.join("task.md").exists());

    // 5. Long title is truncated
    let long_title = "a ".repeat(100); // 200 chars of "a " -> slug would be "a-a-a-..."
    run_add(&ctx, long_title, None, None, None, vec![], vec![]).unwrap();
    let entries: Vec<_> = std::fs::read_dir(&tasks_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("a-a-a")
        })
        .collect();
    assert_eq!(entries.len(), 1);
    let stem = entries[0]
        .path()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    assert!(stem.len() <= 80, "slug was {} chars", stem.len());
}

#[test]
fn test_filtering_and_search() {
    let dir = tempdir().unwrap();
    let current_dir = dir.path();
    let tasks_dir = current_dir.join("docs/pebble");

    let ctx = RunContext {
        project_root: Some(current_dir.to_path_buf()),
        config: Config {
            issue_prefix: "TEST".to_string(),
            tasks_dir: PathBuf::from("docs/pebble"),
        },
        tasks_dir: tasks_dir.clone(),
        json: false,
    };

    // Create tasks
    // 1. Todo, High Priority, Tag: "urgent"
    run_add(
        &ctx,
        "Task 1".to_string(),
        Some("todo".to_string()),
        Some(90),
        Some("body1".to_string()),
        vec![],
        vec!["urgent".to_string()],
    )
    .unwrap();
    // 2. InProgress, Low Priority
    run_add(
        &ctx,
        "Task 2".to_string(),
        Some("in_progress".to_string()),
        Some(10),
        Some("body2 search_me".to_string()),
        vec![],
        vec![],
    )
    .unwrap();
    // 3. Done
    run_add(
        &ctx,
        "Task 3".to_string(),
        Some("done".to_string()),
        None,
        None,
        vec![],
        vec![],
    )
    .unwrap();

    let graph = TaskGraph::load_from_dir(&tasks_dir).unwrap();

    // Test Status Filter
    let filtered =
        crate::commands::filter_tasks(&graph, false, &["todo".to_string()], &[], &[], &[]);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].frontmatter.title, "Task 1");

    // Test Priority Filter
    let filtered = crate::commands::filter_tasks(&graph, false, &[], &[90], &[], &[]);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].frontmatter.title, "Task 1");

    // Test Tag Filter
    let filtered =
        crate::commands::filter_tasks(&graph, false, &[], &[], &["urgent".to_string()], &[]);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].frontmatter.title, "Task 1");

    // Test Search
    let searched = crate::commands::search_tasks(&graph, "search_me");
    assert_eq!(searched.len(), 1);
    assert_eq!(searched[0].frontmatter.title, "Task 2");

    // Test Default List (omit done)
    let filtered = crate::commands::filter_tasks(&graph, false, &[], &[], &[], &[]);
    assert_eq!(filtered.len(), 2); // Task 1 and Task 2

    // Verify Task 3 (done) is filtered out by default
    assert!(!filtered.iter().any(|t| t.frontmatter.title == "Task 3"));
}
