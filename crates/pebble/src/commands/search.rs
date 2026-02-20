use crate::commands::{IssueFilters, get_store};
use color_eyre::Result;
use pebble::cli::OutputFormat;
use pebble::config::Config;

pub fn run(
    config: &Config,
    query: String,
    filters: IssueFilters,
    format: OutputFormat,
) -> Result<()> {
    let (store, _, _) = get_store(config)?;
    let issues = store.read_issues()?;

    let query = query.to_lowercase();
    let IssueFilters {
        status,
        owner,
        priority,
        issue_type,
    } = filters;
    let matches: Vec<_> = issues
        .into_iter()
        .filter(|issue| {
            if status
                .as_deref()
                .is_some_and(|s| issue.status.as_str() != s)
            {
                return false;
            }
            if owner
                .as_deref()
                .is_some_and(|o| issue.owner.as_deref() != Some(o))
            {
                return false;
            }
            if priority.is_some_and(|p| issue.priority != p) {
                return false;
            }
            if issue_type
                .as_deref()
                .is_some_and(|t| issue.issue_type.as_str() != t)
            {
                return false;
            }
            issue.title.to_lowercase().contains(&query)
                || issue
                    .description
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&query)
        })
        .collect();

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&matches)?);
        }
        OutputFormat::Human => {
            if matches.is_empty() {
                println!("No issues found matching '{}'", query);
            } else {
                for issue in matches {
                    println!("{} [{}] {}", issue.id, issue.status, issue.title);
                }
            }
        }
    }

    Ok(())
}
