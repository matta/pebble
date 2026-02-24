//! `pebble` — a CLI task tracker built on Markdown-native, graph-based storage.
//!
//! Tasks are stored as individual Markdown files with TOML frontmatter. The files
//! themselves form a directed dependency graph; no external database is required.
pub mod commands;
pub mod commands_write;

#[cfg(test)]
mod commands_test;
mod config;
pub mod graph;
pub mod models;
pub mod parser;

use clap::{CommandFactory, Parser, Subcommand};
use color_eyre::eyre::Result;
use commands::{ListOptions, RunContext, run_config_get, run_list, run_next, run_search, run_show};
use commands_write::{run_add, run_archive, run_init, run_update};
use models::TaskStatus;
use serde_json::{Map, Value, json};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "pebble",
    version,
    about = "A markdown-native task tracker with graph semantics."
)]
struct Cli {
    /// Change to the given directory before doing anything
    #[arg(short = 'C', long)]
    directory: Option<PathBuf>,

    /// Path to configuration file
    #[arg(short, long, env = "PEBBLE_CONFIG")]
    config: Option<PathBuf>,

    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    /// Path to the tasks directory (overrides config)
    #[arg(long, global = true)]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(alias = "ls")]
    #[command(
        about = "List tasks from the graph.",
        long_about = "List tasks from the markdown task graph. By default, list tasks omits done/canceled. Filters can be combined: status/need/priority are OR, tag is AND. Use --sort to override default dependency-aware ordering.",
        after_help = "Examples:\n  pebble list\n  pebble list --status todo --status in_progress --tag backend --is-ready\n  pebble list --sort -created_at --limit 20"
    )]
    List {
        /// Filter by status (OR). Repeat to include multiple statuses.
        #[arg(long = "status", value_enum)]
        statuses: Vec<TaskStatus>,
        /// Filter by tag (AND). Repeat to require all tags.
        #[arg(long = "tag")]
        tags: Vec<String>,
        /// Filter by dependency ID (OR). Repeat to match any listed need.
        #[arg(long = "need")]
        needs: Vec<String>,
        /// Filter by priority (OR). Repeat to include multiple priorities (0..99).
        #[arg(long = "priority", value_parser = clap::value_parser!(u8).range(0..=99))]
        priorities: Vec<u8>,
        /// Return only ready tasks (actionable with all needs present and closed).
        #[arg(long)]
        is_ready: bool,
        /// Include done/canceled tasks (default omits closed tasks).
        #[arg(long)]
        all: bool,
        /// Limit number of returned rows after filtering/ordering.
        #[arg(long)]
        limit: Option<usize>,
        /// Sort by field. Prefix with '-' for descending.
        #[arg(long, allow_hyphen_values = true)]
        sort: Option<String>,
    },
    #[command(
        about = "Return the next task to work on.",
        long_about = "Return one next task from the ready frontier using Pebble's default ranking.",
        after_help = "Examples:\n  pebble next\n  pebble next --json"
    )]
    Next,
    #[command(
        about = "Search tasks by text.",
        long_about = "Search title and body with case-insensitive substring matching. Results use default list ordering.",
        after_help = "Examples:\n  pebble search bug\n  pebble search \"api timeout\" --json"
    )]
    Search {
        /// Search query (case-insensitive substring over title + body).
        query: String,
    },
    #[command(
        about = "Create a new task.",
        long_about = "Create a task file with generated id/frontmatter and optional metadata.",
        after_help = "Examples:\n  pebble add \"Implement parser\"\n  pebble add \"Fix bug\" --priority 1 --need PROJ-123 --tag backend --json"
    )]
    Add {
        /// Task title.
        title: String,
        /// Initial status.
        #[arg(long, value_enum)]
        status: Option<TaskStatus>,
        /// Priority (0..99, lower is higher priority).
        #[arg(long)]
        priority: Option<u8>,
        /// Initial markdown body content.
        #[arg(long)]
        body: Option<String>,
        /// Add a dependency ID (repeatable).
        #[arg(long = "need")]
        needs: Vec<String>,
        /// Add a tag (repeatable).
        #[arg(long = "tag")]
        tags: Vec<String>,
    },
    #[command(
        about = "Update an existing task.",
        long_about = "Update mutable task fields and body content while preserving immutable id.",
        after_help = "Examples:\n  pebble update PROJ-1 --status in_progress\n  pebble update PROJ-1 --add-tag urgent --remove-need PROJ-0 --append-body \"note\" --json"
    )]
    Update {
        /// Task ID to update.
        id: String,
        /// Replace task title.
        #[arg(long)]
        title: Option<String>,
        /// Set task status.
        #[arg(long, value_enum)]
        status: Option<TaskStatus>,
        /// Set task priority (0..99).
        #[arg(long)]
        priority: Option<u8>,
        /// Clear task priority.
        #[arg(long)]
        clear_priority: bool,
        /// Replace full markdown body.
        #[arg(long)]
        body: Option<String>,
        /// Append markdown content to body.
        #[arg(long)]
        append_body: Option<String>,
        /// Add tag (repeatable).
        #[arg(long = "add-tag")]
        add_tags: Vec<String>,
        /// Remove tag (repeatable).
        #[arg(long = "remove-tag")]
        remove_tags: Vec<String>,
        /// Add dependency id (repeatable).
        #[arg(long = "add-need")]
        add_needs: Vec<String>,
        /// Remove dependency id (repeatable).
        #[arg(long = "remove-need")]
        remove_needs: Vec<String>,
    },
    #[command(
        about = "Archive old closed tasks.",
        long_about = "Move old done/canceled tasks to archive under tasks-dir.",
        after_help = "Examples:\n  pebble archive\n  pebble archive --json"
    )]
    Archive,
    /// Output a specific task in various formats.
    #[command(
        about = "Show one task by ID.",
        long_about = "Show one task's full details, or only its path with --path-only.",
        after_help = "Examples:\n  pebble show PROJ-1\n  pebble show PROJ-1 --path-only --json"
    )]
    Show {
        /// Task ID to show.
        id: String,
        /// Output just the raw filepath instead of the task entity.
        #[arg(long)]
        path_only: bool,
    },
    #[command(
        about = "Initialize a Pebble project.",
        long_about = "Initialize .pebble config and tasks directory in current working directory.",
        after_help = "Examples:\n  pebble init\n  pebble init --issue-prefix PROJ --dir docs/tasks"
    )]
    Init {
        /// Initial issue prefix for generated IDs.
        #[arg(long)]
        issue_prefix: Option<String>,
        /// Initial tasks-dir (must be a relative path).
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    #[command(
        about = "Read configuration values.",
        long_about = "Read resolved configuration values by key.",
        after_help = "Examples:\n  pebble config get issue-prefix\n  pebble config get tasks-dir --json"
    )]
    Config {
        #[command(subcommand)]
        cmd: ConfigCommands,
    },
    #[command(
        about = "Emit machine-readable help schema.",
        long_about = "Print a machine-readable schema of commands, flags, and output shapes.",
        after_help = "Examples:\n  pebble help-json"
    )]
    HelpJson,
}

