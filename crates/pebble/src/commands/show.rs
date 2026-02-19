use crate::commands::get_store;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::config::Config;

pub fn run(config: &Config, id: String, json: bool) -> Result<()> {
    let (store, _, _) = get_store(config)?;

    let issue = store
        .find_issue(&id)?
        .ok_or_else(|| eyre!("Issue {} not found", id))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&issue)?);
    } else {
        println!("ID:          {}", issue.id);
        println!("Status:      {}", issue.status);
        println!("Title:       {}", issue.title);
        println!("Type:        {}", issue.issue_type);
        println!("Priority:    {}", issue.priority);
        println!("Owner:       {}", issue.owner);
        println!("Created At:  {}", issue.created_at);
        println!("Created By:  {}", issue.created_by);
        println!("Updated At:  {}", issue.updated_at);
        if let Some(closed_at) = issue.closed_at {
            println!("Closed At:   {}", closed_at);
        }
        if let Some(reason) = issue.close_reason {
            println!("Close Reason: {}", reason);
        }
        if !issue.description.is_empty() {
            println!(
                "
Description:
{}",
                issue.description
            );
        }
    }
    Ok(())
}
