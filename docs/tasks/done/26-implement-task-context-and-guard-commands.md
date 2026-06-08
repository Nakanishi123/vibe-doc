---
id: 26
title: Implement task context and guard commands
kind: task
type: feature
status: done
priority: medium
specs:
  - 10
  - 12
  - 14
designs:
  - 33
  - 35
adrs: []
depends_on:
  - 17
  - 23
completed_at: 2026-06-07
---

## Goal

Implement task-oriented read checks that support human and agent workflows.

## Scope

- Implement `vdoc context task <id>`.
- Include the task, related specs, related designs, and related ADRs.
- Implement `vdoc guard task <id>`.
- Verify task status, dependencies, and related document existence.
- Return JSON-friendly output.

## Out of Scope

- Running Codex or another agent.
- Generating prompts.
- Web UI context preview.

## Checklist

- [x] Context resolves related document IDs.
- [x] Guard detects missing dependencies.
- [x] Guard rejects completed or dropped active operations.
- [x] Output is stable for automation.

## Done Criteria

- [x] Tests cover ready, blocked, and invalid task cases.
- [x] Context file order is deterministic.
- [x] Related specs remain accurate.

## Result

Implemented `vdoc context task <id>` and `vdoc guard task <id>`.

The core crate now exposes reusable task context and guard APIs. The CLI returns
deterministic text and JSON output, reports guard failures with stable codes,
and exits non-zero when a task is not ready.

Added CLI integration tests for ready guard output, blocked and invalid guard
cases, and deterministic context ordering.

Follow-up review fix: `vdoc context task <id>` now exits non-zero when a
referenced spec, design, or ADR cannot be resolved, instead of returning
incomplete context successfully.

Follow-up usability fix: `vdoc context task <id>` no longer includes
`AGENTS.md`, because Codex and similar agent runners load repository agent
instructions before task context is requested.

Follow-up output cleanup: context item content now omits YAML frontmatter and
returns only the parsed Markdown body. Metadata remains available through item
fields such as `id`, `kind`, `title`, and `path`.
