---
id: 11
title: vibe-doc Web UI Specification
kind: spec
tags:
  - vibe-doc
  - web-ui
  - api
---

## Overview

The Web UI is a React, Vite, Tailwind CSS, and TypeScript SPA.

`vdoc server` serves the SPA and provides APIs that read repository Markdown.

`vdoc server` does not generate static HTML.

## Server Responsibilities

`vdoc server` provides:

- SPA serving
- vibe-doc APIs
- Markdown reads from the repository
- frontmatter parsing
- Markdown-to-HTML rendering
- validation results
- task, ADR, spec, and design lists

## Recommended Web Structure

```txt
apps/web/
├── index.html
├── package.json
├── vite.config.ts
├── tailwind.config.ts
├── tsconfig.json
└── src/
    ├── main.tsx
    ├── App.tsx
    ├── routes/
    ├── components/
    └── lib/
```

## Screens

The Web UI should include:

- overview
- documents list
- document detail
- specs
- designs
- ADRs
- tasks
- validation

## Overview Screen

The overview shows:

- document count
- active task count
- done task count
- ADR count
- validation status
- recently updated documents

## Document Screens

The documents list supports filters by:

- kind
- tag
- title
- ID

Document detail shows:

- frontmatter
- rendered Markdown
- raw Markdown
- related documents
- validation warnings
- file path

## Spec and Design Screens

Spec lists show:

- id
- title
- tags
- related designs
- related tasks

Design lists show:

- id
- title
- related specs
- related ADRs
- related tasks

## ADR Screen

ADR lists show:

- id
- title
- status
- date
- tags
- supersedes
- superseded_by

## Task Screen

Task lists show:

- active tasks
- done tasks
- blocked tasks
- task status
- task type
- priority
- dependencies
- related specs
- related designs
- related ADRs

Task filters include:

- status
- type
- priority
- tag

## Validation Screen

Validation results show:

- errors
- warnings
- affected files
- error code
- message
- suggested fix if available

## API

Recommended initial APIs:

```txt
GET /api/health
GET /api/documents
GET /api/documents/:id
GET /api/specs
GET /api/designs
GET /api/adr
GET /api/tasks
GET /api/tasks/:id
GET /api/validation
GET /api/context/task/:id

POST /api/tasks/:id/start
POST /api/tasks/:id/complete
POST /api/tasks/index/rebuild
```

Future Codex APIs:

```txt
POST /api/tasks/:id/prepare-codex
POST /api/tasks/:id/run-codex
GET /api/runs/:runId
GET /api/runs/:runId/logs
```

## Browser Translation

Markdown body content should render as normal HTML so browser translation works well.

Metadata values and code blocks should not be translated when possible:

```html
<span translate="no">status: accepted</span>
<pre translate="no"><code>status: planned</code></pre>
```

## Security

The Web UI must be safe by default.

Requirements:

- bind to `127.0.0.1` by default
- do not expose arbitrary shell execution
- do not expose arbitrary file writes
- expose only approved vibe-doc operations
- validate task IDs and document IDs
- prevent path traversal
- prefer ID-based operations over path-based operations
- require explicit approval for mutations
- show dry-run results before destructive operations where possible

Avoid APIs like:

```txt
/api/run?cmd=...
```

Prefer APIs like:

```txt
POST /api/tasks/:id/start
POST /api/tasks/:id/complete
```

## Editing

The initial Web UI is read-only by default.

Allowed mutations should be limited to explicit vibe-doc operations such as task start, task complete, and index rebuild.

Future Markdown editing must:

- validate before saving
- prevent duplicate IDs
- prevent kind mismatches
- detect broken references
- show a diff preview
- detect concurrent or external edits
- prevent invalid path writes
- target documents by ID
