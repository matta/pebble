## 2025-03-09 - JSON Output Boilerplate
**Duplication:** Hand-rolled `println!("{}", serde_json::to_string(payload)?);` blocks were duplicated across all CLI subcommands when outputting `--json` payloads.
**Learning:** Common CLI patterns like serialized output formatters should be centralized into a reusable helper function to prevent duplicated serialization errors and boilerplate.
**Abstraction:** Introduced `crate::commands::emit_json<T: Serialize>(payload: &T) -> Result<()>` API.
