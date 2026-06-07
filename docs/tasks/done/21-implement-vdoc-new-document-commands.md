---
id: 21
title: Implement vdoc new document commands
kind: task
type: feature
status: done
completed_at: "2026-06-06T13:47:00Z"
priority: high
specs:
  - 9
  - 10
  - 14
designs:
  - 33
adrs: []
depends_on:
  - 18
  - 19
---

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

- [x] Spec creation works.
- [x] Design creation works.
- [x] ADR creation works.
- [x] Task creation works.
- [x] Metadata flags are supported for task and ADR creation.

## Done Criteria

- [x] Integration tests cover each document kind.
- [x] New tasks are created under `docs/tasks/active/`.
- [x] Related specs remain accurate.

## Result

Implemented `vdoc new spec`, `vdoc new design`, `vdoc new adr`, and `vdoc new task` commands in `vibe-doc-cli` and `vibe-doc-core`. Added `new.rs` module for core allocation, markdown generation, validation, and writing. Integrated JSON outputs and dry runs as specified. Added extensive test suite to cover operations and CLI flag parsing.
