# Initial Concept
Pebble is a simplified Rust re-implementation of the `beads` tool, focusing on a JSONL backend, strict Git-based worktree synchronization, and a Test-Driven Development (TDD) approach.

# Product Guide: Pebble

## Overview
Pebble is a streamlined, "bare-bones" alternative to `beads`. It intentionally strips away peripheral features to focus on a minimal, high-performance core for developers who value simplicity and zero overhead.

## Target Audience
- **Git-Centric Developers**: Users who want their task management to live alongside their code, leveraging Git for synchronization.
- **Minimalists**: Those who prefer a "bare minimum" toolset and value a focused, reduced feature set.
- **Original Beads Users**: People looking for a more predictable, Rust-powered alternative to the original Go implementation.

## Core Goals
1. **Reduced Scope**: Provide only the essential features required for issue tracking, avoiding "feature creep" at all costs.
2. **Minimal Overhead**: Zero daemon processes and a simple, JSONL-only backend.
3. **Small-Scale Focused**: Intentionally not intended to scale to massive databases or huge projects. It's built for individual developers and small teams who value speed over scalability.
4. **Data Integrity**: Using JSONL as the single source of truth to ensure data is human-readable and easily manipulated.
5. **Robust Synchronization**: Leveraging Git worktrees for a reliable, offline-first sync mechanism.
6. **Developer-First Design**: Built with TDD, providing a clean CLI interface and high test coverage.

## Key Features
- **JSONL Storage**: All issues are stored in a human-readable JSONL format.
- **Worktree Integration**: Data resides exclusively in a Git worktree, keeping it separate from the main project files but synchronized via Git.
- **Sync Command**: A straightforward `pebble sync` command that wraps common Git operations.
- **CLI Management**: Commands for listing, adding, showing, and editing issues.

## Constraints
- **No SQLite Support**: Exclusively uses JSONL.
- **No Daemon Mode**: Pebble runs as a simple CLI command and does not maintain a background process.
- **Strict Configuration**: Only supports `sync-branch` configuration.
