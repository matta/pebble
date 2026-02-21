# UX for Agents: Designing for Humans and Machines

This document provides guidance for designing tools —
CLIs, data formats, and protocols — that serve both human operators and
AI coding agents equally well. The advice is opinionated; where
trade-offs exist, we explain *why* we pick one side.

## Core Principle: One Tool, Two Audiences

A well-designed developer tool should not need a separate "agent mode."
Instead, the same interface should degrade gracefully in both directions:

- **Humans** get readable defaults with color, headers, and prose.
- **Agents** get structured data via a flag (`--json`) or an alternate
  protocol (MCP).

If your tool only works for one audience, you will end up maintaining two
tools. Design for composability from day one.

---

## 1. Data Storage in Git

### The Recommendation: Markdown + YAML Frontmatter

For data that lives in a Git repository and needs to be read by both
humans (in code review, in an editor) and agents (via tool calls or
direct file reads), **Markdown with YAML frontmatter** is the clear
winner.

```markdown
---
id: "proj-0kq"
status: "open"
priority: 0
type: "task"
owner: "matt@rfc20.org"
created_at: "2026-01-15T10:30:00Z"
---
# Deploy staging environment

Run the canary deploy pipeline against the `staging` cluster.
Verify health checks pass before promoting.
```

**Why this format?**

| Concern | Markdown + YAML | JSON | JSONL | Plain YAML |
|:---|:---|:---|:---|:---|
| **Git diffs** | ✅ Clean, line-based | ❌ Brace noise, trailing commas | ✅ Line-per-record | ✅ Clean |
| **Human readability** | ✅ Native rendering everywhere | ⚠️ Passable | ⚠️ Dense | ✅ Good |
| **Token efficiency** | ✅ ~20-30% fewer tokens than JSON | ❌ Verbose | ⚠️ OK per record | ✅ Good |
| **Structured metadata** | ✅ Via frontmatter | ✅ Native | ✅ Native | ✅ Native |
| **Free-form content** | ✅ Body is prose/code | ❌ Must escape everything | ❌ Same | ⚠️ Multi-line awkward |
| **RAG / chunking** | ✅ Headers are natural split points | ❌ No structure | ❌ No structure | ⚠️ Possible |
| **Tooling ecosystem** | ✅ Every editor, every renderer | ✅ Universal parsers | ✅ Streaming parsers | ✅ Good parsers |

### When JSONL Is Better

Markdown is the wrong choice for **append-heavy, record-oriented data**
where each record is a self-contained object and you rarely read the
file directly. For that, JSONL wins:

- **Append-only logs**: Audit trails, event streams, issue databases.
- **Large datasets**: When you have hundreds or thousands of records.
- **Machine-first data**: When no human will ever open the file in an
  editor.

JSONL is git-friendly (one line per record = clean diffs for appends)
and trivially parseable by agents and scripts alike.

**Rule of thumb:** If a human would open the file to *read* it, use
Markdown. If a human only interacts with the data through a tool, use
JSONL.

### One File per Record vs. One Big File

For small-to-medium collections (< 200 records), consider
**one Markdown file per record** in a directory:

```
.pebble/issues/
  proj-0kq.md
  proj-1ab.md
  proj-2cd.md
```

Benefits:
- Git history per-issue is trivial (`git log -- .pebble/issues/proj-0kq.md`).
- Agents can read a single issue without parsing or scanning.
- Merge conflicts are limited to the specific records being edited.

For large or append-heavy collections, a single JSONL file is more
practical — but you lose per-record history clarity.

---

## 2. CLI Design for Agent Use

### The Cardinal Rules

1. **Separate data from diagnostics.** `stdout` is for output data.
   `stderr` is for logs, warnings, progress, and errors. Never mix them.
   An agent piping your output into `jq` must not get a "Fetching..." 
   progress line in the middle of a JSON array.

2. **Use meaningful exit codes.** `0` = success. `1` = general error.
   `2` = usage error (bad arguments). Document any additional codes.
   Agents rely on exit codes to decide whether to retry, abort, or
   continue.

3. **Support `--json` everywhere.** Every command that produces output
   should accept a `--json` flag that emits structured JSON to stdout.
   The default without the flag should be human-readable.

