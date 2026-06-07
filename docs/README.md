# Docs

This directory contains vibe-doc-managed project documentation.

The source of truth is Markdown. Schemas describe frontmatter expectations for numbered vibe-doc documents, and implemented `vdoc` commands can validate and rebuild parts of the managed documentation.

## Structure

- `schemas/` contains JSON Schema files for vibe-doc frontmatter.
- `specs/` contains product requirements and externally observable behavior.
- `designs/` contains implementation designs for specs.
- `adr/` contains architectural decision records.
- `tasks/` contains active and completed implementation work.

## Frontmatter

Specs, designs, ADRs, tasks, and the task index must include:

```yaml
---
id: 1
title: Example Title
kind: spec
---
```

`AGENTS.md` and README files do not use frontmatter.

IDs are global across all numbered vibe-doc documents. Do not create separate ID ranges per document kind.

## Manual CLI Equivalent

For behavior not implemented by the CLI yet:

- Create files by following the documented directory layout.
- Assign the next global ID manually.
- Keep references ID-based.
- Update task status and location manually.
- Use `vdoc rebuild index` after task frontmatter changes.
