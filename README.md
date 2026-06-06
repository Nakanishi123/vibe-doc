# vibe-doc

vibe-doc is a Markdown-first document and task management tool for AI-assisted software development.

The product and repository name is `vibe-doc`. The CLI command name is `vdoc`.

This repository contains the vibe-doc product work itself. Until the real `vdoc` CLI exists, the repository is operated manually using the same document layout that `vdoc init` will eventually generate.

## Tooling

Only `pnpm` is managed by mise in this repository.

```sh
mise install
pnpm --version
```

Rust, Node.js, and other tools are intentionally not pinned yet.

## Documentation Source of Truth

The source of truth is the Markdown under `docs/` plus `AGENTS.md`.

Specs, designs, ADRs, tasks, and the task index must include YAML frontmatter with:

- `id`
- `title`
- `kind`

`AGENTS.md` and README files do not use frontmatter.

Document IDs are global positive integers shared across all numbered vibe-doc documents.
