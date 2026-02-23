use anyhow::Result as AnyhowResult;
use clap::{Parser, Subcommand};
use color_eyre::Result;
use color_eyre::eyre::bail;
use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

const DEFAULT_TOKEN_LIMIT: usize = 2500;

mod forbidden_words;
use forbidden_words::check_forbidden_words;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all checks
    Check {
        /// Scan all tracked files instead of just edited ones
        #[arg(long)]
        all: bool,
    },
    /// Check for forbidden words
    CheckForbiddenWords {
        /// Scan all tracked files instead of just edited ones
        #[arg(long)]
        all: bool,
        /// Generate the whitelist from current codebase
        #[arg(long)]
        generate_whitelist: bool,
        /// Remove lines from whitelist that are no longer found
        #[arg(long)]
        minimize_whitelist: bool,
    },
    /// Check for Rust files that are too large (token count)
    CheckRustTokenCount {
        /// Scan all tracked files instead of just edited ones
        #[arg(long)]
        all: bool,

        /// Set the maximum number of non-comment, non-whitespace tokens allowed
        #[arg(long, default_value_t = DEFAULT_TOKEN_LIMIT)]
        limit: usize,

        /// Just print the token counts for all files and exit
        #[arg(long)]
        print_counts: bool,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { all } => {
            check_forbidden_words(all, false, false)?;
            check_rust_token_count(all, DEFAULT_TOKEN_LIMIT, false)?;
            Ok(())
        }
        Commands::CheckForbiddenWords {
            all,
            generate_whitelist,
            minimize_whitelist,
        } => check_forbidden_words(all, generate_whitelist, minimize_whitelist),
        Commands::CheckRustTokenCount {
            all,
            limit,
            print_counts,
        } => check_rust_token_count(all, limit, print_counts),
    }
}

fn get_files_to_check(root: &Path, all: bool) -> Result<HashSet<String>> {
    let mut files: HashSet<String> = get_git_files(root, &["ls-files"])?.into_iter().collect();
    let untracked = get_git_files(root, &["ls-files", "--others", "--exclude-standard"])?;
    files.extend(untracked.clone());

    if !all {
        // Get both staged and unstaged changes
        let mut changed: HashSet<String> = get_git_files(root, &["diff", "--name-only", "HEAD"])?
            .into_iter()
            .collect();
        // Also get untracked files
        changed.extend(untracked);

        if !changed.is_empty() {
            return Ok(changed);
        }
    }

    Ok(files)
}

#[derive(Deserialize, Default)]
struct ExceptionsConfig {
    #[serde(default)]
    exceptions: Vec<String>,
}

fn check_rust_token_count(all: bool, limit: usize, print_counts: bool) -> Result<()> {
    let root = std::env::current_dir()?;
    let config_path = root.join(".rust-line-count-exceptions.toml");

    let exceptions = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let config: ExceptionsConfig = toml::from_str(&content).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to parse .rust-line-count-exceptions.toml: {}", e)
        })?;
        config
            .exceptions
            .into_iter()
            .map(|pattern| {
                Regex::new(&pattern)
                    .map_err(|e| color_eyre::eyre::eyre!("Invalid regex {}: {}", pattern, e))
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        Vec::new()
    };

    let files = get_files_to_check(&root, all)?;
    let mut violations = Vec::new();
    let mut max_count = 0;
    let mut max_file = String::new();

    for file_path in files {
        let path = root.join(&file_path);
        if !path.exists() || path.is_dir() {
            continue;
        }

        // Only check Rust files
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        if exceptions.iter().any(|re| re.is_match(&file_path)) {
            continue;
        }

        let count = count_tokens(&path).map_err(|e| {
            color_eyre::eyre::eyre!("Failed to count tokens in {}: {}", file_path, e)
        })?;
        if count > max_count {
            max_count = count;
            max_file = file_path.clone();
        }

        if print_counts {
            println!("{}: {}", file_path, count);
        }

        if count > limit {
            violations.push((file_path, count));
        }
    }

    if print_counts {
        println!("Max token count: {} (in {})", max_count, max_file);
        return Ok(());
    }

    if !violations.is_empty() {
        violations.sort_by(|a, b| b.1.cmp(&a.1));
        println!("The following Rust files exceed {} tokens:", limit);
        for (file, count) in violations {
            println!("{}: {} tokens", file, count);
        }
        println!("\nSuggestions for corrective action:");
        println!(
            "- Split tests out into separate files (e.g., tests/ directory or separate module)."
        );
        println!("- Improve modularization by extracting large components into new modules.");
        println!("- Refactor long functions into smaller, more manageable pieces.");
        bail!("Files too large. Please refactor or split them.");
    }

    println!(
        "All Rust files are within the token limit ({} tokens).",
        limit
    );
    Ok(())
}

fn count_tokens(path: &Path) -> AnyhowResult<usize> {
    let content = fs::read_to_string(path)?;
    let count = ra_ap_rustc_lexer::tokenize(&content, FrontmatterAllowed::Yes)
        .filter(|token| {
            !matches!(
                token.kind,
                TokenKind::LineComment { .. }
                    | TokenKind::BlockComment { .. }
                    | TokenKind::Whitespace
            )
        })
        .count();
    Ok(count)
}

fn get_git_files(root: &Path, args: &[&str]) -> Result<Vec<String>> {
    let output = Command::new("git").current_dir(root).args(args).output()?;

    if !output.status.success() {
        bail!("Git command failed: git {}", args.join(" "));
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test.rs");

        let code = r##"
            fn main() {
                // This is a comment
                let x = 1; /* This is also a comment */
                let s = "This is a string // with a comment inside";
            }
        "##;
        fs::write(&path, code).unwrap();

        let count = count_tokens(&path).unwrap();
        // Tokens:
        // 1: fn
        // 2: main
        // 3: (
        // 4: )
        // 5: {
        // 6: let
        // 7: x
        // 8: =
        // 9: 1
        // 10: ;
        // 11: let
        // 12: s
        // 13: =
        // 14: "This is a string // with a comment inside"
        // 15: ;
        // 16: }
        assert_eq!(count, 16);
    }
}
