use pebble::worktree::WorktreeManager;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn run_git(args: &[&str], dir: &std::path::Path) {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute git {}: {}", args.join(" "), e));

    if !output.status.success() {
        eprintln!("Git command failed: git {}", args.join(" "));
        eprintln!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn test_worktree_creation_from_remote_branch() {
    // Setup remote repo
    let remote_dir = TempDir::new().unwrap();
    let remote_path = remote_dir.path();
    run_git(&["init", "--bare"], remote_path);

    // Setup a "pusher" repo to push a branch to remote
    let pusher_dir = TempDir::new().unwrap();
    let pusher_path = pusher_dir.path();
    run_git(&["init", "-b", "main"], pusher_path);
    run_git(&["config", "user.email", "test@example.com"], pusher_path);
    run_git(&["config", "user.name", "Test User"], pusher_path);
    run_git(
        &["remote", "add", "origin", remote_path.to_str().unwrap()],
        pusher_path,
    );

    fs::write(pusher_path.join("README.md"), "# Test Repo").unwrap();
    run_git(&["add", "."], pusher_path);
    run_git(&["commit", "-m", "Initial commit"], pusher_path);
    run_git(&["push", "-u", "origin", "main"], pusher_path);

    // Create sync branch and push it
    let sync_branch = "beads-sync-test";
    run_git(&["checkout", "-b", sync_branch], pusher_path);
    fs::write(pusher_path.join("sync.txt"), "sync content").unwrap();
    run_git(&["add", "."], pusher_path);
    run_git(&["commit", "-m", "Sync commit"], pusher_path);
    run_git(&["push", "-u", "origin", sync_branch], pusher_path);

    // Setup "local" repo where we will run the test
    let local_dir = TempDir::new().unwrap();
    let local_path = local_dir.path();
    run_git(&["init", "-b", "main"], local_path);
    run_git(&["config", "user.email", "local@example.com"], local_path);
    run_git(&["config", "user.name", "Local User"], local_path);
    run_git(
        &["remote", "add", "origin", remote_path.to_str().unwrap()],
        local_path,
    );

    // We do NOT fetch explicitly here (or maybe verify without fetch first?)
    // The previous code needed explicit fetch. New code should fetch automatically.
    // So let's NOT run fetch.

    // Verify sync branch does NOT exist locally
    let output = Command::new("git")
        .args(&["branch", "--list", sync_branch])
        .current_dir(local_path)
        .output()
        .unwrap();
    assert!(
        output.stdout.is_empty(),
        "Sync branch should not exist locally yet"
    );

    // Use WorktreeManager
    let manager = WorktreeManager::new(local_path.to_path_buf(), sync_branch.to_string());

    let worktree_path = manager
        .ensure_worktree()
        .expect("Failed to ensure worktree");

    assert!(worktree_path.exists(), "Worktree path should exist");

    // Check if we are on the correct branch in the worktree
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&worktree_path)
        .output()
        .unwrap();
    let branch = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        branch.trim(),
        sync_branch,
        "Worktree should be on sync branch"
    );

    // Check if content exists
    assert!(
        worktree_path.join("sync.txt").exists(),
        "Sync content should exist in worktree"
    );
}

#[test]
fn test_worktree_creation_orphan_fallback() {
    // Test the case where branch exists NOWHERE
    let local_dir = TempDir::new().unwrap();
    let local_path = local_dir.path();
    run_git(&["init", "-b", "main"], local_path);
    run_git(&["config", "user.email", "local@example.com"], local_path);
    run_git(&["config", "user.name", "Local User"], local_path);
    // Commit something so we have a HEAD
    fs::write(local_path.join("README.md"), "init").unwrap();
    run_git(&["add", "."], local_path);
    run_git(&["commit", "-m", "init"], local_path);

    let sync_branch = "beads-orphan-test";
    let manager = WorktreeManager::new(local_path.to_path_buf(), sync_branch.to_string());

    let worktree_path = manager
        .ensure_worktree()
        .expect("Failed to create orphan worktree");

    // Check if we are on the correct branch
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&worktree_path)
        .output()
        .unwrap();
    let branch = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        branch.trim(),
        sync_branch,
        "Worktree should be on sync branch"
    );

    // It should be an orphan branch with clean slate (only .git?)
    // The 'init' commit file should NOT be there.
    assert!(
        !worktree_path.join("README.md").exists(),
        "Orphan branch should be empty"
    );
}
