---
id: 37
title: Implement agent run storage under .vdoc runs
kind: task
type: feature
status: done
priority: medium
specs:
- 12
designs:
- 35
depends_on:
- 32
started_at: 2026-06-10
completed_at: 2026-06-10
---

## Goal

Implement the storage foundation for task-scoped agent runs, using
`.vdoc/runs/` as the repository-local runtime area for run metadata, approved
prompts, logs, and captured diffs.

## Scope

- Define an `AgentRun` data model that records task ID, run ID, agent kind,
  status, worktree path, timestamps, exit result, and artifact paths.
- Store agent run artifacts under `.vdoc/runs/<run-id>/`.
- Add core helpers for locating the repository root and creating run artifact
  directories safely.
- Define the expected artifact files, including `run.json`, `prompt.md`,
  `events.ndjson`, `terminal.log`, and `diff.patch`.
- Ensure run artifacts are outside normal document scanning and do not appear
  in repository diffs.
- Add tests for run ID allocation, artifact path creation, and non-worktree
  repository handling where applicable.

## Out of Scope

- Running Codex or another agent process.
- Creating or managing Git worktrees for agent execution.
- Streaming PTY output with xterm.js.
- Rendering run logs or diffs in the Web UI.
- Recording accepted run summaries as numbered documents.

## Checklist

- [x] `AgentRun` status and metadata shape are implemented.
- [x] `.vdoc/runs/<run-id>/` artifact directory creation is implemented.
- [x] Expected artifact file names are documented in code or tests.
- [x] Runtime artifacts are excluded from Markdown document scanning.
- [x] Tests cover artifact creation and path safety.

## Done Criteria

- [x] Related specs are satisfied.
- [x] Related designs are followed.
- [x] Related ADRs are not violated.
- [x] Tests pass.

## Result

Implemented core AgentRun storage under .vdoc/runs with path-safe run ID allocation, expected artifact path helpers, run.json metadata persistence, repository-root discovery, .vdoc/runs Git ignore coverage, and tests for allocation, artifact creation, safety, scanning exclusion, and non-worktree root discovery.
