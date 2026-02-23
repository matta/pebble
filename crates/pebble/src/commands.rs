use crate::config::{Config, find_project_root, parse_config};
use crate::graph::TaskGraph;
use crate::models::{TaskFrontmatter, TaskNode};
use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use std::env;
use std::path::PathBuf;

/// Serialized version of a TaskObject as expected by the JSON API contract.
#[derive(Serialize)]
pub struct TaskObject<'a> {
    #[serde(flatten)]
    pub frontmatter: &'a TaskFrontmatter,
    pub is_ready: bool,
    pub blocked_by: Vec<String>,
    pub blocking: Vec<String>,
    pub body: &'a str,
    pub path: String,
}

impl<'a> TaskObject<'a> {
    /// Build a serializable task view from a graph node and tasks directory.
    pub fn from_node(node: &'a TaskNode, graph: &TaskGraph, tasks_dir: &std::path::Path) -> Self {
        let is_ready = graph.is_ready(&node.frontmatter.id);

        let blocked_by: Vec<String> = node
            .frontmatter
            .deps
            .iter()
            .filter_map(|dep_id| {
                if let Some(dep_node) = graph.nodes.get(dep_id) {
                    if !dep_node.frontmatter.status.is_closed() {
                        return Some(dep_id.clone());
                    }
                } else {
                    return Some(dep_id.clone()); // Dangling pointers block
                }
                None
            })
            .collect();

        let blocking: Vec<String> = graph
            .reverse_deps
            .get(&node.frontmatter.id)
            .into_iter()
            .flat_map(|deps| deps.iter())
            .filter(|dep_id| {
                graph
                    .nodes
                    .get(*dep_id)
                    .is_some_and(|dep_node| dep_node.frontmatter.status.is_actionable())
            })
            .cloned()
            .collect();

        let rel_path = node.path.strip_prefix(tasks_dir).unwrap_or(&node.path);

        TaskObject {
            frontmatter: &node.frontmatter,
            is_ready,
            blocked_by,
            blocking,
            body: &node.body,
            path: rel_path.display().to_string(),
        }
    }
}

/// Resolved runtime configuration and paths for command execution.
pub struct RunContext {
    pub project_root: Option<PathBuf>,
    pub config: Config,
    pub tasks_dir: PathBuf,
    pub json: bool,
}

impl RunContext {
    /// Load configuration and resolve paths based on CLI overrides and the current directory.
    pub fn load(
        cli_dir_override: Option<PathBuf>,
        cli_config_override: Option<PathBuf>,
        json: bool,
    ) -> Result<Self> {
        let current_dir = env::current_dir()?;
        let project_root = find_project_root(&current_dir);

        let config_path = cli_config_override.unwrap_or_else(|| {
            project_root
                .as_ref()
                .map(|r| r.join(".pebble").join("config.toml"))
                .unwrap_or_else(|| PathBuf::from(".pebble/config.toml"))
        });

        let config = if config_path.exists() {
            let toml_str = std::fs::read_to_string(&config_path)?;
            parse_config(&toml_str)?
        } else {
            Config::default()
        };

        // Resolve tasks_dir
        let tasks_dir = if let Some(dir_override) = cli_dir_override {
            if dir_override.is_absolute() {
                dir_override
            } else {
                current_dir.join(dir_override)
            }
        } else if let Some(root) = &project_root {
            root.join(&config.tasks_dir)
        } else {
            current_dir.join(&config.tasks_dir)
        };

        Ok(Self {
            project_root,
            config,
            tasks_dir,
            json,
        })
    }
}

