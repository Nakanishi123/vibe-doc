---
id: 27
title: Scaffold Web UI app and shared API types
kind: task
type: chore
status: done
priority: medium
specs:
  - 8
  - 11
  - 13
designs:
  - 34
adrs: []
depends_on:
  - 15
completed_at: 2026-06-08
---

## Goal

Create the initial React/Vite/Tailwind/TypeScript app structure and shared API type definitions.

## Scope

- Add `apps/web`.
- Configure Vite, React, TypeScript, and Tailwind.
- Add basic app shell.
- Define initial TypeScript API response types.
- Keep pnpm workspace configuration aligned with the app path.

## Out of Scope

- Building full document views.
- Implementing server APIs.
- Adding Web UI mutations.

## Checklist

- [x] Web app scaffold exists.
- [x] TypeScript config exists.
- [x] Tailwind config exists.
- [x] App starts in development mode.
- [x] API types reflect the current Web UI spec.

## Done Criteria

- [x] Web app build or typecheck succeeds.
- [x] No placeholder marketing page is introduced.
- [x] Related specs remain accurate.

## Result

Added `apps/web` as a React, Vite, Tailwind CSS, and TypeScript workspace app.

The initial UI is a read-only operational documentation shell with overview
metrics, primary screen navigation, recent document rows, API contract entries,
and stable taxonomy displays. It avoids a marketing landing page and leaves
full document, task, ADR, and validation views for the follow-up Web UI tasks.

Added initial TypeScript API response types for health, documents, specs,
designs, ADRs, tasks, validation, task context, and API errors.

Updated pnpm workspace metadata, root Web scripts, lockfile state, and ignored
generated frontend artifacts.
