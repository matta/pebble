use crate::commands::{RunContext, TaskObject};
use crate::graph::TaskGraph;
use crate::models::{TaskFrontmatter, TaskNode, TaskStatus};
use color_eyre::eyre::{Result, eyre};
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

/// Generate the current UTC time as a TOML-compatible datetime.
fn current_toml_time() -> Result<toml_datetime::Datetime> {
    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();
    // TODO: Use `chrono::Utc::now().into()` once `toml_datetime` implements `From<chrono::DateTime<Utc>>`.
    // Currently, it does not seem to be available in the version/feature set we are using.
    toml_datetime::Datetime::from_str(&now_str)
        .map_err(|e| eyre!("Failed to parse datetime for TOML: {}", e))
}

/// Alphabet for generating random ID suffixes: digits 0–9 and lowercase letters a–z.
const ID_ALPHABET: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

/// Calculate the required number of random characters in an ID to keep
/// collision probability below 1e-12. Uses the birthday paradox approximation:
/// P \approx n^2 / (2 * 36^L).
fn required_random_id_length(n: usize) -> usize {
    if n == 0 {
        return 8;
    }
    let n_f: f64 = n as f64;
    let target_prob: f64 = 1e-12;
    let alphabet_size: f64 = 36.0;

    let l: f64 = ((n_f * n_f) / (2.0 * target_prob)).ln() / alphabet_size.ln();
    l.ceil().max(8.0) as usize
}

/// Initializes a new Pebble project in the current directory.
///
/// Creates a `.pebble/` directory containing `config.toml` and `AGENTS.md`,
/// and ensures the configured tasks directory exists. Fails if the project is
/// already initialized.
pub fn run_init(
    cli_dir_override: Option<PathBuf>,
    issue_prefix: Option<String>,
    json: bool,
) -> Result<()> {
    let current_dir = env::current_dir()?;
    let pebble_dir = current_dir.join(".pebble");

    if pebble_dir.exists() {
        return Err(eyre!(
            "Project already initialized at {}",
            current_dir.display()
        ));
    }

    std::fs::create_dir_all(&pebble_dir)?;

    let prefix = issue_prefix.unwrap_or_else(|| crate::config::Config::default().issue_prefix);
    let tasks_dir_path = if let Some(dir) = cli_dir_override {
        if dir.is_absolute() {
            return Err(
                crate::models::UsageError("tasks-dir must be a relative path".to_string()).into(),
            );
        }
        dir
    } else {
        crate::config::Config::default().tasks_dir
    };

    let config_toml = format!(
        r#"issue-prefix = "{prefix}"
tasks-dir = "{tasks_dir_path}"
"#,
        tasks_dir_path = tasks_dir_path.display()
    );
    std::fs::write(pebble_dir.join("config.toml"), config_toml)?;

    let agents_md = r#"# Project: Pebble
See documentation for implementation details.
"#;
    std::fs::write(pebble_dir.join("AGENTS.md"), agents_md)?;

    // Create tasks directory
    std::fs::create_dir_all(current_dir.join(&tasks_dir_path))?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "status": "success",
                "project_root": current_dir.display().to_string(),
                "tasks_dir": tasks_dir_path.display().to_string(),
                "issue_prefix": prefix,
            })
        );
    } else {
        eprintln!("Initialized Pebble repository in {}", current_dir.display());
    }

    Ok(())
}

/// Maximum length for a generated slug, to stay well within filesystem limits.
const MAX_SLUG_LEN: usize = 80;

/// Generates a cross-platform safe filename slug from a task title.
///
/// Slugs are strictly restricted to lowercase alphanumeric characters,
/// dashes, and underscores. Any other characters are collapsed into single dashes.
/// The result is truncated to [`MAX_SLUG_LEN`] characters to avoid filesystem errors.
pub fn slugify(s: &str) -> String {
    let mut slug = String::with_capacity(s.len());
    let mut last_was_dash = false;

    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            last_was_dash = false;
        } else if c == '_' {
            slug.push('_');
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    slug.truncate(slug.trim_end_matches('-').len());
    if slug.len() > MAX_SLUG_LEN {
        slug.truncate(MAX_SLUG_LEN);
        slug.truncate(slug.trim_end_matches('-').len());
    }
    if slug.is_empty() {
        "task".to_string()
    } else {
        slug
    }
}

