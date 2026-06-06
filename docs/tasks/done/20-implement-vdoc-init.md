---
id: 20
title: Implement vdoc init
kind: task
type: feature
status: done
priority: high
specs:
  - 9
  - 10
designs: []
adrs: []
depends_on:
  - 15
completed_at: 2026-06-06
---

## Goal

Implement `vdoc init` to create the initial vibe-doc documentation layout.

## Scope

- Create `AGENTS.md`.
- Create README files without frontmatter.
- Create `docs/schemas/*.json`.
- Create specs, designs, ADR, and task directories.
- Create `docs/tasks/index.md` with task-index frontmatter.
- Support `--dry-run`, `--json`, and `--force`.

## Out of Scope

- Creating product implementation crates.
- Creating initial product specs.
- Creating tasks beyond the empty task index.

## Checklist

- [x] Init command creates expected files and directories.
- [x] Existing files are handled safely.
- [x] Dry-run reports planned writes.
- [x] JSON output is stable.

## Done Criteria

- [x] Integration tests cover a new empty repository.
- [x] README and `AGENTS.md` are created without frontmatter.
- [x] Related specs remain accurate.

## Result

Implemented `vdoc init` with core init planning and CLI execution.

The command now creates the initial docs layout, schema files, README files without frontmatter, `AGENTS.md` without frontmatter, and `docs/tasks/index.md` with task-index frontmatter. It supports `--dry-run`, `--json`, and `--force`, refuses to overwrite existing files unless forced, and reports planned/applied changes in a stable JSON shape.

Verification:

- `cargo fmt --all`
- `cargo test`
