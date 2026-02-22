pub mod commands;
mod config;
pub mod graph;
pub mod models;
pub mod parser;

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use commands::{RunContext, run_list, run_next, run_show};
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
    Show {
        id: String,
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

    let ctx = RunContext::load(cli.dir, cli.config, cli.json)?;

    match cli.command {
        Commands::Init { .. } => todo!("init"),
        Commands::Config { cmd } => match cmd {
            ConfigCommands::Get { key } => todo!("config get {}", key),
        },
        Commands::List { is_ready } => run_list(&ctx, is_ready),
        Commands::Next => run_next(&ctx),
        Commands::Show { id, path_only } => run_show(&ctx, &id, path_only),
    }
}
