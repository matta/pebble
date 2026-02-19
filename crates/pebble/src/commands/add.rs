use crate::commands::{get_git_config, get_store};
use color_eyre::Result;
use pebble::cli::OutputFormat;
use pebble::config::Config;
use pebble::id::generate_unique_id;

pub fn run(
    config: &Config,
    title: String,
    description: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let (store, manager, _) = get_store(config)?;

    let prefix = config.issue_prefix.as_deref().unwrap_or("issue");

    // TODO(matt): Keep add resilient to malformed JSONL by tolerating bad lines.
    let existing_ids = store.read_issue_ids()?;

    let suffix_length = pebble::recommended_id_length(existing_ids.len());
    let id = generate_unique_id(prefix, &existing_ids, suffix_length);

    let now = chrono::Local::now().to_rfc3339();
    let user_name = get_git_config("user.name").unwrap_or_else(|_| "unknown".to_string());
    let user_email = get_git_config("user.email").unwrap_or_else(|_| "unknown".to_string());

    let issue = pebble::store::Issue {
        id: id.clone(),
        title: title.clone(),
        description: description.clone().unwrap_or_default(),
        status: "open".to_string(),
        priority: 0,
        issue_type: "task".to_string(),
        owner: user_email,
        created_at: now.clone(),
        created_by: user_name,
        updated_at: now,
        closed_at: None,
        close_reason: None,
    };

    store.append_issue(&issue)?;
    let commit_message = format!("Add issue {}", id);
    match format {
        OutputFormat::Json => {
            manager.commit_all_quiet(&commit_message)?;
            println!("{}", serde_json::to_string_pretty(&issue)?);
        }
        OutputFormat::Human => {
            manager.commit_all(&commit_message)?;
            println!("Added issue {}", id);
        }
    }
    Ok(())
}
