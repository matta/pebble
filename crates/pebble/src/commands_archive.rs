use crate::commands::RunContext;
use crate::graph::TaskGraph;
use color_eyre::eyre::{Result, eyre};
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Safely moves a file from `src` to `dst` using atomic operations to prevent TOCTOU vulnerabilities.
/// Uses `fs::hard_link` followed by `fs::remove_file` when possible. If the destination already exists,
/// it returns an `AlreadyExists` error. If hard-linking fails for other reasons (like crossing devices),
/// it falls back to an atomic open-and-copy operation.
fn safe_move_file(src: &Path, dst: &Path) -> io::Result<()> {
    match fs::hard_link(src, dst) {
        Ok(_) => fs::remove_file(src),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Err(e),
        Err(_) => {
            // Fall back to atomic file creation and copy if hard link fails (e.g., across filesystems)
            let mut src_file = fs::File::open(src)?;
            let mut dst_file = OpenOptions::new().write(true).create_new(true).open(dst)?;
            io::copy(&mut src_file, &mut dst_file)?;
            fs::remove_file(src)
        }
    }
}

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
            let new_path = move_to_archive(&archive_dir, &node.path)?;

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

fn move_to_archive(archive_dir: &Path, original_path: &Path) -> Result<PathBuf> {
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
        match safe_move_file(original_path, &new_path) {
            Ok(_) => return Ok(new_path),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                filename = if extension.is_empty() {
                    format!("{}-{}", stem, counter)
                } else {
                    format!("{}-{}.{}", stem, counter, extension)
                };
                new_path = archive_dir.join(&filename);
                counter += 1;
            }
            Err(e) => return Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_to_archive() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let archive_dir = temp.path().join("archive");
        fs::create_dir_all(&archive_dir)?;

        let original_file = temp.path().join("PROJ-1.md");
        fs::write(&original_file, "content")?;

        // 1. First move should succeed without a suffix
        let moved_path = move_to_archive(&archive_dir, &original_file)?;
        assert_eq!(moved_path, archive_dir.join("PROJ-1.md"));
        assert!(moved_path.exists());
        assert!(!original_file.exists());

        // 2. Recreate original, move again (should get -2 suffix)
        fs::write(&original_file, "content 2")?;
        let moved_path2 = move_to_archive(&archive_dir, &original_file)?;
        assert_eq!(moved_path2, archive_dir.join("PROJ-1-2.md"));
        assert!(moved_path2.exists());
        assert!(!original_file.exists());

        // 3. Recreate original, move again (should get -3 suffix)
        fs::write(&original_file, "content 3")?;
        let moved_path3 = move_to_archive(&archive_dir, &original_file)?;
        assert_eq!(moved_path3, archive_dir.join("PROJ-1-3.md"));
        assert!(moved_path3.exists());
        assert!(!original_file.exists());

        Ok(())
    }
}
