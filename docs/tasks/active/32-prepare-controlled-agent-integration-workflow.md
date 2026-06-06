---
id: 32
title: Prepare controlled agent integration workflow
kind: task
type: spike
status: planned
priority: low
specs:
  - 12
designs: []
adrs: []
depends_on:
  - 26
---

## Goal

Define and prototype the safe boundary for future Codex or agent integration.

## Scope

- Review task context and guard outputs for agent readiness.
- Define prompt generation inputs.
- Define log and diff capture expectations.
- Define approval points before task completion.
- Identify whether Git worktrees should be required or recommended.

## Out of Scope

- Running Codex from the Web UI.
- Implementing long-running job execution.
- Completing tasks automatically.

## Checklist

- [ ] Prompt input contract is drafted.
- [ ] Approval flow is documented.
- [ ] Log and diff artifacts are described.
- [ ] Worktree recommendation is evaluated.
- [ ] Follow-up implementation tasks are created if needed.

## Done Criteria

- [ ] Findings are recorded in a design or ADR if they become architectural decisions.
- [ ] No arbitrary shell execution path is introduced.
- [ ] Related specs remain accurate.

## Result

Not implemented.