/// Creates a new task file in the tasks directory and prints the result.
///
/// Generates a unique ID using the configured `issue-prefix` and a 6-character
/// nanoid suffix, then writes a Markdown file with TOML frontmatter. If a file
/// with the same slug already exists, a numeric suffix is appended to the name.
/// Outputs JSON when `ctx.json` is set; otherwise prints a human-readable line to stderr.
#[allow(clippy::too_many_arguments)]
pub fn run_add(
    ctx: &RunContext,
    title: String,
    status: Option<TaskStatus>,
    priority: Option<u8>,
    body: Option<String>,
    needs: Vec<String>,
    tags: Vec<String>,
) -> Result<()> {
    let mut graph = TaskGraph::load_from_dir(&ctx.tasks_dir)
        .unwrap_or_else(|_| TaskGraph::new(Default::default()));
    let random_length = required_random_id_length(graph.nodes.len());
    let id_str = nanoid::nanoid!(random_length, ID_ALPHABET);
    let new_id = format!("{}-{}", ctx.config.issue_prefix, id_str);

    let parsed_status = status.unwrap_or(TaskStatus::Todo);

    let created_at = current_toml_time()?;

    let fm = TaskFrontmatter {
        id: new_id.clone(),
        title: title.clone(),
        status: parsed_status,
        priority,
        created_at,
        modified_at: None,
        resolved_at: None,
        needs,
        tags,
    };

    let fm_toml = toml::to_string(&fm)?;
    let body_text = body.unwrap_or_default();

    let content = format!("+++\n{}+++\n{}", fm_toml, body_text);

    std::fs::create_dir_all(&ctx.tasks_dir)?;

    let base_slug = slugify(&title);
    let mut filename = format!("{}.md", base_slug);
    let mut filepath = ctx.tasks_dir.join(&filename);
    let mut counter = 2;

    loop {
        match std::fs::File::create_new(&filepath) {
            Ok(mut file) => {
                file.write_all(content.as_bytes())?;
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                filename = format!("{}-{}.md", base_slug, counter);
                filepath = ctx.tasks_dir.join(&filename);
                counter += 1;
            }
            Err(e) => return Err(e.into()),
        }
    }

    let node = TaskNode {
        path: filepath,
        frontmatter: fm,
        body: body_text,
    };

    if ctx.json {
        graph
            .nodes
            .insert(node.frontmatter.id.clone(), node.clone());
        let obj = TaskObject::from_node(&node, &graph, &ctx.tasks_dir);
        println!("{}", serde_json::to_string(&obj)?);
    } else {
        eprintln!("Created task {} at {}", new_id, node.path.display());
    }

    Ok(())
}

