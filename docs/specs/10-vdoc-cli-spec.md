---
id: 10
title: VDoc CLI Specification
kind: spec
tags:
  - vdoc
  - cli
  - validation
---

# VDoc CLI Specification

## Overview

The CLI command name is `vdoc`.

The CLI is implemented in Rust and designed for both humans and LLMs.

## Principles

The CLI must support:

- non-interactive operation
- JSON output
- dry-run mode for mutation commands where useful
- machine-readable errors
- validation before mutations
- ID-based operations
- safe use by LLM agents

## Commands

MVP CLI commands:

- `vdoc init`
- `vdoc new spec <title>`
- `vdoc new design <title>`
- `vdoc new adr <title>`
- `vdoc new task <title>`
- `vdoc list`
- `vdoc show`
- `vdoc validate`
- `vdoc check`
- `vdoc start task <id>`
- `vdoc complete task <id>`
- `vdoc rebuild index`
- `vdoc context task <id>`
- `vdoc guard task <id>`
- `vdoc schema <kind>`
- `vdoc explain`
- `vdoc server`

Mutation commands should support `--dry-run` where useful. JSON output should be available through `--json`.

Destructive or state-changing commands should require or support `--yes`.

## Init

`vdoc init` creates the VDoc documentation structure:

- `AGENTS.md`
- README files
- `docs/schemas/*.json`
- `docs/specs/`
- `docs/designs/`
- `docs/adr/`
- `docs/tasks/index.md`
- `docs/tasks/active/`
- `docs/tasks/done/`

`AGENTS.md` and README files do not use frontmatter. `docs/tasks/index.md` does use frontmatter.

Useful options:

- `--dry-run`
- `--json`
- `--force`

## New Documents

Creation commands:

```sh
vdoc new spec <title>
vdoc new design <title>
vdoc new adr <title>
vdoc new task <title>
```

Task creation supports metadata such as:

- `--type`
- `--priority`
- `--spec`
- `--design`
- `--adr`
- `--depends-on`

ADR creation supports metadata such as:

- `--status`
- `--tag`
- `--related-design`

## List and Show

List commands:

```sh
vdoc list specs
vdoc list designs
vdoc list adr
vdoc list tasks
```

Task and ADR lists support filters by status, type, priority, and tag where applicable.

Show commands:

```sh
vdoc show 11
vdoc show task 11
vdoc show adr 10
```

Useful show options:

- `--json`
- `--path-only`
- `--frontmatter-only`

## Validation

`vdoc validate` checks:

- frontmatter exists for numbered VDoc documents
- `id`, `title`, and `kind` exist for numbered VDoc documents
- IDs are globally unique positive integers
- `kind` matches file location
- ADR status is allowed
- task status is allowed
- task type is allowed
- priority is allowed
- referenced IDs exist
- task dependencies exist
- done tasks live in `docs/tasks/done/`
- active tasks live in `docs/tasks/active/`
- ADR supersession references are valid
- Markdown links are valid where possible

`vdoc check` performs broader consistency checks:

- task index drift
- active or done folder mismatch
- missing schemas
- missing README files
- suspicious title duplication
- missing related documents
- active design or task references to rejected or superseded ADRs

## Error Codes

Recommended error codes:

- `BAD_FRONTMATTER`
- `MISSING_REQUIRED_FIELD`
- `INVALID_ID`
- `DUPLICATE_ID`
- `INVALID_KIND`
- `INVALID_STATUS`
- `INVALID_TYPE`
- `INVALID_PRIORITY`
- `BROKEN_REFERENCE`
- `MISSING_DEPENDENCY`
- `TASK_DONE_IN_ACTIVE`
- `TASK_ACTIVE_IN_DONE`
- `ADR_SUPERSEDED_WITHOUT_REPLACEMENT`
- `INDEX_OUT_OF_SYNC`
- `SCHEMA_NOT_FOUND`
- `README_NOT_FOUND`

## Task Commands

`vdoc start task <id>`:

- sets `status: doing`
- sets `started_at`
- rebuilds the task index
- runs validation

`vdoc complete task <id>`:

- sets `status: done`
- sets `completed_at`
- updates `Result`
- moves the task from `docs/tasks/active/` to `docs/tasks/done/`
- rebuilds the task index
- runs validation

## Context and Guard

`vdoc context task <id>` returns the files needed to implement or review a task:

- `AGENTS.md`
- the task file
- referenced specs
- referenced designs
- referenced ADRs

`vdoc guard task <id>` verifies that a task is ready to start:

- the task exists
- the task is active
- the task status is `planned` or `doing`
- dependencies are complete
- related specs, designs, and ADRs exist
- related ADRs are not rejected or superseded

## Schema and Explain

`vdoc schema <kind>` prints the schema for a document kind.

`vdoc explain` prints LLM-oriented usage instructions.

## Server Command

`vdoc server` starts the API server and serves the SPA.

Useful options:

- `--host`
- `--port`
- `--json`

`vdoc server` does not generate static HTML.

