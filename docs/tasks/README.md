# Tasks

Tasks define implementation work units.

Tasks are mutable documents. Their frontmatter captures lifecycle state and relationships; their body captures goal, scope, checklist, done criteria, and result.

## Locations

- Active work belongs in `docs/tasks/active/`.
- Completed or dropped work belongs in `docs/tasks/done/`.
- `docs/tasks/index.md` is generated from task frontmatter by `vdoc rebuild index`.

## Frontmatter

```yaml
---
id: 11
title: Example Task
kind: task
type: feature
status: planned
priority: medium
specs: []
designs: []
adrs: []
depends_on: []
---
```

Allowed task types are `feature`, `bug`, `refactor`, `chore`, `docs`, `test`, and `spike`.

Allowed task statuses are `planned`, `doing`, `blocked`, `done`, and `dropped`.

Allowed priorities are `low`, `medium`, `high`, and `critical`.
