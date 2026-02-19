use crate::commands::get_store;
use color_eyre::Result;
use pebble::cli::OutputFormat;
use pebble::config::Config;

pub fn run(
    config: &Config,
    status: Option<String>,
    owner: Option<String>,
    priority: Option<i32>,
    format: OutputFormat,
) -> Result<()> {
    let (store, _, jsonl_path) = get_store(config)?;

    let issues = store.read_issues()?;

    let filtered_issues: Vec<_> = issues
        .into_iter()
        .filter(|issue| {
            if let Some(s) = &status {
                if &issue.status != s {
                    return false;
                }
            }
            if let Some(o) = &owner {
                if issue.owner.as_deref() != Some(o.as_str()) {
                    return false;
                }
            }
            if let Some(p) = priority {
                if issue.priority != p {
                    return false;
                }
            }
            true
        })
        .collect();

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&filtered_issues)?);
        }
        OutputFormat::Human => {
            eprintln!("Using database: {}", jsonl_path.display());
            if filtered_issues.is_empty() {
                eprintln!("No issues found.");
            } else {
                for issue in filtered_issues {
                    println!("{} [{}] {}", issue.id, issue.status, issue.title);
                }
            }
        }
    }
    Ok(())
}
