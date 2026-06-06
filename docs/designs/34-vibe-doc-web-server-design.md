---
id: 34
title: vibe-doc Web Server and UI Design
kind: design
specs:
  - 8
  - 9
  - 11
  - 13
  - 14
adrs: []
tags:
  - vibe-doc
  - server
  - web-ui
---

## Overview

The Web UI is a React SPA served by `vdoc server`.

The server reads the same Markdown repository state as the CLI through `vibe-doc-core`. The initial Web UI is read-only except for approved task lifecycle operations exposed by narrow API endpoints.

## Related Specs

- 8
- 9
- 11
- 13
- 14

## Components

`crates/vibe-doc-server` owns:

- HTTP routing.
- API response serialization.
- Repository root selection.
- SPA asset serving.
- Server-side Markdown rendering, if rendering is done before transport.

`apps/web` owns:

- React routes and screen layout.
- Document, task, ADR, spec, and design views.
- API client types.
- Filtering, navigation, and validation displays.

`vibe-doc-core` remains the source for:

- Repository scanning.
- Frontmatter parsing.
- Validation.
- Document relationships.
- Task mutation helpers.

## API Shape

The server should expose read APIs first:

- `GET /api/health`
- `GET /api/documents`
- `GET /api/documents/:id`
- `GET /api/specs`
- `GET /api/designs`
- `GET /api/adr`
- `GET /api/tasks`
- `GET /api/tasks/:id`
- `GET /api/validation`
- `GET /api/context/task/:id`

Mutation APIs should stay narrow:

- `POST /api/tasks/:id/start`
- `POST /api/tasks/:id/complete`
- `POST /api/tasks/index/rebuild`

The API should use document IDs for lookup and relationships. Paths may be returned for inspection, but they should not be the primary identity.

## Data Flow

1. The server receives a request.
2. The server resolves the repository root.
3. The server calls `vibe-doc-core` to scan, validate, or mutate.
4. The server maps core structs and errors into API response types.
5. The Web UI renders lists, detail views, validation messages, and task state from API responses.

The Web UI should not parse repository files directly.

## UI Structure

The SPA should start as an operational documentation viewer, not a marketing site.

Primary screens:

- Overview.
- Documents list.
- Document detail.
- Specs.
- Designs.
- ADRs.
- Tasks.
- Validation.

The interface should prioritize scanning, filtering, and relationship navigation over decorative presentation.

## Error Handling

API errors should include:

- Stable code.
- Human-readable message.
- Optional path.
- Optional document ID.

The Web UI should show validation and mutation errors inline near the affected workflow, and avoid hiding raw diagnostic details that are useful to agents.

## Testing Strategy

Server tests should cover API response shape and error mapping using temporary repositories.

Web UI tests should cover routing, list filtering, empty states, and detail rendering. Snapshot-like tests are useful only when paired with semantic assertions over IDs, titles, statuses, and relationships.

## Alternatives Considered

Static HTML generation is intentionally out of scope for the initial product.

Letting the browser read repository files directly would make local development awkward and would bypass core validation and safety rules.
