---
id: 24
title: Implement task index rebuild
kind: task
type: feature
status: done
priority: medium
specs:
  - 10
  - 14
designs:
  - 33
adrs: []
depends_on:
  - 17
  - 23
completed_at: "2026-06-07"
---

## Goal

Implement `vdoc rebuild index` to regenerate `docs/tasks/index.md` from task frontmatter.

## Scope

- Read active and done task documents.
- Group tasks into doing, planned, blocked, and done sections.
- Preserve task-index frontmatter.
- Support `--dry-run`.
- Support `--json`.

## Out of Scope

- Starting or completing tasks.
- Defining final advanced grouping policy beyond the documented sections.
- Web UI task dashboard.

## Checklist

- [x] Index generation uses task frontmatter as source of truth.
- [x] Done tasks are listed from `docs/tasks/done/`.
- [x] Dry-run shows proposed output.
- [x] Generated Markdown is stable.

## Done Criteria

- [x] Tests cover empty, active, blocked, and done task sets.
- [x] Rebuilding twice produces no diff.
- [x] Related specs remain accurate.

## Result

Implemented `vdoc rebuild index` with dry-run and JSON output. The core rebuild
logic regenerates `docs/tasks/index.md` from task frontmatter, preserves the
task-index frontmatter, groups tasks into Doing, Planned, Blocked, and Done,
and treats done and dropped tasks as done-section entries.

Added CLI integration tests for empty task sets, active planned/doing tasks,
blocked tasks, done-folder tasks, dry-run output, JSON output, frontmatter
preservation, and repeated rebuild stability. Updated related CLI/task specs
and docs to reflect the implemented command.
