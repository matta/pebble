use crate::commands::get_store;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::cli::OutputFormat;
use pebble::config::Config;

pub fn run(
    config: &Config,
    id: String,
    title: Option<String>,
    description: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let (store, manager, _) = get_store(config)?;
    let mut issues = store.read_issues()?;

    let index = issues
        .iter()
        .position(|i| i.id == id)
        .ok_or_else(|| eyre!("Issue {} not found", id))?;

    let mut changed = false;
    {
        let issue = &mut issues[index];
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
        }
    }

    if changed {
        store.write_issues(&issues)?;
        manager.commit_all(&format!("Edit issue {}", id))?;
    }

    let issue = &issues[index];
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(issue)?),
        OutputFormat::Human => {
            if changed {
                println!("Updated issue {}", id);
            } else {
                println!("No changes provided for issue {}", id);
            }
        }
    }
    Ok(())
}
