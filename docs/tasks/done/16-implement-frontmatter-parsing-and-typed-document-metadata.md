---
id: 16
title: Implement frontmatter parsing and typed document metadata
kind: task
type: feature
status: done
priority: high
specs:
  - 9
  - 10
designs: []
adrs: []
depends_on:
  - 15
completed_at: 2026-06-06
---

# Implement frontmatter parsing and typed document metadata

## Goal

Implement core parsing for numbered vibe-doc Markdown documents.

## Scope

- Parse YAML frontmatter from Markdown files.
- Represent common metadata fields with typed Rust structs.
- Represent spec, design, ADR, task, and task-index metadata.
- Preserve enough source location context for useful validation errors.

## Out of Scope

- Walking the repository tree.
- Allocating new IDs.
- JSON Schema validation.

## Checklist

- [x] Common metadata type exists.
- [x] Per-kind metadata types exist.
- [x] Missing or malformed frontmatter can be reported.
- [x] README and `AGENTS.md` can be ignored by parser callers.

## Done Criteria

- [x] Unit tests cover valid and invalid frontmatter.
- [x] Parser distinguishes numbered VDoc documents from unnumbered docs.
- [x] Related specs remain accurate.

## Result

Implemented in `vibe-doc-core`.

The core crate now parses YAML frontmatter from Markdown, distinguishes numbered VDoc documents from unnumbered Markdown, and exposes typed metadata for specs, designs, ADRs, tasks, and the task index. Parse errors preserve source identity and one-based source locations for missing, unterminated, and invalid frontmatter.

Unit tests cover valid per-kind metadata, unnumbered Markdown, malformed YAML, missing required metadata, and non-positive document IDs.
