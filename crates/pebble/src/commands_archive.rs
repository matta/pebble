use crate::commands::{RunContext, emit_json};
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
            let new_path = get_archive_path(&archive_dir, &node.path, |p| p.exists())?;
            fs::rename(&node.path, &new_path)?;

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
        emit_json(&serde_json::json!({ "archived": archived }))?;
    }

    Ok(())
}

fn get_archive_path(
    archive_dir: &Path,
    original_path: &Path,
    mut exists: impl FnMut(&Path) -> bool,
) -> Result<PathBuf> {
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

    while exists(&new_path) {
        filename = if extension.is_empty() {
            format!("{}-{}", stem, counter)
        } else {
            format!("{}-{}.{}", stem, counter, extension)
        };
        new_path = archive_dir.join(&filename);
        counter += 1;
    }

    Ok(new_path)
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::PathBuf;

    #[test]
    fn test_get_archive_path() {
        let archive_dir = PathBuf::from("archive");
        let mut mock_fs = HashSet::new();

        // 1. With extension, no collision
        let p1 = PathBuf::from("PROJ-1.md");
        assert_eq!(
            get_archive_path(&archive_dir, &p1, |p| mock_fs.contains(p))
                .expect("archive path should be generated"),
            archive_dir.join("PROJ-1.md")
        );

        // 2. Without extension, no collision
        let p2 = PathBuf::from("PROJ-2");
        assert_eq!(
            get_archive_path(&archive_dir, &p2, |p| mock_fs.contains(p))
                .expect("archive path should be generated"),
            archive_dir.join("PROJ-2")
        );

        // 3. With extension, with collision
        mock_fs.insert(archive_dir.join("PROJ-1.md"));
        assert_eq!(
            get_archive_path(&archive_dir, &p1, |p| mock_fs.contains(p))
                .expect("archive path should be generated"),
            archive_dir.join("PROJ-1-2.md")
        );

        // 4. Without extension, with collision
        mock_fs.insert(archive_dir.join("PROJ-2"));
        assert_eq!(
            get_archive_path(&archive_dir, &p2, |p| mock_fs.contains(p))
                .expect("archive path should be generated"),
            archive_dir.join("PROJ-2-2")
        );
    }
}
