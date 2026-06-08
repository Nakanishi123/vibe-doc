---
id: 31
title: Build read-only task and ADR views
kind: task
type: feature
status: done
priority: medium
specs:
- 11
- 14
designs:
- 34
adrs: []
depends_on:
- 27
- 28
started_at: 2026-06-08
completed_at: 2026-06-08
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

- [x] Task list groups active, blocked, and done tasks.
- [x] Task detail shows dependencies and related documents.
- [x] ADR list shows decision metadata.
- [x] Filters are ergonomic for repeated use.

## Done Criteria

- [x] Typecheck or build succeeds.
- [x] Views consume documented API shapes.
- [x] Related specs remain accurate.

## Result

Implemented read-only Web UI task and ADR views. Added task grouping and filters, task detail dependency/related-document panels, ADR decision metadata table, and wired the routes to the documented read APIs. Verified with mise exec -- pnpm web:typecheck, mise exec -- pnpm web:build, cargo test, vdoc validate, and vdoc check.
