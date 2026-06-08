---
id: 28
title: Implement server document APIs
kind: task
type: feature
status: done
priority: medium
specs:
- 9
- 11
designs:
- 34
adrs: []
depends_on:
- 17
- 19
started_at: 2026-06-08
completed_at: 2026-06-08
---

## Goal

Implement read-only API endpoints for documents, specs, designs, ADRs, and tasks.

## Scope

- Implement `GET /api/health`.
- Implement `GET /api/documents`.
- Implement `GET /api/documents/:id`.
- Implement kind-specific list endpoints.
- Resolve document content by ID.
- Return frontmatter, Markdown body, path, and related IDs.

## Out of Scope

- Markdown-to-HTML rendering if it requires a larger renderer decision.
- Mutation APIs.
- Web UI screens.

## Checklist

- [x] Health endpoint works.
- [x] Document list endpoint works.
- [x] Document detail endpoint works.
- [x] Kind-specific endpoints work.
- [x] Path traversal is not possible.

## Done Criteria

- [x] API tests cover list and detail endpoints.
- [x] Missing IDs return stable API errors.
- [x] Related specs remain accurate.

## Result

Implemented read-only server document APIs with health, document list/detail, kind-specific endpoints, stable API errors, and API tests.