4. **Make operations idempotent.** `pebble sync` run twice in a row
   should produce the same end state. Agents retry. Agents crash
   mid-operation. Idempotency means recovery is free.

5. **Never prompt interactively.** If your CLI needs user confirmation,
   require a `--yes` flag or fail with a clear error message. Agents
   cannot type "y" at a prompt. Either accept `--yes` / `--force` or
   read from a config file.

6. **Disable color and formatting in structured mode.** When `--json` is
   active, suppress ANSI color codes, emoji, and decorative formatting.
   Bonus: respect `NO_COLOR` and detect `isatty()`.

### Output Format Design

**Default (human) mode:**

```
$ pebble list
  ID         STATUS   TITLE
  proj-0kq   open     Deploy staging environment
  proj-1ab   closed   Fix login timeout
  proj-2cd   open     Add rate limiting
```

**Structured (agent) mode:**

```
$ pebble list --json
{"tasks":[
  {"id":"proj-0kq","status":"todo","title":"Deploy staging environment"},
  {"id":"proj-1ab","status":"done","title":"Fix login timeout"},
  {"id":"proj-2cd","status":"todo","title":"Add rate limiting"}
]}
```

**Errors (on stderr, exit code non-zero):**

```
$ pebble show nonexistent-id --json
error: no task found with id 'nonexistent-id'
$ echo $?
1
```

Errors are human-readable text on stderr, even in `--json` mode. This
matches the convention of `gh`, `cargo`, `git`, and other widely-used
CLIs. Agents check the exit code to detect failure and read the stderr
text for diagnostics — structured JSON errors add implementation
complexity without meaningful benefit, since agents parse natural
language error messages reliably. The critical design rule is: **never
exit 0 on failure**. Silent failures (exit 0, empty or wrong output)
are far more dangerous to agents than unstructured error text.

### Discoverability: Help Your Agents Help Themselves

Agents read `--help`. Make it count:

- Put the **most common usage** first, not the flag reference.
- Include **concrete examples** in the help text.
- Consider a `--help-json` flag that emits a machine-readable schema of
  all commands, their arguments, and their output shapes. This lets an
  agent self-discover your tool's capabilities without scraping prose.

```
$ pebble --help-json
{
  "commands": [
    {
      "name": "list",
      "description": "List all issues",
      "flags": [
        {"name": "--json", "description": "Output as JSON array"},
        {"name": "--status", "type": "string", "values": ["open", "closed", "all"]}
      ],
      "output_schema": { ... }
    }
  ]
}
```

---

## 3. MCP vs. CLI: When to Use Which

Model Context Protocol (MCP) and CLI are not competitors — they operate
at different layers and serve different integration patterns.

### CLI: The Reliable Workhorse

**Use a CLI when:**

- The tool is invoked by **shell scripts, CI pipelines, `justfile`
  targets, or human operators** in a terminal.
- The agent framework supports **tool calling via shell execution**
  (most do: Gemini CLI, Claude Code, Cursor, Copilot).
- You want **zero infrastructure** — no server process, no port, no
  socket.
- The operation is **stateless and short-lived**: run, produce output,
  exit.
- You need **maximum portability** — CLI tools work everywhere, in
  every agent framework, and in every CI system.

**The CLI is your primary interface.** Build it first, build it well.

### MCP: The Structured Bridge

**Use an MCP server when:**

- The agent needs to **discover tools dynamically** at runtime without
  prior knowledge of the CLI's commands.
- You need **persistent state or sessions** — e.g., keeping a database
  connection open across multiple operations.
- The interaction involves **bidirectional communication** — the tool
  needs to push updates or intermediate results back to the agent.
- You are integrating with an **MCP-native host** (VS Code + Copilot,
  Claude Desktop, Cline) and want first-class tool registration.
- You want **rich type information** — MCP tool definitions include
  JSON Schema for inputs and outputs, giving agents strong type safety.

### The Pragmatic Approach: CLI-First, MCP as a Wrapper

For most project-specific tools (like Pebble), the right architecture
is:

