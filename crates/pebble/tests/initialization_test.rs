use pebble::config::Config;
use pebble::CONFIG_DIR;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_is_initialized_in_empty_dir() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // In an empty directory, it should not be initialized
    assert!(!Config::is_initialized(root));
}

#[test]
fn test_is_initialized_with_pebble_dir() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create a .pebble directory
    fs::create_dir(root.join(CONFIG_DIR)).unwrap();

    // It should now be initialized
    assert!(Config::is_initialized(root));
}