/// Updates an existing task's metadata and/or body in place.
///
/// Reads the task identified by `id` from the graph, applies all supplied
/// mutations (title, status, priority, tags, needs, body), updates `modified_at`,
/// and rewrites the file. Transitioning to a terminal status sets `resolved_at`;
/// transitioning away from one clears it. Outputs JSON when `ctx.json` is set.
#[allow(clippy::too_many_arguments, clippy::cognitive_complexity)]
pub fn run_update(
    ctx: &RunContext,
    id: String,
    title: Option<String>,
    status: Option<TaskStatus>,
    priority: Option<u8>,
    clear_priority: bool,
    body: Option<String>,
    append_body: Option<String>,
    add_tags: Vec<String>,
    remove_tags: Vec<String>,
    add_needs: Vec<String>,
    remove_needs: Vec<String>,
) -> Result<()> {
    let mut graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    if graph.is_duplicate_id(&id) {
        return Err(eyre!(
            "Duplicate task ID '{}' found in multiple files; cannot safely target this ID.",
            id
        ));
    }
    let mut node = graph
        .nodes
        .remove(&id)
        .ok_or_else(|| eyre!("Task '{}' not found", id))?;

    if let Some(t) = title {
        node.frontmatter.title = t;
    }
    if let Some(new_status) = status {
        // Handle transitions
        if !node.frontmatter.status.is_closed() && new_status.is_closed() {
            node.frontmatter.resolved_at = Some(current_toml_time()?);
        } else if node.frontmatter.status.is_closed() && !new_status.is_closed() {
            node.frontmatter.resolved_at = None;
        }

        node.frontmatter.status = new_status;
    }
    if let Some(p) = priority {
        node.frontmatter.priority = Some(p);
    }
    if clear_priority {
        node.frontmatter.priority = None;
    }

    node.frontmatter.modified_at = Some(current_toml_time()?);

    let mut existing_tags: std::collections::HashSet<_> =
        node.frontmatter.tags.iter().cloned().collect();
    for t in add_tags {
        if existing_tags.insert(t.clone()) {
            node.frontmatter.tags.push(t);
        }
    }
    for t in remove_tags {
        node.frontmatter.tags.retain(|tag| tag != &t);
    }

    let mut existing_needs: std::collections::HashSet<_> =
        node.frontmatter.needs.iter().cloned().collect();
    for d in add_needs {
        if existing_needs.insert(d.clone()) {
            node.frontmatter.needs.push(d);
        }
    }
    for d in remove_needs {
        node.frontmatter.needs.retain(|dep| dep != &d);
    }

    if let Some(b) = body {
        node.body = b;
    } else if let Some(a) = append_body {
        if node.body.is_empty() {
            node.body = a;
        } else {
            node.body.push_str("\n\n");
            node.body.push_str(&a);
        }
    }

    let fm_toml = toml::to_string(&node.frontmatter)?;
    let content = format!("+++\n{}+++\n{}", fm_toml, node.body);
    std::fs::write(&node.path, content)?;

    if ctx.json {
        graph
            .nodes
            .insert(node.frontmatter.id.clone(), node.clone());
        let updated_graph = TaskGraph::new(graph.nodes);
        let obj = TaskObject::from_node(&node, &updated_graph, &ctx.tasks_dir);
        println!("{}", serde_json::to_string(&obj)?);
    } else {
        eprintln!("Updated task {}", node.frontmatter.id);
    }

    Ok(())
}

/// Moves completed or canceled tasks older than 30 days into an `archive/` subdirectory.
///
/// Reads the graph from the configured tasks directory, then moves any task whose
/// `resolved_at` timestamp is more than 30 days in the past. Outputs a JSON array
/// of moved tasks when `ctx.json` is set; otherwise prints each archived ID to stderr.
pub fn run_archive(ctx: &RunContext) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let archive_dir = ctx.tasks_dir.join("archive");
    std::fs::create_dir_all(&archive_dir)?;

    let now = chrono::Utc::now();
    let threshold_days = chrono::Duration::days(30);

    let mut archived = vec![];

    for node in graph.nodes.values() {
        if node.frontmatter.status.is_closed()
            && let Some(resolved_at_toml) = node.frontmatter.resolved_at
        {
            let resolved_at = chrono::DateTime::parse_from_rfc3339(&resolved_at_toml.to_string())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|e| eyre!("Failed to parse resolved_at from TOML: {}", e))?;

            if now.signed_duration_since(resolved_at) > threshold_days {
                let new_path = archive_dir.join(node.path.file_name().unwrap());
                std::fs::rename(&node.path, &new_path)?;

                if ctx.json {
                    archived.push(serde_json::json!({
                                "id": node.frontmatter.id,
                                "moved_to": new_path.strip_prefix(&ctx.tasks_dir).unwrap_or(&new_path).display().to_string()
                            }));
                } else {
                    eprintln!("Archived {}", node.frontmatter.id);
                }
            }
        }
    }

    if ctx.json {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({ "archived": archived }))?
        );
    }

    Ok(())
}

#[cfg(test)]
#[path = "commands_write_tests.rs"]
mod commands_write_tests;
