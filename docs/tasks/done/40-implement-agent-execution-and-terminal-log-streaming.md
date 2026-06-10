---
id: 40
title: Implement agent execution and terminal log streaming
kind: task
type: feature
status: done
priority: medium
specs:
- 12
designs:
- 35
depends_on:
- 38
- 39
started_at: 2026-06-10
completed_at: 2026-06-10
---

## Goal

Run approved agent runs in their isolated worktrees and stream terminal output
to the Web UI.

## Scope

- Start an approved agent run from its `AgentRun` metadata.
- Execute only supported agent commands selected by server-side configuration.
- Stream process or PTY output to clients for xterm.js display.
- Append terminal output to `terminal.log`.
- Append status changes and errors to `events.ndjson`.
- Capture exit status and update `run.json`.
- Capture `diff.patch` after the process exits.
- Add tests for status transitions, unsupported commands, log capture, and diff
  capture.

## Out of Scope

- Arbitrary shell command execution from the Web UI.
- Prompt preparation and approval.
- Worktree creation.
- AI review of completed diffs.
- Accepting or merging changes.

## Checklist

- [x] Approved runs can be started.
- [x] Unsupported or unapproved runs are rejected.
- [x] Terminal output is streamed and persisted.
- [x] Run status and exit results are recorded.
- [x] `diff.patch` is captured after execution.
- [x] Tests cover execution and logging behavior.

## Done Criteria

- [x] Related specs are satisfied.
- [x] Related designs are followed.
- [x] Related ADRs are not violated.
- [x] Tests pass.

## Result

Implemented approved agent run execution with configured server-side commands, worktree-backed process streaming, terminal.log persistence, status/error events, exit results, diff.patch capture, and core/server tests for execution, rejection, logging, and diff behavior.
