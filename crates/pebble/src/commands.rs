use crate::config::{Config, find_project_root, parse_config};
use crate::graph::TaskGraph;
use crate::models::{NotFoundError, Priority, TaskNode, TaskStatus, UsageError};
use color_eyre::eyre::{Result, eyre};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

mod listing;
pub use listing::{ListOptions, run_list, run_search};

/// Reads standard input if the provided argument requests it.
///
/// If `value` is `Some("-")`, this function reads the entire contents of standard
/// input into a string and returns it. Otherwise, it returns `value` unchanged.
///
/// # Arguments
///
/// * `value` - An optional string argument. If it equals `"-"`, the function reads from standard input.
///
/// # Errors
///
/// Returns an error if reading from standard input fails.
pub fn read_stdin_if_dash(value: Option<String>) -> Result<Option<String>> {
    match value {
        Some(s) if s == "-" => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(Some(buffer))
        }
        _ => Ok(value),
    }
}

/// Serialized version of a TaskObject as expected by the JSON API contract.
#[derive(Serialize)]
pub struct TaskObject<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub status: TaskStatus,
    pub priority: Option<Priority>,
    pub created_at: String,
    pub modified_at: Option<String>,
    pub resolved_at: Option<String>,
    pub needs: &'a Vec<String>,
    pub tags: &'a Vec<String>,
    pub is_ready: bool,
    pub blocked_by: Vec<String>,
    pub blocking: Vec<String>,
    pub body: &'a str,
    pub path: String,
}

impl<'a> TaskObject<'a> {
    /// Builds a serializable task view from a graph node and tasks directory.
    ///
    /// # Arguments
    ///
    /// * `node` - The [`TaskNode`] representing the task to serialize.
    /// * `graph` - The [`TaskGraph`] used to compute dependency relationships.
    /// * `tasks_dir` - The directory path where tasks are stored.
    pub fn from_node(node: &'a TaskNode, graph: &TaskGraph, tasks_dir: &Path) -> Self {
        let is_ready = graph.is_ready(&node.frontmatter.id);

        let blocked_by: Vec<String> = node
            .frontmatter
            .needs
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
            .blocking
            .get(&node.frontmatter.id)
            .into_iter()
            .flat_map(|ids| ids.iter())
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
            id: &node.frontmatter.id,
            title: &node.frontmatter.title,
            status: node.frontmatter.status,
            priority: node.frontmatter.priority,
            created_at: node
                .frontmatter
                .created_at
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default(),
            modified_at: node
                .frontmatter
                .modified_at
                .as_ref()
                .map(|dt| dt.to_rfc3339()),
            resolved_at: node
                .frontmatter
                .resolved_at
                .as_ref()
                .map(|dt| dt.to_rfc3339()),
            needs: &node.frontmatter.needs,
            tags: &node.frontmatter.tags,
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
    pub current_dir: PathBuf,
    pub project_root: Option<PathBuf>,
    pub config: Config,
    pub tasks_dir: PathBuf,
    pub json: bool,
}

impl RunContext {
    /// Loads configuration and resolves paths based on CLI overrides and the current directory.
    ///
    /// # Arguments
    ///
    /// * `current_dir` - The directory from which the command is executed.
    /// * `cli_dir_override` - An optional directory path provided via CLI override.
    /// * `cli_config_override` - An optional path to a configuration file provided via CLI override.
    /// * `json` - A boolean indicating whether to output in JSON format.
    ///
    /// # Errors
    ///
    /// Returns an error if reading or parsing the configuration file fails, or if
    /// the resolved tasks directory validation fails.
    pub fn load(
        current_dir: PathBuf,
        cli_dir_override: Option<PathBuf>,
        cli_config_override: Option<PathBuf>,
        json: bool,
    ) -> Result<Self> {
        let project_root = find_project_root(&current_dir);

        let config_path = cli_config_override.unwrap_or_else(|| {
            project_root
                .as_ref()
                .map(|r| r.join(".pebble").join("config.toml"))
                .unwrap_or_else(|| PathBuf::from(".pebble/config.toml"))
        });

        let config = if config_path.exists() {
            let toml_str = fs::read_to_string(&config_path)?;
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
            current_dir,
            project_root,
            config,
            tasks_dir,
            json,
        })
    }

