# Agent Instructions

This repository is managed as a VDoc project before the real `vdoc` CLI exists.

## Operating Rules

- Treat repository Markdown as the source of truth.
- Do not run `vdoc` commands until the CLI is implemented.
- Maintain VDoc frontmatter manually for specs, designs, ADRs, tasks, and the task index.
- `AGENTS.md` and README files do not use frontmatter.
- Assign new document IDs by scanning numbered VDoc document frontmatter and using the next global positive integer.
- Keep document references ID-based, not path-based.
- Keep operational documentation English-first. Frontmatter keys and enum values must remain stable English identifiers.

## Managed Documents

Repository documentation files are:

- `AGENTS.md`
- `docs/README.md`
- `docs/specs/README.md`
- `docs/specs/*.md`
- `docs/designs/README.md`
- `docs/designs/*.md`
- `docs/adr/README.md`
- `docs/adr/*.md`
- `docs/tasks/README.md`
- `docs/tasks/index.md`
- `docs/tasks/active/*.md`
- `docs/tasks/done/*.md`

## Manual Task Lifecycle

- Create new tasks in `docs/tasks/active/`.
- Use task statuses `planned`, `doing`, `blocked`, `done`, or `dropped`.
- Move completed or dropped tasks to `docs/tasks/done/`.
- Update `docs/tasks/index.md` by hand until `vdoc rebuild index` exists.
- Record implementation results in the task body, not in frontmatter.

## Validation Expectations

Before finishing documentation work, check that:

- IDs are unique positive integers.
- `kind` matches the file location.
- ADR and task statuses use allowed values.
- Task references point to existing document IDs.
- The task index reflects current active and done tasks.
