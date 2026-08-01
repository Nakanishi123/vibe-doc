# Documentation Guide

Documents are plain Markdown files with YAML Front Matter. Keep this guide focused on the conventions of this project.

## Directory structure

```text
docs/
├── README.md
├── architecture/
├── decisions/
│   ├── architecture/
│   ├── product/
│   ├── domain/
│   └── operations/
├── research/
└── tasks/
    ├── todo/
    ├── in-progress/
    ├── done/
    └── wont-do/
```

- `architecture/` describes the current system structure and behavior.
- `decisions/` records choices and their context and consequences.
- `research/` records investigations that have not necessarily produced a decision.
- `tasks/` records work, organized by its current status.

## Front Matter

Start managed documents with Front Matter like this:

```yaml
---
vibedoc: 1
id: TASK-0010
kind: task
status: todo
tags:
  - example
related: []
depends_on: []
---
```

Supported kinds are `architecture`, `decision`, `task`, and `research`. Research documents omit `status`.

Task files belong in the directory matching their status: `todo`, `in-progress`, `done`, or `wont-do`.

## IDs and filenames

Use these ID prefixes:

- Architecture: `ARCH-0001`
- Decision: `DEC-0001`
- Research: `RES-0001`
- Task: `TASK-0010`

Use lowercase kebab-case filenames with the numeric portion first, such as `0010-add-login.md`.

Get the next candidate number with:

```bash
vibe-doc next-index decision
vibe-doc next-index research
vibe-doc next-index task
```

## Relationships

Use `related` for general relationships and `depends_on` for Task dependencies. Markdown links to managed documents are also indexed and shown as backlinks.

## Workflow

1. Choose the appropriate document kind and directory.
2. Get the next candidate number for a Decision, Research record, or Task.
3. Add only useful tags and relationships.
4. Move Task files and update their `status` together as work progresses.
5. Run `vibe-doc lint` and resolve errors before finishing.
