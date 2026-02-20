use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::CONFIG_DIR;
use pebble::cli::OutputFormat;
use pebble::config::Config;

pub fn run(sync_branch: String, format: OutputFormat) -> Result<()> {
    pebble::config::validate_branch_name(&sync_branch)?;

    let current_dir = std::env::current_dir()?;

    if !pebble::worktree::WorktreeManager::<pebble::git_provider::RealGit>::is_inside_git_repo(
        &current_dir,
    ) {
        return Err(eyre!(
            "Error: 'pebble init' must be run inside a Git repository."
        ));
    }

    let repo_root =
        pebble::worktree::find_git_root(
            &current_dir,
        )?;

    let manager =
        pebble::worktree::WorktreeManager::new(repo_root.clone(), sync_branch.to_string());

    if matches!(format, OutputFormat::Human) {
        println!("Creating orphaned sync branch: {}...", sync_branch);
    }
    manager.create_orphaned_sync_branch()?;

    let worktree_path = manager.get_worktree_path();
    if matches!(format, OutputFormat::Human) {
        println!("Initializing worktree at {}...", worktree_path.display());
    }
    manager.init_worktree(&worktree_path)?;

    let pebble_dir = repo_root.join(CONFIG_DIR);
    if !pebble_dir.exists() {
        std::fs::create_dir_all(&pebble_dir)?;
    }
    let config_path = Config::default_path(&repo_root);
    if matches!(format, OutputFormat::Human) {
        println!("Saving configuration to {}...", config_path.display());
    }
    let config = Config {
        sync_branch: Some(sync_branch.clone()),
        ..Default::default()
    };
    config.save(&config_path)?;

    match format {
        OutputFormat::Human => {
            println!("Sync branch:  {}", sync_branch);
            println!("Worktree:     {}", worktree_path.display());
            println!("Config file:  {}", config_path.display());
            println!("Pebble initialized successfully!");
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "sync_branch": sync_branch,
                    "worktree_path": worktree_path.display().to_string(),
                    "config_path": config_path.display().to_string(),
                }))?
            );
        }
    }
    Ok(())
}
