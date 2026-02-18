use crate::commands::{get_git_config, get_store};
use color_eyre::Result;
use pebble::config::Config;
use rand::RngExt;

/// Computes the required random ID length to maintain a collision probability
/// of less than 1 in 1 trillion (10^-12) for a given population size.
///
/// This assumes the Birthday Paradox applies: P ≈ k^2 / (2 * N)
/// where k = current_population, N = alphabet_size^length.
fn recommended_id_length(current_population: u64) -> usize {
    // If there is 0 or 1 item, a collision is impossible.
    // However, to maintain the safety margin for the *next* item
    // or simply to establish a baseline, we return 1.
    if current_population <= 1 {
        return 1;
    }

    // Alphabet: a-z (26) + 0-9 (10) = 36
    const ALPHABET_SIZE: f64 = 36.0;

    // Target safety: 1 in 1,000,000,000,000
    const TARGET_PROBABILITY: f64 = 1.0e-12;

    let k = current_population as f64;

    // Derived from the Birthday Paradox approximation:
    // P ≈ k^2 / (2 * N)
    //
    // Solving for N (Required Pool Size):
    // N ≈ k^2 / (2 * P)
    let required_pool_size = (k * k) / (2.0 * TARGET_PROBABILITY);

    // Solving for Length (L):
    // ALPHABET_SIZE^L = N
    // L = log_alphabet(N)
    let length = required_pool_size.log(ALPHABET_SIZE);

    length.ceil() as usize
}

pub fn run(config: &Config, title: String, description: Option<String>) -> Result<()> {
    let (store, manager, _) = get_store(config)?;

    let prefix = config.issue_prefix.as_deref().unwrap_or("issue");

    let existing_issues = store.read_issues()?;
    let existing_ids: std::collections::HashSet<&str> =
        existing_issues.iter().map(|i| i.id.as_str()).collect();

    let suffix_length = recommended_id_length(existing_issues.len() as u64);

    let mut id;
    loop {
        // rand::rng() returns ThreadRng which is cryptographically secure (ChaCha12)
        let suffix: String = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(suffix_length)
            .map(char::from)
            .collect::<String>()
            .to_lowercase();
        id = format!("{}-{}", prefix, suffix);

        if !existing_ids.contains(id.as_str()) {
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
