use clap::{Parser, Subcommand};
use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use pebble::command::CommandExt;
use pebble::config::Config;
use rand::RngExt;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(name = "pebble")]
struct Cli {
    /// Change to this directory before doing anything else
    #[arg(short = 'C', long)]
    directory: Option<std::path::PathBuf>,

    /// Path to the configuration file
    #[arg(short, long, env = "PEBBLE_CONFIG")]
    config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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

const DEFAULT_CONFIG_PATH: &str = ".beads/config.yaml";

fn load_config(path: Option<&std::path::Path>) -> Result<Config> {
    let config_path = path
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(DEFAULT_CONFIG_PATH));
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file at {}", config_path.display()))?;
    let config: Config = serde_yaml::from_str(&content).context("Failed to parse config")?;
    Ok(config)
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if let Some(ref dir) = cli.directory {
        std::env::set_current_dir(dir)
            .with_context(|| format!("Failed to change directory to {}", dir.display()))?;
    }

    if let Some(command) = &cli.command {
        let config = load_config(cli.config.as_deref())?;

        if config.no_daemon == Some(false) || config.auto_start_daemon == Some(true) {
            return Err(eyre!("Daemon mode is not supported"));
        }

        match command {
            Commands::Config { command } => match command {
                ConfigCommands::Get { key } => {
                    let val = match key.as_str() {
                        "sync-branch" => config.sync_branch.clone(),
                        "issue-prefix" => config.issue_prefix.clone(),
                        "no-db" => config.no_db.map(|v| v.to_string()),
                        "no-daemon" => config.no_daemon.map(|v| v.to_string()),
                        "auto-start-daemon" => config.auto_start_daemon.map(|v| v.to_string()),
                        _ => return Err(eyre!("Unknown config key '{}'", key)),
                    };

                    if let Some(v) = val {
                        println!("{}", v);
                    } else {
                        return Err(eyre!("Config key '{}' not set", key));
                    }
                }
            },
            Commands::Sync => {
                let sync_branch = config
                    .sync_branch
                    .as_deref()
                    .ok_or_else(|| eyre!("sync-branch not configured"))?;

                let repo_root = std::env::current_dir()?;
                let manager =
                    pebble::worktree::WorktreeManager::new(repo_root, sync_branch.to_string());

                println!("Syncing...");
                manager.sync()?;
                println!("Sync complete.");
            }
            Commands::List { json } => {
                let sync_branch = config
                    .sync_branch
                    .as_deref()
                    .ok_or_else(|| eyre!("sync-branch not configured"))?;

                let repo_root = std::env::current_dir()?;
                let manager =
                    pebble::worktree::WorktreeManager::new(repo_root, sync_branch.to_string());

                let jsonl_path = manager.get_absolute_jsonl_path()?;
                if !*json {
                    println!("Using database: {}", jsonl_path.display());
                }
                let store = pebble::store::JsonlStore::new(jsonl_path.to_str().unwrap());
                let issues = store.read_issues()?;

                if *json {
                    println!("{}", serde_json::to_string_pretty(&issues)?);
                } else if issues.is_empty() {
                    println!("No issues found.");
                } else {
                    for issue in issues {
                        println!("{} [{}] {}", issue.id, issue.status, issue.title);
                    }
                }
            }
            Commands::Add { title, description } => {
                let sync_branch = config
                    .sync_branch
                    .as_deref()
                    .ok_or_else(|| eyre!("sync-branch not configured"))?;

                let repo_root = std::env::current_dir()?;
                let manager =
                    pebble::worktree::WorktreeManager::new(repo_root, sync_branch.to_string());

                let jsonl_path = manager.get_absolute_jsonl_path()?;
                let store = pebble::store::JsonlStore::new(jsonl_path.to_str().unwrap());

                let prefix = config.issue_prefix.as_deref().unwrap_or("issue");
                let suffix: String = rand::rng()
                    .sample_iter(&rand::distr::Alphanumeric)
                    .take(3)
                    .map(char::from)
                    .collect::<String>()
                    .to_lowercase();
                let id = format!("{}-{}", prefix, suffix);

                let now = chrono::Local::now().to_rfc3339();
                let user_name =
                    get_git_config("user.name").unwrap_or_else(|_| "unknown".to_string());
                let user_email =
                    get_git_config("user.email").unwrap_or_else(|_| "unknown".to_string());

                let issue = pebble::store::Issue {
                    id: id.clone(),
                    title: title.clone(),
                    description: description.clone().unwrap_or_default(),
                    status: "open".to_string(),
                    priority: 0,
                    issue_type: "task".to_string(),
                    owner: user_email,
                    created_at: now.clone(),
                    created_by: user_name,
                    updated_at: now,
                    closed_at: None,
                    close_reason: None,
                    dependencies: vec![],
                    extra: std::collections::HashMap::new(),
                };

                store.append_issue(&issue)?;
                println!("Added issue {}", id);
            }
            Commands::Show { id, json } => {
                let sync_branch = config
                    .sync_branch
                    .as_deref()
                    .ok_or_else(|| eyre!("sync-branch not configured"))?;

                let repo_root = std::env::current_dir()?;
                let manager =
                    pebble::worktree::WorktreeManager::new(repo_root, sync_branch.to_string());

                let jsonl_path = manager.get_absolute_jsonl_path()?;
                let store = pebble::store::JsonlStore::new(jsonl_path.to_str().unwrap());
                let issues = store.read_issues()?;

                let issue = issues
                    .into_iter()
                    .find(|i| i.id == *id)
                    .ok_or_else(|| eyre!("Issue {} not found", id))?;

                if *json {
                    println!("{}", serde_json::to_string_pretty(&issue)?);
                } else {
                    println!("ID:          {}", issue.id);
                    println!("Status:      {}", issue.status);
                    println!("Title:       {}", issue.title);
                    println!("Type:        {}", issue.issue_type);
                    println!("Priority:    {}", issue.priority);
                    println!("Owner:       {}", issue.owner);
                    println!("Created At:  {}", issue.created_at);
                    println!("Created By:  {}", issue.created_by);
                    println!("Updated At:  {}", issue.updated_at);
                    if let Some(closed_at) = issue.closed_at {
                        println!("Closed At:   {}", closed_at);
                    }
                    if let Some(reason) = issue.close_reason {
                        println!("Close Reason: {}", reason);
                    }
                    if !issue.description.is_empty() {
                        println!("\nDescription:\n{}", issue.description);
                    }
                }
            }
            Commands::Edit {
                id,
                title,
                description,
            } => {
                let sync_branch = config
                    .sync_branch
                    .as_deref()
                    .ok_or_else(|| eyre!("sync-branch not configured"))?;

                let repo_root = std::env::current_dir()?;
                let manager =
                    pebble::worktree::WorktreeManager::new(repo_root, sync_branch.to_string());

                let jsonl_path = manager.get_absolute_jsonl_path()?;
                let store = pebble::store::JsonlStore::new(jsonl_path.to_str().unwrap());
                let mut issues = store.read_issues()?;

                let issue = issues
                    .iter_mut()
                    .find(|i| i.id == *id)
                    .ok_or_else(|| eyre!("Issue {} not found", id))?;

                let mut changed = false;
                if let Some(t) = title {
                    issue.title = t.clone();
                    changed = true;
                }
                if let Some(d) = description {
                    issue.description = d.clone();
                    changed = true;
                }

                if changed {
                    issue.updated_at = chrono::Local::now().to_rfc3339();
                    store.write_issues(&issues)?;
                    println!("Updated issue {}", id);
                } else {
                    println!("No changes provided for issue {}", id);
                }
            }
        }
    }

    Ok(())
}

fn get_git_config(key: &str) -> Result<String> {
    std::process::Command::new("git")
        .args(["config", key])
        .check_output()
        .map(|s| s.trim().to_string())
        .map_err(Into::into)
}
