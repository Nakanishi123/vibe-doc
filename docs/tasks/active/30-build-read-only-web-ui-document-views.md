---
id: 30
title: Build read-only Web UI document views
kind: task
type: feature
status: planned
priority: medium
specs:
  - 11
designs:
  - 34
adrs: []
depends_on:
  - 27
  - 28
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

- [ ] Documents list works.
- [ ] Filters work.
- [ ] Document detail works.
- [ ] Related IDs are visible.
- [ ] Validation warnings can be displayed if available.

## Done Criteria

- [ ] Typecheck or build succeeds.
- [ ] Basic responsive layout works.
- [ ] Related specs remain accurate.

## Result

Not implemented.
