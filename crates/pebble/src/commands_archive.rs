use crate::commands::RunContext;
use crate::graph::TaskGraph;
use color_eyre::eyre::{Result, eyre};
use std::fs;
use std::io::{ErrorKind, Write};
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
            let new_path = archive_task_atomically(&archive_dir, &node.path)?;

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

fn archive_task_atomically(archive_dir: &Path, original_path: &Path) -> Result<PathBuf> {
    // Read the file and metadata before attempting to create the destination.
    // This prevents creating empty files on read failures and ensures the entire file is valid.
    let metadata = fs::metadata(original_path)?;
    let content = fs::read(original_path)?;

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
        let filename = if extension.is_empty() {
            if counter == 1 {
                stem.to_string()
            } else {
                format!("{}-{}", stem, counter)
            }
        } else if counter == 1 {
            format!("{}.{}", stem, extension)
        } else {
            format!("{}-{}.{}", stem, counter, extension)
        };

        let new_path = archive_dir.join(&filename);

        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&new_path)
        {
            Ok(mut dest_file) => {
                dest_file.write_all(&content)?;

                // Attempt to preserve permissions. Ignore errors as some filesystems may not support it.
                let _ = fs::set_permissions(&new_path, metadata.permissions());

                // Use the filetime crate if it were available to copy timestamps,
                // but std::fs::set_permissions provides the most standard available metadata copy.

                fs::remove_file(original_path)?;
                return Ok(new_path);
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                counter += 1;
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod tests {
    use super::*;

    #[test]
    fn test_archive_task_atomically() {
        let temp_dir = tempfile::tempdir().expect("tempdir created");
        let tasks_dir = temp_dir.path().join("tasks");
        let archive_dir = temp_dir.path().join("archive");

        fs::create_dir_all(&tasks_dir).expect("created tasks dir");
        fs::create_dir_all(&archive_dir).expect("created archive dir");

        // 1. With extension, no collision
        let p1_source = tasks_dir.join("PROJ-1.md");
        fs::write(&p1_source, "content 1").expect("file written");
        let p1_archived =
            archive_task_atomically(&archive_dir, &p1_source).expect("archive successful");
        assert_eq!(p1_archived, archive_dir.join("PROJ-1.md"));
        assert_eq!(
            fs::read_to_string(p1_archived).expect("should read"),
            "content 1"
        );
        assert!(!p1_source.exists());

        // 2. Without extension, no collision
        let p2_source = tasks_dir.join("PROJ-2");
        fs::write(&p2_source, "content 2").expect("file written");
        let p2_archived =
            archive_task_atomically(&archive_dir, &p2_source).expect("archive successful");
        assert_eq!(p2_archived, archive_dir.join("PROJ-2"));
        assert_eq!(
            fs::read_to_string(p2_archived).expect("should read"),
            "content 2"
        );
        assert!(!p2_source.exists());

        // 3. With extension, with collision
        // (Re-create p1 in source, and we know PROJ-1.md exists in archive from step 1)
        let p1_source_2 = tasks_dir.join("PROJ-1.md");
        fs::write(&p1_source_2, "content 1b").expect("file written");
        let p1_archived_2 =
            archive_task_atomically(&archive_dir, &p1_source_2).expect("archive successful");
        assert_eq!(p1_archived_2, archive_dir.join("PROJ-1-2.md"));
        assert_eq!(
            fs::read_to_string(p1_archived_2).expect("should read"),
            "content 1b"
        );
        // Verify the original wasn't overwritten
        assert_eq!(
            fs::read_to_string(archive_dir.join("PROJ-1.md")).expect("should read"),
            "content 1"
        );
        assert!(!p1_source_2.exists());

        // 4. Without extension, with collision
        // (Re-create p2 in source, PROJ-2 exists in archive from step 2)
        let p2_source_2 = tasks_dir.join("PROJ-2");
        fs::write(&p2_source_2, "content 2b").expect("file written");
        let p2_archived_2 =
            archive_task_atomically(&archive_dir, &p2_source_2).expect("archive successful");
        assert_eq!(p2_archived_2, archive_dir.join("PROJ-2-2"));
        assert_eq!(
            fs::read_to_string(p2_archived_2).expect("should read"),
            "content 2b"
        );
        // Verify the original wasn't overwritten
        assert_eq!(
            fs::read_to_string(archive_dir.join("PROJ-2")).expect("should read"),
            "content 2"
        );
        assert!(!p2_source_2.exists());
    }
}
