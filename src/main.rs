use clap::{Parser, Subcommand};
use pebble::config::Config;
use anyhow::{Context, Result};

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
    Get {
        key: String,
    },
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
                let config: Config = serde_yaml::from_str(&content)
                    .context("Failed to parse config")?;

                if key == "sync-branch" {
                    if let Some(val) = config.sync_branch {
                        println!("{}", val);
                    }
                }
            }
        },
        Some(Commands::Sync) => {
            let config_path = ".beads/config.yaml";
            let _content = std::fs::read_to_string(config_path)
                .with_context(|| format!("Failed to read config file at {}", config_path))?;
            println!("Sync command not implemented yet");
        },
        None => {}
    }
    
    Ok(())
}
