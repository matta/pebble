# Product Guidelines: Pebble

## Documentation & Prose
- **Brevity & Completeness**: Documentation, comments, and design specs should aim for conciseness while ensuring thoroughness. Every entry must be comprehensive and clear, providing all necessary details without unnecessary verbosity.

## CLI Design
- **Human-Friendly Output**: By default, Pebble commands should output clean, formatted text (e.g., tables or lists) for easy readability by developers.
- **Machine-Readable Options**: Every command that outputs data must include an optional `--json` flag to provide a structured, machine-readable format for automation and tooling.
- **Aesthetic**: Use standard ANSI colors (Green for success, Red for errors, Blue for info) to provide clear visual cues while maintaining broad terminal compatibility.

## Git & Development Workflow
- **Narrative Commit Messages**: Use clear, descriptive sentences in commit messages to explain the context and purpose of each change.
- **Test-Driven Development (TDD)**: Mandatory. Every new feature or bug fix must start with a failing test case that defines the desired behavior.
- **High Test Coverage**: Strive for comprehensive test coverage, ensuring that core logic, edge cases, and error handling are thoroughly exercised.

## Code Style & Documentation
- **Extensive Public API Documentation**: Every public function, struct, and module must have a clear documentation comment explaining its purpose, parameters, and return values.
- **Internal Logic Comments**: While public APIs require extensive documentation, internal logic should be commented only where necessary to explain non-obvious design choices.
