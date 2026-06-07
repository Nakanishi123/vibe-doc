---
id: 25
title: Implement task start and complete lifecycle
kind: task
type: feature
status: done
priority: high
specs:
  - 10
  - 14
designs:
  - 33
adrs: []
depends_on:
  - 23
  - 24
completed_at: "2026-06-07"
---

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

- [x] Start updates status to `doing`.
- [x] Complete updates status to `done`.
- [x] Complete moves task file to `docs/tasks/done/`.
- [x] Index rebuild is triggered.
- [x] Dry-run and JSON behavior are supported.

## Done Criteria

- [x] Tests cover start, complete, invalid IDs, and invalid statuses.
- [x] Command behavior remains non-interactive when flags are provided.
- [x] Related specs remain accurate.

## Result

Implemented task lifecycle mutation support in `vibe-doc-core` and wired it to
the CLI as `vdoc start task <id>` and `vdoc complete task <id>`. Start updates
planned or blocked tasks to `doing` and records `started_at`; complete requires a
`doing` task, records `completed_at`, updates `Result` when `--result` is
provided, moves the file into `docs/tasks/done/`, rebuilds the task index, and
runs repository validation after writes.

Added dry-run and JSON output for lifecycle commands, machine-readable JSON
errors for missing tasks and invalid statuses, CLI spec coverage for the
supported flags, and integration tests covering start, complete, dry-run JSON
behavior, invalid IDs, and invalid statuses.
