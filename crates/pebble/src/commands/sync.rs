use crate::commands::get_worktree_manager;
use color_eyre::Result;
use pebble::cli::OutputFormat;
use pebble::config::Config;

pub fn run(config: &Config, format: OutputFormat) -> Result<()> {
    let manager = get_worktree_manager(config, std::env::current_dir()?)?;

    match format {
        OutputFormat::Human => {
            println!("Syncing...");
            manager.sync(false)?;
            println!("Sync complete.");
        }
        OutputFormat::Json => {
            manager.sync(true)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "ok",
                }))?
            );
        }
    }
    Ok(())
}
