use crate::commands::get_store;
use color_eyre::Result;
use pebble::cli::OutputFormat;
use pebble::config::Config;

pub fn run(config: &Config, query: String, format: OutputFormat) -> Result<()> {
    let (store, _, _) = get_store(config)?;
    let issues = store.read_issues()?;

    let query = query.to_lowercase();
    let matches: Vec<_> = issues
        .into_iter()
        .filter(|issue| {
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
