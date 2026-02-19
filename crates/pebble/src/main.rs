use crate::commands::config_cmd::ConfigCommand;
use clap::error::ErrorKind;
use clap::{Parser, Subcommand};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::cli::{EXIT_ERROR, EXIT_OK, EXIT_USAGE, OutputFormat, UsageError};
use pebble::config::Config;

mod commands;
mod help_json;

#[derive(Parser)]
#[command(name = "pebble")]
#[command(version, about = "A distributed issue tracking system built on Git.", long_about = None)]
#[command(
    after_help = "Examples:\n  pebble init\n  pebble add \"Fix login\" --description \"Investigate session timeout\"\n  pebble list\n  pebble show issue-abc123\n  pebble update issue-abc123 --title \"Fix login flow\"\n  pebble sync\n  pebble import issues.jsonl\n  pebble config get sync-branch"
)]
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
    #[command(
        after_help = "Examples:\n  pebble init\n  pebble init --sync-branch my-sync-branch\n  pebble init --json"
    )]
    Init {
        /// Name of the synchronization branch
        #[arg(long, default_value = pebble::DEFAULT_SYNC_BRANCH)]
        sync_branch: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Inspect or read configuration values
    #[command(
        after_help = "Examples:\n  pebble config get sync-branch\n  pebble config get issue-prefix --json"
    )]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Import issues from a JSONL file
    #[command(
        after_help = "Examples:\n  pebble import issues.jsonl\n  pebble import /path/to/issues.jsonl --json"
    )]
    Import {
        /// Path to the JSONL file to import
        path: std::path::PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Search issues by title or description
    #[command(
        after_help = "Examples:\n  pebble search \"login\"\n  pebble search \"login\" --json"
    )]
    Search {
        query: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a new issue
    #[command(
        after_help = "Examples:\n  pebble add \"Fix login\"\n  pebble add \"Fix login\" --description \"Investigate session timeout\"\n  pebble add \"Fix login\" --json"
    )]
    Add {
        title: String,
        #[arg(long)]
        description: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Update an existing issue (broader field set)
    #[command(
        after_help = "Examples:\n  pebble update issue-abc123 --status closed\n  pebble update issue-abc123 --priority 2 --owner you@example.com\n  pebble update issue-abc123 --json"
    )]
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
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show a single issue
    #[command(
        after_help = "Examples:\n  pebble show issue-abc123\n  pebble show issue-abc123 --json"
    )]
    Show {
        id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List issues in the data worktree
    #[command(
        after_help = "Examples:\n  pebble list\n  pebble list --status open\n  pebble list --json"
    )]
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        priority: Option<i32>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Sync the data worktree with the remote
    #[command(after_help = "Examples:\n  pebble sync\n  pebble sync --json")]
    Sync {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Get a configuration value
    #[command(
        after_help = "Examples:\n  pebble config get sync-branch\n  pebble config get issue-prefix --json"
    )]
    Get {
        key: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    if let Err(err) = color_eyre::install() {
        eprintln!("Error: {}", err);
    }

    if std::env::args().any(|arg| arg == "--help-json") {
        if let Err(err) = help_json::print() {
            eprintln!("Error: {}", err);
            std::process::exit(EXIT_ERROR);
        }
        std::process::exit(EXIT_OK);
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

    let requires_init = !matches!(
        &cli.command,
        Commands::Init { .. } | Commands::Config { .. }
    );
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

    dispatch_command(cli.command, &config)
}

fn dispatch_command(command: Commands, config: &Option<Config>) -> Result<()> {
    match command {
        Commands::Init { sync_branch, json } => {
            let format = OutputFormat::from_json_flag(json);
            commands::init::run(sync_branch, format)?;
        }
        Commands::Import { path, json } => {
            let format = OutputFormat::from_json_flag(json);
            commands::import::run(require_config(config)?, path, format)?;
        }
        Commands::Config { command } => match command {
            ConfigCommands::Get { key, json } => {
                let format = OutputFormat::from_json_flag(json);
                commands::config_cmd::run(
                    require_config(config)?,
                    ConfigCommand::Get { key, format },
                )?;
            }
        },
        Commands::Sync { json } => {
            let format = OutputFormat::from_json_flag(json);
            commands::sync::run(require_config(config)?, format)?;
        }
        Commands::List {
            status,
            owner,
            priority,
            json,
        } => {
            let format = OutputFormat::from_json_flag(json);
            commands::list::run(require_config(config)?, status, owner, priority, format)?;
        }
        Commands::Add {
            title,
            description,
            json,
        } => {
            let format = OutputFormat::from_json_flag(json);
            commands::add::run(require_config(config)?, title, description, format)?;
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
            let format = OutputFormat::from_json_flag(json);
            let fields = commands::update::UpdateFields {
                title,
                description,
                status,
                priority,
                owner,
                issue_type,
            };
            commands::update::run(require_config(config)?, id, fields, format)?;
        }
        Commands::Show { id, json } => {
            let format = OutputFormat::from_json_flag(json);
            commands::show::run(require_config(config)?, id, format)?;
        }
        Commands::Search { query, json } => {
            let format = OutputFormat::from_json_flag(json);
            commands::search::run(require_config(config)?, query, format)?;
        }
    }

    Ok(())
}

fn require_config(config: &Option<Config>) -> Result<&Config> {
    config.as_ref().ok_or_else(|| eyre!("Config not loaded"))
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
