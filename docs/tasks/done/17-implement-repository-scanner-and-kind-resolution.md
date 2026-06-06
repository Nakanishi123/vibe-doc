---
id: 17
title: Implement repository scanner and kind resolution
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
  - 16
completed_at: 2026-06-06
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

- [x] Scanner discovers every numbered document location.
- [x] Scanner ignores unnumbered operational docs.
- [x] Expected kind rules match the spec.
- [x] Scanner returns stable sorted results.

## Done Criteria

- [x] Unit or integration tests cover the supported directory layout.
- [x] Kind mismatch can be represented for later validation.
- [x] Related specs remain accurate.

## Result

Implemented in `vibe-doc-core`.

The core crate now scans the supported docs layout, ignores README and operational Markdown files, infers expected document kind from path, and returns parsed numbered documents with their expected kind for later validation. Scan results are sorted by repository-relative path for stable output, and parse or IO failures are represented through a scanner error type.

Unit tests cover expected kind resolution, supported layout discovery, stable sorting, ignored operational docs, parse errors, and representing kind mismatches.
