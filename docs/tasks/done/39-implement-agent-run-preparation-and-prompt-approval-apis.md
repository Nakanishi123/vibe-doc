---
id: 39
title: Implement agent run preparation and prompt approval APIs
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

Implement APIs for preparing an agent run from a task ID and approving the
generated prompt before execution.

## Scope

- Add a prepare endpoint that runs task guard and task context collection.
- Generate a task-oriented prompt from the structured context contract.
- Persist `prompt.md` and initial `run.json` under `.vdoc/runs/<run-id>/`.
- Return the generated prompt and run metadata to the Web UI.
- Add an explicit prompt approval mutation that records an approval event.
- Add tests for guard failures, prompt generation, artifact writes, and approval
  state transitions.

## Out of Scope

- Running Codex or any other agent.
- Creating Git worktrees.
- Streaming logs or terminal output.
- Building the full review and acceptance UI.

## Checklist

- [x] Prepare API creates an `AgentRun` from a task ID.
- [x] Prompt generation uses guard and context outputs.
- [x] Prompt approval is explicit and recorded.
- [x] Failed guard results do not create executable runs.
- [x] Tests cover prepare and approval behavior.

## Done Criteria

- [x] Related specs are satisfied.
- [x] Related designs are followed.
- [x] Related ADRs are not violated.
- [x] Tests pass.

## Result

Implemented Codex run preparation and prompt approval APIs. The server can prepare a guarded task run, persist run.json and prompt.md under .vdoc/runs, return prompt and run metadata, and record explicit prompt approval events.