#[derive(Subcommand)]
enum ConfigCommands {
    #[command(
        about = "Get one resolved config value.",
        long_about = "Get one resolved configuration value by key (issue-prefix or tasks-dir).",
        after_help = "Examples:\n  pebble config get issue-prefix\n  pebble config get tasks-dir --json"
    )]
    Get { key: String },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    if let Some(ref dir) = cli.directory {
        std::env::set_current_dir(dir)?;
    }

    let ctx = RunContext::load(cli.dir.clone(), cli.config, cli.json)?;

    match cli.command {
        Commands::Init { issue_prefix, dir } => run_init(cli.dir.or(dir), issue_prefix, cli.json),
        Commands::Config { cmd } => match cmd {
            ConfigCommands::Get { key } => run_config_get(&ctx, &key),
        },
        Commands::List {
            statuses,
            tags,
            needs,
            priorities,
            is_ready,
            all,
            limit,
            sort,
        } => {
            let options = ListOptions {
                statuses,
                tags,
                needs,
                priorities,
                is_ready,
                all,
                limit,
                sort,
            };
            run_list(&ctx, &options)
        }
        Commands::Next => run_next(&ctx),
        Commands::Search { query } => run_search(&ctx, &query),
        Commands::Add {
            title,
            status,
            priority,
            body,
            needs,
            tags,
        } => run_add(&ctx, title, status, priority, body, needs, tags),
        Commands::Update {
            id,
            title,
            status,
            priority,
            clear_priority,
            body,
            append_body,
            add_tags,
            remove_tags,
            add_needs,
            remove_needs,
        } => run_update(
            &ctx,
            id,
            title,
            status,
            priority,
            clear_priority,
            body,
            append_body,
            add_tags,
            remove_tags,
            add_needs,
            remove_needs,
        ),
        Commands::Archive => run_archive(&ctx),
        Commands::Show { id, path_only } => run_show(&ctx, &id, path_only),
        Commands::HelpJson => {
            println!("{}", serde_json::to_string(&help_json_schema())?);
            Ok(())
        }
    }
}

