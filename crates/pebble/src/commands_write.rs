use crate::commands::{RunContext, TaskObject, validate_task_references};
use crate::config::{Config, validate_tasks_dir};
use crate::graph::TaskGraph;
use crate::models::{Priority, TaskNode, TaskStatus, UsageError};
use crate::task_io::current_task_time;
use color_eyre::eyre::{Result, eyre};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn path_to_lossy_json_string(path: &Path) -> String {
    path.as_os_str().to_string_lossy().into_owned()
}

fn apply_reverse_update(
    graph: &mut TaskGraph,
    source_node: &mut TaskNode,
    add_targets: Vec<String>,
    remove_targets: Vec<String>,
) -> Result<()> {
    let source_id = source_node.frontmatter.id.clone();

    for target_id in add_targets {
        if target_id == source_id {
            if !source_node
                .frontmatter
                .needs
                .iter()
                .any(|need| need == &source_id)
            {
                source_node.frontmatter.needs.push(source_id.clone());
            }
            continue;
        }
        let mut target_node = graph
            .nodes
            .get(&target_id)
            .cloned()
            .ok_or_else(|| eyre!("Task '{}' not found for --blocks", target_id))?;
        if target_node
            .frontmatter
            .needs
            .iter()
            .any(|need| need == &source_id)
        {
            continue;
        }
        target_node.frontmatter.needs.push(source_id.clone());
        target_node.frontmatter.modified_at = Some(current_task_time());
        target_node.write_to_disk()?;
        graph.nodes.insert(target_id, target_node);
    }

    for target_id in remove_targets {
        if target_id == source_id {
            source_node
                .frontmatter
                .needs
                .retain(|need| need != &source_id);
            continue;
        }
        let mut target_node = graph
            .nodes
            .get(&target_id)
            .cloned()
            .ok_or_else(|| eyre!("Task '{}' not found for --remove-blocks", target_id))?;
        let before_len = target_node.frontmatter.needs.len();
        target_node
            .frontmatter
            .needs
            .retain(|need| need != &source_id);
        if target_node.frontmatter.needs.len() == before_len {
            continue;
        }
        target_node.frontmatter.modified_at = Some(current_task_time());
        target_node.write_to_disk()?;
        graph.nodes.insert(target_id, target_node);
    }

    Ok(())
}

struct UpdateMutations {
    title: Option<String>,
    status: Option<TaskStatus>,
    priority: Option<Priority>,
    clear_priority: bool,
    body: Option<String>,
    append_body: Option<String>,
    add_tags: Vec<String>,
    remove_tags: Vec<String>,
    add_needs: Vec<String>,
    remove_needs: Vec<String>,
}

pub struct RunUpdateInput {
    pub id: String,
    pub title: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<Priority>,
    pub clear_priority: bool,
    pub body: Option<String>,
    pub append_body: Option<String>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    pub add_needs: Vec<String>,
    pub remove_needs: Vec<String>,
    pub blocks: Vec<String>,
    pub remove_blocks: Vec<String>,
}

fn apply_update_mutations(node: &mut TaskNode, mutations: UpdateMutations) -> Result<()> {
    if let Some(t) = mutations.title {
        node.frontmatter.title = t;
    }
    if let Some(new_status) = mutations.status {
        if !node.frontmatter.status.is_closed() && new_status.is_closed() {
            node.frontmatter.resolved_at = Some(current_task_time());
        } else if node.frontmatter.status.is_closed() && !new_status.is_closed() {
            node.frontmatter.resolved_at = None;
        }
        node.frontmatter.status = new_status;
    }
    if let Some(p) = mutations.priority {
        node.frontmatter.priority = Some(p);
    }
    if mutations.clear_priority {
        node.frontmatter.priority = None;
    }
    node.frontmatter.modified_at = Some(current_task_time());

    let mut existing_tags: HashSet<_> = node.frontmatter.tags.iter().cloned().collect();
    for t in mutations.add_tags {
        if existing_tags.insert(t.clone()) {
            node.frontmatter.tags.push(t);
        }
    }
    for t in mutations.remove_tags {
        node.frontmatter.tags.retain(|tag| tag != &t);
    }

    let mut existing_needs: HashSet<_> = node.frontmatter.needs.iter().cloned().collect();
    for d in mutations.add_needs {
        if existing_needs.insert(d.clone()) {
            node.frontmatter.needs.push(d);
        }
    }
    for d in mutations.remove_needs {
        node.frontmatter.needs.retain(|dep| dep != &d);
    }

    if let Some(b) = mutations.body {
        node.body = b;
    } else if let Some(a) = mutations.append_body {
        if node.body.trim().is_empty() {
            node.body = a;
        } else {
            node.body = node.body.trim_end().to_string();
            node.body.push_str("\n\n");
            node.body.push_str(&a);
        }
    }

    Ok(())
}

