---
id: 35
title: vibe-doc Controlled Agent Workflow Design
kind: design
specs:
  - 10
  - 12
  - 13
  - 14
adrs: []
tags:
  - vibe-doc
  - agents
  - safety
---

## Overview

Agent integration is task-ID based and intentionally controlled.

The first implementation should prepare context and validate readiness. Running agents, streaming logs, and accepting changes are future extensions that must preserve user approval boundaries.

## Related Specs

- 10
- 12
- 13
- 14

## Components

`vdoc guard task <id>` checks whether a task is ready for agent work.

`vdoc context task <id>` collects the files and documents an agent needs:

- the task document
- referenced specs
- referenced designs
- referenced ADRs

`AGENTS.md` is intentionally excluded because Codex and similar agent runners
load repository agent instructions before command output is passed in.

Future server endpoints should reuse the same core logic rather than inventing a separate context model.

An `AgentRun` represents one attempt to run an agent for a task. A task can have
many agent runs, and an agent run belongs to exactly one task. Task documents
remain the source of truth for planned work and reviewed outcomes; agent runs
hold operational state such as prompts, process status, logs, terminal output,
and temporary diffs.

Agent run state must not be stored in task frontmatter or task prose while the
run is in progress. The task should only be updated after a user has reviewed
the run result and explicitly completes or otherwise edits the task.

## Prompt Input Contract

Generated prompts should be derived from structured, task-oriented inputs:

- repository identity and root path
- task ID, title, status, priority, and task body
- guard result and stable guard codes
- related specs, designs, and ADRs from task context
- explicit repository instructions loaded by the agent runner
- selected execution mode and working directory policy

The generated prompt should be saved before execution and shown to the user for
approval. Free-form prompt text may be appended by the user, but the run entry
point remains the task ID.

## Agent Run Artifacts

Managed agent runs should store runtime artifacts under
`.vdoc/runs/<run-id>/`. This keeps live execution state tied to the repository
while keeping it out of normal document scanning and Git diffs.

Expected artifacts:

- `run.json` for task ID, run ID, agent kind, status, worktree path, timestamps,
  exit result, and artifact paths
- `prompt.md` for the approved prompt
- `events.ndjson` for approval events, status changes, cancellation, and errors
- `terminal.log` for process or PTY output
- `diff.patch` for the captured Git diff after execution
- `review.md` for optional AI review output

Long-term summaries may be written to Markdown documents only after the user
chooses to preserve the run outcome. Live state, process IDs, PTY state, and
temporary diffs should stay in the run artifact directory.

## Guard Rules

Guard checks should verify:

- The task exists.
- The task is active.
- The task status is `planned` or `doing`.
- Dependencies are complete.
- Referenced specs, designs, and ADRs exist.
- Referenced ADRs are not rejected or superseded without a replacement.

Guard output should be readable by humans and agents. JSON output should preserve stable codes.

## Context Output

Context output should be deterministic.

Recommended order:

1. Agent instructions.
2. Task document.
3. Related specs sorted by ID.
4. Related designs sorted by ID.
5. Related ADRs sorted by ID.

Each context item should include its document ID when available, kind, title,
path, and content or content reference depending on output mode. Context content
should use the parsed Markdown body and omit YAML frontmatter, because metadata
is already represented as structured fields.

## Safety Boundaries

The Web UI must not expose arbitrary shell command execution.

Future agent runs should:

- Start from a task ID.
- Run guard before preparing a prompt.
- Show the generated prompt before execution.
- Require explicit user approval.
- Use an isolated Git worktree for managed execution.
- Capture logs.
- Display changed files and diffs.
- Require user approval before completing a task.

The Web UI may embed a terminal with xterm.js, but xterm.js is only a display
and input surface for a specific `AgentRun`. The server-side run record is the
source of truth for status, artifacts, and review decisions.

## Approval Points

Agent workflows should require explicit approval at these points:

1. Prepare a run from a task ID after guard succeeds.
2. Approve the generated prompt before execution.
3. Start the agent in the isolated worktree.
4. Review the resulting diff and logs.
5. Accept or reject the run result.
6. Complete the task only after reviewed changes are accepted.

## Data Flow

1. User selects a task.
2. System runs guard.
3. System builds context.
4. System generates or displays a prompt.
5. User approves execution.
6. System creates an `AgentRun` under `.vdoc/runs/<run-id>/`.
7. System creates or selects an isolated Git worktree.
8. Agent runs in the worktree.
9. System records logs, events, and changed files.
10. User reviews logs, AI review output, and diff.
11. User accepts or rejects the run result.
12. User decides whether to complete the task.

Only the first three steps are required for the initial CLI support.

## Testing Strategy

Tests should cover:

- Missing task IDs.
- Blocked or done tasks.
- Incomplete dependencies.
- Missing related documents.
- Deterministic context ordering.
- JSON output shape for guard failures and context entries.

## Alternatives Considered

Starting with free-form prompts would be flexible but would lose the main value of vibe-doc: keeping agent work tied to documented tasks and explicit repository context.

Completing tasks automatically after an agent run is not acceptable because task completion records a human-reviewed outcome.
