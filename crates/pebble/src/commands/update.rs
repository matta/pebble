use crate::commands::get_store;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::config::Config;

pub fn run(
    config: &Config,
    id: String,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<i32>,
    owner: Option<String>,
    issue_type: Option<String>,
    json: bool,
) -> Result<()> {
    let (store, manager, _) = get_store(config)?;
    let mut issues = store.read_issues()?;

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
            issue.description = d;
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
            issue.owner = o;
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
        manager.commit_all(&format!("Update issue {}", id))?;
        if json {
            println!("{}", serde_json::to_string_pretty(&issues[idx])?);
        } else {
            println!("Updated issue {}", id);
        }
    } else if json {
        println!("{}", serde_json::to_string_pretty(&issues[idx])?);
    } else {
        println!("No changes provided for issue {}", id);
    }
    Ok(())
}
