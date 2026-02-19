use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::CONFIG_DIR;
use pebble::config::Config;

pub fn run(sync_branch: String) -> Result<()> {
    let repo_root = std::env::current_dir()?;

    if !pebble::worktree::WorktreeManager::<pebble::git_provider::RealGit>::is_inside_git_repo(
        &repo_root,
    ) {
        return Err(eyre!(
            "Error: 'pebble init' must be run inside a Git repository."
        ));
    }

    let manager =
        pebble::worktree::WorktreeManager::new(repo_root.clone(), sync_branch.to_string());

    println!("Creating orphaned sync branch: {}...", sync_branch);
    manager.create_orphaned_sync_branch()?;

    let worktree_path = manager.get_worktree_path();
    println!("Initializing worktree at {}...", worktree_path.display());
    manager.init_worktree(&worktree_path)?;

    let pebble_dir = repo_root.join(CONFIG_DIR);
    if !pebble_dir.exists() {
        std::fs::create_dir_all(&pebble_dir)?;
    }
    let config_path = Config::default_path(&repo_root);
    println!("Saving configuration to {}...", config_path.display());
    let config = Config {
        sync_branch: Some(sync_branch.clone()),
        ..Default::default()
    };
    config.save(&config_path)?;

    println!("Pebble initialized successfully!");
    Ok(())
}
