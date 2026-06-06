---
id: 25
title: Implement task start and complete lifecycle
kind: task
type: feature
status: planned
priority: high
specs:
  - 10
  - 14
designs: []
adrs: []
depends_on:
  - 23
  - 24
---

# Implement task start and complete lifecycle

## Goal

Implement safe CLI operations for starting and completing tasks.

## Scope

- Implement `vdoc start task <id>`.
- Implement `vdoc complete task <id>`.
- Set `started_at` and `completed_at`.
- Move completed tasks from active to done.
- Update task result content from command flags.
- Rebuild task index after lifecycle changes.
- Run validation after mutations.

## Out of Scope

- Web UI mutation APIs.
- Agent-run acceptance workflow.
- Dropped-task command support unless it naturally falls out of the implementation.

## Checklist

- [ ] Start updates status to `doing`.
- [ ] Complete updates status to `done`.
- [ ] Complete moves task file to `docs/tasks/done/`.
- [ ] Index rebuild is triggered.
- [ ] Dry-run and JSON behavior are supported.

## Done Criteria

- [ ] Tests cover start, complete, invalid IDs, and invalid statuses.
- [ ] Command behavior remains non-interactive when flags are provided.
- [ ] Related specs remain accurate.

## Result

Not implemented.

