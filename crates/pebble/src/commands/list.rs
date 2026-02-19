use crate::commands::get_store;
use color_eyre::Result;
use pebble::cli::OutputFormat;
use pebble::config::Config;

pub fn run(config: &Config, format: OutputFormat) -> Result<()> {
    let (store, _, jsonl_path) = get_store(config)?;

    let issues = store.read_issues()?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&issues)?);
        }
        OutputFormat::Human => {
            eprintln!("Using database: {}", jsonl_path.display());
            if issues.is_empty() {
                eprintln!("No issues found.");
            } else {
                for issue in issues {
                    println!("{} [{}] {}", issue.id, issue.status, issue.title);
                }
            }
        }
    }
    Ok(())
}
