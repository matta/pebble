use crate::config::{Config, find_project_root, parse_config};
use crate::graph::TaskGraph;
use crate::models::{TaskNode, TaskObject, TaskStatus};
use color_eyre::eyre::{Result, eyre};
use std::env;
use std::path::PathBuf;

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

/// Filter tasks based on provided criteria.
/// Public for testing.
#[allow(clippy::too_many_arguments)]
pub fn filter_tasks<'a>(
    graph: &'a TaskGraph,
    is_ready: bool,
    status: &[String],
    priority: &[u8],
    tags: &[String],
    deps: &[String],
) -> Vec<&'a TaskNode> {
    let mut tasks: Vec<&TaskNode> = graph.nodes.values().collect();

    // 1. Status Filter
    if !status.is_empty() {
        // Parse status strings into TaskStatus
        // We do loose matching (case insensitive)
        let target_statuses: Vec<TaskStatus> = status
            .iter()
            .filter_map(|s| serde_json::from_str(&format!("\"{}\"", s)).ok())
            .collect();

        if !target_statuses.is_empty() {
            tasks.retain(|n| target_statuses.contains(&n.frontmatter.status));
        } else {
            // If statuses were provided but none parsed validly, strictly we should probably match nothing?
            // Or maybe the user meant custom status? Currently TaskStatus is an enum.
            // If they pass garbage, filter everything out.
            tasks.retain(|_| false);
        }
    } else {
        // Default behavior: omit done/canceled if no status filter provided
        // EXCEPT if is_ready is true, which implies actionable tasks anyway.
        // But the original code said: "Default: omit done/canceled".
        tasks.retain(|n| {
            !matches!(
                n.frontmatter.status,
                TaskStatus::Done | TaskStatus::Canceled
            )
        });
    }

    // 2. Priority Filter (OR)
    if !priority.is_empty() {
        tasks.retain(|n| {
            if let Some(p) = n.frontmatter.priority {
                priority.contains(&p)
            } else {
                false
            }
        });
    }

    // 3. Tags Filter (AND)
    if !tags.is_empty() {
        tasks.retain(|n| tags.iter().all(|tag| n.frontmatter.tags.contains(tag)));
    }

    // 4. Deps Filter (OR) - Task must depend on ANY of these
    if !deps.is_empty() {
        tasks.retain(|n| {
            deps.iter()
                .any(|dep_id| n.frontmatter.deps.contains(dep_id))
        });
    }

    // 5. Readiness Filter
    if is_ready {
        tasks.retain(|n| graph.is_ready(&n.frontmatter.id));
    }

    graph.default_order(tasks)
}

/// List tasks using the default ordering, optionally filtering to ready tasks.
#[allow(clippy::too_many_arguments)]
pub fn run_list(
    ctx: &RunContext,
    is_ready: bool,
    status: Vec<String>,
    priority: Vec<u8>,
    tags: Vec<String>,
    deps: Vec<String>,
) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;

    let tasks = filter_tasks(&graph, is_ready, &status, &priority, &tags, &deps);

    if ctx.json {
        let objects: Vec<TaskObject> = tasks
            .into_iter()
            .map(|n| TaskObject::from_node(n, &graph, &ctx.tasks_dir))
            .collect();
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({ "tasks": objects }))?
        );
    } else if tasks.is_empty() {
        eprintln!("No tasks found.");
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

pub fn search_tasks<'a>(graph: &'a TaskGraph, query: &str) -> Vec<&'a TaskNode> {
    let query_lower = query.to_lowercase();

    let tasks: Vec<&TaskNode> = graph
        .nodes
        .values()
        .filter(|n| {
            n.frontmatter.title.to_lowercase().contains(&query_lower)
                || n.body.to_lowercase().contains(&query_lower)
        })
        .collect();

    graph.default_order(tasks)
}

pub fn run_search(ctx: &RunContext, query: &str) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let tasks = search_tasks(&graph, query);

    if ctx.json {
        let objects: Vec<TaskObject> = tasks
            .into_iter()
            .map(|n| TaskObject::from_node(n, &graph, &ctx.tasks_dir))
            .collect();
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({ "tasks": objects }))?
        );
    } else if tasks.is_empty() {
        eprintln!("No tasks found matching '{}'.", query);
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

pub fn run_config_get(ctx: &RunContext, key: &str) -> Result<()> {
    let value = match key {
        "issue-prefix" => serde_json::json!(ctx.config.issue_prefix),
        "tasks-dir" => serde_json::json!(ctx.config.tasks_dir),
        _ => return Err(eyre!("Unknown config key: {}", key)),
    };

    if ctx.json {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({ key: value }))?
        );
    } else {
        // For simple string values, just print the string? Or the JSON value representation?
        // "Get" usually implies raw value.
        if let Some(s) = value.as_str() {
            println!("{}", s);
        } else {
            println!("{}", value);
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
    } else if ctx.json {
        println!("null");
    } else {
        eprintln!("No ready tasks found.");
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
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::str::FromStr;

    use crate::models::TaskFrontmatter;

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
