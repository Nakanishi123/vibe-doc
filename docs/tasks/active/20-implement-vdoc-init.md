---
id: 20
title: Implement vdoc init
kind: task
type: feature
status: planned
priority: high
specs:
  - 9
  - 10
designs: []
adrs: []
depends_on:
  - 15
---

# Implement vdoc init

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

- [ ] Init command creates expected files and directories.
- [ ] Existing files are handled safely.
- [ ] Dry-run reports planned writes.
- [ ] JSON output is stable.

## Done Criteria

- [ ] Integration tests cover a new empty repository.
- [ ] README and `AGENTS.md` are created without frontmatter.
- [ ] Related specs remain accurate.

## Result

Not implemented.

