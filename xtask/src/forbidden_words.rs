use crate::get_files_to_check;
use color_eyre::eyre::bail;
use color_eyre::Result;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn check_forbidden_words(
    all: bool,
    generate_whitelist: bool,
    minimize_whitelist: bool,
) -> Result<()> {
    let root = std::env::current_dir()?;
    let whitelist_path = root.join(".forbidden-word-whitelist");

    let scan_all = all || generate_whitelist || minimize_whitelist;
    let files = get_files_to_check(&root, scan_all)?;

    let whitelist = load_whitelist(&whitelist_path, generate_whitelist)?;
    let mut state = WhitelistState::new(whitelist, generate_whitelist);
    let forbidden = format!("{}ad", "be");
    let violations = scan_forbidden_words(&root, files, &forbidden, &mut state)?;

    let config = ForbiddenConfig {
        whitelist_path: &whitelist_path,
        scan_all,
        generate_whitelist,
        minimize_whitelist,
    };
    handle_forbidden_results(config, violations, &state)?;

    Ok(())
}

fn load_whitelist(path: &Path, generate_whitelist: bool) -> Result<HashSet<String>> {
    if path.exists() && !generate_whitelist {
        let content = fs::read_to_string(path)?;
        Ok(content.lines().map(canonicalize).collect())
    } else {
        Ok(HashSet::new())
    }
}

type Violation = (String, usize, String);
type Violations = Vec<Violation>;

struct ForbiddenConfig<'a> {
    whitelist_path: &'a Path,
    scan_all: bool,
    generate_whitelist: bool,
    minimize_whitelist: bool,
}

fn scan_forbidden_words(
    root: &Path,
    files: HashSet<String>,
    forbidden: &str,
    state: &mut WhitelistState,
) -> Result<Violations> {
    let mut violations: Violations = Vec::new();

    for file_path in files {
        let path = root.join(&file_path);
        if should_skip_forbidden_check(&path, &file_path) {
            continue;
        }

        let file = match fs::File::open(&path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };

            if process_line(&line, forbidden, state) {
                violations.push((file_path.clone(), line_num + 1, line.clone()));
            }
        }
    }

    Ok(violations)
}

fn should_skip_forbidden_check(path: &Path, relative_path: &str) -> bool {
    if !path.exists() || path.is_dir() {
        return true;
    }

    const IGNORED_FILES: &[&str] = &["crates/pebble/tests/fixtures/golden.jsonl"];
    let normalized_relative = relative_path.replace('\\', "/");
    if IGNORED_FILES
        .iter()
        .any(|ignored| *ignored == normalized_relative)
    {
        return true;
    }

    if path
        .file_name()
        .is_some_and(|name| name == ".forbidden-word-whitelist")
    {
        return true;
    }

    const BINARY_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "ico", "pdf", "bin"];
    path.extension()
        .is_some_and(|ext| BINARY_EXTENSIONS.iter().any(|&b| ext == b))
}

fn handle_forbidden_results(
    config: ForbiddenConfig<'_>,
    violations: Violations,
    state: &WhitelistState,
) -> Result<()> {
    if config.generate_whitelist {
        write_whitelist(
            config.whitelist_path,
            state.new_whitelist.clone(),
            "Generated",
        )?;
        return Ok(());
    }

    if config.minimize_whitelist {
        write_whitelist(
            config.whitelist_path,
            state.found_whitelisted.clone(),
            "Minimized",
        )?;
        return Ok(());
    }

    if !violations.is_empty() {
        println!("Found forbidden words in the following locations:");
        for (file, line, _) in violations {
            println!("{}:{}", file, line);
        }
        bail!("Found forbidden words. Please remove them or add the line(s) to .forbidden-word-whitelist if intended.");
    }

    if config.scan_all && state.found_whitelisted.len() < state.whitelist.len() {
        let unused: Vec<_> = state
            .whitelist
            .difference(&state.found_whitelisted)
            .collect();
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
    Ok(())
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

fn process_line(line: &str, forbidden: &str, state: &mut WhitelistState) -> bool {
    let lower_line = line.to_lowercase();
    if !lower_line.contains(forbidden) {
        return false;
    }

    let canonical = canonicalize(line);
    if state.generate_whitelist {
        state.new_whitelist.insert(canonical);
        return false;
    }

    if state.whitelist.contains(&canonical) {
        state.found_whitelisted.insert(canonical);
        return false;
    }

    true
}

struct WhitelistState {
    generate_whitelist: bool,
    whitelist: HashSet<String>,
    new_whitelist: HashSet<String>,
    found_whitelisted: HashSet<String>,
}

impl WhitelistState {
    fn new(whitelist: HashSet<String>, generate_whitelist: bool) -> Self {
        Self {
            generate_whitelist,
            whitelist,
            new_whitelist: HashSet::new(),
            found_whitelisted: HashSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_process_line() {
        let mut whitelist = HashSet::new();
        let entry = "foo baz bar";
        whitelist.insert(canonicalize(entry));

        let mut state = WhitelistState::new(whitelist, false);

        assert!(!process_line("foo baz bar", "baz", &mut state));
        assert!(!process_line("  foo   baz   bar  ", "baz", &mut state));
        assert!(!process_line("FOO BAZ BAR", "baz", &mut state));
        assert!(process_line("other baz", "baz", &mut state));
    }

    #[test]
    fn test_should_skip_forbidden_check_ignores_configured_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let relative = "crates/pebble/tests/fixtures/golden.jsonl";
        let file_path = temp_dir.path().join(relative);
        std::fs::create_dir_all(file_path.parent().expect("fixture parent"))
            .expect("create fixture dirs");
        std::fs::write(&file_path, "forbidden").expect("write fixture");

        assert!(should_skip_forbidden_check(&file_path, relative));
    }
}
