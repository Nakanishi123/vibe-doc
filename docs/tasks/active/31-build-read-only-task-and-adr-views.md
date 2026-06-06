---
id: 31
title: Build read-only task and ADR views
kind: task
type: feature
status: planned
priority: medium
specs:
  - 11
  - 14
designs: []
adrs: []
depends_on:
  - 27
  - 28
---

## Goal

Build focused Web UI views for tasks and ADRs.

## Scope

- Add task list view.
- Add task detail view.
- Add task filters for status, type, priority, and tag.
- Add ADR list view.
- Show ADR status, date, tags, supersedes, and superseded_by.
- Surface validation state when available.

## Out of Scope

- Task start and complete buttons.
- Agent integration UI.
- Markdown editing.

## Checklist

- [ ] Task list groups active, blocked, and done tasks.
- [ ] Task detail shows dependencies and related documents.
- [ ] ADR list shows decision metadata.
- [ ] Filters are ergonomic for repeated use.

## Done Criteria

- [ ] Typecheck or build succeeds.
- [ ] Views consume documented API shapes.
- [ ] Related specs remain accurate.

## Result

Not implemented.
