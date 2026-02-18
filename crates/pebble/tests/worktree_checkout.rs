use pebble::worktree::WorktreeManager;
use std::process::Command;
use tempfile::TempDir;

fn run_git(args: &[&str], dir: &std::path::Path) {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("Failed to execute git command");

    if !output.status.success() {
        eprintln!("Git command failed: git {}", args.join(" "));
        eprintln!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Git command failed");
    }
}

#[test]
fn test_worktree_checkout_target() {
    // Setup a "remote" repo
    let remote_dir = TempDir::new().unwrap();
    let remote_root = remote_dir.path().to_path_buf();
    run_git(&["init", "--bare", "--initial-branch=main"], &remote_root);

    // Setup the "local" repo
    let local_dir = TempDir::new().unwrap();
    let local_root = local_dir.path().to_path_buf();
    run_git(&["init", "--initial-branch=main"], &local_root);
    run_git(&["config", "user.email", "test@example.com"], &local_root);
    run_git(&["config", "user.name", "Test User"], &local_root);

    // Create main branch content
    std::fs::write(local_root.join("main.txt"), "main content").unwrap();
    run_git(&["add", "."], &local_root);
    run_git(&["commit", "-m", "main commit"], &local_root);

    // Push main
    run_git(
        &["remote", "add", "origin", remote_root.to_str().unwrap()],
        &local_root,
    );
    run_git(&["push", "-u", "origin", "main"], &local_root);

    // Create sync-branch content DIFFERENT from main
    run_git(&["checkout", "-b", "sync-branch"], &local_root);
    std::fs::write(local_root.join("sync.txt"), "sync content").unwrap();
    run_git(&["add", "sync.txt"], &local_root);
    // Remove main.txt to make it distinct
    run_git(&["rm", "main.txt"], &local_root);
    run_git(&["commit", "-m", "sync commit"], &local_root);
    run_git(&["push", "-u", "origin", "sync-branch"], &local_root);

    // Switch back to main and delete local sync-branch
    run_git(&["checkout", "main"], &local_root);
    run_git(&["branch", "-D", "sync-branch"], &local_root);

    // Now run WorktreeManager::ensure_worktree
    let manager = WorktreeManager::new(local_root.clone(), "sync-branch".to_string());

    let worktree_path = manager
        .ensure_worktree()
        .expect("Failed to ensure worktree");

    // Verify content of worktree
    let has_main_txt = worktree_path.join("main.txt").exists();
    let has_sync_txt = worktree_path.join("sync.txt").exists();

    println!(
        "Worktree content check: main.txt={}, sync.txt={}",
        has_main_txt, has_sync_txt
    );

    if has_main_txt && !has_sync_txt {
        panic!("Worktree checked out HEAD (main) instead of sync-branch!");
    }

    assert!(has_sync_txt, "Worktree should contain sync.txt");
    assert!(!has_main_txt, "Worktree should NOT contain main.txt");
}
