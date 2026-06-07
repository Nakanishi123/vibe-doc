---
id: 33
title: vibe-doc CLI Core Design
kind: design
specs:
  - 9
  - 10
  - 13
  - 14
adrs: []
tags:
  - vibe-doc
  - cli
  - core
---

## Overview

The CLI core is split into a small command-line crate and a reusable core crate.

`vibe-doc-core` owns repository rules, document parsing, ID allocation, validation, task lifecycle helpers, and filesystem mutation planning. `vibe-doc-cli` owns process concerns: argument parsing, current directory selection, stdout and stderr formatting, JSON output, and exit codes.

This keeps the same behavior available to the future server and Web UI without routing shared logic through the CLI binary.

## Related Specs

- 9
- 10
- 13
- 14

## Components

`crates/vibe-doc-core` contains:

- Markdown frontmatter parsing and typed metadata.
- Repository scanning and expected-kind resolution.
- Global ID allocation and filename generation.
- Schema loading and built-in validation rules.
- Init planning and write execution.
- Task index rebuild planning and write execution.
- Future task lifecycle mutation helpers.

`crates/vibe-doc-cli` contains:

- Command parsing and option handling through `clap`.
- Human-readable output.
- Stable JSON output.
- Machine-readable error payloads.
- Exit status mapping.

## Command Flow

Mutation commands should follow this shape:

1. Parse CLI arguments into command options with `clap`.
2. Resolve the repository root from the current directory.
3. Ask `vibe-doc-core` to build a plan or perform the operation.
4. For `--dry-run`, print the plan without writing files.
5. For `--json`, emit stable field names and enum values.
6. For human output, keep messages short and path-oriented.

Read-only commands should scan through `vibe-doc-core`, sort predictably, and format results in the CLI layer.

## Data Model

The core crate should expose Rust structs and enums for shared behavior instead of returning preformatted strings.

Stable external strings belong behind methods such as `as_str()` so JSON output and future API responses can reuse the same values.

Paths returned from core APIs should be repository-relative where possible. CLI and server layers may convert path separators for display or transport.

## Error Handling

Core errors should preserve source `io::Error` values for diagnostics while exposing stable variants that callers can map to external codes.

The CLI should:

- Print human errors to stderr by default.
- Print JSON error objects when `--json` is passed.
- Return non-zero exit codes for failed commands.
- Refuse unsafe overwrites unless an explicit force or confirmation option exists.

## Templates

Built-in Markdown and schema templates live in `vibe-doc-core` while the CLI remains a caller.

Initial templates should be conservative:

- Create repository structure and README guidance.
- Create schemas.
- Create an empty task index.
- Do not create product-specific specs, designs, ADRs, or implementation tasks.

Future template customization should use a non-managed path such as `.vibe-doc/templates/` and should not change the default managed layout.

## Testing Strategy

Core tests should cover planning and filesystem behavior with temporary repositories.

CLI integration tests should execute the `vdoc` binary and verify:

- Created files and directories.
- Frontmatter presence or absence.
- Existing-file conflict behavior.
- Dry-run behavior.
- Stable JSON shape.

Later commands should add similar tests for JSON output, dry-run planning, and repository mutation safety.

## Alternatives Considered

Putting command logic directly in `vibe-doc-cli` would be faster at first, but it would force the server to duplicate CLI behavior or shell out to `vdoc`.

Keeping argument parsing hand-written avoids a dependency, but it becomes brittle
as nested commands, repeated flags, enum values, and generated help text expand.
`clap` should be used for parsing while repository behavior stays in
`vibe-doc-core`.
