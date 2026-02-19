use crate::commands::get_store;
use color_eyre::Result;
use pebble::config::Config;

pub fn run(
    config: &Config,
    status: Option<String>,
    owner: Option<String>,
    priority: Option<i32>,
    json: bool,
) -> Result<()> {
    let (store, _, jsonl_path) = get_store(config)?;

    if !json {
        eprintln!("Using database: {}", jsonl_path.display());
    }
    let issues = store.read_issues()?;

    let filtered_issues: Vec<_> = issues
        .into_iter()
        .filter(|issue| {
            if status.as_ref().is_some_and(|s| &issue.status != s) {
                return false;
            }
            if owner.as_ref().is_some_and(|o| &issue.owner != o) {
                return false;
            }
            if priority.is_some_and(|p| issue.priority != p) {
                return false;
            }
            true
        })
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&filtered_issues)?);
    } else if filtered_issues.is_empty() {
        eprintln!("No issues found.");
    } else {
        for issue in filtered_issues {
            println!("{} [{}] {}", issue.id, issue.status, issue.title);
        }
    }
    Ok(())
}
