use crate::commands::RunContext;
use crate::graph::TaskGraph;
use color_eyre::eyre::{Result, eyre};
use std::fs;
use std::io;
use std::path::Path;

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
            let mut counter = 1;
            let new_path = loop {
                let filename = get_archive_filename(&node.path, counter)?;
                let path = archive_dir.join(&filename);
                match fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                {
                    Ok(mut file) => {
                        let mut original_file = fs::File::open(&node.path)?;
                        io::copy(&mut original_file, &mut file)?;
                        file.sync_all()?;
                        drop(file);
                        drop(original_file);
                        fs::remove_file(&node.path)?;
                        break path;
                    }
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                        counter += 1;
                        continue;
                    }
                    Err(e) => return Err(e.into()),
                }
            };

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

fn get_archive_filename(original_path: &Path, counter: u32) -> Result<String> {
    let stem = original_path
        .file_stem()
        .ok_or_else(|| eyre!("Invalid task path: {}", original_path.display()))?
        .to_string_lossy();
    let extension = original_path
        .extension()
        .map(|e| e.to_string_lossy())
        .unwrap_or_default();

    let filename = if counter <= 1 {
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

    Ok(filename)
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_archive_filename() {
        // 1. With extension, no collision
        let p1 = PathBuf::from("PROJ-1.md");
        assert_eq!(
            get_archive_filename(&p1, 1).expect("archive filename should be generated"),
            "PROJ-1.md".to_string()
        );

        // 2. Without extension, no collision
        let p2 = PathBuf::from("PROJ-2");
        assert_eq!(
            get_archive_filename(&p2, 1).expect("archive filename should be generated"),
            "PROJ-2".to_string()
        );

        // 3. With extension, with collision
        assert_eq!(
            get_archive_filename(&p1, 2).expect("archive filename should be generated"),
            "PROJ-1-2.md".to_string()
        );

        // 4. Without extension, with collision
        assert_eq!(
            get_archive_filename(&p2, 2).expect("archive filename should be generated"),
            "PROJ-2-2".to_string()
        );
    }
}
