pub mod command;
pub mod config;
pub mod store;
pub mod worktree;

pub const CONFIG_DIR: &str = ".pebble";
pub const CONFIG_FILE: &str = "config.toml";
pub const ISSUES_FILE: &str = "issues.jsonl";
pub const WORKTREE_DIR: &str = ".git/x-pebble";

/// Computes the required random ID length to maintain a collision probability
/// of less than 1 in 1 trillion (10^-12) for a given population size.
///
/// This assumes the Birthday Paradox applies: P ≈ k^2 / (2 * N)
/// where k = current_population, N = alphabet_size^length.
pub fn recommended_id_length(current_population: usize) -> usize {
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
