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

const DEFAULT_CONFIG_PATH_PEBBLE: &str = ".pebble/config.yaml";

fn load_config(path: Option<&std::path::Path>) -> Result<Config> {
    let config_path = if let Some(p) = path {
        std::path::PathBuf::from(p)
    } else {
        std::path::PathBuf::from(DEFAULT_CONFIG_PATH_PEBBLE)
    };
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file at {}", config_path.display()))?;
    let config: Config = serde_yaml::from_str(&content).context("Failed to parse config")?;
    config.validate()?;
    Ok(config)
}

fn get_worktree_manager(config: &Config) -> Result<pebble::worktree::WorktreeManager> {
    let sync_branch = config
        .sync_branch
        .as_deref()
        .ok_or_else(|| eyre!("sync-branch not configured"))?;

    let repo_root = std::env::current_dir()?;
    Ok(pebble::worktree::WorktreeManager::new(
        repo_root,
        sync_branch.to_string(),
    ))
}

fn get_store(
    config: &Config,
) -> Result<(
    pebble::store::JsonlStore,
    pebble::worktree::WorktreeManager,
    std::path::PathBuf,
)> {
    let manager = get_worktree_manager(config)?;
    let jsonl_path = manager.get_absolute_jsonl_path()?;
    let store = pebble::store::JsonlStore::new(
        jsonl_path
            .to_str()
            .ok_or_else(|| eyre!("Path contains invalid UTF-8 characters"))?,
    );
    Ok((store, manager, jsonl_path))
}

fn get_git_config(key: &str) -> Result<String> {
    std::process::Command::new("git")
        .args(["config", key])
        .check_output()
        .map(|s| s.trim().to_string())
        .map_err(Into::into)
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    if let Some(ref dir) = cli.directory {
        std::env::set_current_dir(dir)
            .with_context(|| format!("Failed to change directory to {}", dir.display()))?;
    }

    if let Some(command) = &cli.command {
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
            Some(load_config(cli.config.as_deref())?)
        };

        match command {
            Commands::Init { sync_branch } => {
                let repo_root = std::env::current_dir()?;

                if !pebble::worktree::WorktreeManager::<pebble::worktree::RealGit>::is_inside_git_repo(
                    &repo_root,
                ) {
                    eprintln!("Error: 'pebble init' must be run inside a Git repository.");
                    std::process::exit(1);
                }

                let manager = pebble::worktree::WorktreeManager::new(
                    repo_root.clone(),
                    sync_branch.to_string(),
                );

                println!("Creating orphaned sync branch: {}...", sync_branch);
                manager.create_orphaned_sync_branch()?;

                let worktree_path = manager.get_worktree_path();
                println!("Initializing worktree at {}...", worktree_path.display());
                manager.init_worktree(&worktree_path)?;

                let pebble_dir = repo_root.join(".pebble");
                if !pebble_dir.exists() {
                    std::fs::create_dir_all(&pebble_dir)?;
                }
                println!(
                    "Saving configuration to {}...",
                    pebble_dir.join("config.yaml").display()
                );
                let config = pebble::config::Config {
                    sync_branch: Some(sync_branch.clone()),
                    ..Default::default()
                };
                config.save(&pebble_dir.join("config.yaml"))?;

                println!("Pebble initialized successfully!");
            }
            Commands::Import { path } => {
                let config = config.as_ref().unwrap();
                let (store, manager, _jsonl_path) = get_store(config)?;

                if manager.is_dirty()? {
                    eprintln!(
                        "Error: Pebble data worktree has uncommitted changes. Please commit or stash them before importing."
                    );
                    std::process::exit(1);
                }

                let mut issues = store.read_issues()?;

                let external_store = pebble::store::JsonlStore::new(
                    path.to_str().ok_or_else(|| eyre!("Invalid path"))?,
                );
                let external_issues = external_store.read_issues()?;

                let mut updated_count = 0;
                let mut added_count = 0;

                for ext_issue in external_issues {
                    if let Some(existing) = issues.iter_mut().find(|i| i.id == ext_issue.id) {
                        let old_updated = existing.updated_at.clone();
                        existing.merge(ext_issue);
                        if existing.updated_at != old_updated {
                            updated_count += 1;
                        }
                    } else {
                        issues.push(ext_issue);
                        added_count += 1;
                    }
                }

                if updated_count > 0 || added_count > 0 {
                    store.write_issues(&issues)?;
                    manager.commit_all(&format!("Imported data from {}", path.display()))?;
                    println!(
                        "Import complete: {} added, {} updated.",
                        added_count, updated_count
                    );
                } else {
                    println!("Import complete: No changes.");
                }
            }
            Commands::Config { command } => {
                let config = config.as_ref().unwrap();
                match command {
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
                }
            }

            Commands::Sync => {
                let config = config.as_ref().unwrap();
                let manager = get_worktree_manager(config)?;

                println!("Syncing...");
                manager.sync()?;
                println!("Sync complete.");
            }
            Commands::List { json } => {
                let config = config.as_ref().unwrap();
                let (store, _, jsonl_path) = get_store(config)?;

                if !*json {
                    println!("Using database: {}", jsonl_path.display());
                }
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
                let config = config.as_ref().unwrap();
                let (store, manager, _) = get_store(config)?;

                let prefix = config.issue_prefix.as_deref().unwrap_or("issue");

                let existing_issues = store.read_issues()?;
                let existing_ids: std::collections::HashSet<&str> =
                    existing_issues.iter().map(|i| i.id.as_str()).collect();

                let mut id;
                loop {
                    let suffix: String = rand::rng()
                        .sample_iter(&rand::distr::Alphanumeric)
                        .take(6)
                        .map(char::from)
                        .collect::<String>()
                        .to_lowercase();
                    id = format!("{}-{}", prefix, suffix);

                    if !existing_ids.contains(id.as_str()) {
                        break;
                    }
                }

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
                };

                store.append_issue(&issue)?;
                manager.commit_all(&format!("Add issue {}", id))?;
                println!("Added issue {}", id);
            }
            Commands::Show { id, json } => {
                let config = config.as_ref().unwrap();
                let (store, _, _) = get_store(config)?;
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
                let config = config.as_ref().unwrap();
                let (store, manager, _) = get_store(config)?;
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
                    manager.commit_all(&format!("Edit issue {}", id))?;
                    println!("Updated issue {}", id);
                } else {
                    println!("No changes provided for issue {}", id);
                }
            }
        }
    }

    Ok(())
}
