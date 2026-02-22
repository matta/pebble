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
use commands::{RunContext, run_list, run_next, run_show};
use commands_write::{run_add, run_archive, run_init, run_update};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "pebble",
    version,
    about = "A distributed issue tracking system built on Git."
)]
struct Cli {
    #[arg(short = 'C', long)]
    directory: Option<PathBuf>,

    #[arg(short, long, env = "PEBBLE_CONFIG")]
    config: Option<PathBuf>,

    #[arg(long, global = true)]
    json: bool,

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
    },
    Next,
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
        Commands::List { is_ready } => run_list(&ctx, is_ready),
        Commands::Next => run_next(&ctx),
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
