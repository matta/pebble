use clap::{Arg, ArgAction, ColorChoice, Command, CommandFactory};
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct HelpJson {
    name: &'static str,
    version: &'static str,
    top_level_help: String,
    top_level_long_help: String,
    commands: Vec<CommandHelp>,
    schemas: BTreeMap<&'static str, serde_json::Value>,
}

#[derive(Serialize)]
struct CommandHelp {
    name: String,
    description: String,
    help: String,
    long_help: String,
    args: Vec<ArgHelp>,
    flags: Vec<FlagHelp>,
    output_schema: Option<&'static str>,
    subcommands: Vec<CommandHelp>,
}

#[derive(Serialize)]
struct ArgHelp {
    name: String,
    description: String,
    required: bool,
}

#[derive(Serialize)]
struct FlagHelp {
    name: String,
    description: String,
    value_type: Option<&'static str>,
}

pub fn print() -> color_eyre::Result<()> {
    let mut cmd = crate::Cli::command();
    cmd = cmd.color(ColorChoice::Never);
    let top_level_help = cmd.render_help().to_string();
    let top_level_long_help = cmd.render_long_help().to_string();

    let commands = cmd
        .get_subcommands()
        .map(|sub| command_to_help(sub, sub.get_name()))
        .collect();

    let help = HelpJson {
        name: "pebble",
        version: env!("CARGO_PKG_VERSION"),
        top_level_help,
        top_level_long_help,
        commands,
        schemas: schema_map(),
    };

    println!("{}", serde_json::to_string_pretty(&help)?);
    Ok(())
}

fn command_to_help(cmd: &Command, path: &str) -> CommandHelp {
    let mut cmd = cmd.clone().color(ColorChoice::Never);
    let help = cmd.render_help().to_string();
    let long_help = cmd.render_long_help().to_string();
    let description = cmd
        .get_about()
        .or_else(|| cmd.get_long_about())
        .unwrap_or_default()
        .to_string();

    let mut args = Vec::new();
    let mut flags = Vec::new();

    for arg in cmd.get_arguments() {
        if is_builtin_arg(arg) {
            continue;
        }

        if arg.is_positional() {
            args.push(ArgHelp {
                name: arg.get_id().to_string(),
                description: arg_description(arg),
                required: arg.is_required_set(),
            });
        } else {
            flags.push(FlagHelp {
                name: flag_name(arg),
                description: arg_description(arg),
                value_type: flag_value_type(arg),
            });
        }
    }

    let subcommands = cmd
        .get_subcommands()
        .map(|sub| {
            let sub_path = format!("{} {}", path, sub.get_name());
            command_to_help(sub, &sub_path)
        })
        .collect();

    CommandHelp {
        name: cmd.get_name().to_string(),
        description,
        help,
        long_help,
        args,
        flags,
        output_schema: output_schema_for(path),
        subcommands,
    }
}

fn arg_description(arg: &Arg) -> String {
    arg.get_long_help()
        .or_else(|| arg.get_help())
        .unwrap_or_default()
        .to_string()
}

fn flag_name(arg: &Arg) -> String {
    if let Some(long) = arg.get_long() {
        format!("--{}", long)
    } else if let Some(short) = arg.get_short() {
        format!("-{}", short)
    } else {
        arg.get_id().to_string()
    }
}

fn flag_value_type(arg: &Arg) -> Option<&'static str> {
    match arg.get_action() {
        ArgAction::Set | ArgAction::Append => Some("string"),
        _ => None,
    }
}

fn is_builtin_arg(arg: &Arg) -> bool {
    matches!(
        arg.get_action(),
        ArgAction::Help | ArgAction::HelpShort | ArgAction::HelpLong | ArgAction::Version
    ) || matches!(arg.get_long(), Some("help" | "version"))
}

fn output_schema_for(path: &str) -> Option<&'static str> {
    match path {
        "init" => Some("InitResult"),
        "import" => Some("ImportResult"),
        "config get" => Some("ConfigValue"),
        "sync" => Some("SyncResult"),
        "list" => Some("IssueList"),
        "add" | "show" | "edit" => Some("Issue"),
        _ => None,
    }
}

fn schema_map() -> BTreeMap<&'static str, serde_json::Value> {
    let mut schemas = BTreeMap::new();
    schemas.insert("Issue", issue_schema());
    schemas.insert("IssueList", issue_list_schema());
    schemas.insert("ConfigValue", config_value_schema());
    schemas.insert("ImportResult", import_result_schema());
    schemas.insert("SyncResult", sync_result_schema());
    schemas.insert("InitResult", init_result_schema());
    schemas
}

fn issue_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "string"},
            "title": {"type": "string"},
            "description": {"type": "string"},
            "status": {"type": "string"},
            "priority": {"type": "integer"},
            "issue_type": {"type": "string"},
            "owner": {"type": "string"},
            "created_at": {"type": "string"},
            "created_by": {"type": "string"},
            "updated_at": {"type": "string"},
            "closed_at": {"type": ["string", "null"]},
            "close_reason": {"type": ["string", "null"]}
        },
        "required": [
            "id",
            "title",
            "description",
            "status",
            "priority",
            "issue_type",
            "owner",
            "created_at",
            "created_by",
            "updated_at",
            "closed_at",
            "close_reason"
        ]
    })
}

fn issue_list_schema() -> serde_json::Value {
    json!({
        "type": "array",
        "items": {"$ref": "Issue"}
    })
}

fn config_value_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "key": {"type": "string"},
            "value": {"type": "string"}
        },
        "required": ["key", "value"]
    })
}

fn import_result_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "added": {"type": "integer"},
            "updated": {"type": "integer"}
        },
        "required": ["added", "updated"]
    })
}

fn sync_result_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "status": {"type": "string"}
        },
        "required": ["status"]
    })
}

fn init_result_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "sync_branch": {"type": "string"},
            "worktree_path": {"type": "string"},
            "config_path": {"type": "string"}
        },
        "required": ["sync_branch", "worktree_path", "config_path"]
    })
}