    /// Ensure that we are in a valid project context (root exists or tasks dir exists).
    pub fn ensure_project(&self) -> Result<()> {
        if self.project_root.is_some() || self.tasks_dir.exists() {
            Ok(())
        } else {
            Err(eyre!(
                "No pebble project found. Run 'pebble init' to create one, or use '--dir' to specify a tasks directory."
            ))
        }
    }
}

/// Emit the highest-scoring ready tasks according to the default ranking.
pub fn run_next(ctx: &RunContext, limit: usize) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let next_tasks = graph.get_next_tasks();
    let tasks: Vec<&TaskNode> = next_tasks.into_iter().take(limit).collect();

    if ctx.json {
        let objects: Vec<TaskObject> = tasks
            .into_iter()
            .map(|n| TaskObject::from_node(n, &graph, &ctx.tasks_dir))
            .collect();
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({ "tasks": objects }))?
        );
        return Ok(());
    }

    if tasks.is_empty() {
        return Err(NotFoundError("No ready tasks found.".to_string()).into());
    }

    for task in tasks {
        println!("{} {}", task.frontmatter.id, task.frontmatter.title);
    }
    Ok(())
}

/// Show a task by ID, or just its path when `path_only` is set.
pub fn run_show(ctx: &RunContext, id: &str, path_only: bool) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let node = graph
        .nodes
        .get(id)
        .ok_or_else(|| NotFoundError(format!("Task '{}' not found", id)))?;

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
        println!("Task: {} ({})", obj.title, obj.id);
        println!("Status: {}", obj.status);
        println!("Path: {}", obj.path);
        if !obj.tags.is_empty() {
            println!("Tags: {:?}", obj.tags);
        }
        println!("Blocked by: {:?}", obj.blocked_by);
        println!("Blocking: {:?}", obj.blocking);
        println!("\n{}", obj.body.trim());
    }
    Ok(())
}

/// Read a resolved configuration value by key.
pub fn run_config_get(ctx: &RunContext, key: &str) -> Result<()> {
    let config_values = config_values_map(&ctx.config)?;
    let value = config_values.get(key).cloned().ok_or_else(|| {
        let supported_keys = config_values.keys().cloned().collect::<Vec<_>>().join(", ");
        UsageError(format!(
            "Unknown config key '{}'. Supported keys: {}",
            key, supported_keys
        ))
    })?;

    if ctx.json {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({ "key": key, "value": value }))?
        );
    } else {
        println!("{value}");
    }

    Ok(())
}

/// Validates and deduplicates a list of task IDs against the graph.
///
/// Ensures that:
/// 1. Referenced tasks exist in the graph (except optionally `self_id`).
/// 2. Referenced tasks are not marked as duplicates (ambiguous).
/// 3. The returned list contains unique IDs.
///
/// # Arguments
///
/// * `graph` - The task graph to validate against.
/// * `targets` - A list of task IDs to validate.
/// * `self_id` - An optional task ID to exclude from existence checks, used when a task refers to itself.
/// * `flag_name` - The name of the CLI flag that provided the targets, used for error message context.
///
/// # Errors
///
/// Returns an error if a target ID is marked as duplicate in the graph or if it is not found.
pub fn validate_task_references(
    graph: &TaskGraph,
    targets: Vec<String>,
    self_id: Option<&str>,
    flag_name: &str,
) -> Result<Vec<String>> {
    let mut deduped = Vec::new();
    let mut seen = HashSet::new();

    for target_id in targets {
        if !seen.insert(target_id.clone()) {
            continue;
        }

        if let Some(sid) = self_id
            && target_id == sid
        {
            deduped.push(target_id);
            continue;
        }

        if graph.is_duplicate_id(&target_id) {
            return Err(eyre!(
                "Duplicate task ID '{}' found in multiple files; cannot safely target this ID.",
                target_id
            ));
        }

        if !graph.nodes.contains_key(&target_id) {
            return Err(
                UsageError(format!("Task '{}' not found for {}", target_id, flag_name)).into(),
            );
        }

        deduped.push(target_id);
    }

    Ok(deduped)
}

fn config_values_map(config: &Config) -> Result<BTreeMap<String, String>> {
    let serialized = serde_json::to_value(config)?;
    let object = serialized
        .as_object()
        .ok_or_else(|| eyre!("Internal error: serialized config should be a JSON object"))?;

    Ok(object
        .iter()
        .map(|(key, value)| {
            let rendered = value
                .as_str()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| value.to_string());
            (key.clone(), rendered)
        })
        .collect())
}

#[cfg(test)]
mod tests;