pub fn run_init(
    current_dir: PathBuf,
    cli_dir_override: Option<PathBuf>,
    issue_prefix: Option<String>,
    json: bool,
) -> Result<()> {
    let pebble_dir = current_dir.join(".pebble");

    if pebble_dir.exists() {
        return Err(eyre!(
            "Project already initialized at {}",
            current_dir.display()
        ));
    }

    fs::create_dir_all(&pebble_dir)?;

    let prefix = issue_prefix.unwrap_or_else(|| Config::default().issue_prefix);
    let tasks_dir_path = if let Some(dir) = cli_dir_override {
        if let Err(e) = validate_tasks_dir(&dir) {
            return Err(UsageError(e.to_string()).into());
        }
        dir
    } else {
        Config::default().tasks_dir
    };

    let config_toml = format!(
        r#"issue-prefix = "{prefix}"
tasks-dir = "{tasks_dir_path}"
"#,
        tasks_dir_path = tasks_dir_path.display()
    );
    fs::write(pebble_dir.join("config.toml"), config_toml)?;

    let agents_md = "This project uses `pebble` for task tracking. \
Use `pebble --help` to see available commands.\n";
    fs::write(pebble_dir.join("AGENTS.md"), agents_md)?;

    fs::create_dir_all(current_dir.join(&tasks_dir_path))?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "status": "success",
                "project_root": path_to_lossy_json_string(current_dir.as_path()),
                "tasks_dir": path_to_lossy_json_string(tasks_dir_path.as_path()),
                "issue_prefix": prefix,
            })
        );
    } else {
        eprintln!("Initialized Pebble repository in {}", current_dir.display());
    }

    Ok(())
}

pub fn run_update(ctx: &RunContext, input: RunUpdateInput) -> Result<()> {
    let RunUpdateInput {
        id,
        title,
        status,
        priority,
        clear_priority,
        body,
        append_body,
        add_tags,
        remove_tags,
        add_needs,
        remove_needs,
        blocks,
        remove_blocks,
    } = input;
    let mut graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    if graph.is_duplicate_id(&id) {
        return Err(eyre!(
            "Duplicate task ID '{}' found in multiple files; cannot safely target this ID.",
            id
        ));
    }
    let mut node = graph
        .nodes
        .remove(&id)
        .ok_or_else(|| eyre!("Task '{}' not found", id))?;
    let add_blocks_targets =
        validate_task_references(&graph, blocks, Some(id.as_str()), "--blocks")?;
    let remove_blocks_targets =
        validate_task_references(&graph, remove_blocks, Some(id.as_str()), "--remove-blocks")?;
    apply_update_mutations(
        &mut node,
        UpdateMutations {
            title,
            status,
            priority,
            clear_priority,
            body,
            append_body,
            add_tags,
            remove_tags,
            add_needs,
            remove_needs,
        },
    )?;
    apply_reverse_update(
        &mut graph,
        &mut node,
        add_blocks_targets,
        remove_blocks_targets,
    )?;

    node.write_to_disk()?;

    if ctx.json {
        graph
            .nodes
            .insert(node.frontmatter.id.clone(), node.clone());
        let updated_graph = TaskGraph::new(graph.nodes);
        let obj = TaskObject::from_node(&node, &updated_graph, &ctx.tasks_dir);
        println!("{}", serde_json::to_string(&obj)?);
    } else {
        eprintln!("Updated task {}", node.frontmatter.id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::ffi::OsString;
    use std::path::Path;
    #[cfg(unix)]
    use std::path::PathBuf;

    #[cfg(unix)]
    #[test]
    fn path_to_json_string_lossy_replaces_invalid_utf8() {
        use std::os::unix::ffi::OsStringExt;

        let path = PathBuf::from(OsString::from_vec(vec![b'd', b'i', b'r', b'-', 0xFF]));
        assert_eq!(
            super::path_to_lossy_json_string(path.as_path()),
            "dir-\u{FFFD}"
        );
    }

    #[test]
    fn path_to_json_string_lossy_preserves_utf8() {
        let path = Path::new("docs/pebble");
        assert_eq!(super::path_to_lossy_json_string(path), "docs/pebble");
    }
}
