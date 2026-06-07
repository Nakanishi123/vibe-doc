---
id: 14
title: vibe-doc Task Model Specification
kind: spec
tags:
  - vibe-doc
  - tasks
  - lifecycle
---

## Overview

This spec defines vibe-doc task metadata, task lifecycle rules, task body conventions, and task index behavior.

## Tasks

A task defines an implementation work unit.

Task examples:

- feature
- bug fix
- refactor
- chore
- documentation update
- test work
- investigation

Required task frontmatter includes `status`.

Allowed task types:

- `feature`
- `bug`
- `refactor`
- `chore`
- `docs`
- `test`
- `spike`

Allowed task statuses:

- `planned`
- `doing`
- `blocked`
- `done`
- `dropped`

Allowed priorities:

- `low`
- `medium`
- `high`
- `critical`

## Task Lifecycle

Tasks start in `docs/tasks/active/`.

Standard transitions:

```txt
planned -> doing -> done
planned -> blocked -> doing -> done
planned -> dropped
```

When a task is completed:

1. Update the checklist.
2. Set `status: done`.
3. Set `completed_at`.
4. Add a result summary.
5. Move the file from `docs/tasks/active/` to `docs/tasks/done/`.
6. Rebuild `docs/tasks/index.md`.
7. Run validation.

When a task is dropped:

1. Set `status: dropped`.
2. Record the reason in `Result` or `Notes`.
3. Move the file to `docs/tasks/done/`.

## Task Body Template

Recommended task body:

```md
## Goal

Describe the purpose of the task.

## Scope

- Scope item

## Out of Scope

- Out-of-scope item

## Checklist

- [ ] Work item

## Done Criteria

- [ ] Related specs are satisfied.
- [ ] Related designs are followed.
- [ ] Related ADRs are not violated.
- [ ] Tests pass.

## Result

Not implemented.
```

After completion, `Result` should summarize implementation, changed files, and follow-ups.

## Task Index

`docs/tasks/index.md` is a human-oriented task list.

It has its own frontmatter and `kind: task-index`.

The task index is not the source of truth. Each task file frontmatter is the source of truth.

The CLI regenerates the index with `vdoc rebuild index`.

The generated index preserves the task-index frontmatter and replaces the body
with stable sections:

- Doing
- Planned
- Blocked
- Done

Tasks with `done` or `dropped` status are listed in the Done section.

## Reference Rules

Task frontmatter references related documents by ID.

Example:

```yaml
specs:
  - 8
designs:
  - 9
adrs:
  - 10
depends_on:
  - 11
```

Task dependencies refer to other task IDs. A task is ready only when every dependency is complete.
