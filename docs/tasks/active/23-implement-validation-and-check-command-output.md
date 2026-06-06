---
id: 23
title: Implement validation and check command output
kind: task
type: feature
status: planned
priority: high
specs:
  - 9
  - 10
designs:
  - 33
adrs: []
depends_on:
  - 19
---

## Goal

Expose validation and consistency checks through CLI commands.

## Scope

- Implement `vdoc validate`.
- Implement path-specific validation.
- Implement `vdoc check`.
- Support human-readable output.
- Support JSON output with stable error codes.
- Include task index drift and missing README/schema checks where feasible.

## Out of Scope

- Implementing `--fix`.
- Web UI validation display.
- Auto-generating task index content.

## Checklist

- [ ] Validate command reports frontmatter and reference issues.
- [ ] Check command reports broader consistency issues.
- [ ] JSON output has stable shape.
- [ ] Exit codes distinguish success from failure.

## Done Criteria

- [ ] Tests cover successful and failing validation.
- [ ] Error codes match the CLI spec.
- [ ] Related specs remain accurate.

## Result

Not implemented.
