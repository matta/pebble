use clap::{Parser, Subcommand};
use color_eyre::Result;
use pebble::config::Config;

mod commands;

#[derive(Parser)]
#[command(name = "pebble")]
#[command(version, about = "A distributed issue tracking system built on Git.", long_about = None)]
struct Cli {
    /// Change to this directory before doing anything else
    #[arg(short = 'C', long)]
    directory: Option<std::path::PathBuf>,

    /// Path to the configuration file
    #[arg(short, long, env = "PEBBLE_CONFIG")]
    config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Pebble repository
    Init {
        /// Name of the synchronization branch
        #[arg(long, default_value = "pebble-data")]
        sync_branch: String,
    },
    /// Import issues from a JSONL file
    Import {
        /// Path to the JSONL file to import
        path: std::path::PathBuf,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    Sync,
    List {
        #[arg(long)]
        json: bool,
    },
    Add {
        title: String,
        #[arg(long)]
        description: Option<String>,
    },
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
    Edit {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
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

    let command = &cli.command;
    // Initialization check for commands that require it
    let requires_init = !matches!(command, Commands::Init { .. } | Commands::Config { .. });
    if requires_init && !Config::is_initialized(&std::env::current_dir()?) {
        eprintln!(
            "Error: Pebble is not initialized in this repository. Run 'pebble init' to get started."
        );
        std::process::exit(1);
    }

    let config = if matches!(command, Commands::Init { .. }) {
        // Init doesn't need to load config first
        None
    } else {
        Some(commands::load_config(cli.config.as_deref())?)
    };

    match command {
        Commands::Init { sync_branch } => {
            commands::init::run(sync_branch.clone())?;
        }
        Commands::Import { path } => {
            commands::import::run(config.as_ref().unwrap(), path.clone())?;
        }
        Commands::Config { command } => {
            let config = config.as_ref().unwrap();
            match command {
                ConfigCommands::Get { key } => {
                    commands::config_cmd::run(
                        config,
                        commands::config_cmd::ConfigCommand::Get { key: key.clone() },
                    )?;
                }
            }
        }
        Commands::Sync => {
            commands::sync::run(config.as_ref().unwrap())?;
        }
        Commands::List { json } => {
            commands::list::run(config.as_ref().unwrap(), *json)?;
        }
        Commands::Add { title, description } => {
            commands::add::run(config.as_ref().unwrap(), title.clone(), description.clone())?;
        }
        Commands::Show { id, json } => {
            commands::show::run(config.as_ref().unwrap(), id.clone(), *json)?;
        }
        Commands::Edit {
            id,
            title,
            description,
        } => {
            commands::edit::run(
                config.as_ref().unwrap(),
                id.clone(),
                title.clone(),
                description.clone(),
            )?;
        }
    }

    Ok(())
}
