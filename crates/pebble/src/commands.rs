use crate::config::{Config, find_project_root, parse_config};
use crate::graph::TaskGraph;
use crate::models::{TaskFrontmatter, TaskNode, TaskStatus};
use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use std::env;
use std::path::PathBuf;

/// Serialized version of a TaskObject as expected by the JSON API contract
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
    pub fn from_node(node: &'a TaskNode, graph: &TaskGraph, tasks_dir: &std::path::Path) -> Self {
        let is_ready = graph.is_ready(&node.frontmatter.id);

        let blocked_by: Vec<String> = node
            .frontmatter
            .deps
            .iter()
            .filter_map(|dep_id| {
                if let Some(dep_node) = graph.nodes.get(dep_id) {
                    if !matches!(
                        dep_node.frontmatter.status,
                        TaskStatus::Done | TaskStatus::Canceled
                    ) {
                        return Some(dep_id.clone());
                    }
                } else {
                    return Some(dep_id.clone()); // Dangling pointers block
                }
                None
            })
            .collect();

        let blocking = graph
            .reverse_deps
            .get(&node.frontmatter.id)
            .cloned()
            .unwrap_or_default();

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

pub struct RunContext {
    pub project_root: Option<PathBuf>,
    pub config: Config,
    pub tasks_dir: PathBuf,
    pub json: bool,
}

impl RunContext {
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

pub fn run_list(ctx: &RunContext, is_ready: bool) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;

    // Default: omit done/canceled
    let mut tasks: Vec<&TaskNode> = graph
        .nodes
        .values()
        .filter(|n| {
            !matches!(
                n.frontmatter.status,
                TaskStatus::Done | TaskStatus::Canceled
            )
        })
        .collect();

    if is_ready {
        tasks.retain(|n| graph.is_ready(&n.frontmatter.id));
    }

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
    } else if ctx.json {
        println!("null");
    } else {
        eprintln!("No ready tasks found.");
    }
    Ok(())
}

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