/// List tasks using the default ordering, optionally filtering to ready tasks.
pub fn run_list(ctx: &RunContext, is_ready: bool) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;

    // Default: omit done/canceled
    let mut tasks: Vec<&TaskNode> = graph
        .nodes
        .values()
        .filter(|n| !n.frontmatter.status.is_closed())
        .collect();

    if is_ready {
        tasks.retain(|n| graph.is_ready(&n.frontmatter.id));
    }

    let tasks = graph.default_order(tasks);

    if ctx.json {
        let objects: Vec<TaskObject> = tasks
            .into_iter()
            .map(|n| TaskObject::from_node(n, &graph, &ctx.tasks_dir))
            .collect();
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({ "tasks": objects }))?
        );
    } else {
        for task in tasks {
            println!(
                "{} {} ({:?})",
                task.frontmatter.id, task.frontmatter.title, task.frontmatter.status
            );
        }
    }

    Ok(())
}

/// Emit the highest-scoring ready task according to the default ranking.
pub fn run_next(ctx: &RunContext) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let next_tasks = graph.get_next_tasks();

    if let Some(task) = next_tasks.first() {
        if ctx.json {
            let obj = TaskObject::from_node(task, &graph, &ctx.tasks_dir);
            println!("{}", serde_json::to_string(&obj)?);
        } else {
            println!("{} {}", task.frontmatter.id, task.frontmatter.title);
        }
    } else {
        return Err(eyre!("No ready tasks found."));
    }
    Ok(())
}

/// Show a task by ID, or just its path when `path_only` is set.
pub fn run_show(ctx: &RunContext, id: &str, path_only: bool) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let node = graph
        .nodes
        .get(id)
        .ok_or_else(|| eyre!("Task '{}' not found", id))?;

    if path_only {
        let rel_path = node.path.strip_prefix(&ctx.tasks_dir).unwrap_or(&node.path);
        if ctx.json {
            println!(
                "{}",
                serde_json::to_string(
                    &serde_json::json!({ "path": rel_path.display().to_string() })
                )?
            );
        } else {
            println!("{}", rel_path.display());
        }
    } else if ctx.json {
        let obj = TaskObject::from_node(node, &graph, &ctx.tasks_dir);
        println!("{}", serde_json::to_string(&obj)?);
    } else {
        let obj = TaskObject::from_node(node, &graph, &ctx.tasks_dir);
        println!("Task: {} ({})", obj.frontmatter.title, obj.frontmatter.id);
        println!("Status: {:?}", obj.frontmatter.status);
        println!("Path: {}", obj.path);
        if !obj.frontmatter.tags.is_empty() {
            println!("Tags: {:?}", obj.frontmatter.tags);
        }
        println!("Blocked by: {:?}", obj.blocked_by);
        println!("Blocking: {:?}", obj.blocking);
        println!("\n{}", obj.body.trim());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TaskStatus;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::str::FromStr;

    fn make_test_node(id: &str, status: TaskStatus, deps: Vec<&str>) -> TaskNode {
        TaskNode {
            path: PathBuf::from(format!("{id}.md")),
            body: String::new(),
            frontmatter: TaskFrontmatter {
                id: id.to_string(),
                title: id.to_string(),
                status,
                priority: None,
                created_at: toml_datetime::Datetime::from_str("2026-01-01T00:00:00Z").unwrap(),
                modified_at: None,
                resolved_at: None,
                deps: deps.into_iter().map(|s| s.to_string()).collect(),
                tags: vec![],
            },
        }
    }

    #[test]
    fn test_blocking_list_excludes_terminal_dependents() {
        let mut nodes = HashMap::new();
        nodes.insert(
            "A".to_string(),
            make_test_node("A", TaskStatus::Todo, vec![]),
        );
        nodes.insert(
            "B".to_string(),
            make_test_node("B", TaskStatus::Todo, vec!["A"]),
        );
        nodes.insert(
            "C".to_string(),
            make_test_node("C", TaskStatus::Done, vec!["A"]),
        );

        let graph = TaskGraph::new(nodes);
        let node = graph.nodes.get("A").unwrap();
        let tasks_dir = PathBuf::from(".");
        let obj = TaskObject::from_node(node, &graph, tasks_dir.as_path());

        let mut blocking = obj.blocking.clone();
        blocking.sort();
        assert_eq!(blocking, vec!["B"]);
    }
}
