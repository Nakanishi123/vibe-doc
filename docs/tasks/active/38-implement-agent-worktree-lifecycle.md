---
id: 38
title: Implement agent worktree lifecycle
kind: task
type: feature
status: planned
priority: medium
specs:
- 12
designs:
- 35
depends_on:
- 37
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

- [ ] Worktree creation is implemented for managed agent runs.
- [ ] Worktree path validation is implemented.
- [ ] `AgentRun` metadata records the selected worktree.
- [ ] Cleanup behavior is implemented.
- [ ] Tests cover lifecycle success and failure cases.

## Done Criteria

- [ ] Related specs are satisfied.
- [ ] Related designs are followed.
- [ ] Related ADRs are not violated.
- [ ] Tests pass.

## Result

Not implemented.
