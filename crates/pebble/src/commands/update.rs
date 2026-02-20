use crate::commands::get_store;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::cli::OutputFormat;
use pebble::config::Config;

#[derive(Debug, Default)]
pub struct UpdateFields {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub owner: Option<String>,
    pub issue_type: Option<String>,
}

pub fn run(config: &Config, id: String, fields: UpdateFields, format: OutputFormat) -> Result<()> {
    let (store, manager, _) = get_store(config)?;
    let mut issues = store.read_issues()?;

    let UpdateFields {
        title,
        description,
        status,
        priority,
        owner,
        issue_type,
    } = fields;

    let idx = issues
        .iter()
        .position(|i| i.id == id)
        .ok_or_else(|| eyre!("Issue {} not found", id))?;

    let mut changed = false;
    {
        let issue = &mut issues[idx];
        if let Some(t) = title {
            issue.title = t;
            changed = true;
        }
        if let Some(d) = description {
            issue.description = Some(d);
            changed = true;
        }
        if let Some(s) = status {
            issue.status = s;
            changed = true;
        }
        if let Some(p) = priority {
            issue.priority = p;
            changed = true;
        }
        if let Some(o) = owner {
            issue.owner = Some(o);
            changed = true;
        }
        if let Some(it) = issue_type {
            issue.issue_type = it;
            changed = true;
        }

        if changed {
            issue.updated_at = chrono::Local::now().to_rfc3339();
        }
    }

    if changed {
        store.write_issues(&issues)?;
        let commit_message = format!("Update issue {}", id);
        match format {
            OutputFormat::Json => {
                manager.commit_all(&commit_message, true)?;
                println!("{}", serde_json::to_string_pretty(&issues[idx])?);
            }
            OutputFormat::Human => {
                manager.commit_all(&commit_message, false)?;
                println!("Updated issue {}", id);
            }
        }
    } else {
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&issues[idx])?);
            }
            OutputFormat::Human => {
                println!("No changes provided for issue {}", id);
            }
        }
    }
    Ok(())
}
