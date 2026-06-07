---
id: 23
title: Implement validation and check command output
kind: task
type: feature
status: done
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

- [x] Validate command reports frontmatter and reference issues.
- [x] Check command reports broader consistency issues.
- [x] JSON output has stable shape.
- [x] Exit codes distinguish success from failure.

## Done Criteria

- [x] Tests cover successful and failing validation.
- [x] Error codes match the CLI spec.
- [x] Related specs remain accurate.

## Result

Implemented `vdoc validate` and `vdoc check` in the CLI. Both commands support human-readable and JSON output, optional path filters, stable validation issue codes, and non-zero exit codes when issues are reported.

The core crate now exposes `check_repository`, which extends validation with missing README detection and task index drift detection. CLI integration tests cover success, JSON failure output, path filtering, missing README checks, and task index drift checks.
