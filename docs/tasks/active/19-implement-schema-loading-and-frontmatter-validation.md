---
id: 19
title: Implement schema loading and frontmatter validation
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
  - 17
---

## Goal

Validate numbered VDoc document frontmatter against the repository schema files and built-in rules.

## Scope

- Load schemas from `docs/schemas/`.
- Validate common document metadata.
- Validate spec, design, ADR, task, and task-index metadata.
- Report validation errors with stable codes.
- Keep validation reusable from CLI and server.

## Out of Scope

- Implementing CLI formatting.
- Implementing Web UI validation display.
- Auto-fixing invalid documents.

## Checklist

- [ ] Schema loader exists.
- [ ] Kind-specific validation exists.
- [ ] Error codes match the CLI spec.
- [ ] Validation can run against scanner output.

## Done Criteria

- [ ] Tests cover valid docs and representative invalid docs.
- [ ] README files and `AGENTS.md` are not required to have frontmatter.
- [ ] Related specs remain accurate.

## Result

Not implemented.
