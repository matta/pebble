use crate::commands::RunContext;
use crate::graph::TaskGraph;
use color_eyre::eyre::{Result, eyre};
use std::fs;
use std::path::{Path, PathBuf};

/// Moves completed or canceled tasks older than the configured threshold into an `archive/` subdirectory.
///
/// Reads the graph from the configured tasks directory, then moves any task whose
/// `resolved_at` timestamp is more than the configured threshold in the past.
/// Outputs a JSON array of moved tasks when `ctx.json` is set; otherwise prints each archived ID to stderr.
pub fn run_archive(ctx: &RunContext) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let archive_dir = ctx.tasks_dir.join("archive");
    fs::create_dir_all(&archive_dir)?;

    let now = chrono::Utc::now();
    let threshold_days = chrono::Duration::days(ctx.config.archive_threshold_days);

    let mut archived = vec![];

    for node in graph.nodes.values() {
        if node.frontmatter.status.is_closed()
            && let Some(resolved_at) = node.frontmatter.resolved_at
            && now.signed_duration_since(resolved_at) >= threshold_days
        {
            let new_path = safe_rename(&node.path, &archive_dir)?;

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

    if ctx.json {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({ "archived": archived }))?
        );
    }

    Ok(())
}

fn safe_rename(original_path: &Path, archive_dir: &Path) -> Result<PathBuf> {
    let stem = original_path
        .file_stem()
        .ok_or_else(|| eyre!("Invalid task path: {}", original_path.display()))?
        .to_string_lossy();
    let extension = original_path
        .extension()
        .map(|e| e.to_string_lossy())
        .unwrap_or_default();

    let mut filename = if extension.is_empty() {
        stem.to_string()
    } else {
        format!("{}.{}", stem, extension)
    };
    let mut new_path = archive_dir.join(&filename);
    let mut counter = 2;

    loop {
        match fs::hard_link(original_path, &new_path) {
            Ok(_) => {
                fs::remove_file(original_path)?;
                return Ok(new_path);
            }
            Err(e) => {
                use std::io::ErrorKind;
                // If hard_link is unsupported by the filesystem (e.g. FAT32), fallback to a non-atomic rename
                // assuming that the probability of a TOCTOU collision is low in such environments.
                if e.kind() == ErrorKind::Unsupported || e.kind() == ErrorKind::InvalidInput {
                    if !new_path.exists() {
                        fs::rename(original_path, &new_path)?;
                        return Ok(new_path);
                    }
                    // Fall through to collision logic if exists
                } else if e.kind() != ErrorKind::AlreadyExists {
                    return Err(e.into());
                }

                // Collision occurred
                filename = if extension.is_empty() {
                    format!("{}-{}", stem, counter)
                } else {
                    format!("{}-{}.{}", stem, counter, extension)
                };
                new_path = archive_dir.join(&filename);
                counter += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_safe_rename_paths() -> Result<()> {
        let temp = tempdir()?;
        let source_dir = temp.path().join("source");
        let archive_dir = temp.path().join("archive");
        fs::create_dir_all(&source_dir)?;
        fs::create_dir_all(&archive_dir)?;

        let original_file = source_dir.join("PROJ-1.md");
        fs::write(&original_file, "content")?;

        // 1. No collision
        let new_path = safe_rename(&original_file, &archive_dir)?;
        assert_eq!(new_path, archive_dir.join("PROJ-1.md"));
        assert!(!original_file.exists());
        assert!(new_path.exists());

        // Setup for collision test
        let original_file = source_dir.join("PROJ-1.md");
        fs::write(&original_file, "content2")?;

        // 2. Collision with "PROJ-1.md"
        let new_path2 = safe_rename(&original_file, &archive_dir)?;
        assert_eq!(new_path2, archive_dir.join("PROJ-1-2.md"));
        assert!(!original_file.exists());
        assert!(new_path2.exists());

        // 3. Without extension
        let original_file_no_ext = source_dir.join("PROJ-2");
        fs::write(&original_file_no_ext, "content3")?;
        let new_path3 = safe_rename(&original_file_no_ext, &archive_dir)?;
        assert_eq!(new_path3, archive_dir.join("PROJ-2"));

        // 4. Without extension collision
        let original_file_no_ext = source_dir.join("PROJ-2");
        fs::write(&original_file_no_ext, "content4")?;
        let new_path4 = safe_rename(&original_file_no_ext, &archive_dir)?;
        assert_eq!(new_path4, archive_dir.join("PROJ-2-2"));

        Ok(())
    }
}
