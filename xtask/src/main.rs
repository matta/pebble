use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
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
    /// Check for forbidden words like "bead" or "beads"
    CheckBeads {
        /// Scan all tracked files instead of just edited ones
        #[arg(long)]
        all: bool,
        /// Generate the whitelist from current codebase
        #[arg(long)]
        generate_whitelist: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::CheckBeads {
            all,
            generate_whitelist,
        } => check_beads(all, generate_whitelist),
    }
}

fn check_beads(all: bool, generate_whitelist: bool) -> Result<()> {
    let root = std::env::current_dir()?;
    let whitelist_path = root.join(".bead-whitelist");

    let files: HashSet<String> = if all || generate_whitelist {
        get_git_files(&root, &["ls-files"])?.into_iter().collect()
    } else {
        // Get both staged and unstaged changes
        let mut files: HashSet<String> = get_git_files(&root, &["diff", "--name-only", "HEAD"])?
            .into_iter()
            .collect();
        // Also get untracked files
        files.extend(get_git_files(
            &root,
            &["ls-files", "--others", "--exclude-standard"],
        )?);

        if files.is_empty() {
            // If the repo is clean, default to checking all tracked files
            get_git_files(&root, &["ls-files"])?.into_iter().collect()
        } else {
            files
        }
    };

    let mut violations = Vec::new();
    let whitelist = if whitelist_path.exists() && !generate_whitelist {
        let content = fs::read_to_string(&whitelist_path)?;
        content.lines().map(|l| l.trim().to_string()).collect()
    } else {
        HashSet::new()
    };

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
            .is_some_and(|name| name == ".bead-whitelist")
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

            // Check for "bead" or "beads" case-insensitive
            let lower_line = line.to_lowercase();
            if lower_line.contains("bead") {
                let trimmed = line.trim().to_string();
                if generate_whitelist {
                    new_whitelist.insert(trimmed);
                } else if !whitelist.contains(&trimmed) {
                    violations.push((file_path.clone(), line_num + 1, line.clone()));
                }
            }
        }
    }

    if generate_whitelist {
        let mut sorted_whitelist: Vec<_> = new_whitelist.into_iter().collect();
        sorted_whitelist.sort();
        fs::write(&whitelist_path, sorted_whitelist.join("\n"))?;
        println!("Generated whitelist at {:?}", whitelist_path);
    } else if !violations.is_empty() {
        println!("Found forbidden words 'bead' or 'beads' in the following locations:");
        for (file, line, content) in violations {
            println!("{}:{}: {}", file, line, content.trim());
        }
        bail!("Found forbidden words. Please remove them or add the line to .bead-whitelist if intended.");
    } else {
        println!("No forbidden words found.");
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
