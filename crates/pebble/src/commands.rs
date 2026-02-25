use crate::config::{Config, find_project_root, parse_config};
use crate::graph::TaskGraph;
use crate::models::UsageError;
use crate::task_view::TaskObject;
use color_eyre::eyre::{Result, eyre};
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

mod listing;
pub use listing::{ListOptions, run_list, run_search};

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
mod tests {
    use super::*;

    #[test]
    fn test_config_values_map_matches_serialized_config_keys() {
        let config = Config::default();
        let extracted = config_values_map(&config).expect("Should extract config values");
        let serialized = serde_json::to_value(&config).expect("Should serialize config");
        let serialized_keys = serialized
            .as_object()
            .expect("Serialized config should be an object")
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let extracted_keys = extracted
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(extracted_keys, serialized_keys);
    }
}
