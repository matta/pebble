use crate::commands::get_store;
use color_eyre::Result;
use pebble::config::Config;

pub fn run(config: &Config, query: String, json: bool) -> Result<()> {
    let (store, _, _) = get_store(config)?;
    let issues = store.read_issues()?;

    let query = query.to_lowercase();
    let matches: Vec<_> = issues
        .into_iter()
        .filter(|issue| {
            issue.title.to_lowercase().contains(&query)
                || issue.description.to_lowercase().contains(&query)
        })
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&matches)?);
    } else if matches.is_empty() {
        println!("No issues found matching '{}'", query);
    } else {
        for issue in matches {
            println!("{} [{}] {}", issue.id, issue.status, issue.title);
        }
    }

    Ok(())
}
