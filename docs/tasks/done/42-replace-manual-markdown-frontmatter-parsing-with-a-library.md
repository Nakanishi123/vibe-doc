---
id: 42
title: Replace manual Web UI Markdown rendering with a library
kind: task
type: refactor
status: done
priority: medium
specs:
- 9
- 10
designs:
- 33
started_at: 2026-06-08
completed_at: 2026-06-08
---

## Goal

Replace the hand-written Web UI Markdown renderer with a maintained JavaScript
Markdown rendering library so document bodies render more accurately and become
easier to style.

## Scope

- Adopt `react-markdown` with `remark-gfm` for rendering Markdown body content
  in the Web UI.
- Replace `apps/web/src/lib/markdown.ts` and the block-based rendering in
  `MarkdownView` with library-backed rendering.
- Preserve the server API shape where document detail responses return parsed
  frontmatter plus raw Markdown body text.
- Keep raw HTML disabled or sanitized so user-authored Markdown cannot inject
  unsafe markup into the Web UI.
- Add component mappings for headings, paragraphs, lists, links, tables, inline
  code, and fenced code blocks.
- Ensure code blocks and frontmatter/raw views keep `translate="no"` where
  appropriate.
- Add or update frontend tests for headings, lists, fenced code, links, tables,
  and task checklists if a test harness exists.

## Out of Scope

- Changing the Rust frontmatter parser used by `vibe-doc-core`.
- Changing the server API to return rendered HTML.
- Changing the vibe-doc Markdown document model.
- Adding Markdown editing.
- Redesigning the entire document detail screen beyond renderer-specific
  styling.

## Checklist

- [x] `react-markdown` and `remark-gfm` are added to the Web UI dependencies.
- [x] The manual `markdownToBlocks` renderer is removed or no longer used.
- [x] Markdown body content renders through the library-backed `MarkdownView`.
- [x] Raw HTML handling is explicitly safe.
- [x] Common Markdown structures render with polished document styling.
- [x] Relevant frontend tests and build checks pass.

## Done Criteria

- [x] Related specs are satisfied.
- [x] Related designs are followed.
- [x] Related ADRs are not violated.
- [x] Tests pass.

## Result

Replaced the Web UI's hand-written Markdown renderer with react-markdown and remark-gfm, added safe library-backed component mappings for common Markdown structures, and removed the old parser.
