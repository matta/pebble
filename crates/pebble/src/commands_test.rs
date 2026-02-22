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
