use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::config::Config;

pub enum ConfigCommand {
    Get { key: String },
}

pub fn run(config: &Config, command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Get { key } => {
            let val = match key.as_str() {
                "sync-branch" => config.sync_branch.clone(),
                "issue-prefix" => config.issue_prefix.clone(),
                _ => return Err(eyre!("Unknown config key '{}'", key)),
            };

            if let Some(v) = val {
                println!("{}", v);
            } else {
                return Err(eyre!("Config key '{}' not set", key));
            }
        }
    }
    Ok(())
}
