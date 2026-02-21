use crate::commands::get_store;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use pebble::cli::{OutputFormat, UsageError};
use pebble::config::Config;
use pebble::store::Issue;

#[derive(Debug, Default)]
pub struct UpdateFields {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub owner: Option<String>,
    pub close_reason: Option<String>,
    pub issue_type: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub defer_until: Option<String>,
    pub add_labels: Vec<String>,
    pub remove_labels: Vec<String>,
    pub add_notes: Vec<String>,
}

pub fn run(config: &Config, id: String, fields: UpdateFields, format: OutputFormat) -> Result<()> {
    let (store, manager, _) = get_store(config)?;
    let mut issues = store.read_issues()?;

    let idx = issues
        .iter()
        .position(|i| i.id == id)
        .ok_or_else(|| eyre!("Issue {} not found", id))?;

    let changed = {
        let issue = &mut issues[idx];
        apply_updates(issue, fields)?
    };

    if changed {
        store.write_issues(&issues)?;
        let commit_message = format!("Update issue {}", id);
        match format {
            OutputFormat::Json => {
                manager.commit_all_quiet(&commit_message)?;
                println!("{}", serde_json::to_string_pretty(&issues[idx])?);
            }
            OutputFormat::Human => {
                manager.commit_all(&commit_message)?;
                println!("Updated issue {}", id);
            }
        }
    } else {
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&issues[idx])?);
            }
            OutputFormat::Human => {
                println!("No changes provided for issue {}", id);
            }
        }
    }
    Ok(())
}

fn apply_updates(issue: &mut Issue, fields: UpdateFields) -> Result<bool> {
    let UpdateFields {
        title,
        description,
        status,
        priority,
        owner,
        close_reason,
        issue_type,
        acceptance_criteria,
        defer_until,
        add_labels,
        remove_labels,
        add_notes,
    } = fields;

    let status_next = status.clone().unwrap_or_else(|| issue.status.clone());
    validate_close_reason(issue.status.as_str(), status_next.as_str(), &close_reason)?;

    let mut changed = apply_scalar_updates(
        issue,
        title,
        description,
        status,
        priority,
        owner,
        close_reason,
        issue_type,
        acceptance_criteria,
        defer_until,
    );

    if update_labels(issue, add_labels, remove_labels) {
        changed = true;
    }

    if !add_notes.is_empty() {
        issue.notes.extend(add_notes);
        changed = true;
    }

    let now = chrono::Local::now().to_rfc3339();
    if status_next == "closed" && issue.closed_at.is_none() {
        issue.closed_at = Some(now.clone());
        changed = true;
    } else if status_next != "closed" && issue.closed_at.is_some() {
        issue.closed_at = None;
        changed = true;
    }

    if changed {
        issue.updated_at = now;
    }

    Ok(changed)
}

#[allow(clippy::too_many_arguments)]
fn apply_scalar_updates(
    issue: &mut Issue,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<i32>,
    owner: Option<String>,
    close_reason: Option<String>,
    issue_type: Option<String>,
    acceptance_criteria: Option<String>,
    defer_until: Option<String>,
) -> bool {
    let mut changed = false;
    if let Some(t) = title {
        issue.title = t;
        changed = true;
    }
    if let Some(d) = description {
        issue.description = Some(d);
        changed = true;
    }
    if let Some(s) = status {
        issue.status = s;
        changed = true;
    }
    if let Some(p) = priority {
        issue.priority = p;
        changed = true;
    }
    if let Some(o) = owner {
        issue.owner = Some(o);
        changed = true;
    }
    if let Some(reason) = close_reason {
        issue.close_reason = Some(reason);
        changed = true;
    }
    if let Some(it) = issue_type {
        issue.issue_type = it;
        changed = true;
    }
    if let Some(ac) = acceptance_criteria {
        issue.acceptance_criteria = Some(ac);
        changed = true;
    }
    if let Some(du) = defer_until {
        issue.defer_until = Some(du);
        changed = true;
    }
    changed
}

fn update_labels(issue: &mut Issue, add_labels: Vec<String>, remove_labels: Vec<String>) -> bool {
    let mut changed = false;
    if !add_labels.is_empty() {
        let mut labels_set: std::collections::HashSet<_> = issue.labels.drain(..).collect();
        for label in add_labels {
            labels_set.insert(label);
        }
        issue.labels = labels_set.into_iter().collect();
        issue.labels.sort();
        changed = true;
    }

    if !remove_labels.is_empty() {
        let original_len = issue.labels.len();
        issue.labels.retain(|l| !remove_labels.contains(l));
        if issue.labels.len() != original_len {
            changed = true;
        }
    }
    changed
}

fn validate_close_reason(
    current_status: &str,
    status_next: &str,
    close_reason: &Option<String>,
) -> Result<()> {
    let closing_transition = status_next == "closed" && current_status != "closed";

    if close_reason.is_some() && status_next != "closed" {
        return Err(UsageError::new("close_reason requires status 'closed'").into());
    }
    if close_reason.is_none() && closing_transition {
        return Err(UsageError::new("close_reason is required when closing an issue").into());
    }

    Ok(())
}
