---
id: 22
title: Implement vdoc list and show
kind: task
type: feature
status: planned
priority: medium
specs:
  - 9
  - 10
designs:
  - 33
adrs: []
depends_on:
  - 17
---

## Goal

Implement read-only CLI commands for listing and displaying documents.

## Scope

- Implement `vdoc list specs`.
- Implement `vdoc list designs`.
- Implement `vdoc list adr`.
- Implement `vdoc list tasks`.
- Implement `vdoc show <id>`.
- Support JSON output.
- Support path-only and frontmatter-only display modes.

## Out of Scope

- Mutation commands.
- Web UI document rendering.
- Full validation reports.

## Checklist

- [ ] List commands sort by numeric ID.
- [ ] Task and ADR filters work.
- [ ] Show resolves by ID.
- [ ] JSON output is stable.

## Done Criteria

- [ ] Tests cover list filters and show modes.
- [ ] Missing IDs produce machine-readable errors.
- [ ] Related specs remain accurate.

## Result

Not implemented.
