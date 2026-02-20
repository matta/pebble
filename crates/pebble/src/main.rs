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
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by owner
        #[arg(long)]
        owner: Option<String>,
        /// Filter by priority
        #[arg(long)]
        priority: Option<i32>,
        /// Filter by type
        #[arg(long = "type")]
        issue_type: Option<String>,
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
        /// New title for the issue
        #[arg(long)]
        title: Option<String>,
        /// New description for the issue
        #[arg(long)]
        description: Option<String>,
        /// New status for the issue
        #[arg(long)]
        status: Option<String>,
        /// New priority for the issue
        #[arg(long)]
        priority: Option<i32>,
        /// New owner for the issue
        #[arg(long)]
        owner: Option<String>,
        /// Close reason (required when closing)
        #[arg(long)]
        close_reason: Option<String>,
        /// New type for the issue
        #[arg(long = "type")]
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
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by owner
        #[arg(long)]
        owner: Option<String>,
        /// Filter by priority
        #[arg(long)]
        priority: Option<i32>,
        /// Filter by type
        #[arg(long = "type")]
        issue_type: Option<String>,
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
    use Commands::*;
    match command {
        Init { sync_branch, json } => run_init(sync_branch, json),
        Import { path, json } => run_import(require_config(config)?, path, json),
        Config { command } => run_config_command(command, require_config(config)?),
        Sync { json } => run_sync(require_config(config)?, json),
        List {
            status,
            owner,
            priority,
            issue_type,
            json,
        } => {
            let filters = commands::IssueFilters::new(status, owner, priority, issue_type);
            run_list(require_config(config)?, filters, json)
        }
        Add {
            title,
            description,
            json,
        } => run_add(require_config(config)?, title, description, json),
        Update {
            id,
            title,
            description,
            status,
            priority,
            owner,
            close_reason,
            issue_type,
            json,
        } => {
            let fields = commands::update::UpdateFields {
                title,
                description,
                status,
                priority,
                owner,
                close_reason,
                issue_type,
            };
            run_update(require_config(config)?, id, fields, json)
        }
        Show { id, json } => run_show(require_config(config)?, id, json),
        Search {
            query,
            status,
            owner,
            priority,
            issue_type,
            json,
        } => {
            let filters = commands::IssueFilters::new(status, owner, priority, issue_type);
            run_search(require_config(config)?, query, filters, json)
        }
    }
}

fn require_config(config: &Option<Config>) -> Result<&Config> {
    config.as_ref().ok_or_else(|| eyre!("Config not loaded"))
}

fn format(json: bool) -> OutputFormat {
    OutputFormat::from_json_flag(json)
}

fn run_init(sync_branch: String, json: bool) -> Result<()> {
    commands::init::run(sync_branch, format(json))
}

fn run_import(config: &Config, path: std::path::PathBuf, json: bool) -> Result<()> {
    commands::import::run(config, path, format(json))
}

fn run_config_command(command: ConfigCommands, config: &Config) -> Result<()> {
    match command {
        ConfigCommands::Get { key, json } => {
            let format = format(json);
            commands::config_cmd::run(config, ConfigCommand::Get { key, format })
        }
    }
}

fn run_sync(config: &Config, json: bool) -> Result<()> {
    commands::sync::run(config, format(json))
}

fn run_list(config: &Config, filters: commands::IssueFilters, json: bool) -> Result<()> {
    commands::list::run(config, filters, format(json))
}

fn run_add(config: &Config, title: String, description: Option<String>, json: bool) -> Result<()> {
    commands::add::run(config, title, description, format(json))
}

fn run_update(
    config: &Config,
    id: String,
    fields: commands::update::UpdateFields,
    json: bool,
) -> Result<()> {
    commands::update::run(config, id, fields, format(json))
}

fn run_show(config: &Config, id: String, json: bool) -> Result<()> {
    commands::show::run(config, id, format(json))
}

fn run_search(
    config: &Config,
    query: String,
    filters: commands::IssueFilters,
    json: bool,
) -> Result<()> {
    commands::search::run(config, query, filters, format(json))
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
