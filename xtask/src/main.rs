use clap::{Parser, Subcommand};
use color_eyre::eyre::bail;
use color_eyre::Result;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

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
    /// Check for forbidden words like "bead" or "beads"
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
    /// Check for files that are too long
    CheckFileLength {
        /// Scan all tracked files instead of just edited ones
        #[arg(long)]
        all: bool,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    match cli.command {
        Commands::Check { all } => {
            check_forbidden_words(all, false, false)?;
            check_file_length(all)?;
            Ok(())
        }
        Commands::CheckForbiddenWords {
            all,
            generate_whitelist,
            minimize_whitelist,
        } => check_forbidden_words(all, generate_whitelist, minimize_whitelist),
        Commands::CheckFileLength { all } => check_file_length(all),
    }
}

fn get_files_to_check(root: &Path, all: bool) -> Result<HashSet<String>> {
    let mut files: HashSet<String> = get_git_files(root, &["ls-files"])?.into_iter().collect();
    // Also get untracked files
    files.extend(get_git_files(
        root,
        &["ls-files", "--others", "--exclude-standard"],
    )?);

    if !all {
        // Get both staged and unstaged changes
        let mut changed: HashSet<String> = get_git_files(root, &["diff", "--name-only", "HEAD"])?
            .into_iter()
            .collect();
        // Also get untracked files
        changed.extend(get_git_files(
            root,
            &["ls-files", "--others", "--exclude-standard"],
        )?);

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
        if path.extension().is_some_and(|ext| {
            ext == "png"
                || ext == "jpg"
                || ext == "jpeg"
                || ext == "gif"
                || ext == "ico"
                || ext == "pdf"
                || ext == "bin"
        }) {
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

        for (line_num, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue, // Skip binary or invalid utf8
            };

            if process_line(
                &line,
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
        let mut sorted_whitelist: Vec<_> = new_whitelist.into_iter().collect();
        sorted_whitelist.sort();
        fs::write(&whitelist_path, sorted_whitelist.join("\n"))?;
        println!("Generated whitelist at {:?}", whitelist_path);
    } else if minimize_whitelist {
        let mut sorted_whitelist: Vec<_> = found_whitelisted.into_iter().collect();
        sorted_whitelist.sort();
        fs::write(&whitelist_path, sorted_whitelist.join("\n"))?;
        println!("Minimized whitelist at {:?}", whitelist_path);
    } else {
        if !violations.is_empty() {
            println!("Found forbidden words 'bead' or 'beads' in the following locations:");
            for (file, line, content) in violations {
                println!("{}:{}: {}", file, line, content.trim());
            }
            bail!("Found forbidden words. Please remove them or add the line to .forbidden-word-whitelist if intended.");
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

fn check_file_length(all: bool) -> Result<()> {
    let root = std::env::current_dir()?;
    let files = get_files_to_check(&root, all)?;
    let mut violations = Vec::new();
    const MAX_LINES: usize = 500;

    for file_path in files {
        let path = root.join(&file_path);
        if !path.exists() || path.is_dir() {
            continue;
        }

        // Only check Rust files
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }

        let file = match fs::File::open(&path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let reader = BufReader::new(file);
        let line_count = reader.lines().count();

        if line_count > MAX_LINES {
            violations.push((file_path, line_count));
        }
    }

    if !violations.is_empty() {
        println!("The following Rust files exceed {} lines:", MAX_LINES);
        for (file, lines) in violations {
            println!("{}: {} lines", file, lines);
        }
        println!("\nSuggestions for corrective action:");
        println!(
            "- Split tests out into separate files (e.g., tests/ directory or separate module)."
        );
        println!("- Improve modularization by extracting large components into new modules.");
        println!("- Refactor long functions into smaller, more manageable pieces.");
        bail!("Files too long. Please refactor or split them.");
    } else {
        println!(
            "All Rust files are within the line limit ({} lines).",
            MAX_LINES
        );
    }

    Ok(())
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

fn process_line(
    line: &str,
    generate_whitelist: bool,
    whitelist: &HashSet<String>,
    new_whitelist: &mut HashSet<String>,
    found_whitelisted: &mut HashSet<String>,
) -> bool {
    let lower_line = line.to_lowercase();
    if lower_line.contains("bead") {
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
        let entry = "foo bead bar";
        whitelist.insert(canonicalize(entry));

        let mut new_whitelist = HashSet::new();
        let mut found_whitelisted = HashSet::new();

        // Exact match
        assert!(!process_line(
            "foo bead bar",
            false,
            &whitelist,
            &mut new_whitelist,
            &mut found_whitelisted
        ));

        // Whitespace mismatch
        assert!(!process_line(
            "  foo   bead   bar  ",
            false,
            &whitelist,
            &mut new_whitelist,
            &mut found_whitelisted
        ));

        // Case mismatch (should be whitelisted now)
        assert!(!process_line(
            "FOO BEAD BAR",
            false,
            &whitelist,
            &mut new_whitelist,
            &mut found_whitelisted
        ));

        // Violation
        assert!(process_line(
            "other bead",
            false,
            &whitelist,
            &mut new_whitelist,
            &mut found_whitelisted
        ));
    }
}
