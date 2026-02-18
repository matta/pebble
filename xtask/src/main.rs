use anyhow::Result as AnyhowResult;
use clap::{Parser, Subcommand};
use color_eyre::eyre::bail;
use color_eyre::Result;
use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

const DEFAULT_TOKEN_LIMIT: usize = 2500;

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

fn check_forbidden_words(
    all: bool,
    generate_whitelist: bool,
    minimize_whitelist: bool,
) -> Result<()> {
    let root = std::env::current_dir()?;
    let whitelist_path = root.join(".forbidden-word-whitelist");

    // Whitelist generation and minimization always scan all files
    let scan_all = all || generate_whitelist || minimize_whitelist;
    let files = get_files_to_check(&root, scan_all)?;

    let mut violations = Vec::new();
    let whitelist: HashSet<String> = if whitelist_path.exists() && !generate_whitelist {
        let content = fs::read_to_string(&whitelist_path)?;
        content.lines().map(canonicalize).collect()
    } else {
        HashSet::new()
    };

    let mut found_whitelisted = HashSet::new();
    let mut new_whitelist = HashSet::new();

    for file_path in files {
        let path = root.join(&file_path);
        if !path.exists() || path.is_dir() {
            continue;
        }

        // Skip the whitelist file itself and binary files (basic check)
        const BINARY_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "ico", "pdf", "bin"];
        if path
            .extension()
            .is_some_and(|ext| BINARY_EXTENSIONS.iter().any(|&b| ext == b))
        {
            continue;
        }

        if path
            .file_name()
            .is_some_and(|name| name == ".forbidden-word-whitelist")
        {
            continue;
        }

        let file = match fs::File::open(&path) {
            Ok(f) => f,
            Err(_) => continue, // Skip if cannot open
        };
        let reader = BufReader::new(file);
        let forbidden = format!("{}ad", "be");

        for (line_num, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue, // Skip binary or invalid utf8
            };

            if process_line(
                &line,
                &forbidden,
                generate_whitelist,
                &whitelist,
                &mut new_whitelist,
                &mut found_whitelisted,
            ) {
                violations.push((file_path.clone(), line_num + 1, line.clone()));
            }
        }
    }

    if generate_whitelist {
        write_whitelist(&whitelist_path, new_whitelist, "Generated")?;
    } else if minimize_whitelist {
        write_whitelist(&whitelist_path, found_whitelisted, "Minimized")?;
    } else {
        if !violations.is_empty() {
            println!("Found forbidden words in the following locations:");
            for (file, line, _) in violations {
                println!("{}:{}", file, line);
            }
            bail!("Found forbidden words. Please remove them or add the line(s) to .forbidden-word-whitelist if intended.");
        }

        if scan_all && found_whitelisted.len() < whitelist.len() {
            let unused: Vec<_> = whitelist.difference(&found_whitelisted).collect();
            println!(
                "Found {} unused lines in .forbidden-word-whitelist:",
                unused.len()
            );
            for line in unused {
                println!("  {}", line);
            }
            bail!("Whitelist contains unused entries. Run 'cargo xtask check-forbidden-words --minimize-whitelist' to clean it up.");
        }

        println!("No forbidden words found.");
    }

    Ok(())
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

fn canonicalize(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn write_whitelist(path: &Path, whitelist: HashSet<String>, label: &str) -> Result<()> {
    let mut sorted_whitelist: Vec<_> = whitelist.into_iter().collect();
    sorted_whitelist.sort();
    let mut content = sorted_whitelist.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    fs::write(path, content)?;
    println!("{} whitelist at {:?}", label, path);
    Ok(())
}

fn process_line(
    line: &str,
    forbidden: &str,
    generate_whitelist: bool,
    whitelist: &HashSet<String>,
    new_whitelist: &mut HashSet<String>,
    found_whitelisted: &mut HashSet<String>,
) -> bool {
    let lower_line = line.to_lowercase();
    if lower_line.contains(forbidden) {
        let canonical = canonicalize(line);
        if generate_whitelist {
            new_whitelist.insert(canonical);
            false
        } else if whitelist.contains(&canonical) {
            found_whitelisted.insert(canonical);
            false
        } else {
            true
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_line() {
        let mut whitelist = HashSet::new();
        let entry = "foo baz bar";
        whitelist.insert(canonicalize(entry));

        let mut new_whitelist = HashSet::new();
        let mut found_whitelisted = HashSet::new();

        // Exact match
        assert!(!process_line(
            "foo baz bar",
            "baz",
            false,
            &whitelist,
            &mut new_whitelist,
            &mut found_whitelisted
        ));

        // Whitespace mismatch
        assert!(!process_line(
            "  foo   baz   bar  ",
            "baz",
            false,
            &whitelist,
            &mut new_whitelist,
            &mut found_whitelisted
        ));

        // Case mismatch (should be whitelisted now)
        assert!(!process_line(
            "FOO BAZ BAR",
            "baz",
            false,
            &whitelist,
            &mut new_whitelist,
            &mut found_whitelisted
        ));

        // Violation
        assert!(process_line(
            "other baz",
            "baz",
            false,
            &whitelist,
            &mut new_whitelist,
            &mut found_whitelisted
        ));
    }

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
