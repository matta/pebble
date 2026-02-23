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

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use commands::{ListOptions, RunContext, run_list, run_next, run_search, run_show};
use commands_write::{run_add, run_archive, run_init, run_update};
use models::TaskStatus;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "pebble",
    version,
    about = "A distributed issue tracking system built on Git."
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
    List {
        #[arg(long = "status", value_enum)]
        statuses: Vec<TaskStatus>,
        #[arg(long = "tag")]
        tags: Vec<String>,
        #[arg(long = "need")]
        needs: Vec<String>,
        #[arg(long = "priority", value_parser = clap::value_parser!(u8).range(0..=99))]
        priorities: Vec<u8>,
        #[arg(long)]
        is_ready: bool,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long, allow_hyphen_values = true)]
        sort: Option<String>,
    },
    Next,
    Search {
        query: String,
    },
    Add {
        title: String,
        #[arg(long, value_enum)]
        status: Option<TaskStatus>,
        #[arg(long)]
        priority: Option<u8>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long = "need")]
        needs: Vec<String>,
        #[arg(long = "tag")]
        tags: Vec<String>,
    },
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long, value_enum)]
        status: Option<TaskStatus>,
        #[arg(long)]
        priority: Option<u8>,
        #[arg(long)]
        clear_priority: bool,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        append_body: Option<String>,
        #[arg(long = "add-tag")]
        add_tags: Vec<String>,
        #[arg(long = "remove-tag")]
        remove_tags: Vec<String>,
        #[arg(long = "add-need")]
        add_needs: Vec<String>,
        #[arg(long = "remove-need")]
        remove_needs: Vec<String>,
    },
    Archive,
    /// Output a specific task in various formats.
    Show {
        id: String,
        /// Output just the raw filepath instead of the task entity.
        #[arg(long)]
        path_only: bool,
    },
    Init {
        #[arg(long)]
        issue_prefix: Option<String>,
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    Config {
        #[command(subcommand)]
        cmd: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
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
            ConfigCommands::Get { key } => todo!("config get {}", key),
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
    }
}
