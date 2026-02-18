use crate::commands::{get_git_config, get_store};
use color_eyre::Result;
use pebble::config::Config;
use rand::RngExt;

pub fn run(config: &Config, title: String, description: Option<String>) -> Result<()> {
    let (store, manager, _) = get_store(config)?;

    let prefix = config.issue_prefix.as_deref().unwrap_or("issue");

    let mut id;
    loop {
        let suffix: String = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(6)
            .map(char::from)
            .collect::<String>()
            .to_lowercase();
        id = format!("{}-{}", prefix, suffix);

        // Optimization: Use issue_exists to check for collision without loading all issues
        if !store.issue_exists(&id)? {
            break;
        }
    }

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
    manager.commit_all(&format!("Add issue {}", id))?;
    println!("Added issue {}", id);
    Ok(())
}
