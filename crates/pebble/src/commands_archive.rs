use crate::commands::RunContext;
use crate::graph::TaskGraph;
use color_eyre::eyre::{Result, eyre};
use std::fs;
use std::io::{self, ErrorKind};
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
            let (new_path, mut file) = get_archive_path(&archive_dir, &node.path, |p| {
                fs::OpenOptions::new().write(true).create_new(true).open(p)
            })?;

            let mut original_file = fs::File::open(&node.path)?;
            io::copy(&mut original_file, &mut file)?;
            fs::remove_file(&node.path)?;

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

fn get_archive_path(
    archive_dir: &Path,
    original_path: &Path,
    mut try_create: impl FnMut(&Path) -> io::Result<fs::File>,
) -> Result<(PathBuf, fs::File)> {
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
        match try_create(&new_path) {
            Ok(file) => return Ok((new_path, file)),
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
#[expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::path::PathBuf;

    #[test]
    fn test_get_archive_path() {
        let archive_dir = PathBuf::from("archive");
        let mock_fs = RefCell::new(HashSet::new());

        let mut mock_create = |p: &Path| -> io::Result<fs::File> {
            if mock_fs.borrow().contains(p) {
                Err(io::Error::from(ErrorKind::AlreadyExists))
            } else {
                tempfile::tempfile()
            }
        };

        // 1. With extension, no collision
        let p1 = PathBuf::from("PROJ-1.md");
        assert_eq!(
            get_archive_path(&archive_dir, &p1, &mut mock_create)
                .expect("archive path should be generated")
                .0,
            archive_dir.join("PROJ-1.md")
        );

        // 2. Without extension, no collision
        let p2 = PathBuf::from("PROJ-2");
        assert_eq!(
            get_archive_path(&archive_dir, &p2, &mut mock_create)
                .expect("archive path should be generated")
                .0,
            archive_dir.join("PROJ-2")
        );

        // 3. With extension, with collision
        mock_fs.borrow_mut().insert(archive_dir.join("PROJ-1.md"));
        assert_eq!(
            get_archive_path(&archive_dir, &p1, &mut mock_create)
                .expect("archive path should be generated")
                .0,
            archive_dir.join("PROJ-1-2.md")
        );

        // 4. Without extension, with collision
        mock_fs.borrow_mut().insert(archive_dir.join("PROJ-2"));
        assert_eq!(
            get_archive_path(&archive_dir, &p2, &mut mock_create)
                .expect("archive path should be generated")
                .0,
            archive_dir.join("PROJ-2-2")
        );
    }
}
