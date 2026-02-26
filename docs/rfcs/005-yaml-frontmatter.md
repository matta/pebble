[STATUS: DRAFT]
Supersedes: RFC 004

# RFC 005: YAML Frontmatter (via `serde-saphyr`)

## 1. Summary

Revert the decision from RFC 004 to use TOML frontmatter, and return to YAML as the exclusive metadata format for Pebble task files. Delimiters revert from `+++` to `---`. We will use the `serde-saphyr` crate for YAML deserialization, which bypasses the problematic `unsafe-libyaml` AST vulnerabilities entirely.

## 2. Motivation

### 2.1 Parsing Issues with TOML Frontmatter Boundary Detection

The core difficulty in parsing TOML frontmatter lies in boundary detection. 

* **State Dependency:** TOML allows multi-line strings (denoted by `"""` or `'''`). Inside these strings, the sequence `+++` on a new line is perfectly valid literal text. 
* **Non-Regularity:** TOML is not a regular language. A simple line-by-line scanner or regular expression cannot reliably extract TOML frontmatter because it cannot track whether it is currently inside a multi-line string. 
* **Correctness Strategy:** To guarantee we do not prematurely truncate the frontmatter, we must use a backtracking approach where candidate slices are repeatedly fed into a full TOML parser until a structurally valid document is returned. This adds unacceptable complexity to our parsing pipeline.

### 2.2 Why YAML is Superior for Boundary Detection

In the specific context of frontmatter boundary detection, YAML is structurally superior. 

* **Direct Evidence:** The YAML 1.2 specification designates `---` at the beginning of a line as a root-level "Document Marker". The specification mandates that document markers function entirely outside the formatting of the document and cannot be used as the content of a node.
* **Inference:** Because a YAML parser will forcibly terminate the document if it encounters `---` at column 0—even if it is inside a string—we can safely infer that a simple `O(N)` string scanner looking for `\n---` is semantically correct. It achieves parity with a full parser's boundary detection without requiring complex state management.

### 2.3 The `serde-saphyr` Solution

RFC 004 correctly noted that traditional YAML parsers relying on `unsafe-libyaml` pose security and ecosystem risks. However, we now plan to use `serde-saphyr`. `serde-saphyr` is a pure-Rust crate that deserializes YAML directly into Rust structures via Serde *without* building an intermediate abstract syntax tree. Because it bypasses an intermediate Document Object Model (DOM), it parses strictly driven by the types defined in our struct. If the YAML contains unexpected types, it rejects the input early, preventing intermediate memory bloat and mitigating traditional YAML vulnerabilities (like the Billion Laughs attack).

### 2.4 Ecosystem Familiarity

Beyond technical advantages, YAML is the undisputed de facto standard for Markdown frontmatter across the software ecosystem (e.g., Hugo, Jekyll, Obsidian, GitHub). Returning to YAML eliminates the learning curve for new users reading or authoring task files. Contributors naturally expect standard `---` delimiters and basic block structures, removing unnecessary friction when interacting with Pebble's text-based format.

## 3. Specification

### 3.1 Format Rules and Schema
- Frontmatter will strictly use YAML 1.2.
- The YAML block must begin at the first line of the file with `---` and end with `---` alone on a line.
- Pebble's `TaskFrontmatter` Rust `struct` will represent the expected frontmatter schema.
- We do not require the original AST for write-back modifications, as `serde-saphyr` does not retain it. Write-backs will be achieved via a robust template or serialization approach that guarantees schema conformance.

### 3.2 Parsing Implementation Plan

1. **Schema Definition:** Define a Rust `struct` representing the frontmatter and derive `serde::Deserialize` and validation mechanisms (e.g., via the `garde` crate for declarative constraints).
2. **Extract the Payload:** Use a naive string scanner to locate the second instance of `---` anchored to the start of a line. Slice the string to isolate the frontmatter payload. 
3. **Deserialize Directly:** Pass the isolated string slice to `serde_saphyr::from_str`. 

Example of implementation pattern:

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Frontmatter {
    title: String,
    status: String,
}

fn parse_file(content: &str) -> Result<Frontmatter, Box<dyn std::error::Error>> {
    // 1. Locate boundaries (Assuming strictly formatted YAML frontmatter)
    let end_index = content[4..].find("\n---").ok_or("No closing delimiter")?;
    let yaml_slice = &content[4..end_index + 4];

    // 2. Parse directly into the struct using serde-saphyr
    let frontmatter: Frontmatter = serde_saphyr::from_str(yaml_slice)?;
    
    Ok(frontmatter)
}
```

We will also integrate the `garde` crate with `serde-saphyr` to add declarative validation constraints to this frontmatter struct, ensuring safe and correct data types despite YAML's flexible type coercion.

## 4. Impact

### 4.1 Parser Changes
- Revert delimiters from `+++` to `---`.
- Add `serde-saphyr` and `garde` to `Cargo.toml`.
- Remove `toml` for frontmatter deserialization (though it may remain for configuration files).
- The `TaskFrontmatter` struct will derive its validation via `garde` to enforce constraints (e.g., date formats, valid enums, lengths) that YAML naturally allows to be looser than TOML.

### 4.2 Documentation
- `docs/schema.md`, `AGENTS.md`, and other documentation will revert references from TOML to YAML.
- `docs/cli-contract.md` and related format references will be updated.

### 4.3 Existing Data
- Any existing test fixtures or existing markdown tasks using `+++` will need to be updated to `---` and converted to YAML syntax.

## 5. Alternatives Considered

* **Continue using TOML:** Requires complex loop-and-retry parsing to safely extract frontmatter boundaries without breaking on TOML multi-line strings that contain `+++`. This is deemed too complex and brittle.
* **Use `serde_yaml` or other `unsafe-libyaml` wrappers:** Rejected due to memory safety and unmaintained ecosystem reasons as originally detailed in RFC 004.

## 6. Verification Plan

* Rewrite parsing and serialization tests in `crates/pebble/src/parser.rs` and `crates/pebble/src/commands_write.rs` using `serde-saphyr` and `---`.
* Ensure `cargo test` and `cargo run -- check` pass consistently for valid YAML payloads.
* Validate that a `\n---` inside a multi-line YAML string correctly forces premature termination, proving the O(N) boundary scanner is practically aligned with the YAML document specification.
