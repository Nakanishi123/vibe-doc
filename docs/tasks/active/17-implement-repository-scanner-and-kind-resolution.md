---
id: 17
title: Implement repository scanner and kind resolution
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
  - 16
---

## Goal

Scan a repository and discover numbered vibe-doc documents with expected kinds.

## Scope

- Walk the supported docs layout.
- Include specs, designs, ADRs, tasks, and task index.
- Exclude README files and `AGENTS.md` from numbered document parsing.
- Infer expected kind from file path.
- Return document path, parsed metadata, and expected kind.

## Out of Scope

- Full validation command output.
- ID allocation.
- Index rebuild.

## Checklist

- [ ] Scanner discovers every numbered document location.
- [ ] Scanner ignores unnumbered operational docs.
- [ ] Expected kind rules match the spec.
- [ ] Scanner returns stable sorted results.

## Done Criteria

- [ ] Unit or integration tests cover the supported directory layout.
- [ ] Kind mismatch can be represented for later validation.
- [ ] Related specs remain accurate.

## Result

Not implemented.
