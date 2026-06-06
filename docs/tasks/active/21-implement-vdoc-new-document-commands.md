---
id: 21
title: Implement vdoc new document commands
kind: task
type: feature
status: planned
priority: high
specs:
  - 9
  - 10
  - 14
designs: []
adrs: []
depends_on:
  - 18
  - 19
---

# Implement vdoc new document commands

## Goal

Implement commands that create new numbered VDoc documents.

## Scope

- Implement `vdoc new spec <title>`.
- Implement `vdoc new design <title>`.
- Implement `vdoc new adr <title>`.
- Implement `vdoc new task <title>`.
- Use global ID allocation.
- Generate default frontmatter and body templates.
- Validate generated documents before writing.

## Out of Scope

- Completing or starting tasks.
- Custom user templates.
- Interactive prompts.

## Checklist

- [ ] Spec creation works.
- [ ] Design creation works.
- [ ] ADR creation works.
- [ ] Task creation works.
- [ ] Metadata flags are supported for task and ADR creation.

## Done Criteria

- [ ] Integration tests cover each document kind.
- [ ] New tasks are created under `docs/tasks/active/`.
- [ ] Related specs remain accurate.

## Result

Not implemented.

