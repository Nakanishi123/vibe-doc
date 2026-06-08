# Agent Instructions

This repository is managed with `vdoc`. Treat Markdown as the source of truth.

## Rules

- Use `vdoc` for supported document discovery, creation, validation, task
  lifecycle, task index rebuild, and task context workflows.
- Use `--dry-run` before mutation commands when the planned changes are not
  obvious.
- Do not run `vdoc init --force` unless the user explicitly asks for repository
  reinitialization.
- `AGENTS.md` and README files do not use frontmatter.
- Keep references ID-based, not path-based.
- Pass bare numeric IDs to `vdoc` commands. For example, use `vdoc show 36`
  and `vdoc context task 36`, not `task-36`.
- Keep operational docs English-first. Frontmatter keys and enum values must
  remain stable English identifiers.

## Common Commands

- Inspect: `vdoc list <specs|designs|adr|tasks>`, `vdoc show <id>`.
- Validate: `vdoc validate`, `vdoc check`.
- Tasks: `vdoc guard task <id>`, `vdoc context task <id>`,
  `vdoc start task <id>`, `vdoc complete task <id> --result "<summary>"`.
- Index: `vdoc rebuild index`.

## Manual Edits

Manual edits are still expected for existing document prose, README and
`AGENTS.md` changes, blocked or dropped tasks, reference rewrites, and ADR/spec
supersession work. After manual edits to numbered docs, run `vdoc rebuild index`
if task frontmatter or placement changed, then run `vdoc validate` and
`vdoc check`.
