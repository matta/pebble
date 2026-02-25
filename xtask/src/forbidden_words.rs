use crate::get_files_to_check;
use color_eyre::Result;
use color_eyre::eyre::bail;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Scans tracked files for occurrences of the forbidden word.
///
/// When `generate_whitelist` is `true`, writes every found occurrence to
/// `.forbidden-word-whitelist` instead of failing. When `minimize_whitelist`
/// is `true`, rewrites the whitelist retaining only entries still present in
/// the codebase. Fails if any non-whitelisted occurrences are found.
pub fn check_forbidden_words(
    all: bool,
    generate_whitelist: bool,
    minimize_whitelist: bool,
) -> Result<()> {
    let root = env::current_dir()?;
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

/// Loads the forbidden-word whitelist from `path`, returning an empty set when regenerating.
fn load_whitelist(path: &Path, generate_whitelist: bool) -> Result<HashSet<String>> {
    if path.exists() && !generate_whitelist {
        let content = fs::read_to_string(path)?;
        Ok(content.lines().map(canonicalize).collect())
    } else {
        Ok(HashSet::new())
    }
}

/// A single forbidden-word hit: `(relative_file_path, 1-based_line_number, line_content)`.
type Violation = (String, usize, String);
/// Collected list of forbidden-word violations.
type Violations = Vec<Violation>;

/// Configuration bundle passed to [`handle_forbidden_results`].
struct ForbiddenConfig<'a> {
    /// Path to the `.forbidden-word-whitelist` file.
    whitelist_path: &'a Path,
    /// Whether the full file set (not just changed files) was scanned.
    scan_all: bool,
    /// Whether to write a fresh whitelist from found occurrences.
    generate_whitelist: bool,
    /// Whether to rewrite the whitelist retaining only still-present entries.
    minimize_whitelist: bool,
}

/// Walks `files`, returning every line that contains `forbidden` and is not whitelisted.
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

/// Returns `true` if a file should be excluded from forbidden-word scanning.
///
/// Skips non-existent paths, directories, explicitly ignored files, the
/// whitelist file itself, Markdown files, and known binary extensions.
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

    if path.extension().is_some_and(|ext| ext == "md") {
        return true;
    }

    const BINARY_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "ico", "pdf", "bin"];
    path.extension()
        .is_some_and(|ext| BINARY_EXTENSIONS.iter().any(|&b| ext == b))
}

/// Handles the outcome of a forbidden-word scan according to `config` mode.
///
/// In generate or minimize mode, writes the whitelist file. Otherwise, fails
/// on any violations and warns about stale whitelist entries when scanning all files.
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
        bail!(
            "Found forbidden words. Please remove them or add the line(s) to .forbidden-word-whitelist if intended."
        );
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
        bail!(
            "Whitelist contains unused entries. Run 'cargo xtask check-forbidden-words --minimize-whitelist' to clean it up."
        );
    }

    println!("No forbidden words found.");
    Ok(())
}

/// Normalizes a line to lowercase with runs of whitespace collapsed to single spaces.
fn canonicalize(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Writes `whitelist` entries to `path`, one per line, sorted lexicographically.
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

/// Returns `true` if `line` contains `forbidden` and is not covered by the whitelist.
///
/// In generate-whitelist mode, records the line instead of flagging it.
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

/// Mutable state accumulated during a forbidden-word scan.
struct WhitelistState {
    /// When `true`, all hits are collected into `new_whitelist` instead of reported.
    generate_whitelist: bool,
    /// The canonical entries loaded from the existing whitelist file.
    whitelist: HashSet<String>,
    /// Canonical entries collected when regenerating the whitelist.
    new_whitelist: HashSet<String>,
    /// Whitelist entries that were actually encountered during the scan.
    found_whitelisted: HashSet<String>,
}

impl WhitelistState {
    /// Creates a new `WhitelistState` with the given existing whitelist.
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
        fs::create_dir_all(file_path.parent().expect("fixture parent"))
            .expect("create fixture dirs");
        fs::write(&file_path, "forbidden").expect("write fixture");

        assert!(should_skip_forbidden_check(&file_path, relative));
    }
}
