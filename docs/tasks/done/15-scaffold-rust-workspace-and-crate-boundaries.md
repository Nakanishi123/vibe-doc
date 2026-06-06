---
id: 15
title: Scaffold Rust workspace and crate boundaries
kind: task
type: chore
status: done
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

- [x] Workspace manifest exists.
- [x] Core crate builds.
- [x] CLI crate builds.
- [x] Server crate builds.
- [x] Shared dependency direction is documented.

## Done Criteria

- [x] `cargo check` succeeds for the workspace.
- [x] No crate contains behavior that belongs in a later task.
- [x] Related specs remain accurate.

## Result

Implemented the initial Rust workspace with `vibe-doc-core`, `vibe-doc-cli`, and `vibe-doc-server`. The CLI and server crates depend on the core crate, and the repository README documents the intended dependency direction.
