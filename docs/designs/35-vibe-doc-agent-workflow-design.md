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
- Prefer isolated Git worktrees.
- Capture logs.
- Display changed files and diffs.
- Require user approval before completing a task.

## Data Flow

1. User selects a task.
2. System runs guard.
3. System builds context.
4. System generates or displays a prompt.
5. User approves execution.
6. Agent runs in a controlled working directory.
7. System records logs and changed files.
8. User reviews diff.
9. User decides whether to complete the task.

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
