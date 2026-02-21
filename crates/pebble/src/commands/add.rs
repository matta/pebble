use crate::commands::{get_git_config, get_store};
use color_eyre::Result;
use pebble::cli::OutputFormat;
use pebble::config::Config;
use pebble::id::generate_unique_id;

#[derive(Debug)]
pub struct AddOptions {
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i32,
    pub issue_type: String,
    pub owner: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub defer_until: Option<String>,
    pub labels: Vec<String>,
    pub notes: Vec<String>,
}

pub fn run(config: &Config, options: AddOptions, format: OutputFormat) -> Result<()> {
    let (store, manager, _) = get_store(config)?;

    let prefix = config.issue_prefix.as_deref().unwrap_or("issue");

    // TODO(matt): Keep add resilient to malformed JSONL by tolerating bad lines.
    let existing_ids = store.read_issue_ids()?;

    let suffix_length = pebble::recommended_id_length(existing_ids.len());
    let id = generate_unique_id(prefix, &existing_ids, suffix_length);

    let now = chrono::Local::now().to_rfc3339();
    let user_name = get_git_config("user.name").unwrap_or_else(|_| "unknown".to_string());
    let user_email = get_git_config("user.email").unwrap_or_else(|_| "unknown".to_string());

    let owner = options.owner.or(Some(user_email));

    let issue = pebble::store::Issue {
        id: id.clone(),
        title: options.title,
        description: options.description,
        status: options.status,
        priority: options.priority,
        issue_type: options.issue_type,
        owner,
        created_at: now.clone(),
        created_by: Some(user_name),
        updated_at: now,
        closed_at: None,
        close_reason: None,
        acceptance_criteria: options.acceptance_criteria,
        defer_until: options.defer_until,
        labels: options.labels,
        notes: options.notes,
        ..Default::default()
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
