---
id: 24
title: Implement task index rebuild
kind: task
type: feature
status: planned
priority: medium
specs:
  - 10
  - 14
designs: []
adrs: []
depends_on:
  - 17
  - 23
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

- [ ] Index generation uses task frontmatter as source of truth.
- [ ] Done tasks are listed from `docs/tasks/done/`.
- [ ] Dry-run shows proposed output.
- [ ] Generated Markdown is stable.

## Done Criteria

- [ ] Tests cover empty, active, blocked, and done task sets.
- [ ] Rebuilding twice produces no diff.
- [ ] Related specs remain accurate.

## Result

Not implemented.
