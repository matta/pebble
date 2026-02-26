use crate::commands::RunContext;
use crate::graph::TaskGraph;
use color_eyre::eyre::Result;
use serde::Serialize;
use std::env;

/// A standard error shape emitted by the doctor's checks (suitable for JSON).
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

/// Represents the JSON payload containing the results of the `doctor` command.
/// Includes an overall true/false health indicator and an array of individual issues.
#[derive(Serialize)]
pub struct DoctorOutput {
    pub ok: bool,
    pub errors: Vec<DiagnosticError>,
}

/// Executes the read-only graph check for `pebble doctor`.
///
/// It emits warnings for duplicated files, dangling downstream dependencies,
/// and unknown keys left in a file's frontend parser map (the `extra` property).
/// Exits with Code 0 under all healthy and non-healthy valid execution runs.
pub fn run_doctor(ctx: &RunContext) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let mut errors = Vec::new();

    let current_dir = env::current_dir()?;

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
            .strip_prefix(&current_dir)
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
            let message = format!("Dependency cycle detected: {}", cycle_ids.join(" -> "));

            for id in scc {
                if let Some(node) = graph.nodes.get(id) {
                    let rel_path = node
                        .path
                        .strip_prefix(&current_dir)
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

    let ok = errors.is_empty();

    if ctx.json {
        let out = DoctorOutput { ok, errors };
        println!("{}", serde_json::to_string(&out)?);
    } else if ok {
        println!("Graph is healthy. No issues found.");
    } else {
        for err in &errors {
            eprintln!("{}: {}", err.file, err.message);
        }
    }

    Ok(())
}
