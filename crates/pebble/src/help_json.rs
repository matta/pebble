use crate::cli::Cli;
use clap::CommandFactory;
use serde_json::{Map, Value, json};

/// Build the machine-readable help schema describing all commands, flags, and output shapes.
pub fn help_json_schema() -> serde_json::Value {
    let cli_command = Cli::command();
    let commands: Vec<Value> = cli_command
        .get_subcommands()
        .filter(|subcommand| subcommand.get_name() != "help")
        .map(help_json_command_entry)
        .collect();

    json!({
        "name": "pebble",
        "global_options": [
            { "name": "--json", "description": "Output in JSON format" },
            { "name": "--dir <PATH>", "description": "Override tasks directory" }
        ],
        "commands": commands
    })
}

fn help_json_command_entry(subcommand: &clap::Command) -> Value {
    let mut entry = Map::new();
    let command_name = subcommand.get_name();

    entry.insert("name".to_string(), json!(command_name));
    entry.insert(
        "description".to_string(),
        json!(
            subcommand
                .get_about()
                .map(|about| about.to_string())
                .unwrap_or_default()
        ),
    );
    entry.insert("options".to_string(), help_json_options(subcommand));

    if subcommand.has_subcommands() {
        let subcommands: Vec<Value> = subcommand
            .get_subcommands()
            .filter(|nested| nested.get_name() != "help")
            .map(|nested| help_json_nested_command_entry(command_name, nested))
            .collect();
        entry.insert("subcommands".to_string(), Value::Array(subcommands));
    } else {
        let output = help_json_output_schema(command_name, None);
        entry.insert("output".to_string(), output);
    }

    Value::Object(entry)
}

fn help_json_nested_command_entry(parent_name: &str, subcommand: &clap::Command) -> Value {
    let mut entry = Map::new();
    let subcommand_name = subcommand.get_name();

    entry.insert("name".to_string(), json!(subcommand_name));
    entry.insert(
        "description".to_string(),
        json!(
            subcommand
                .get_about()
                .map(|about| about.to_string())
                .unwrap_or_default()
        ),
    );
    entry.insert("options".to_string(), help_json_options(subcommand));

    let output = help_json_output_schema(parent_name, Some(subcommand_name));
    entry.insert("output".to_string(), output);

    Value::Object(entry)
}

fn help_json_options(subcommand: &clap::Command) -> Value {
    let options: Vec<Value> = subcommand
        .get_arguments()
        .filter(|arg| !arg.is_hide_set())
        .map(|arg| {
            let mut arg_entry = Map::new();
            let name = if let Some(long) = arg.get_long() {
                format!("--{}", long)
            } else if let Some(short) = arg.get_short() {
                format!("-{}", short)
            } else {
                arg.get_id().to_string()
            };
            arg_entry.insert("name".to_string(), json!(name));
            arg_entry.insert(
                "description".to_string(),
                json!(arg.get_help().map(|h| h.to_string()).unwrap_or_default()),
            );
            Value::Object(arg_entry)
        })
        .collect();
    Value::Array(options)
}

fn help_json_output_schema(command_name: &str, subcommand_name: Option<&str>) -> Value {
    match (command_name, subcommand_name) {
        ("init", None) => json!({
            "status": "string",
            "project_root": "string",
            "tasks_dir": "string",
            "issue_prefix": "string"
        }),
        ("config", Some("get")) => json!({ "key": "string", "value": "string" }),
        ("list", None) => json!({ "tasks": ["TaskObject"] }),
        ("next", None) => json!("TaskObject|null"),
        ("search", None) => json!({ "tasks": ["TaskObject"] }),
        ("show", None) => json!("TaskObject|{path:string}"),
        ("add", None) => json!("TaskObject"),
        ("update", None) => json!("TaskObject"),
        ("archive", None) => json!({ "archived": [{ "id": "string", "moved_to": "string" }] }),
        ("check", None) => {
            json!({ "ok": "bool", "errors": [{ "file": "string", "line": "number|null", "message": "string", "code": "string|null" }] })
        }
        ("help-json", None) => json!("HelpSchema"),
        // Intentional panic: Tests validate exhaustiveness before release.
        _ => panic!(
            "Unhandled help-json output schema mapping for command '{}' and subcommand {:?}",
            command_name, subcommand_name
        ),
    }
}

#[cfg(test)]
mod help_json_schema_tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Unhandled help-json output schema mapping")]
    fn test_help_json_output_schema_panics_on_unhandled_command() {
        let _ = help_json_output_schema("unknown-command", None);
    }

    #[test]
    fn test_all_commands_have_help_json_output_schema() {
        let cli_command = Cli::command();

        for subcommand in cli_command.get_subcommands() {
            let command_name = subcommand.get_name();
            if command_name == "help" {
                continue;
            }

            if subcommand.has_subcommands() {
                for nested in subcommand.get_subcommands() {
                    let nested_name = nested.get_name();
                    if nested_name == "help" {
                        continue;
                    }
                    let _ = help_json_output_schema(command_name, Some(nested_name));
                }
            } else {
                let _ = help_json_output_schema(command_name, None);
            }
        }
    }
}
