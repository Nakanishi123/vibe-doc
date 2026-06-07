---
id: 22
title: Implement vdoc list and show
kind: task
type: feature
status: done
priority: medium
specs:
  - 9
  - 10
designs:
  - 33
adrs: []
depends_on:
  - 17
---

## Goal

Implement read-only CLI commands for listing and displaying documents.

## Scope

- Implement `vdoc list specs`.
- Implement `vdoc list designs`.
- Implement `vdoc list adr`.
- Implement `vdoc list tasks`.
- Implement `vdoc show <id>`.
- Support JSON output.
- Support path-only and frontmatter-only display modes.

## Out of Scope

- Mutation commands.
- Web UI document rendering.
- Full validation reports.

## Checklist

- [x] List commands sort by numeric ID.
- [x] Task and ADR filters work.
- [x] Show resolves by ID.
- [x] JSON output is stable.

## Done Criteria

- [x] Tests cover list filters and show modes.
- [x] Missing IDs produce machine-readable errors.
- [x] Related specs remain accurate.

## Result

Implemented `vdoc list specs`, `vdoc list designs`, `vdoc list adr`, `vdoc list tasks`,
and `vdoc show [spec|design|adr|task] <id>` through `clap`-based command parsing.

List output is sorted by numeric document ID and supports stable JSON summaries. ADR
lists support `--status` and `--tag`; task lists support `--status`, `--type`,
`--priority`, and `--tag`.

Show resolves documents by global ID, optionally narrows by kind, supports full
Markdown output, `--path-only`, `--frontmatter-only`, and JSON output. Missing
documents with `--json` emit a machine-readable `DOCUMENT_NOT_FOUND` error.
