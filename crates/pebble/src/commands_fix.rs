use crate::commands::RunContext;
use crate::graph::collect_markdown_files;
use crate::parser::split_frontmatter;
use chrono::Utc;
use color_eyre::eyre::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::str::FromStr;
use toml::Table;
use toml_datetime::Datetime;

pub fn run_fix(ctx: &RunContext) -> Result<()> {
    let tasks_dir = &ctx.tasks_dir;
    if !tasks_dir.exists() {
        // Nothing to fix if dir doesn't exist
        return Ok(());
    }

    let paths = collect_markdown_files(tasks_dir)?;

    let valid_keys: HashSet<&'static str> = [
        "id",
        "title",
        "status",
        "priority",
        "created_at",
        "modified_at",
        "resolved_at",
        "needs",
        "tags",
    ]
    .into_iter()
    .collect();

    for path in paths {
        let content = fs::read_to_string(&path)
            .wrap_err_with(|| format!("Failed to read file: {:?}", path))?;

        // 1. Split frontmatter and body
        let (toml_str, body) = match split_frontmatter(&content) {
            Ok((t, b)) => (t, b),
            Err(e) => {
                eprintln!(
                    "Skipping file {:?} due to parsing error: {}",
                    path.strip_prefix(tasks_dir).unwrap_or(&path),
                    e
                );
                continue;
            }
        };

        // 2. Parse into TOML Table
        let mut table: Table = match toml::from_str(&toml_str) {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "Skipping file {:?} due to invalid TOML: {}",
                    path.strip_prefix(tasks_dir).unwrap_or(&path),
                    e
                );
                continue;
            }
        };

        // 3. Check for unknown keys and warn
        for key in table.keys() {
            if !valid_keys.contains(key.as_str()) {
                eprintln!(
                    "Warning: Unknown frontmatter key '{}' in {:?}",
                    key,
                    path.strip_prefix(tasks_dir).unwrap_or(&path)
                );
            }
        }

        // 4. Backfill created_at if missing
        let mut changed = false;
        if !table.contains_key("created_at") {
            let now = Utc::now().to_rfc3339();
            #[allow(clippy::expect_used)]
            let datetime =
                Datetime::from_str(&now).expect("RFC3339 string should be valid Datetime");
            table.insert("created_at".to_string(), toml::Value::Datetime(datetime));
            changed = true;
        }

        // 5. Rewrite file if needed
        // Even if only formatted differently (e.g. key sort order), we rewrite to enforce deterministic formatting.
        let new_toml_str = toml::to_string_pretty(&table)?;

        // Reconstruct content
        let mut new_content = format!("+++\n{}+++\n{}", new_toml_str, body);
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }

        // Check if content actually changed
        if content != new_content {
            fs::write(&path, new_content)?;
            if changed {
                // Only log if we did a semantic fix
                eprintln!(
                    "Fixed file: {:?}",
                    path.strip_prefix(tasks_dir).unwrap_or(&path)
                );
            }
        }
    }

    Ok(())
}
