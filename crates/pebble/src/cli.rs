//! Command-line interface definition for Pebble.
//!
//! This module defines the `Cli` parser and its subcommands using `clap`.
//! It serves as the primary entry point for parsing arguments and dispatching
//! to command handlers.

use crate::models::{Priority, TaskStatus};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Top-level command-line argument parser.
#[derive(Parser)]
#[command(
    name = "pebble",
    version,
    about = "A markdown-native task tracker with graph semantics."
)]
pub struct Cli {
    /// Change to the given directory before doing anything
    #[arg(short = 'C', long, value_name = "PATH")]
    pub directory: Option<PathBuf>,

    /// Path to configuration file
    #[arg(short, long, env = "PEBBLE_CONFIG", value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    /// Path to the tasks directory (overrides config)
    #[arg(long, global = true, value_name = "PATH")]
    pub dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Top-level Pebble subcommands accepted by the CLI.
#[derive(Subcommand)]
pub enum Commands {
    /// List tasks matching specified filters.
    #[command(alias = "ls")]
    #[command(
        about = "List tasks from the graph.",
        long_about = "List tasks from the markdown task graph. By default, list omits closed (done/canceled) tasks. Filters can be combined: status/need/priority are OR (match any), tag is AND (match all). Results use dependency-aware ordering unless --sort is specified.",
        after_help = "Examples:\n  pebble list\n  pebble list --status todo --status in_progress --tag backend --is-ready\n  pebble list --sort -created_at --limit 20"
    )]
    List {
        /// Filter by status (OR). Repeat for multiple.
        #[arg(long = "status", value_enum, value_name = "STATUS")]
        statuses: Vec<TaskStatus>,
        /// Filter by tag (AND). Repeat to require multiple tags.
        #[arg(long = "tag", value_name = "TAG")]
        tags: Vec<String>,
        /// Show tasks that have this ID as a prerequisite (OR; repeatable).
        #[arg(long = "need", value_name = "ID")]
        needs: Vec<String>,
        /// Filter by priority (OR). Repeat for multiple. Valid range: 0..99.
        #[arg(long = "priority", value_name = "PRIORITY")]
        priorities: Vec<Priority>,
        /// Return only tasks whose dependencies are all terminal (done/canceled).
        #[arg(long)]
        is_ready: bool,
        /// Include done/canceled tasks (default omits closed tasks).
        #[arg(long)]
        all: bool,
        /// Limit number of results returned after filtering and ordering.
        #[arg(long, value_name = "COUNT")]
        limit: Option<usize>,
        /// Sort by field. Prefix with '-' for descending. Ties broken by created_at, then id.
        #[arg(long, allow_hyphen_values = true, value_name = "SORT_KEY")]
        sort: Option<String>,
    },
    #[command(
        about = "Return the next task to work on.",
        long_about = "Return the single highest-ranked ready task using Pebble's default scoring: (transitive_blocking_count DESC, priority ASC, created_at ASC, id ASC).",
        after_help = "Examples:\n  pebble next\n  pebble next --json"
    )]
    /// Return the single highest-priority task that is ready to work on.
    Next,
    #[command(
        about = "Search tasks by text.",
        long_about = "Search title and body content using case-insensitive substring matching. Results follow the default list ordering. Default omits closed tasks.",
        after_help = "Examples:\n  pebble search bug\n  pebble search \"database timeout\" --json"
    )]
    /// Search for tasks using primitive text matching.
    Search {
        /// Search query string (case-insensitive substring over title + body).
        #[arg(value_name = "TEXT")]
        query: String,
    },
    #[command(
        about = "Create a new task.",
        long_about = "Create a new task file with generated ID and frontmatter. Filename is slugified from the title. ID suffix length scales with task count to avoid collisions.",
        after_help = "Examples:\n  pebble add \"Implement file scanning\"\n  pebble add \"Fix bug\" --priority 5 --need PEBL-123 --tag urgent --json\n  pebble add \"Build foundation\" --blocks PEBL-456 --json"
    )]
    /// Create a new task file in the tasks directory.
    Add {
        /// Task title.
        #[arg(value_name = "TITLE")]
        title: String,
        /// Initial status (defaults to todo).
        #[arg(long, value_enum, value_name = "STATUS")]
        status: Option<TaskStatus>,
        /// Priority (0..99, lower is higher).
        #[arg(long, value_name = "PRIORITY")]
        priority: Option<Priority>,
        /// Initial markdown body content.
        #[arg(long, value_name = "TEXT")]
        body: Option<String>,
        /// Add the task with this ID as a prerequisite of the new task (repeatable).
        #[arg(long = "need", value_name = "ID")]
        needs: Vec<String>,
        /// Add a tag (repeatable).
        #[arg(long = "tag", value_name = "TAG")]
        tags: Vec<String>,
        /// Make the new task a prerequisite of the task with this ID (repeatable).
        #[arg(long = "blocks", value_name = "ID")]
        blocks: Vec<String>,
    },
    #[command(
        about = "Update an existing task.",
        long_about = "Update mutable fields and body content. Immutable id is preserved. modified_at is updated automatically; resolved_at is set/cleared based on status transitions.",
        after_help = "Examples:\n  pebble update PEBL-1 --status in_progress\n  pebble update PEBL-1 --add-tag docs --remove-need PEBL-0 --append-body \"See RFC 001\" --json\n  pebble update PEBL-1 --blocks PEBL-2 --json"
    )]
    /// Update an existing task's frontmatter or body.
    Update {
        /// Task ID to update.
        #[arg(value_name = "ID")]
        id: String,
        /// Replace task title.
        #[arg(long, value_name = "TITLE")]
        title: Option<String>,
        /// Set task status. Terminal transitions manage resolved_at.
        #[arg(long, value_enum, value_name = "STATUS")]
        status: Option<TaskStatus>,
        /// Set task priority (0..99).
        #[arg(long, value_name = "PRIORITY")]
        priority: Option<Priority>,
        /// Clear existing priority (sets to None).
        #[arg(long)]
        clear_priority: bool,
        /// Replace the entire markdown body.
        #[arg(long, value_name = "TEXT")]
        body: Option<String>,
        /// Append content to the end of the markdown body.
        #[arg(long, value_name = "TEXT")]
        append_body: Option<String>,
        /// Add a tag (repeatable).
        #[arg(long = "add-tag", value_name = "TAG")]
        add_tags: Vec<String>,
        /// Remove a tag (repeatable).
        #[arg(long = "remove-tag", value_name = "TAG")]
        remove_tags: Vec<String>,
        /// Add the task with this ID as a prerequisite of this task (repeatable).
        #[arg(long = "add-need", value_name = "ID")]
        add_needs: Vec<String>,
        /// Remove the task with this ID as a prerequisite of this task (repeatable).
        #[arg(long = "remove-need", value_name = "ID")]
        remove_needs: Vec<String>,
        /// Make this task a prerequisite of the task with this ID (repeatable).
        #[arg(long = "blocks", value_name = "ID")]
        blocks: Vec<String>,
        /// Remove this task as a prerequisite of the task with this ID (repeatable).
        #[arg(long = "remove-blocks", value_name = "ID")]
        remove_blocks: Vec<String>,
    },
    #[command(
        about = "Archive old closed tasks.",
        long_about = "Sweep done/canceled tasks older than the configured threshold into the archive/ subdirectory. Resolves filename collisions with numeric suffixes.",
        after_help = "Examples:\n  pebble archive\n  pebble archive --json"
    )]
    /// Archive terminal tasks older than the configured threshold.
    Archive,
    #[command(
        about = "Read-only diagnostics for the task graph.",
        long_about = "Performs a read-only health check on the graph. Does not rewrite state. Exits with status code 0 and emits warnings for unknown frontmatter keys, duplicate IDs, and dangling dependencies.",
        after_help = "Examples:\n  pebble doctor\n  pebble doctor --json"
    )]
    /// Perform a read-only health check on the graph.
    Doctor,
    #[command(
        about = "Show one task by ID.",
        long_about = "Show full task details or only the relative file path. Exit code 1 if task is not found.",
        after_help = "Examples:\n  pebble show PEBL-1\n  pebble show PEBL-1 --path-only --json"
    )]
    /// Display full details of a single task.
    Show {
        /// Task ID to show.
        #[arg(value_name = "ID")]
        id: String,
        /// Output only the file path relative to tasks-dir.
        #[arg(long)]
        path_only: bool,
    },
    #[command(
        about = "Initialize a Pebble project.",
        long_about = "Bootstrap .pebble config and tasks directory in the current directory. All paths in config must be relative to project root.",
        after_help = "Examples:\n  pebble init\n  pebble init --issue-prefix PROJ --dir tasks"
    )]
    /// Initialize a new Pebble project.
    Init {
        /// Initial issue-prefix for generated IDs.
        #[arg(long, value_name = "PREFIX")]
        issue_prefix: Option<String>,
        /// Initial tasks-dir relative to project root.
        #[arg(long, value_name = "RELATIVE_PATH")]
        dir: Option<PathBuf>,
    },
    #[command(
        about = "Read configuration values.",
        long_about = "Read resolved configuration values (issue-prefix or tasks-dir) by key.",
        after_help = "Examples:\n  pebble config get issue-prefix\n  pebble config get tasks-dir --json"
    )]
    /// View or modify Pebble configuration.
    Config {
        #[command(subcommand)]
        cmd: ConfigCommands,
    },
    #[command(
        about = "Emit machine-readable help schema.",
        long_about = "Print a machine-readable JSON schema of all commands, flags, and output shapes for tool integration.",
        after_help = "Examples:\n  pebble help-json"
    )]
    /// Output the CLI structure in a machine-readable JSON format.
    HelpJson,
    #[command(
        about = "Check for common issues.",
        long_about = "Scan the project for potential configuration issues or data corruption.",
        after_help = "Examples:\n  pebble doctor"
    )]
    /// Verify the health of the project configuration and data.
    Doctor,
}

/// Configuration-related subcommands.
#[derive(Subcommand)]
pub enum ConfigCommands {
    #[command(
        about = "Get one resolved config value.",
        long_about = "Get one resolved configuration value by key (issue-prefix or tasks-dir). Output for unknown keys is a usage error.",
        after_help = "Examples:\n  pebble config get issue-prefix\n  pebble config get tasks-dir --json"
    )]
    /// Retrieve the value of a specific configuration key.
    Get {
        /// Configuration key to fetch (issue-prefix or tasks-dir).
        #[arg(value_name = "KEY")]
        key: String,
    },
}
