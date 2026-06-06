---
id: 26
title: Implement task context and guard commands
kind: task
type: feature
status: planned
priority: medium
specs:
  - 10
  - 12
  - 14
designs: []
adrs: []
depends_on:
  - 17
  - 23
---

## Goal

Implement task-oriented read checks that support human and agent workflows.

## Scope

- Implement `vdoc context task <id>`.
- Include `AGENTS.md`, the task, related specs, related designs, and related ADRs.
- Implement `vdoc guard task <id>`.
- Verify task status, dependencies, and related document existence.
- Return JSON-friendly output.

## Out of Scope

- Running Codex or another agent.
- Generating prompts.
- Web UI context preview.

## Checklist

- [ ] Context resolves related document IDs.
- [ ] Guard detects missing dependencies.
- [ ] Guard rejects completed or dropped active operations.
- [ ] Output is stable for automation.

## Done Criteria

- [ ] Tests cover ready, blocked, and invalid task cases.
- [ ] Context file order is deterministic.
- [ ] Related specs remain accurate.

## Result

Not implemented.
