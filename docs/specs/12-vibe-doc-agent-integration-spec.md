---
id: 12
title: vibe-doc Agent Integration Specification
kind: spec
tags:
  - vibe-doc
  - codex
  - agents
---

## Overview

vibe-doc may integrate with Codex and other coding agents in the future.

Initial implementations must not expose arbitrary shell execution through the Web UI.

Agent integration should be task-ID based and controlled.

## Codex Runner

The Codex Runner flow:

1. Receive a task ID.
2. Run `vdoc guard task <id>`.
3. Run `vdoc context task <id>`.
4. Generate a prompt from task context.
5. Show the prompt to the user.
6. Require user approval.
7. Run Codex in an isolated Git worktree.
8. Save execution logs.
9. List changed files.
10. Show Git diff.
11. Let the user accept or reject changes.
12. Complete the task only after user approval.

## Safety Requirements

Codex execution must:

- use task IDs as the entry point
- fix the repository root
- avoid arbitrary Web UI command input
- use isolated Git worktrees for managed execution
- save execution logs
- display changed files
- display Git diff
- require user approval before marking a task done
- never complete a task without user confirmation

## Recommended Workflow

1. The user opens a task detail screen.
2. The user selects Prepare Codex Run.
3. vibe-doc validates task readiness.
4. vibe-doc collects task context.
5. vibe-doc generates a Codex prompt.
6. The user approves the prompt.
7. vibe-doc runs a Codex job in an isolated Git worktree.
8. vibe-doc streams logs.
9. vibe-doc displays a diff.
10. The user accepts or rejects changes.
11. The user completes the task if appropriate.

## Future APIs

Future Codex APIs:

```txt
POST /api/tasks/:id/prepare-codex
POST /api/tasks/:id/run-codex
GET /api/runs/:runId
GET /api/runs/:runId/logs
```
