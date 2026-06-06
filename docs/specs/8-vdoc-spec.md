---
id: 8
title: VDoc Product Specification
kind: spec
tags:
  - vdoc
  - product
---

# VDoc Product Specification

## Overview

VDoc is a Markdown-first document and task management tool for AI-assisted development, especially vibe coding workflows.

VDoc manages specifications, designs, ADRs, tasks, README files, and agent instructions as repository files. Specs, designs, ADRs, tasks, and the task index are numbered VDoc documents with YAML frontmatter. `AGENTS.md` and README files do not use frontmatter.

The source of truth is always the repository file tree. The CLI and Web UI must not introduce a separate canonical database.

## Product Components

VDoc provides:

- a Rust CLI named `vdoc`
- a Rust API server
- a React, Vite, Tailwind CSS, and TypeScript SPA

`vdoc server` serves the React SPA and exposes APIs that read Markdown from the repository.

## Goals

VDoc should:

- provide an AI-readable development documentation structure
- keep documentation human-manageable with Markdown
- assign a unique global integer ID to every numbered VDoc document
- preserve frontmatter consistency through CLI validation
- separate specs, designs, ADRs, and tasks by role
- manage the active and done lifecycle of tasks
- show documents clearly in a Web UI
- render browser-translation-friendly HTML
- leave room for future safe Codex integration

## Non-Goals

The initial product is not:

- a complete replacement for GitHub Issues, Linear, or Jira
- a real-time collaborative editor
- a UI for arbitrary shell command execution
- a cloud-first service
- a database-backed canonical document system
- a WYSIWYG editor
- a complex permission system
- a static site generator
- a fully automated Codex runner

## Core Principles

- Markdown is the body format.
- YAML frontmatter is used only for numbered VDoc document metadata.
- README files and `AGENTS.md` are unnumbered operational documents.
- Frontmatter references use document IDs, not file paths.
- IDs are positive integers without zero padding.
- Metadata keys and enum values are stable English identifiers.
- Long-term operational documentation should be English-first.

## Related Specs

- Document model and repository layout: `9`
- CLI and validation behavior: `10`
- Web UI and API behavior: `11`
- Codex and agent integration: `12`
- Delivery plan and MVP scope: `13`
- Task model and lifecycle: `14`

## Product Repository Shape

VDoc itself is intended to become a monorepo using a Rust workspace and a pnpm workspace.

Recommended final structure:

```txt
vdoc/
├── Cargo.toml
├── package.json
├── pnpm-workspace.yaml
├── README.md
├── crates/
│   ├── vdoc-core/
│   ├── vdoc-cli/
│   └── vdoc-server/
└── apps/
    └── web/
```

Responsibilities:

- `crates/vdoc-core`: frontmatter parsing, Markdown scanning, ID management, validation, document resolution, and task lifecycle logic
- `crates/vdoc-cli`: Rust CLI
- `crates/vdoc-server`: Rust API server and SPA host
- `apps/web`: React SPA

The CLI and server share core behavior through `vdoc-core`.
