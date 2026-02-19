use clap::{Parser, Subcommand};
use clap::error::ErrorKind;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::config::Config;
use pebble::cli::{EXIT_ERROR, EXIT_OK, EXIT_USAGE, UsageError};
use crate::commands::config_cmd::ConfigCommand;

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

fn main() {
    if let Err(err) = color_eyre::install() {
        eprintln!("Error: {}", err);
    }

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => handle_clap_error(err),
    };

    if let Err(err) = run(cli) {
        exit_with_error(err);
    }
}

fn run(cli: Cli) -> Result<()> {
    if let Some(ref dir) = cli.directory {
        std::env::set_current_dir(dir)?;
    }

    let requires_init =
        !matches!(&cli.command, Commands::Init { .. } | Commands::Config { .. });
    if requires_init && !Config::is_initialized(&std::env::current_dir()?) {
        return Err(eyre!(
            "Error: Pebble is not initialized in this repository. Run 'pebble init' to get started."
        ));
    }

    let config = if matches!(&cli.command, Commands::Init { .. }) {
        None
    } else {
        Some(commands::load_config(cli.config.as_deref())?)
    };

    match cli.command {
        Commands::Init { sync_branch } => {
            commands::init::run(sync_branch)?;
        }
        Commands::Import { path } => {
            commands::import::run(config.as_ref().unwrap(), path)?;
        }
        Commands::Config { command } => {
            let config = config.as_ref().unwrap();
            match command {
                ConfigCommands::Get { key } => {
                    commands::config_cmd::run(
                        config,
                        ConfigCommand::Get { key },
                    )?;
                }
            }
        }
        Commands::Sync => {
            commands::sync::run(config.as_ref().unwrap())?;
        }
        Commands::List { json } => {
            commands::list::run(config.as_ref().unwrap(), json)?;
        }
        Commands::Add { title, description } => {
            commands::add::run(config.as_ref().unwrap(), title, description)?;
        }
        Commands::Show { id, json } => {
            commands::show::run(config.as_ref().unwrap(), id, json)?;
        }
        Commands::Edit {
            id,
            title,
            description,
        } => {
            commands::edit::run(config.as_ref().unwrap(), id, title, description)?;
        }
    }

    Ok(())
}

fn handle_clap_error(err: clap::Error) -> ! {
    let code = match err.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => EXIT_OK,
        _ => EXIT_USAGE,
    };
    let _ = err.print();
    std::process::exit(code);
}

fn exit_with_error(err: color_eyre::Report) -> ! {
    let code = if err.downcast_ref::<UsageError>().is_some() {
        EXIT_USAGE
    } else {
        EXIT_ERROR
    };
    eprintln!("{err}");
    std::process::exit(code);
}
