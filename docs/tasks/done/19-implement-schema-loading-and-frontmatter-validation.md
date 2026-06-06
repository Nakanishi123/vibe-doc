---
id: 19
title: Implement schema loading and frontmatter validation
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
  - 17
---

## Goal

Validate numbered VDoc document frontmatter against the repository schema files and built-in rules.

## Scope

- Load schemas from `docs/schemas/`.
- Validate common document metadata.
- Validate spec, design, ADR, task, and task-index metadata.
- Report validation errors with stable codes.
- Keep validation reusable from CLI and server.

## Out of Scope

- Implementing CLI formatting.
- Implementing Web UI validation display.
- Auto-fixing invalid documents.

## Checklist

- [x] Schema loader exists.
- [x] Kind-specific validation exists.
- [x] Error codes match the CLI spec.
- [x] Validation can run against scanner output.

## Done Criteria

- [x] Tests cover valid docs and representative invalid docs.
- [x] README files and `AGENTS.md` are not required to have frontmatter.
- [x] Related specs remain accurate.

## Result

Implemented reusable core validation for schema loading and parsed repository documents. The core crate now loads `docs/schemas/*.json`, exposes stable validation codes and reports, validates duplicate IDs, kind/location mismatches, empty required titles, task folder/status consistency, ADR supersession requirements, and kind-aware ID references.

Validation can run directly from scanner output through `validate_documents` or scan a repository through `validate_repository`. Tests cover schema loading, valid repositories, unnumbered README/`AGENTS.md` files, parse-error code mapping, duplicate IDs, kind mismatches, broken references, missing dependencies, task folder status mismatches, and superseded ADRs without replacements.
