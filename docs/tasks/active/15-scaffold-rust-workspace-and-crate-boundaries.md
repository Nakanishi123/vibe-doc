---
id: 15
title: Scaffold Rust workspace and crate boundaries
kind: task
type: chore
status: planned
priority: high
specs:
  - 8
  - 10
  - 13
designs: []
adrs: []
depends_on: []
---

# Scaffold Rust workspace and crate boundaries

## Goal

Create the initial Rust workspace structure for the CLI, server, and shared core logic.

## Scope

- Add root `Cargo.toml`.
- Add `crates/vibe-doc-core`.
- Add `crates/vibe-doc-cli`.
- Add `crates/vibe-doc-server`.
- Wire crate dependencies so CLI and server can depend on core.
- Add placeholder tests or smoke checks for workspace compilation.

## Out of Scope

- Implementing document parsing behavior.
- Implementing CLI commands.
- Implementing the Web UI.

## Checklist

- [ ] Workspace manifest exists.
- [ ] Core crate builds.
- [ ] CLI crate builds.
- [ ] Server crate builds.
- [ ] Shared dependency direction is documented.

## Done Criteria

- [ ] `cargo check` succeeds for the workspace.
- [ ] No crate contains behavior that belongs in a later task.
- [ ] Related specs remain accurate.

## Result

Not implemented.