/// Build the machine-readable help schema describing all commands, flags, and output shapes.
fn help_json_schema() -> serde_json::Value {
    let cli_command = Cli::command();
    let commands: Vec<Value> = cli_command
        .get_subcommands()
        .filter(|subcommand| subcommand.get_name() != "help")
        .map(help_json_command_entry)
        .collect();

    json!({
        "name": "pebble",
        "global_options": [
            { "name": "--json", "description": "Output in JSON format" },
            { "name": "--dir <PATH>", "description": "Override tasks directory" }
        ],
        "commands": commands
    })
}

fn help_json_command_entry(subcommand: &clap::Command) -> Value {
    let mut entry = Map::new();
    let command_name = subcommand.get_name();

    entry.insert("name".to_string(), json!(command_name));
    entry.insert(
        "description".to_string(),
        json!(
            subcommand
                .get_about()
                .map(|about| about.to_string())
                .unwrap_or_default()
        ),
    );

    if subcommand.has_subcommands() {
        let subcommands: Vec<Value> = subcommand
            .get_subcommands()
            .filter(|nested| nested.get_name() != "help")
            .map(|nested| help_json_nested_command_entry(command_name, nested))
            .collect();
        entry.insert("subcommands".to_string(), Value::Array(subcommands));
    } else {
        let output = help_json_output_schema(command_name, None);
        entry.insert("output".to_string(), output);
    }

    Value::Object(entry)
}

fn help_json_nested_command_entry(parent_name: &str, subcommand: &clap::Command) -> Value {
    let mut entry = Map::new();
    let subcommand_name = subcommand.get_name();

    entry.insert("name".to_string(), json!(subcommand_name));
    entry.insert(
        "description".to_string(),
        json!(
            subcommand
                .get_about()
                .map(|about| about.to_string())
                .unwrap_or_default()
        ),
    );

    let output = help_json_output_schema(parent_name, Some(subcommand_name));
    entry.insert("output".to_string(), output);

    Value::Object(entry)
}

fn help_json_output_schema(command_name: &str, subcommand_name: Option<&str>) -> Value {
    match (command_name, subcommand_name) {
        ("init", None) => json!("status"),
        ("config", Some("get")) => json!({ "key": "string", "value": "string" }),
        ("list", None) => json!({ "tasks": ["TaskObject"] }),
        ("next", None) => json!("TaskObject|null"),
        ("search", None) => json!({ "tasks": ["TaskObject"] }),
        ("show", None) => json!("TaskObject|{path:string}"),
        ("add", None) => json!("TaskObject"),
        ("update", None) => json!("TaskObject"),
        ("archive", None) => json!({ "archived": [{ "id": "string", "moved_to": "string" }] }),
        ("help-json", None) => json!("HelpSchema"),
        _ => panic!(
            "Unhandled help-json output schema mapping for command '{}' and subcommand {:?}",
            command_name, subcommand_name
        ),
    }
}

#[cfg(test)]
mod help_json_schema_tests {
    use super::help_json_output_schema;

    #[test]
    #[should_panic(expected = "Unhandled help-json output schema mapping")]
    fn test_help_json_output_schema_panics_on_unhandled_command() {
        let _ = help_json_output_schema("unknown-command", None);
    }
}
