---
id: 9
title: vibe-doc Document Model Specification
kind: spec
tags:
  - vibe-doc
  - documents
  - frontmatter
---

# vibe-doc Document Model Specification

## Overview

This spec defines the repository layout, document kinds, frontmatter rules, ID rules, and references for vibe-doc-managed projects.

## Managed Project Structure

A project using vibe-doc has this structure:

```txt
.
├── AGENTS.md
└── docs/
    ├── README.md
    ├── schemas/
    │   ├── document.schema.json
    │   ├── spec.schema.json
    │   ├── design.schema.json
    │   ├── adr.schema.json
    │   └── task.schema.json
    ├── specs/
    │   ├── README.md
    │   └── *.md
    ├── designs/
    │   ├── README.md
    │   └── *.md
    ├── adr/
    │   ├── README.md
    │   └── *.md
    └── tasks/
        ├── README.md
        ├── index.md
        ├── active/
        │   └── *.md
        └── done/
            └── *.md
```

## Repository Documentation Files

Repository documentation files are:

- `AGENTS.md`
- `docs/README.md`
- `docs/specs/README.md`
- `docs/specs/*.md`
- `docs/designs/README.md`
- `docs/designs/*.md`
- `docs/adr/README.md`
- `docs/adr/*.md`
- `docs/tasks/README.md`
- `docs/tasks/index.md`
- `docs/tasks/active/*.md`
- `docs/tasks/done/*.md`

Specs, designs, ADRs, tasks, and the task index are numbered vibe-doc documents. `AGENTS.md` and README files are unnumbered and do not use frontmatter.

## Schemas

vibe-doc uses JSON Schema to describe frontmatter validation rules. Schemas live in `docs/schemas/`.

Schemas validate:

- required frontmatter
- document kind
- ADR status
- task status
- task type
- priority
- ID reference shape

Markdown body templates are built into the CLI. If custom templates are added later, they should use a non-managed extension such as `.vibe-doc/templates/*.md.tmpl`.

## Global ID Rules

Every numbered vibe-doc document has one global positive integer ID.

IDs are shared across specs, designs, ADRs, tasks, and the task index. README files and `AGENTS.md` do not participate in ID assignment.

The frontmatter `id` is the source of truth. File names may include IDs, but ID resolution must use frontmatter.

IDs are not zero-padded.

Example:

```yaml
---
id: 11
title: Auth User Model
kind: task
---
```

The CLI and Web UI sort IDs numerically.

## Kinds

Allowed `kind` values:

- `spec`
- `design`
- `adr`
- `task`
- `task-index`

Expected kind by location:

```txt
docs/specs/*.md                   -> spec
docs/designs/*.md                 -> design
docs/adr/*.md                     -> adr
docs/tasks/index.md               -> task-index
docs/tasks/active/*.md            -> task
docs/tasks/done/*.md              -> task
```

## Common Frontmatter

Required:

- `id`
- `title`
- `kind`

Optional common fields:

- `created_at`
- `updated_at`
- `tags`

`status` is not required for all documents. ADRs and tasks require status.

## Agent Instructions

`AGENTS.md` records rules for AI coding agents.

It should include:

- coding conventions
- test policy
- forbidden actions
- documentation update rules
- spec, design, ADR, and task usage rules
- vibe-doc CLI usage rules
- project-specific LLM instructions

`AGENTS.md` does not use frontmatter.

## README Documents

README files explain directory purpose and operating rules.

They should describe:

- what belongs in the directory
- what does not belong in the directory
- naming rules
- frontmatter rules for numbered vibe-doc documents in the directory
- schema references
- CLI usage

README files do not use frontmatter.

## Specs

A spec defines what to build.

Specs should include externally observable behavior, requirements, constraints, APIs, and acceptance criteria. They should avoid unnecessary implementation detail.

Recommended sections:

- Goal
- Background
- User Stories
- Functional Requirements
- Non-functional Requirements
- API Contract
- Error Cases
- Acceptance Criteria

Spec status is optional. Existing specs are considered active. Deprecated specs may use:

```yaml
status: deprecated
superseded_by: 20
```

MVP validation does not need strict spec status handling.

## Designs

A design defines how to build something.

Designs should include implementation approach, components, responsibility boundaries, data flow, data model, error handling, testing strategy, and alternatives considered.

Recommended sections:

- Overview
- Related Specs
- Related ADRs
- Components
- Data Flow
- Data Model
- Error Handling
- Testing Strategy
- Alternatives Considered

Design status is optional. Deprecated designs may use:

```yaml
status: deprecated
superseded_by: 21
```

MVP validation does not need strict design status handling.

## ADRs

An ADR records why a design decision was made.

ADRs are mostly immutable. Accepted ADRs should generally be superseded by a new ADR instead of being heavily rewritten.

Required ADR frontmatter includes `status`.

Allowed ADR statuses:

- `proposed`
- `accepted`
- `rejected`
- `deprecated`
- `superseded`

Recommended ADR sections:

- Context
- Decision
- Consequences
- Alternatives Considered
- Related Documents

Task references from ADRs are not required because tasks are short-lived.

## Reference Rules

Document references use IDs.

Good:

```yaml
specs:
  - 8
designs:
  - 9
adrs:
  - 10
depends_on:
  - 11
```

Avoid path references in frontmatter.

ID references survive file moves, task completion, and file renames.

The CLI and Web UI must resolve paths from IDs.
