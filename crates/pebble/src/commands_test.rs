use crate::commands::RunContext;
use crate::commands_add::{RunAddInput, run_add};
use crate::commands_write::{RunUpdateInput, run_update};
use crate::config::Config;
use crate::graph::TaskGraph;
use crate::models::{Priority, TaskStatus};
use std::path::PathBuf;
use tempfile::tempdir;

fn add_task(ctx: &RunContext, title: &str) {
    run_add(
        ctx,
        RunAddInput {
            title: title.to_string(),
            status: None,
            priority: None,
            body: None,
            needs: vec![],
            tags: vec![],
            blocks: vec![],
        },
    )
    .unwrap();
}

#[test]
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
        RunAddInput {
            title: "My First Task".to_string(),
            status: None,
            priority: Some(Priority::try_from(5).unwrap()),
            body: Some("Body text".to_string()),
            needs: vec![],
            tags: vec!["urgent".to_string()],
            blocks: vec![],
        },
    )
    .unwrap();

    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir).unwrap();
    assert_eq!(graph.nodes.len(), 1);
    let node = graph.nodes.values().next().unwrap();

    assert_eq!(node.frontmatter.title, "My First Task");
    assert_eq!(node.frontmatter.status, TaskStatus::Todo);
    assert_eq!(
        node.frontmatter.priority,
        Some(Priority::try_from(5).unwrap())
    );
    assert_eq!(node.frontmatter.tags, vec!["urgent".to_string()]);
    assert_eq!(node.body, "Body text");

    let id = node.frontmatter.id.clone();

    run_update(
        &ctx,
        RunUpdateInput {
            id: id.clone(),
            title: Some("Updated Title".to_string()),
            status: Some(TaskStatus::InProgress),
            priority: None,
            clear_priority: true,
            body: None,
            append_body: Some("Appended body".to_string()),
            add_tags: vec!["new_tag".to_string()],
            remove_tags: vec!["urgent".to_string()],
            add_needs: vec![],
            remove_needs: vec![],
            blocks: vec![],
            remove_blocks: vec![],
        },
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
    add_task(&ctx, "Implement Task Node");
    assert!(tasks_dir.join("implement-task-node.md").exists());

    // 2. Collision handling
    add_task(&ctx, "Implement Task Node");
    assert!(tasks_dir.join("implement-task-node-2.md").exists());

    // 3. Punctuation and spaces
    add_task(&ctx, "Fix: Bug #123! (now)");
    assert!(tasks_dir.join("fix-bug-123-now.md").exists());

    // 4. Empty slug fallback
    add_task(&ctx, "!!!");
    assert!(tasks_dir.join("task.md").exists());

    // 5. Long title is truncated
    let long_title = "a ".repeat(100); // 200 chars of "a " -> slug would be "a-a-a-..."
    run_add(
        &ctx,
        RunAddInput {
            title: long_title,
            status: None,
            priority: None,
            body: None,
            needs: vec![],
            tags: vec![],
            blocks: vec![],
        },
    )
    .unwrap();
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
