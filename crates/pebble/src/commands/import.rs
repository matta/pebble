use crate::commands::get_store;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::config::Config;
use std::path::PathBuf;

pub fn run(config: &Config, path: PathBuf) -> Result<()> {
    let (store, manager, _jsonl_path) = get_store(config)?;

    if manager.is_dirty()? {
        return Err(eyre!(
            "Error: Pebble data worktree has uncommitted changes. Please commit or stash them before importing."
        ));
    }

    let mut issues = store.read_issues()?;

    let external_store =
        pebble::store::JsonlStore::new(path.to_str().ok_or_else(|| eyre!("Invalid path"))?);
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
    Ok(())
}
