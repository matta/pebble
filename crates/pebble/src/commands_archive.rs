use crate::commands::RunContext;
use crate::graph::TaskGraph;
use color_eyre::eyre::{Result, eyre};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

fn safe_move_file(src: &Path, dst: &Path) -> io::Result<()> {
    match fs::hard_link(src, dst) {
        Ok(_) => {
            let _ = fs::remove_file(src);
            Ok(())
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Err(e),
        Err(_) => match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dst)
        {
            Ok(mut dst_file) => {
                let result = fs::File::open(src)
                    .and_then(|mut src_file| io::copy(&mut src_file, &mut dst_file));
                match result {
                    Ok(_) => {
                        let _ = fs::remove_file(src);
                        Ok(())
                    }
                    Err(e) => {
                        let _ = fs::remove_file(dst);
                        Err(e)
                    }
                }
            }
            Err(e) => Err(e),
        },
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
            let stem = node
                .path
                .file_stem()
                .ok_or_else(|| eyre!("Invalid task path: {}", node.path.display()))?
                .to_string_lossy();
            let extension = node
                .path
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
                match safe_move_file(&node.path, &new_path) {
                    Ok(_) => break,
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
