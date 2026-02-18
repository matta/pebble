use crate::commands::get_store;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::config::Config;

pub fn run(
    config: &Config,
    id: String,
    title: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let (store, manager, _) = get_store(config)?;
    let mut issues = store.read_issues()?;

    let issue = issues
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| eyre!("Issue {} not found", id))?;

    let mut changed = false;
    if let Some(t) = title {
        issue.title = t;
        changed = true;
    }
    if let Some(d) = description {
        issue.description = d;
        changed = true;
    }

    if changed {
        issue.updated_at = chrono::Local::now().to_rfc3339();
        store.write_issues(&issues)?;
        manager.commit_all(&format!("Edit issue {}", id))?;
        println!("Updated issue {}", id);
    } else {
        println!("No changes provided for issue {}", id);
    }
    Ok(())
}
