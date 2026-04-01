use crate::commands::RunContext;
use crate::graph::TaskGraph;
use color_eyre::eyre::{Result, eyre};
use std::fs;
use std::io::{ErrorKind, copy};
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
            let new_path = atomic_move_to_archive(&archive_dir, &node.path)?;

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

fn atomic_move_to_archive(archive_dir: &Path, original_path: &Path) -> Result<PathBuf> {
    let stem = original_path
        .file_stem()
        .ok_or_else(|| eyre!("Invalid task path: {}", original_path.display()))?
        .to_string_lossy();
    let extension = original_path
        .extension()
        .map(|e| e.to_string_lossy())
        .unwrap_or_default();

    let mut counter = 1;
    loop {
        let filename = if counter == 1 {
            if extension.is_empty() {
                stem.to_string()
            } else {
                format!("{}.{}", stem, extension)
            }
        } else if extension.is_empty() {
            format!("{}-{}", stem, counter)
        } else {
            format!("{}-{}.{}", stem, counter, extension)
        };

        let new_path = archive_dir.join(&filename);

        match fs::hard_link(original_path, &new_path) {
            Ok(_) => {
                fs::remove_file(original_path)?;
                return Ok(new_path);
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                counter += 1;
            }
            Err(_e) => {
                // If hard link fails for another reason (e.g., cross-device link or not supported),
                // fallback to atomic file creation and manual copy.
                match fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&new_path)
                {
                    Ok(mut dest_file) => {
                        let mut src_file = fs::File::open(original_path)?;
                        copy(&mut src_file, &mut dest_file)?;
                        // Ensure all data is written before removing source
                        dest_file.sync_all()?;
                        fs::remove_file(original_path)?;
                        return Ok(new_path);
                    }
                    Err(create_err) if create_err.kind() == ErrorKind::AlreadyExists => {
                        counter += 1;
                    }
                    Err(create_err) => {
                        return Err(create_err.into());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_atomic_move_to_archive() -> Result<()> {
        let temp_dir = tempdir()?;
        let archive_dir = temp_dir.path().join("archive");
        fs::create_dir_all(&archive_dir)?;

        // 1. With extension, no collision
        let p1 = temp_dir.path().join("PROJ-1.md");
        File::create(&p1)?;
        let res1 = atomic_move_to_archive(&archive_dir, &p1)?;
        assert_eq!(res1, archive_dir.join("PROJ-1.md"));
        assert!(!p1.exists());
        assert!(res1.exists());

        // 2. Without extension, no collision
        let p2 = temp_dir.path().join("PROJ-2");
        File::create(&p2)?;
        let res2 = atomic_move_to_archive(&archive_dir, &p2)?;
        assert_eq!(res2, archive_dir.join("PROJ-2"));
        assert!(!p2.exists());
        assert!(res2.exists());

        // 3. With extension, with collision
        let p3 = temp_dir.path().join("PROJ-3.md");
        File::create(&p3)?;
        // Create the collision target manually
        File::create(archive_dir.join("PROJ-3.md"))?;

        let res3 = atomic_move_to_archive(&archive_dir, &p3)?;
        assert_eq!(res3, archive_dir.join("PROJ-3-2.md"));
        assert!(!p3.exists());
        assert!(res3.exists());

        // 4. Without extension, with collision
        let p4 = temp_dir.path().join("PROJ-4");
        File::create(&p4)?;
        // Create the collision target manually
        File::create(archive_dir.join("PROJ-4"))?;

        let res4 = atomic_move_to_archive(&archive_dir, &p4)?;
        assert_eq!(res4, archive_dir.join("PROJ-4-2"));
        assert!(!p4.exists());
        assert!(res4.exists());

        Ok(())
    }
}
