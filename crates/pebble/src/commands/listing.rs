use crate::graph::TaskGraph;
use crate::models::{ClosedStatus, LiveStatus, NotFoundError, Priority, TaskNode, TaskStatus};
use color_eyre::eyre::{Result, eyre};
use std::cmp::Ordering;
use std::collections::HashSet;

use super::{RunContext, TaskObject};

/// Filters and switches accepted by `pebble list`.
pub struct ListOptions {
    /// Filter by status values (OR logic).
    pub statuses: Vec<TaskStatus>,
    /// Filter by tags (AND logic — task must have all specified tags).
    pub tags: Vec<String>,
    /// Filter by task dependencies (OR logic).
    pub needs: Vec<String>,
    /// Filter by priority values (OR logic).
    pub priorities: Vec<Priority>,
    /// Show only tasks that are ready to start.
    pub is_ready: bool,
    /// Include closed tasks (done/canceled) in results.
    pub all: bool,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
    /// Explicit sort field (prefix with '-' for descending).
    pub sort: Option<String>,
}

#[derive(Copy, Clone, Debug)]
enum ListSortField {
    Priority,
    Blocking,
    CreatedAt,
    ModifiedAt,
    Status,
    Title,
}

#[derive(Copy, Clone, Debug)]
struct SortSpec {
    field: ListSortField,
    descending: bool,
}

impl SortSpec {
    fn parse(raw: &str) -> Result<Self> {
        let (descending, field_str) = if let Some(stripped) = raw.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, raw)
        };

        let field = match field_str {
            "priority" => ListSortField::Priority,
            "blocking" => ListSortField::Blocking,
            "created_at" => ListSortField::CreatedAt,
            "modified_at" => ListSortField::ModifiedAt,
            "status" => ListSortField::Status,
            "title" => ListSortField::Title,
            _ => {
                return Err(eyre!(
                    "Invalid sort field '{}'. Valid fields: priority, blocking, created_at, modified_at, status, title",
                    raw
                ));
            }
        };

        Ok(Self { field, descending })
    }
}

fn filter_list_tasks<'a>(graph: &'a TaskGraph, options: &ListOptions) -> Vec<&'a TaskNode> {
    let mut tasks: Vec<&TaskNode> = graph.nodes.values().collect();

    if !options.statuses.is_empty() {
        let statuses: HashSet<TaskStatus> = options.statuses.iter().cloned().collect();
        tasks.retain(|n| statuses.contains(&n.frontmatter.status));
    } else if !options.all {
        // Default: omit done/canceled unless --all is set.
        tasks.retain(|n| !n.frontmatter.status.is_closed());
    }

    if !options.tags.is_empty() {
        tasks.retain(|n| {
            options
                .tags
                .iter()
                .all(|tag| n.frontmatter.tags.iter().any(|task_tag| task_tag == tag))
        });
    }

    if !options.needs.is_empty() {
        let filter_needs: HashSet<_> = options.needs.iter().collect();
        tasks.retain(|n| {
            n.frontmatter
                .needs
                .iter()
                .any(|need| filter_needs.contains(need))
        });
    }

    if !options.priorities.is_empty() {
        tasks.retain(|n| {
            n.frontmatter
                .priority
                .is_some_and(|p| options.priorities.contains(&p))
        });
    }

    if options.is_ready {
        tasks.retain(|n| graph.is_ready(&n.frontmatter.id));
    }

    tasks
}

fn sort_list_tasks<'a>(
    graph: &'a TaskGraph,
    mut tasks: Vec<&'a TaskNode>,
    sort: Option<&str>,
) -> Result<Vec<&'a TaskNode>> {
    let Some(sort_raw) = sort else {
        return graph.default_order(tasks);
    };

    let spec = SortSpec::parse(sort_raw)?;
    let min_priority = Priority::MIN;
    let status_rank = |status: &TaskStatus| -> u8 {
        match status {
            TaskStatus::Live(LiveStatus::Todo) => 0,
            TaskStatus::Live(LiveStatus::InProgress) => 1,
            TaskStatus::Closed(ClosedStatus::Done) => 2,
            TaskStatus::Closed(ClosedStatus::Canceled) => 3,
        }
    };

    tasks.sort_by(|a, b| {
        let key_order = match spec.field {
            ListSortField::Priority => {
                // Unset priority always sorts after explicit priorities.
                let a_key = a
                    .frontmatter
                    .priority
                    .map(|p| (false, p))
                    .unwrap_or((true, min_priority));
                let b_key = b
                    .frontmatter
                    .priority
                    .map(|p| (false, p))
                    .unwrap_or((true, min_priority));
                a_key.cmp(&b_key)
            }
            ListSortField::Blocking => {
                let a_blocking = graph.count_blocking(&a.frontmatter.id);
                let b_blocking = graph.count_blocking(&b.frontmatter.id);
                a_blocking.cmp(&b_blocking)
            }
            ListSortField::CreatedAt => a.frontmatter.created_at.cmp(&b.frontmatter.created_at),
            ListSortField::ModifiedAt => a.frontmatter.modified_at.cmp(&b.frontmatter.modified_at),
            ListSortField::Status => {
                status_rank(&a.frontmatter.status).cmp(&status_rank(&b.frontmatter.status))
            }
            ListSortField::Title => a.frontmatter.title.cmp(&b.frontmatter.title),
        };

        let key_order = if spec.descending {
            key_order.reverse()
        } else {
            key_order
        };

        if key_order != Ordering::Equal {
            return key_order;
        }

        // Required tie-breakers under explicit --sort.
        a.frontmatter
            .created_at
            .cmp(&b.frontmatter.created_at)
            .then_with(|| a.frontmatter.id.cmp(&b.frontmatter.id))
    });

    Ok(tasks)
}

fn emit_task_list(ctx: &RunContext, graph: &TaskGraph, tasks: Vec<&TaskNode>) -> Result<()> {
    if ctx.json {
        let objects: Vec<TaskObject> = tasks
            .into_iter()
            .map(|n| TaskObject::from_node(n, graph, &ctx.tasks_dir))
            .collect();
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({ "tasks": objects }))?
        );
    } else {
        for task in tasks {
            println!(
                "{} {} ({})",
                task.frontmatter.id, task.frontmatter.title, task.frontmatter.status
            );
        }
    }
    Ok(())
}

/// List tasks using the default ordering, with optional filters.
pub fn run_list(ctx: &RunContext, options: &ListOptions) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let tasks = filter_list_tasks(&graph, options);
    let mut tasks = sort_list_tasks(&graph, tasks, options.sort.as_deref())?;
    if let Some(limit) = options.limit {
        tasks.truncate(limit);
    }
    emit_task_list(ctx, &graph, tasks)
}

/// Search tasks by case-insensitive substring across title and body.
pub fn run_search(ctx: &RunContext, query: &str) -> Result<()> {
    let graph = TaskGraph::load_from_dir(&ctx.tasks_dir)?;
    let needle = query.to_lowercase();

    let tasks: Vec<&TaskNode> = graph
        .nodes
        .values()
        .filter(|n| !n.frontmatter.status.is_closed())
        .filter(|n| {
            n.frontmatter.title.to_lowercase().contains(&needle)
                || n.body.to_lowercase().contains(&needle)
        })
        .collect();

    if tasks.is_empty() {
        return Err(NotFoundError(format!("No tasks found matching query '{}'", query)).into());
    }

    let tasks = graph.default_order(tasks)?;
    emit_task_list(ctx, &graph, tasks)
}
