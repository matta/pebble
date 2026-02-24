//! `pebble` — a CLI task tracker built on Markdown-native, graph-based storage.
//!
//! Tasks are stored as individual Markdown files with TOML frontmatter. The files
//! themselves form a directed dependency graph; no external database is required.
pub mod cli;
pub mod commands;
pub mod commands_write;

#[cfg(test)]
mod commands_test;
mod config;
pub mod graph;
pub mod help_json;
pub mod models;
pub mod parser;

use crate::cli::{Cli, Commands, ConfigCommands};
use crate::help_json::help_json_schema;
use crate::models::UsageError;
use clap::Parser;
use color_eyre::eyre::Result;
use commands::{ListOptions, RunContext, run_config_get, run_list, run_next, run_search, run_show};
use commands_write::{run_add, run_archive, run_init, run_update};

fn main() -> std::process::ExitCode {
    if let Err(err) = run() {
        if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
            if clap_err.kind() == clap::error::ErrorKind::DisplayHelp
                || clap_err.kind() == clap::error::ErrorKind::DisplayVersion
            {
                let _ = clap_err.print();
                return std::process::ExitCode::SUCCESS;
            }
            let _ = clap_err.print();
            return std::process::ExitCode::from(2);
        }

        if err.is::<UsageError>() {
            eprintln!("Usage error: {}", err);
            return std::process::ExitCode::from(2);
        }

        eprintln!("Runtime error: {:?}", err);
        return std::process::ExitCode::from(1);
    }
    std::process::ExitCode::SUCCESS
}

fn run() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::try_parse()?;

    if let Some(ref dir) = cli.directory {
        std::env::set_current_dir(dir)?;
    }

    let ctx = RunContext::load(cli.dir.clone(), cli.config, cli.json)?;

    match cli.command {
        Commands::Init { issue_prefix, dir } => run_init(cli.dir.or(dir), issue_prefix, cli.json),
        Commands::Config { cmd } => match cmd {
            ConfigCommands::Get { key } => run_config_get(&ctx, &key),
        },
        Commands::List {
            statuses,
            tags,
            needs,
            priorities,
            is_ready,
            all,
            limit,
            sort,
        } => {
            let options = ListOptions {
                statuses,
                tags,
                needs,
                priorities,
                is_ready,
                all,
                limit,
                sort,
            };
            run_list(&ctx, &options)
        }
        Commands::Next => run_next(&ctx),
        Commands::Search { query } => run_search(&ctx, &query),
        Commands::Add {
            title,
            status,
            priority,
            body,
            needs,
            tags,
        } => run_add(&ctx, title, status, priority, body, needs, tags),
        Commands::Update {
            id,
            title,
            status,
            priority,
            clear_priority,
            body,
            append_body,
            add_tags,
            remove_tags,
            add_needs,
            remove_needs,
        } => run_update(
            &ctx,
            id,
            title,
            status,
            priority,
            clear_priority,
            body,
            append_body,
            add_tags,
            remove_tags,
            add_needs,
            remove_needs,
        ),
        Commands::Archive => run_archive(&ctx),
        Commands::Show { id, path_only } => run_show(&ctx, &id, path_only),
        Commands::HelpJson => {
            println!("{}", serde_json::to_string(&help_json_schema())?);
            Ok(())
        }
    }
}
