use crate::commands::get_store;
use color_eyre::Result;
use pebble::config::Config;

pub fn run(config: &Config, json: bool) -> Result<()> {
    let (store, _, jsonl_path) = get_store(config)?;

    if !json {
        println!("Using database: {}", jsonl_path.display());
    }
    let issues = store.read_issues()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&issues)?);
    } else if issues.is_empty() {
        println!("No issues found.");
    } else {
        for issue in issues {
            println!("{} [{}] {}", issue.id, issue.status, issue.title);
        }
    }
    Ok(())
}
