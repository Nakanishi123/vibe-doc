---
id: 29
title: Implement validation and task mutation APIs
kind: task
type: feature
status: done
priority: medium
specs:
- 11
- 14
designs:
- 34
adrs: []
depends_on:
- 23
- 25
- 28
started_at: 2026-06-08
completed_at: 2026-06-08
---

## Goal

Expose validation results and approved task lifecycle mutations through the server API.

## Scope

- Implement `GET /api/validation`.
- Implement `GET /api/context/task/:id`.
- Implement `POST /api/tasks/:id/start`.
- Implement `POST /api/tasks/:id/complete`.
- Implement `POST /api/tasks/index/rebuild`.
- Reuse CLI/core lifecycle logic.

## Out of Scope

- Arbitrary file writes.
- Arbitrary command execution.
- Codex run APIs.
- Web UI screens.

## Checklist

- [x] Validation endpoint returns stable errors.
- [x] Context endpoint returns task context.
- [x] Task start endpoint uses core lifecycle behavior.
- [x] Task complete endpoint uses core lifecycle behavior.
- [x] Index rebuild endpoint is constrained to task index generation.

## Done Criteria

- [x] API tests cover successful and invalid mutations.
- [x] Mutation APIs reject invalid IDs and path traversal attempts.
- [x] Related specs remain accurate.

## Result

Implemented server validation, task context, task lifecycle mutation, and task index rebuild APIs. Added focused server API tests for validation/context, successful lifecycle mutations, invalid IDs, invalid statuses, traversal-shaped paths, and constrained index rebuild behavior. Added shared Web API types for task lifecycle and index rebuild responses.
