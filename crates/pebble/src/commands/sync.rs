use crate::commands::get_worktree_manager;
use color_eyre::Result;
use pebble::config::Config;

pub fn run(config: &Config) -> Result<()> {
    let manager = get_worktree_manager(config)?;

    println!("Syncing...");
    manager.sync()?;
    println!("Sync complete.");
    Ok(())
}
