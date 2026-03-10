use crate::commands::RunContext;
use crate::graph::TaskGraph;
use crate::models::TaskNode;
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

pub(crate) fn collect_diagnostics(ctx: &RunContext) -> Result<Vec<DiagnosticError>> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let mut errors = Vec::new();

    // 1. Check for duplicate IDs
    check_duplicate_ids(&graph, &mut errors);

    // 2. Iterate through loaded valid nodes and check dangling needs, unknown keys, and canonicalization.
    for node in graph.nodes.values() {
        check_node_diagnostics(&graph, node, ctx, &mut errors);
    }

    // 3. Cycle detection
    check_dependency_cycles(&graph, ctx, &mut errors);

    // Sort to make testing easier
    errors.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));

    Ok(errors)
}

fn check_duplicate_ids(graph: &TaskGraph, errors: &mut Vec<DiagnosticError>) {
    for id in &graph.duplicate_ids {
        errors.push(DiagnosticError {
            file: "<multiple files>".to_string(),
            line: None,
            message: format!("Duplicate task ID found: '{}'", id),
            code: Some("duplicate_id".to_string()),
        });
    }
}

fn check_node_diagnostics(
    graph: &TaskGraph,
    node: &TaskNode,
    ctx: &RunContext,
    errors: &mut Vec<DiagnosticError>,
) {
    let rel_path = node
        .path
        .strip_prefix(&ctx.current_dir)
        .unwrap_or(&node.path)
        .display()
        .to_string();

    // Missing required keys
    if node.frontmatter.created_at.is_none() {
        errors.push(DiagnosticError {
            file: rel_path.clone(),
            line: None,
            message: "Missing required frontmatter key: 'created_at'".to_string(),
            code: Some("missing_created_at".to_string()),
        });
    }

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

    // 4. Check for uncanonical task file content (e.g., frontmatter formatting or trailing newlines)
    if let Ok(false) = node.is_canonical() {
        errors.push(DiagnosticError {
            file: rel_path.clone(),
            line: None,
            message: "Task file is not canonical".to_string(),
            code: Some("uncanonical_file".to_string()),
        });
    }
}

fn check_dependency_cycles(graph: &TaskGraph, ctx: &RunContext, errors: &mut Vec<DiagnosticError>) {
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
                        .strip_prefix(&ctx.current_dir)
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
}
