use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble_legacy::cli::{OutputFormat, UsageError};
use pebble_legacy::config::Config;

pub enum ConfigCommand {
    Get { key: String, format: OutputFormat },
}

pub fn run(config: &Config, command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Get { key, format } => {
            let val = match key.as_str() {
                "sync-branch" => config.sync_branch.clone(),
                "issue-prefix" => config.issue_prefix.clone(),
                _ => return Err(UsageError::new(format!("Unknown config key '{}'", key)).into()),
            };

            if let Some(v) = val {
                match format {
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "key": key,
                                "value": v,
                            }))?
                        );
                    }
                    OutputFormat::Human => {
                        println!("{}", v);
                    }
                }
            } else {
                return Err(eyre!("Config key '{}' not set", key));
            }
        }
    }
    Ok(())
}
