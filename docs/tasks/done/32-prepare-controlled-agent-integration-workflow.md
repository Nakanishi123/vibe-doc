---
id: 32
title: Prepare controlled agent integration workflow
kind: task
type: spike
status: done
priority: low
specs:
- 12
designs:
- 35
adrs: []
depends_on:
- 26
started_at: 2026-06-08
completed_at: 2026-06-08
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

- [x] Prompt input contract is drafted.
- [x] Approval flow is documented.
- [x] Log and diff artifacts are described.
- [x] Worktree recommendation is evaluated.
- [x] Follow-up implementation tasks are created if needed.

## Done Criteria

- [x] Findings are recorded in a design or ADR if they become architectural decisions.
- [x] No arbitrary shell execution path is introduced.
- [x] Related specs remain accurate.

## Result

Recorded the controlled agent workflow decisions in design 35 and spec 12. Defined AgentRun as a task-scoped execution record, selected .git/vdoc/runs/<run-id>/ for runtime artifacts, made managed runs use isolated Git worktrees, documented approval points, and split follow-up implementation tasks 37-41.
