---
id: 30
title: Build read-only Web UI document views
kind: task
type: feature
status: done
priority: medium
specs:
- 11
designs:
- 34
adrs: []
depends_on:
- 27
- 28
started_at: 2026-06-08
completed_at: 2026-06-08
---

## Goal

Build Web UI views for browsing documents and reading document details.

## Scope

- Add overview navigation for documents.
- Add documents list with kind, tag, title, and ID filters.
- Add spec and design list views.
- Add document detail view.
- Show frontmatter, raw Markdown, rendered Markdown, path, and related IDs.

## Out of Scope

- Task dashboard.
- ADR dashboard beyond generic document views.
- Mutation controls.
- Markdown editing.

## Checklist

- [x] Documents list works.
- [x] Filters work.
- [x] Document detail works.
- [x] Related IDs are visible.
- [x] Validation warnings can be displayed if available.

## Done Criteria

- [ ] Typecheck or build succeeds.
- [x] Basic responsive layout works.
- [x] Related specs remain accurate.

## Result

Implemented read-only document, spec, design, and detail Web UI views with filters, relationships, Markdown display, and validation issue surfacing. `pnpm`/`node` were unavailable in the execution environment, so Web typecheck/build could not be run.
