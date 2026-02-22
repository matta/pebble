use crate::commands::get_worktree_manager;
use color_eyre::Result;
use pebble_legacy::cli::OutputFormat;
use pebble_legacy::config::Config;

pub fn run(config: &Config, format: OutputFormat) -> Result<()> {
    let manager = get_worktree_manager(config, std::env::current_dir()?)?;

    match format {
        OutputFormat::Human => {
            println!("Syncing...");
            manager.sync()?;
            println!("Sync complete.");
        }
        OutputFormat::Json => {
            manager.sync_quiet()?;
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
