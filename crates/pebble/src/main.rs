//! `pebble` — a CLI task tracker built on Markdown-native, graph-based storage.
//!
//! Tasks are stored as individual Markdown files with TOML frontmatter. The files
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
mod config;
pub mod graph;
pub mod help_json;
pub mod models;
pub mod parser;
mod task_io;

use crate::cli::{Cli, Commands, ConfigCommands};
use crate::help_json::help_json_schema;
use crate::models::{NotFoundError, UsageError};
use clap::Parser;
use clap::error::ErrorKind;
use color_eyre::eyre::Result;
use commands::{ListOptions, RunContext, run_config_get, run_list, run_next, run_search, run_show};
use commands_add::{RunAddInput, run_add};
use commands_archive::run_archive;
use commands_fix::run_fix;
use commands_write::{RunUpdateInput, run_init, run_update};
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn run_help_json() -> Result<()> {
    println!("{}", serde_json::to_string(&help_json_schema())?);
    Ok(())
}

fn run_list_command(ctx: &RunContext, options: ListOptions) -> Result<()> {
    run_list(ctx, &options)
}

enum DispatchCommand {
    ConfigGet { key: String },
    List(ListOptions),
    Next,
    Search { query: String },
    Add(RunAddInput),
    Update(RunUpdateInput),
    Archive,
    Check { warn_only: bool, fix: bool },
    Show { id: String, path_only: bool },
}

fn prepare_dispatch_command(
    current_dir: &Path,
    command: Commands,
    global_dir: Option<PathBuf>,
    json: bool,
) -> Result<Option<DispatchCommand>> {
    match command {
        Commands::HelpJson => {
            run_help_json()?;
            Ok(None)
        }
        Commands::Init { issue_prefix, dir } => {
            run_init(
                current_dir.to_path_buf(),
                global_dir.or(dir),
                issue_prefix,
                json,
            )?;
            Ok(None)
        }
        cmd => Ok(Some(to_dispatch_command(cmd))),
    }
}

fn to_dispatch_command(command: Commands) -> DispatchCommand {
    match command {
        Commands::Config { cmd } => match cmd {
            ConfigCommands::Get { key } => DispatchCommand::ConfigGet { key },
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
        } => DispatchCommand::List(ListOptions {
            statuses,
            tags,
            needs,
            priorities,
            is_ready,
            all,
            limit,
            sort,
        }),
        Commands::Next => DispatchCommand::Next,
        Commands::Search { query } => DispatchCommand::Search { query },
        Commands::Add {
            title,
            status,
            priority,
            body,
            needs,
            tags,
            blocks,
        } => DispatchCommand::Add(RunAddInput {
            title,
            status,
            priority,
            body,
            needs,
            tags,
            blocks,
        }),
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
            blocks,
            remove_blocks,
        } => DispatchCommand::Update(RunUpdateInput {
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
            blocks,
            remove_blocks,
        }),
        Commands::Archive => DispatchCommand::Archive,
        Commands::Check { warn_only, fix } => DispatchCommand::Check { warn_only, fix },
        Commands::Show { id, path_only } => DispatchCommand::Show { id, path_only },
        Commands::HelpJson | Commands::Init { .. } => {
            unreachable!("handled before dispatch conversion")
        }
    }
}

fn main() -> ExitCode {
    if let Err(err) = run() {
        if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
            if clap_err.kind() == ErrorKind::DisplayHelp
                || clap_err.kind() == ErrorKind::DisplayVersion
            {
                let _ = clap_err.print();
                return ExitCode::SUCCESS;
            }
            let _ = clap_err.print();
            return ExitCode::from(2);
        }

        if err.is::<UsageError>() {
            eprintln!("Usage error: {}", err);
            return ExitCode::from(2);
        }

        if err.is::<NotFoundError>() {
            eprintln!("{}", err);
            return ExitCode::from(1);
        }

        eprintln!("Runtime error: {:?}", err);
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn run() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::try_parse()?;

    if let Some(ref dir) = cli.directory {
        env::set_current_dir(dir)?;
    }

    let current_dir = env::current_dir()?;

    if let Some(command) =
        prepare_dispatch_command(&current_dir, cli.command, cli.dir.clone(), cli.json)?
    {
        let ctx = RunContext::load(current_dir, cli.dir.clone(), cli.config, cli.json)?;
        dispatch_command(&ctx, command)
    } else {
        Ok(())
    }
}

fn dispatch_command(ctx: &RunContext, command: DispatchCommand) -> Result<()> {
    match &command {
        DispatchCommand::Add(_)
        | DispatchCommand::List(_)
        | DispatchCommand::Next
        | DispatchCommand::Search { .. }
        | DispatchCommand::Update(_)
        | DispatchCommand::Archive
        | DispatchCommand::Show { .. }
        | DispatchCommand::Check { .. } => {
            ctx.ensure_project()?;
        }
        DispatchCommand::ConfigGet { .. } => {
            // No project required for config inspection.
        }
    }

    match command {
        DispatchCommand::ConfigGet { key } => run_config_get(ctx, &key),
        DispatchCommand::List(options) => run_list_command(ctx, options),
        DispatchCommand::Next => run_next(ctx),
        DispatchCommand::Search { query } => run_search(ctx, &query),
        DispatchCommand::Add(input) => run_add(ctx, input),
        DispatchCommand::Update(input) => run_update(ctx, input),
        DispatchCommand::Archive => run_archive(ctx),
        DispatchCommand::Check { warn_only, fix } => {
            if fix {
                run_fix(ctx)
            } else {
                commands_diagnostics::run_check(ctx, warn_only)
            }
        }
        DispatchCommand::Show { id, path_only } => run_show(ctx, &id, path_only),
    }
}
