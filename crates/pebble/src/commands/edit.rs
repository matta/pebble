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
            issue.description = Some(d);
            changed = true;
        }
        if changed {
            issue.updated_at = chrono::Local::now().to_rfc3339();
        }
    }

    if changed {
        store.write_issues(&issues)?;
        let commit_message = format!("Edit issue {}", id);
        match format {
            OutputFormat::Json => {
                manager.commit_all_quiet(&commit_message)?;
            }
            OutputFormat::Human => {
                manager.commit_all(&commit_message)?;
            }
        }
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
