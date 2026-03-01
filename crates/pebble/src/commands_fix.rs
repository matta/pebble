use crate::commands::RunContext;
use crate::commands_diagnostics::collect_diagnostics;
use crate::graph::TaskGraph;
use crate::task_io::current_task_time;
use color_eyre::eyre::{Result, eyre};

/// Executes repair behavior for `pebble check --fix`.
pub fn run_fix(ctx: &RunContext) -> Result<()> {
    let mut graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let mut modified_ids = Vec::new();
    let current_time = current_task_time();

    for node in graph.nodes.values_mut() {
        let mut modified = false;

        // Backfill missing created_at
        if node.frontmatter.created_at.is_none() {
            node.frontmatter.created_at = Some(current_time);
            modified = true;
        }

        if modified {
            node.write_to_disk()?;
            modified_ids.push(node.frontmatter.id.clone());
        }
    }

    modified_ids.sort();

    // Determine whether any findings remain after repairs.
    let errors = collect_diagnostics(ctx)?;
    let ok = errors.is_empty();

    if !ok {
        eprintln!("Graph is unhealthy.");
        for err in &errors {
            eprintln!("{}: {}", err.file, err.message);
        }
        eprintln!("\nFound {} issue(s).", errors.len());
    }

    if ctx.json {
        println!(
            "{}",
            serde_json::to_string(
                &serde_json::json!({ "ok": ok, "fixed_tasks": modified_ids, "errors": errors })
            )?
        );
    } else if modified_ids.is_empty() {
        println!("No repairs needed.");
    } else {
        println!("Fixed {} task(s).", modified_ids.len());
    }

    if ok {
        Ok(())
    } else {
        Err(eyre!("Check --fix failed: unresolved findings remain."))
    }
}
