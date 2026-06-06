---
id: 28
title: Implement server document APIs
kind: task
type: feature
status: planned
priority: medium
specs:
  - 9
  - 11
designs: []
adrs: []
depends_on:
  - 17
  - 19
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

- [ ] Health endpoint works.
- [ ] Document list endpoint works.
- [ ] Document detail endpoint works.
- [ ] Kind-specific endpoints work.
- [ ] Path traversal is not possible.

## Done Criteria

- [ ] API tests cover list and detail endpoints.
- [ ] Missing IDs return stable API errors.
- [ ] Related specs remain accurate.

## Result

Not implemented.
