---
id: 13
title: VDoc Delivery Plan Specification
kind: spec
tags:
  - vdoc
  - roadmap
  - mvp
---

# VDoc Delivery Plan Specification

## Overview

This spec records the intended implementation phases, MVP scope, important product decisions, and open questions.

## Implementation Phases

Phase 1: CLI Core

- Rust workspace
- `vdoc-core`
- `vdoc-cli`
- init, new, list, show, validate, and index rebuild
- global ID management
- frontmatter and schema validation

Phase 2: Task Lifecycle

- task start
- task complete
- active and done movement
- result updates
- dependency and related document validation

Phase 3: Web UI Server

- `vdoc-server`
- `vdoc server`
- SPA serving
- document, validation, task, and context APIs

Phase 4: Web UI Viewer

- React SPA
- Markdown display
- frontmatter display
- task dashboard
- ADR dashboard
- validation result display
- browser translation support

Phase 5: Web UI Operations

- safe VDoc operations
- task start
- task complete
- index rebuild
- context preview

Phase 6: Web UI Editing

- Markdown body editing
- frontmatter editing
- pre-save validation
- diff preview
- conflict detection

Phase 7: Codex Runner

- prompt generation
- Codex job execution
- log streaming
- diff display
- task completion integration

## MVP

MVP CLI:

- `vdoc init`
- `vdoc new spec`
- `vdoc new design`
- `vdoc new adr`
- `vdoc new task`
- `vdoc list`
- `vdoc show`
- `vdoc validate`
- `vdoc rebuild index`
- `vdoc start task`
- `vdoc complete task`
- `vdoc context task`

MVP Web UI:

- `vdoc server`
- SPA serving
- documents list
- document detail
- task list
- task detail
- ADR list
- validation result

Codex Runner and Web UI editing are not included in the MVP.

## Important Decisions

- Every numbered VDoc document requires `id`, `title`, and `kind`.
- `AGENTS.md` and README files do not use frontmatter.
- IDs are positive integers without zero padding.
- IDs are globally unique across all document kinds.
- Frontmatter references use IDs, not file paths.
- Completed tasks move to `docs/tasks/done/`.
- The CLI preserves consistency with `validate` and `check`.
- The CLI and API server are implemented in Rust.
- Shared logic lives in `vdoc-core`.
- The Web UI is a React SPA served by `vdoc server`.
- Static site generation is out of scope for the initial product.

## Open Questions

- Which Rust crates should be used?
- Which API server framework should be used?
- Which Markdown renderer should be used?
- What HTML sanitization policy should be used?
- How strict should JSON Schema validation be?
- How should ID assignment be locked?
- Should `updated_at` be automatically maintained by the CLI?
- What are the exact task index grouping rules?
- Should Git worktrees be required or only recommended for agent runs?
- Should dropped tasks live in `done/` or a separate folder?
- How should Web UI edit conflict detection work?
- Should the Web UI require Git diff display before saves?

