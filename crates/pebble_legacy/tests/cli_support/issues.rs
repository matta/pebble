use pebble::ISSUES_FILE;
use pebble::worktree::generate_worktree_path;

use crate::cli_support::TestEnv;
use crate::common::TEST_SYNC_BRANCH;

pub trait TestEnvIssues {
    fn add_issue_to_worktree(&self, issue: &serde_json::Value);
}

impl TestEnvIssues for TestEnv {
    fn add_issue_to_worktree(&self, issue: &serde_json::Value) {
        let worktree_path = generate_worktree_path(self.root(), TEST_SYNC_BRANCH);
        std::fs::create_dir_all(&worktree_path).unwrap();
        let issues_path = worktree_path.join(ISSUES_FILE);

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&issues_path)
            .unwrap();

        let json = serde_json::to_string(issue).unwrap();
        use std::io::Write;
        writeln!(file, "{}", json).unwrap();
    }
}

pub fn create_test_issue(id: &str, title: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "title": title,
        "status": "open",
        "priority": 0,
        "issue_type": "task",
        "owner": "test@example.com",
        "created_at": "2026-01-01T00:00:00Z",
        "created_by": "Tester",
        "updated_at": "2026-01-01T00:00:00Z",
        "description": "A test fixture issue",
        "closed_at": null,
        "close_reason": null
    })
}
