use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use pebble::config::Config;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(name = "pebble")]
struct Cli {
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
}

#[derive(Subcommand)]
enum ConfigCommands {
    Get { key: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Config { command }) => match command {
            ConfigCommands::Get { key } => {
                // Determine config path.
                // For now, assume .beads/config.yaml in CWD.
                // The test sets CWD to ../mydoo, so .beads/config.yaml should be present there.
                let config_path = ".beads/config.yaml";
                let content = std::fs::read_to_string(config_path)
                    .with_context(|| format!("Failed to read config file at {}", config_path))?;
                let config: Config =
                    serde_yaml::from_str(&content).context("Failed to parse config")?;

                if key == "sync-branch"
                    && let Some(val) = config.sync_branch
                {
                    println!("{}", val);
                }
            }
        },
        Some(Commands::Sync) => {
            let config_path = ".beads/config.yaml";
            let content = std::fs::read_to_string(config_path)
                .with_context(|| format!("Failed to read config file at {}", config_path))?;
            let config: Config =
                serde_yaml::from_str(&content).context("Failed to parse config")?;

            let sync_branch = config
                .sync_branch
                .ok_or_else(|| anyhow::anyhow!("sync-branch not configured"))?;

            let repo_root = std::env::current_dir()?;
            let manager = pebble::worktree::WorktreeManager::new(repo_root, sync_branch);

            println!("Syncing...");
            manager.sync()?;
            println!("Sync complete.");
        }
        None => {}
    }

    Ok(())
}
