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
        #[arg(long, default_value = pebble::DEFAULT_SYNC_BRANCH)]
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
        status: Option<String>,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        priority: Option<i32>,
        #[arg(long)]
        json: bool,
    },
    Search {
        query: String,
        #[arg(long)]
        json: bool,
    },
    Add {
        title: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<i32>,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        issue_type: Option<String>,
        #[arg(long)]
        json: bool,
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
        Commands::List {
            status,
            owner,
            priority,
            json,
        } => {
            commands::list::run(
                config.as_ref().unwrap(),
                status.clone(),
                owner.clone(),
                *priority,
                *json,
            )?;
        }
        Commands::Search { query, json } => {
            commands::search::run(config.as_ref().unwrap(), query.clone(), *json)?;
        }
        Commands::Add {
            title,
            description,
            json,
        } => {
            commands::add::run(
                config.as_ref().unwrap(),
                title.clone(),
                description.clone(),
                *json,
            )?;
        }
        Commands::Show { id, json } => {
            commands::show::run(config.as_ref().unwrap(), id.clone(), *json)?;
        }
        Commands::Update {
            id,
            title,
            description,
            status,
            priority,
            owner,
            issue_type,
            json,
        } => {
            commands::update::run(
                config.as_ref().unwrap(),
                commands::update::UpdateArgs {
                    id: id.clone(),
                    title: title.clone(),
                    description: description.clone(),
                    status: status.clone(),
                    priority: *priority,
                    owner: owner.clone(),
                    issue_type: issue_type.clone(),
                    json: *json,
                },
            )?;
        }
    }

    Ok(())
}