```
┌─────────────┐     ┌─────────────┐     ┌──────────────┐
│  Agent /    │     │  MCP Server │     │  CLI Binary  │
│  Human      │────▶│  (optional) │────▶│  (pebble)    │
│             │     │  thin shim  │     │  core logic  │
└─────────────┘     └─────────────┘     └──────────────┘
                           │
                    Translates MCP tool
                    calls into CLI
                    invocations with
                    --json flag
```

1. **Build the CLI first.** It must be excellent for both humans and
   agents. This means `--json` output, clean exit codes, and
   idempotent operations.

2. **Wrap it in MCP later (if needed).** An MCP server can be a thin
   shim that translates `tools/call` messages into `pebble <command>
   --json` invocations and returns the parsed output. This gives you
   MCP discoverability and type safety without duplicating logic.

3. **Avoid building MCP-only tools.** If your tool only works via MCP,
   you lose the ability to use it in CI, in shell scripts, or with
   agent frameworks that use shell-based tool calling.

### Decision Matrix

| Factor | CLI | MCP | Both |
|:---|:---|:---|:---|
| CI/CD pipelines | ✅ | ❌ | ✅ |
| Shell scripts / `justfile` | ✅ | ❌ | ✅ |
| Human terminal use | ✅ | ❌ | ✅ |
| Agent frameworks (shell-based) | ✅ | ⚠️ | ✅ |
| Agent frameworks (MCP-native) | ⚠️ | ✅ | ✅ |
| Dynamic tool discovery | ❌ | ✅ | ✅ |
| Typed schemas for I/O | ⚠️ | ✅ | ✅ |
| Zero infrastructure | ✅ | ❌ | ⚠️ |
| Persistent connections | ❌ | ✅ | ✅ |

---

## 4. Model-Specific Format Considerations

Different LLM families have different training biases. This matters
less than it used to — all major models handle JSON well — but is still
worth noting when optimizing for specific contexts.

| Model Family | Preferred Structured Format | Notes |
|:---|:---|:---|
| **OpenAI** (GPT-4o+) | JSON | Heavy fine-tuning on JSON schemas. Native structured outputs. |
| **Anthropic** (Claude) | JSON for tool I/O; XML for large context | Claude distinguishes instructions from data better with XML wrapping. |
| **Google** (Gemini) | JSON | Robust with multiple formats; defaults to JSON for tool calls. |
| **DeepSeek / Qwen** | Markdown or JSON | Better reasoning performance with Markdown tables/lists vs. deep JSON trees. |
| **Llama / OSS** | JSON | Follows the mainstream convention. |

**Practical takeaway:** JSON is the universal format for tool I/O. Use
it as your `--json` output. You do not need to support XML or Markdown
output modes for agent consumption — JSON covers all models adequately.

If your tool produces *large context* that an agent needs to reason
over (e.g., a multi-page log or a complex state dump), consider
Markdown formatting for readability — all models handle it well, and it
uses fewer tokens than JSON for prose-heavy content.

---

## 5. Putting It All Together: A Checklist

When building a tool intended for dual human/agent use:

### Data Storage
- [ ] Prose-heavy records → Markdown with YAML frontmatter
- [ ] Record-oriented data → JSONL (one record per line)
- [ ] Small collections → consider one file per record
- [ ] All data files → committed to Git, diffable, no binary formats

### CLI Design
- [ ] Every command supports `--json` for structured output
- [ ] `stdout` = data, `stderr` = diagnostics (never mixed)
- [ ] Meaningful exit codes (0, 1, 2 minimum)
- [ ] No interactive prompts (use `--yes` flag instead)
- [ ] Idempotent operations where possible
- [ ] `NO_COLOR` / `isatty()` support
- [ ] `--help` with concrete examples
- [ ] Consider `--help-json` for machine-readable discoverability

### Protocol Layer
- [ ] CLI is the primary, always-supported interface
- [ ] MCP server is optional, wraps the CLI
- [ ] MCP tool definitions include JSON Schema for inputs/outputs
- [ ] No MCP-only functionality — everything is also available via CLI

### Testing
- [ ] Test both human and `--json` output modes
- [ ] Test exit codes for success, failure, and usage errors
- [ ] Test idempotency (run the same command twice, same result)
- [ ] Test `stderr` isolation (structured output never leaks to stderr)
- [ ] Test `stdout` purity (diagnostics never leak to `stdout` in `--json` mode)
