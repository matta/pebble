use crate::commands::{RunContext, TaskObject};
use crate::graph::TaskGraph;
use crate::models::{TaskFrontmatter, TaskNode, TaskStatus};
use color_eyre::eyre::{Result, eyre};
use std::env;
use std::path::PathBuf;

pub fn run_init(
    cli_dir_override: Option<PathBuf>,
    issue_prefix: Option<String>,
    json: bool,
) -> Result<()> {
    let current_dir = env::current_dir()?;
    let pebble_dir = current_dir.join(".pebble");

    if pebble_dir.exists() {
        return Err(eyre!(
            "Project already initialized at {}",
            current_dir.display()
        ));
    }

    std::fs::create_dir_all(&pebble_dir)?;

    let prefix = issue_prefix.unwrap_or_else(|| crate::config::Config::default().issue_prefix);
    let tasks_dir_path = if let Some(dir) = cli_dir_override {
        if dir.is_absolute() {
            return Err(eyre!("tasks-dir must be a relative path"));
        }
        dir
    } else {
        crate::config::Config::default().tasks_dir
    };

    let config_toml = format!(
        r#"issue-prefix = "{prefix}"
tasks-dir = "{tasks_dir_path}"
"#,
        tasks_dir_path = tasks_dir_path.display()
    );
    std::fs::write(pebble_dir.join("config.toml"), config_toml)?;

    let agents_md = r#"# Project: Pebble
See documentation for implementation details.
"#;
    std::fs::write(pebble_dir.join("AGENTS.md"), agents_md)?;

    // Create tasks directory
    std::fs::create_dir_all(current_dir.join(tasks_dir_path))?;

    if !json {
        eprintln!("Initialized Pebble repository in {}", current_dir.display());
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn run_add(
    ctx: &RunContext,
    title: String,
    status: Option<String>,
    priority: Option<u8>,
    body: Option<String>,
    deps: Vec<String>,
    tags: Vec<String>,
) -> Result<()> {
    let id_str = nanoid::nanoid!(6, &nanoid::alphabet::SAFE); // Short ID
    let new_id = format!("{}-{}", ctx.config.issue_prefix, id_str);

    let parsed_status = if let Some(s) = status {
        serde_yaml::from_str::<TaskStatus>(&s).map_err(|_| {
            eyre!("Invalid status: {s}. Expected todo, in_progress, done, or canceled.")
        })?
    } else {
        TaskStatus::Todo
    };

    let now = chrono::Utc::now();
    let fm = TaskFrontmatter {
        id: new_id.clone(),
        title: title.clone(),
        status: parsed_status,
        priority,
        created_at: now,
        modified_at: None,
        resolved_at: None,
        deps,
        tags,
    };

    let fm_yaml = serde_yaml::to_string(&fm)?;
    let body_text = body.unwrap_or_default();

    let content = format!("---\n{}---\n{}", fm_yaml, body_text);

    std::fs::create_dir_all(&ctx.tasks_dir)?;
    let filepath = ctx.tasks_dir.join(format!("{}.md", new_id));
    std::fs::write(&filepath, content)?;

    let node = TaskNode {
        path: filepath,
        frontmatter: fm,
        body: body_text,
    };

    if ctx.json {
        let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
        let obj = TaskObject::from_node(&node, &graph, &ctx.tasks_dir);
        println!("{}", serde_json::to_string(&obj)?);
    } else {
        eprintln!("Created task {} at {}", new_id, node.path.display());
    }

    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::cognitive_complexity)]
pub fn run_update(
    ctx: &RunContext,
    id: String,
    title: Option<String>,
    status: Option<String>,
    priority: Option<u8>,
    clear_priority: bool,
    body: Option<String>,
    append_body: Option<String>,
    add_tags: Vec<String>,
    remove_tags: Vec<String>,
    add_deps: Vec<String>,
    remove_deps: Vec<String>,
) -> Result<()> {
    let mut graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let mut node = graph
        .nodes
        .remove(&id)
        .ok_or_else(|| eyre!("Task '{}' not found", id))?;

    if let Some(t) = title {
        node.frontmatter.title = t;
    }
    if let Some(s) = status {
        let new_status: TaskStatus =
            serde_yaml::from_str(&s).map_err(|_| eyre!("Invalid status"))?;

        // Handle transitions
        if !matches!(
            node.frontmatter.status,
            TaskStatus::Done | TaskStatus::Canceled
        ) && matches!(new_status, TaskStatus::Done | TaskStatus::Canceled)
        {
            node.frontmatter.resolved_at = Some(chrono::Utc::now());
        } else if matches!(
            node.frontmatter.status,
            TaskStatus::Done | TaskStatus::Canceled
        ) && !matches!(new_status, TaskStatus::Done | TaskStatus::Canceled)
        {
            node.frontmatter.resolved_at = None;
        }

        node.frontmatter.status = new_status;
    }
    if let Some(p) = priority {
        node.frontmatter.priority = Some(p);
    }
    if clear_priority {
        node.frontmatter.priority = None;
    }

    node.frontmatter.modified_at = Some(chrono::Utc::now());

    for t in add_tags {
        if !node.frontmatter.tags.contains(&t) {
            node.frontmatter.tags.push(t);
        }
    }
    for t in remove_tags {
        node.frontmatter.tags.retain(|tag| tag != &t);
    }

    for d in add_deps {
        if !node.frontmatter.deps.contains(&d) {
            node.frontmatter.deps.push(d);
        }
    }
    for d in remove_deps {
        node.frontmatter.deps.retain(|dep| dep != &d);
    }

    if let Some(b) = body {
        node.body = b;
    } else if let Some(a) = append_body {
        if node.body.is_empty() {
            node.body = a;
        } else {
            node.body.push_str("\n\n");
            node.body.push_str(&a);
        }
    }

    let fm_yaml = serde_yaml::to_string(&node.frontmatter)?;
    let content = format!("---\n{}---\n{}", fm_yaml, node.body);
    std::fs::write(&node.path, content)?;

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

pub fn run_archive(ctx: &RunContext) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let archive_dir = ctx.tasks_dir.join("archive");
    std::fs::create_dir_all(&archive_dir)?;

    let now = chrono::Utc::now();
    let threshold_days = chrono::Duration::days(30);

    let mut archived = vec![];

    for node in graph.nodes.values() {
        if matches!(
            node.frontmatter.status,
            TaskStatus::Done | TaskStatus::Canceled
        ) && let Some(resolved_at) = node.frontmatter.resolved_at
            && now.signed_duration_since(resolved_at) > threshold_days
        {
            let new_path = archive_dir.join(node.path.file_name().unwrap());
            std::fs::rename(&node.path, &new_path)?;

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
