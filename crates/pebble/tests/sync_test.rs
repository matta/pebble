use pebble::worktree::WorktreeManager;
use std::process::Command;
use tempfile::TempDir;

fn run_git(args: &[&str], dir: &std::path::Path) {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("Failed to execute git");

    if !output.status.success() {
        panic!(
            "Git command failed: git {}\nStdout: {}\nStderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn setup_git_repo(path: &std::path::Path) {
    run_git(&["init", "-b", "main"], path);
    run_git(&["config", "user.email", "test@example.com"], path);
    run_git(&["config", "user.name", "Test User"], path);
}

#[test]
fn test_sync_conflict_resolution() {
    // 1. Setup remote
    let remote_dir = TempDir::new().unwrap();
    let remote_path = remote_dir.path();
    run_git(&["init", "--bare"], remote_path);

    // 2. Setup User A
    let user_a_dir = TempDir::new().unwrap();
    let user_a_path = user_a_dir.path();
    setup_git_repo(user_a_path);
    run_git(
        &["remote", "add", "origin", remote_path.to_str().unwrap()],
        user_a_path,
    );

    // Initial commit on main
    std::fs::write(user_a_path.join("README.md"), "# Test Repo").unwrap();
    run_git(&["add", "."], user_a_path);
    run_git(&["commit", "-m", "Initial commit"], user_a_path);
    run_git(&["push", "-u", "origin", "main"], user_a_path);

    // Create sync branch on remote
    run_git(&["checkout", "-b", "beads-sync"], user_a_path);
    std::fs::create_dir_all(user_a_path.join(".beads")).unwrap();
    std::fs::write(user_a_path.join(".beads/dummy"), "init").unwrap(); // ensure dir exists
    run_git(&["add", "."], user_a_path); // add .beads dir
    // We need an issues.jsonl to conflict on
    let issues_json = r#"{"id":"1","title":"Original Title","status":"open"}"#;
    std::fs::write(
        user_a_path.join(".beads/issues.jsonl"),
        format!("{}\n", issues_json),
    )
    .unwrap();
    run_git(&["add", ".beads/issues.jsonl"], user_a_path);
    run_git(&["commit", "-m", "Add issues.jsonl"], user_a_path);
    run_git(&["push", "-u", "origin", "beads-sync"], user_a_path);

    // 3. Setup User B
    let user_b_dir = TempDir::new().unwrap();
    let user_b_path = user_b_dir.path();
    setup_git_repo(user_b_path);
    run_git(
        &["remote", "add", "origin", remote_path.to_str().unwrap()],
        user_b_path,
    );
    run_git(&["fetch", "origin"], user_b_path);
    run_git(&["checkout", "-b", "main", "origin/main"], user_b_path);

    // 4. User A modifies issue and syncs (via WorktreeManager essentially, but we simulate changes on origin)
    // User A changes title to "Title A"
    let issues_json_a = r#"{"id":"1","title":"Title A","status":"open"}"#;
    std::fs::write(
        user_a_path.join(".beads/issues.jsonl"),
        format!("{}\n", issues_json_a),
    )
    .unwrap();
    run_git(&["add", ".beads/issues.jsonl"], user_a_path);
    run_git(&["commit", "-m", "User A change"], user_a_path);
    run_git(&["push", "origin", "beads-sync"], user_a_path);

    // 5. User B tries to sync, but has local changes (simulated)
    // Create mock editor script
    let editor_script_path = user_b_path.join("mock_editor.sh");
    let script_content = r#"#!/bin/sh
echo '{"id":"1","title":"Title Resolved","status":"open"}' > "$1"
"#;
    std::fs::write(&editor_script_path, script_content).unwrap();

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&editor_script_path)
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&editor_script_path, perms).unwrap();
    }

    let manager_b = WorktreeManager::new(user_b_path.to_path_buf(), "beads-sync".to_string())
        .with_editor(editor_script_path.to_str().unwrap().to_string());

    let worktree_path = manager_b
        .ensure_worktree()
        .expect("Failed to create worktree");

    // Create local change in B's worktree
    let issues_json_b = r#"{"id":"1","title":"Title B","status":"open"}"#;
    std::fs::create_dir_all(worktree_path.join(".beads")).unwrap();
    std::fs::write(
        worktree_path.join(".beads/issues.jsonl"),
        format!("{}\n", issues_json_b),
    )
    .unwrap();

    // Now call sync.
    // It should:
    // 1. Commit local changes ("Title B").
    // 2. Fetch "Title A".
    // 3. Merge -> Conflict.
    // 4. Call EDITOR (resolve to "Title Resolved").
    // 5. Commit merge.
    // 6. Push.

    let result = manager_b.sync();

    assert!(result.is_ok(), "Sync failed: {:?}", result.err());

    // Verify remote has the resolved content
    let verify_dir = TempDir::new().unwrap();
    let verify_path = verify_dir.path();
    run_git(&["clone", remote_path.to_str().unwrap(), "."], verify_path);
    run_git(&["checkout", "beads-sync"], verify_path);

    let content = std::fs::read_to_string(verify_path.join(".beads/issues.jsonl")).unwrap();
    assert!(
        content.contains("Title Resolved"),
        "Content was: {}",
        content
    );
}
