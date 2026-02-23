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
use commands::{RunContext, run_config_get, run_list, run_next, run_search, run_show};
use commands_write::{run_add, run_archive, run_init, run_update};
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

    /// Output help in JSON format
    #[arg(long, global = true)]
    help_json: bool,

    /// Path to the tasks directory (overrides config)
    #[arg(long, global = true)]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[arg(long)]
        is_ready: bool,
        #[arg(long)]
        status: Vec<String>,
        #[arg(long)]
        priority: Vec<u8>,
        #[arg(long = "tag")]
        tags: Vec<String>,
        #[arg(long = "dep")]
        deps: Vec<String>,
    },
    Next,
    Search {
        query: String,
    },
    Add {
        title: String,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<u8>,
        #[arg(long)]
        body: Option<String>,
        #[arg(long = "dep")]
        deps: Vec<String>,
        #[arg(long = "tag")]
        tags: Vec<String>,
    },
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        status: Option<String>,
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
        #[arg(long = "add-dep")]
        add_deps: Vec<String>,
        #[arg(long = "remove-dep")]
        remove_deps: Vec<String>,
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

fn dump_help_json(cmd: &clap::Command) {
    fn build_json(cmd: &clap::Command) -> serde_json::Value {
        serde_json::json!({
            "name": cmd.get_name(),
            "about": cmd.get_about().map(|s| s.to_string()),
            "args": cmd.get_arguments().map(|a| {
                serde_json::json!({
                    "name": a.get_id().as_str(),
                    "help": a.get_help().map(|s| s.to_string()),
                    "required": a.is_required_set(),
                })
            }).collect::<Vec<_>>(),
            "subcommands": cmd.get_subcommands().map(|sub| {
                 build_json(sub)
            }).collect::<Vec<_>>()
        })
    }

    let json = build_json(cmd);
    println!("{}", serde_json::to_string(&json).unwrap());
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if cli.help_json {
        use clap::CommandFactory;
        dump_help_json(&Cli::command());
        return Ok(());
    }

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
            is_ready,
            status,
            priority,
            tags,
            deps,
        } => run_list(&ctx, is_ready, status, priority, tags, deps),
        Commands::Next => run_next(&ctx),
        Commands::Search { query } => run_search(&ctx, &query),
        Commands::Add {
            title,
            status,
            priority,
            body,
            deps,
            tags,
        } => run_add(&ctx, title, status, priority, body, deps, tags),
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
            add_deps,
            remove_deps,
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
            add_deps,
            remove_deps,
        ),
        Commands::Archive => run_archive(&ctx),
        Commands::Show { id, path_only } => run_show(&ctx, &id, path_only),
    }
}
