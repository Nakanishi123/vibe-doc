---
id: 10
title: vibe-doc CLI Specification
kind: spec
tags:
  - vibe-doc
  - cli
  - validation
---

## Overview

The product and repository name is `vibe-doc`. The CLI command name is `vdoc`.

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

## Argument Parsing

The CLI should use `clap` for argument parsing and help generation.

The command tree, option names, and accepted enum values remain part of this
specification. `clap` is an implementation dependency for parsing and usage
text, not the owner of product behavior. Command handlers should still translate
parsed arguments into typed options and call `vibe-doc-core` for repository
behavior.

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

`vdoc init` creates the vibe-doc documentation structure:

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

- frontmatter exists for numbered vibe-doc documents
- `id`, `title`, and `kind` exist for numbered vibe-doc documents
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

`vdoc rebuild index`:

- reads task frontmatter from `docs/tasks/active/` and `docs/tasks/done/`
- preserves `docs/tasks/index.md` frontmatter
- groups tasks into Doing, Planned, Blocked, and Done sections
- lists `done` and `dropped` task statuses in the Done section
- supports `--dry-run`
- supports `--json`

`vdoc start task <id>`:

- sets `status: doing`
- sets `started_at` to the local current date by default
- rebuilds the task index
- runs validation
- supports `--date YYYY-MM-DD`
- supports `--dry-run`
- supports `--json`

`vdoc complete task <id>`:

- sets `status: done`
- sets `completed_at` to the local current date by default
- updates `Result`
- moves the task from `docs/tasks/active/` to `docs/tasks/done/`
- rebuilds the task index
- runs validation
- supports `--date YYYY-MM-DD`
- supports `--result`
- supports `--dry-run`
- supports `--json`

## Context and Guard

`vdoc context task <id>` returns the files needed to implement or review a task:

- the task file
- referenced specs
- referenced designs
- referenced ADRs

Context output is deterministic. It orders items as task, specs by ID, designs
by ID, and ADRs by ID.

Context does not include `AGENTS.md`. Agent runners such as Codex read
repository agent instructions through their own bootstrap flow, so including it
in task context would duplicate that input.

Context exits non-zero when the task references a spec, design, or ADR that
cannot be resolved. It must not silently omit referenced documents.

With `--json`, context output includes:

- `command: "context task"`
- `task_id`
- `items`

Each context item includes `kind`, `path`, and `content`. Numbered documents
also include `id` and `title`. Context `content` contains the Markdown body
without YAML frontmatter; metadata needed for ordering and identification is
returned through item fields instead.

`vdoc guard task <id>` verifies that a task is ready to start:

- the task exists
- the task is active
- the task status is `planned` or `doing`
- dependencies are complete
- related specs, designs, and ADRs exist
- related ADRs are not rejected or superseded

Guard exits successfully only when the task is ready. Guard exits non-zero when
readiness issues are reported.

With `--json`, guard output includes:

- `command: "guard task"`
- `task_id`
- `ready`
- `issue_count`
- `issues`

Guard issue objects include stable `code` and `message` fields, plus `id` or
`path` when available.

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
