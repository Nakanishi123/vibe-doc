---
id: 39
title: Implement agent run preparation and prompt approval APIs
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

- [ ] Prepare API creates an `AgentRun` from a task ID.
- [ ] Prompt generation uses guard and context outputs.
- [ ] Prompt approval is explicit and recorded.
- [ ] Failed guard results do not create executable runs.
- [ ] Tests cover prepare and approval behavior.

## Done Criteria

- [ ] Related specs are satisfied.
- [ ] Related designs are followed.
- [ ] Related ADRs are not violated.
- [ ] Tests pass.

## Result

Not implemented.
