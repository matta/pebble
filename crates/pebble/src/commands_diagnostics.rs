use crate::commands::RunContext;
use crate::graph::TaskGraph;
use color_eyre::eyre::Result;
use serde::Serialize;

/// A standard error shape emitted by diagnostics checks (suitable for JSON).
/// Contains a human-readable message, an identifier for the file (if applicable),
/// and an optional machine-readable error code.
#[derive(Serialize)]
pub struct DiagnosticError {
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Represents the JSON payload containing diagnostics results.
/// Includes an overall true/false health indicator and an array of individual issues.
#[derive(Serialize)]
pub struct DiagnosticsOutput {
    pub ok: bool,
    pub errors: Vec<DiagnosticError>,
}

/// Executes a strict graph check for `pebble check`.
///
/// By default this exits with an error status if any issues are found. With
/// `warn_only` enabled it still reports issues but always exits successfully.
pub fn run_check(ctx: &RunContext, warn_only: bool) -> Result<()> {
    let errors = collect_diagnostics(ctx)?;
    let ok = report_diagnostics(ctx, errors)?;

    if !ok && !warn_only {
        // We use a generic Result error here which will be handled by main/color_eyre
        // and usually results in exit code 1.
        return Err(color_eyre::eyre::eyre!("Check failed: graph has issues."));
    }

    Ok(())
}

/// Reports diagnostic results to the user based on the requested output format.
///
/// Returns `Ok(true)` if the graph is healthy (no issues found).
/// Returns `Ok(false)` if the graph has diagnostic issues (which are printed to stderr).
/// Returns `Err` only if an operational failure occurred (e.g., IO error or serialization failure)
/// that prevented the report from being generated.
fn report_diagnostics(ctx: &RunContext, errors: Vec<DiagnosticError>) -> Result<bool> {
    let ok = errors.is_empty();

    if ctx.json {
        let out = DiagnosticsOutput { ok, errors };
        println!("{}", serde_json::to_string(&out)?);
    } else if ok {
        println!("Graph is healthy. No issues found.");
    } else {
        eprintln!("Graph is unhealthy.");
        for err in &errors {
            eprintln!("{}: {}", err.file, err.message);
        }
        eprintln!("\nFound {} issue(s).", errors.len());
    }

    Ok(ok)
}

fn collect_diagnostics(ctx: &RunContext) -> Result<Vec<DiagnosticError>> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let mut errors = Vec::new();

    let current_dir = &ctx.current_dir;

    // 1. Check for duplicate IDs
    for id in &graph.duplicate_ids {
        errors.push(DiagnosticError {
            file: "<multiple files>".to_string(),
            line: None,
            message: format!("Duplicate task ID found: '{}'", id),
            code: Some("duplicate_id".to_string()),
        });
    }

    // 2. Iterate through loaded valid nodes and check dangling needs and extra keys.
    for node in graph.nodes.values() {
        let rel_path = node
            .path
            .strip_prefix(current_dir)
            .unwrap_or(&node.path)
            .display()
            .to_string();

        // Unknown keys
        for key in node.frontmatter.extra.keys() {
            errors.push(DiagnosticError {
                file: rel_path.clone(),
                line: None,
                message: format!("Unknown frontmatter key: '{}'", key),
                code: Some("unknown_key".to_string()),
            });
        }

        // Dangling references
        for need in &node.frontmatter.needs {
            if !graph.nodes.contains_key(need) {
                errors.push(DiagnosticError {
                    file: rel_path.clone(),
                    line: None,
                    message: format!("Dangling reference in 'needs': '{}' not found", need),
                    code: Some("dangling_need".to_string()),
                });
            }
        }
    }

    // 3. Cycle detection
    let scc_data = graph.compute_sccs();
    for scc in &scc_data.sccs {
        if scc_data.is_cycle(scc) {
            let mut cycle_ids = scc.clone();
            cycle_ids.sort();
            let message = format!("Dependency cycle detected: {}", cycle_ids.join(", "));

            for id in scc {
                if let Some(node) = graph.nodes.get(id) {
                    let rel_path = node
                        .path
                        .strip_prefix(current_dir)
                        .unwrap_or(&node.path)
                        .display()
                        .to_string();

                    errors.push(DiagnosticError {
                        file: rel_path,
                        line: None,
                        message: message.clone(),
                        code: Some("dependency_cycle".to_string()),
                    });
                }
            }
        }
    }

    // Sort to make testing easier
    errors.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));

    Ok(errors)
}
