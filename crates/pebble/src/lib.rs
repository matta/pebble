//! `pebble` — a CLI task tracker built on Markdown-native, graph-based storage.
//!
//! Tasks are stored as individual Markdown files with YAML frontmatter. The files
//! themselves form a directed dependency graph; no external database is required.
pub mod cli;
pub mod commands;
pub mod commands_add;
pub mod commands_archive;
pub mod commands_diagnostics;
pub mod commands_fix;
pub mod commands_write;

#[cfg(test)]
mod commands_test;
pub mod config;
pub mod graph;
pub mod help_json;
pub mod models;
pub mod parser;
pub mod task_io;
