use crate::commands::get_store;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::config::Config;

pub struct UpdateArgs {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub owner: Option<String>,
    pub issue_type: Option<String>,
    pub json: bool,
}

pub fn run(config: &Config, args: UpdateArgs) -> Result<()> {
    let (store, manager, _) = get_store(config)?;
    let mut issues = store.read_issues()?;

    let idx = issues
        .iter()
        .position(|i| i.id == args.id)
        .ok_or_else(|| eyre!("Issue {} not found", args.id))?;

    let mut changed = false;
    {
        let issue = &mut issues[idx];
        if let Some(t) = args.title {
            issue.title = t;
            changed = true;
        }
        if let Some(d) = args.description {
            issue.description = d;
            changed = true;
        }
        if let Some(s) = args.status {
            issue.status = s;
            changed = true;
        }
        if let Some(p) = args.priority {
            issue.priority = p;
            changed = true;
        }
        if let Some(o) = args.owner {
            issue.owner = o;
            changed = true;
        }
        if let Some(it) = args.issue_type {
            issue.issue_type = it;
            changed = true;
        }

        if changed {
            issue.updated_at = chrono::Local::now().to_rfc3339();
        }
    }

    if changed {
        store.write_issues(&issues)?;
        manager.commit_all(&format!("Update issue {}", args.id))?;
        if args.json {
            println!("{}", serde_json::to_string_pretty(&issues[idx])?);
        } else {
            println!("Updated issue {}", args.id);
        }
    } else if args.json {
        println!("{}", serde_json::to_string_pretty(&issues[idx])?);
    } else {
        println!("No changes provided for issue {}", args.id);
    }
    Ok(())
}
