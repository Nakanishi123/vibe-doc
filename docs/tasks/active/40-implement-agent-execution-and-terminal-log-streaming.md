---
id: 40
title: Implement agent execution and terminal log streaming
kind: task
type: feature
status: planned
priority: medium
specs:
- 12
designs:
- 35
depends_on:
- 38
- 39
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

- [ ] Approved runs can be started.
- [ ] Unsupported or unapproved runs are rejected.
- [ ] Terminal output is streamed and persisted.
- [ ] Run status and exit results are recorded.
- [ ] `diff.patch` is captured after execution.
- [ ] Tests cover execution and logging behavior.

## Done Criteria

- [ ] Related specs are satisfied.
- [ ] Related designs are followed.
- [ ] Related ADRs are not violated.
- [ ] Tests pass.

## Result

Not implemented.
