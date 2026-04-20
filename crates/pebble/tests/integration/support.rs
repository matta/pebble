#![expect(clippy::expect_used, reason = "TODO: remove all calls to expect")]
//! Shared test helpers for CLI integration tests.

use assert_cmd::{Command, cargo_bin};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Temp Pebble workspace with an initialized config and tasks directory.
pub struct TestEnv {
    _temp_dir: TempDir,
    /// Root directory of the temporary test workspace.
    pub root: PathBuf,
    /// Tasks directory within the temporary workspace.
    pub tasks_dir: PathBuf,
}

impl TestEnv {
    /// Returns a pre-configured `assert_cmd::Command` for the pebble CLI.
    pub fn pebble(&self) -> Command {
        pebble(&self.root)
    }
}

/// Returns a pre-configured `assert_cmd::Command` for the pebble CLI.
pub fn pebble_cli() -> Command {
    Command::new(cargo_bin!("pebble"))
}

/// Returns a pre-configured `assert_cmd::Command` for the pebble CLI in the given directory.
pub fn pebble<P: AsRef<Path>>(dir: P) -> Command {
    let mut cmd = pebble_cli();
    cmd.current_dir(dir.as_ref());
    cmd
}

/// Create a temp Pebble project with a config and empty tasks dir.
pub fn setup_test_env() -> TestEnv {
    let dir = tempfile::tempdir().expect("temp directory should be created");
    let root = dir.path().to_path_buf();

    let config_dir = root.join(".pebble");
    fs::create_dir(&config_dir).expect("config directory should be created");
    fs::write(
        config_dir.join("config.toml"),
        r#"
        issue-prefix = "PROJ"
        tasks-dir = "tasks"
        "#,
    )
    .expect("config file should be written");

    let tasks_dir = root.join("tasks");
    fs::create_dir(&tasks_dir).expect("tasks directory should be created");

    TestEnv {
        _temp_dir: dir,
        root,
        tasks_dir,
    }
}

/// Write a simple task file into the tasks directory.
pub fn write_task(tasks_dir: &Path, id: &str, title: &str, status: &str) {
    let content = format!(
        r#"---
id: "{id}"
title: "{title}"
status: "{status}"
created_at: "2024-01-01T00:00:00Z"
---
Body
"#,
        id = id,
        title = title,
        status = status
    );
    fs::write(tasks_dir.join(format!("{id}.md")), content).expect("task file should be written");
}

/// Write a simple task file with title equal to the id.
pub fn write_task_with_id(tasks_dir: &Path, id: &str) {
    write_task(tasks_dir, id, id, "todo");
}

/// A builder for creating task files in tests.
#[derive(Default)]
pub struct TaskBuilder<'a> {
    id: &'a str,
    title: Option<&'a str>,
    status: Option<&'a str>,
    created_at: Option<&'a str>,
    priority: Option<u8>,
    needs: Vec<&'a str>,
    tags: Vec<&'a str>,
    body: Option<&'a str>,
}

impl<'a> TaskBuilder<'a> {
    pub fn new(id: &'a str) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn status(mut self, status: &'a str) -> Self {
        self.status = Some(status);
        self
    }

    pub fn created_at(mut self, created_at: &'a str) -> Self {
        self.created_at = Some(created_at);
        self
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn needs(mut self, needs: &[&'a str]) -> Self {
        self.needs = needs.to_vec();
        self
    }

    pub fn tags(mut self, tags: &[&'a str]) -> Self {
        self.tags = tags.to_vec();
        self
    }

    pub fn body(mut self, body: &'a str) -> Self {
        self.body = Some(body);
        self
    }

    pub fn write(self, tasks_dir: &Path) {
        let mut frontmatter = format!(
            "id: \"{}\"\ntitle: \"{}\"\nstatus: \"{}\"\ncreated_at: \"{}\"\n",
            self.id,
            self.title.unwrap_or(self.id),
            self.status.unwrap_or("todo"),
            self.created_at.unwrap_or("2024-01-01T00:00:00Z")
        );

        if let Some(p) = self.priority {
            frontmatter.push_str(&format!("priority: {p}\n"));
        }

        fn append_str_list(fm: &mut String, key: &str, values: &[&str]) {
            if !values.is_empty() {
                let items = values
                    .iter()
                    .map(|v| format!("\"{v}\""))
                    .collect::<Vec<_>>()
                    .join(", ");
                fm.push_str(&format!("{}: [{}]\n", key, items));
            }
        }

        append_str_list(&mut frontmatter, "needs", &self.needs);
        append_str_list(&mut frontmatter, "tags", &self.tags);

        let content = format!("---\n{frontmatter}---\n{}\n", self.body.unwrap_or("Body"));
        fs::write(tasks_dir.join(format!("{}.md", self.id)), content)
            .expect("task file should be written");
    }
}
