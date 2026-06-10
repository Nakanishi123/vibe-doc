---
id: 38
title: Implement agent worktree lifecycle
kind: task
type: feature
status: done
priority: medium
specs:
- 12
designs:
- 35
depends_on:
- 37
started_at: 2026-06-10
completed_at: 2026-06-10
---

## Goal

Implement the Git worktree lifecycle used by managed agent runs.

## Scope

- Create task-scoped worktrees for agent execution.
- Use deterministic, collision-safe worktree names derived from task ID and run
  ID.
- Record the worktree path in the corresponding `AgentRun` metadata.
- Validate that worktree paths stay within the approved repository-local
  execution area.
- Provide cleanup behavior for failed, rejected, or completed runs.
- Add tests for worktree creation, existing path conflicts, and cleanup.

## Out of Scope

- Running Codex or any other agent.
- Streaming terminal output.
- Rendering worktree status in the Web UI.
- Merging or accepting worktree changes.

## Checklist

- [x] Worktree creation is implemented for managed agent runs.
- [x] Worktree path validation is implemented.
- [x] `AgentRun` metadata records the selected worktree.
- [x] Cleanup behavior is implemented.
- [x] Tests cover lifecycle success and failure cases.

## Done Criteria

- [x] Related specs are satisfied.
- [x] Related designs are followed.
- [x] Related ADRs are not violated.
- [x] Tests pass.

## Result

Implemented core managed agent worktree lifecycle helpers under .vdoc/worktrees with deterministic task/run-derived names, repository-local path validation, git worktree creation, AgentRun metadata updates, cleanup that removes worktrees and clears metadata, and tests for creation, path conflicts, validation, and cleanup.
