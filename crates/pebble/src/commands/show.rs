use crate::commands::get_store;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::cli::OutputFormat;
use pebble::config::Config;

pub fn run(config: &Config, id: String, format: OutputFormat) -> Result<()> {
    let (store, _, _) = get_store(config)?;

    let issue = store
        .find_issue(&id)?
        .ok_or_else(|| eyre!("Issue {} not found", id))?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&issue)?);
        }
        OutputFormat::Human => {
            println!("ID:          {}", issue.id);
            println!("Status:      {}", issue.status);
            println!("Title:       {}", issue.title);
            println!("Type:        {}", issue.issue_type);
            println!("Priority:    {}", issue.priority);
            println!("Owner:       {}", issue.owner.as_deref().unwrap_or(""));
            println!("Created At:  {}", issue.created_at);
            println!("Created By:  {}", issue.created_by.as_deref().unwrap_or(""));
            println!("Updated At:  {}", issue.updated_at);
            if let Some(closed_at) = issue.closed_at {
                println!("Closed At:   {}", closed_at);
            }
            if let Some(reason) = issue.close_reason {
                println!("Close Reason: {}", reason);
            }
            if let Some(description) = issue.description.as_deref()
                && !description.is_empty()
            {
                println!(
                    "
Description:
{}",
                    description
                );
            }
        }
    }
    Ok(())
}
