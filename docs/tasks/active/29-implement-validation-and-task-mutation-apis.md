---
id: 29
title: Implement validation and task mutation APIs
kind: task
type: feature
status: planned
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

- [ ] Validation endpoint returns stable errors.
- [ ] Context endpoint returns task context.
- [ ] Task start endpoint uses core lifecycle behavior.
- [ ] Task complete endpoint uses core lifecycle behavior.
- [ ] Index rebuild endpoint is constrained to task index generation.

## Done Criteria

- [ ] API tests cover successful and invalid mutations.
- [ ] Mutation APIs reject invalid IDs and path traversal attempts.
- [ ] Related specs remain accurate.

## Result

Not implemented.
