---
id: 41
title: Build agent run review and acceptance UI
kind: task
type: feature
status: planned
priority: medium
specs:
- 11
- 12
designs:
- 35
depends_on:
- 40
---

## Goal

Build the Web UI flow for reviewing completed agent runs, inspecting diffs, and
accepting or rejecting run results.

## Scope

- Show completed run metadata, prompt, logs, and captured diff.
- Provide a diff viewer for changed files produced by the run.
- Provide an AI review action that stores review output in the run artifacts.
- Let the user accept or reject a run result explicitly.
- Keep task completion as a separate user-approved action after acceptance.
- Add UI states for pending review, accepted, rejected, and failed runs.
- Add frontend and server tests for review state transitions where applicable.

## Out of Scope

- Running agents or streaming live terminal output.
- Creating worktrees.
- Automatically completing tasks.
- Automatically merging rejected or unreviewed changes.

## Checklist

- [ ] Run detail UI shows prompt, logs, and diff.
- [ ] AI review output can be generated or attached.
- [ ] Accept and reject actions are explicit.
- [ ] Accepted runs do not automatically complete the task.
- [ ] Tests cover review and acceptance behavior.

## Done Criteria

- [ ] Related specs are satisfied.
- [ ] Related designs are followed.
- [ ] Related ADRs are not violated.
- [ ] Tests pass.

## Result

Not implemented.
