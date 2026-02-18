# Tech Stack: Pebble

## Language & Core
- **Rust**: The primary language for Pebble, chosen for its performance, safety, and strong support for CLI tools.

## Data Serialization
- **Serde**: Used for serializing and deserializing the core `Issue` struct to and from JSONL format.

## Synchronization & Storage
- **Git (Worktree-Native)**: All Pebble data is managed directly within a dedicated Git worktree. This ensures that the data is synchronized using reliable Git protocols while remaining separate from the user's main working directory.

## Tooling & CI
- **Just**: The project's command runner, providing simple commands for testing, linting, and other development tasks.
