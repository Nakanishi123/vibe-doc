---
id: 16
title: Implement frontmatter parsing and typed document metadata
kind: task
type: feature
status: planned
priority: high
specs:
  - 9
  - 10
designs: []
adrs: []
depends_on:
  - 15
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

- [ ] Common metadata type exists.
- [ ] Per-kind metadata types exist.
- [ ] Missing or malformed frontmatter can be reported.
- [ ] README and `AGENTS.md` can be ignored by parser callers.

## Done Criteria

- [ ] Unit tests cover valid and invalid frontmatter.
- [ ] Parser distinguishes numbered VDoc documents from unnumbered docs.
- [ ] Related specs remain accurate.

## Result

Not implemented.

