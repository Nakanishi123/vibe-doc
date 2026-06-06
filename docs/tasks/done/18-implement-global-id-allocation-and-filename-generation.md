---
id: 18
title: Implement global ID allocation and filename generation
kind: task
type: feature
status: done
priority: high
specs:
  - 9
  - 10
designs: []
adrs: []
depends_on:
  - 17
---

## Goal

Provide core utilities for assigning the next global ID and generating document filenames.

## Scope

- Scan existing numbered VDoc documents for IDs.
- Detect duplicate and invalid IDs.
- Return the next positive integer ID.
- Generate slugged filenames with unpadded IDs.
- Keep ID allocation independent from any specific CLI command.

## Out of Scope

- Cross-process locking.
- Writing new documents.
- Full validation command output.

## Checklist

- [x] Next-ID utility exists.
- [x] Duplicate ID detection exists.
- [x] Slug generation handles common title input.
- [x] Filename generation follows the documented directory conventions.

## Done Criteria

- [x] Tests cover empty repositories, normal repositories, gaps, and duplicates.
- [x] IDs are sorted numerically, not lexicographically.
- [x] Related specs remain accurate.

## Result

Implemented in `vibe-doc-core` as CLI-independent allocation and filename helpers.

- Added repository scanning based next-ID allocation.
- Added duplicate global ID detection.
- Added numeric ID sorting helpers.
- Added title slug generation and unpadded filename generation.
- Added repository-relative path generation for specs, designs, ADRs, active tasks, done tasks, and the task index.
- Added unit tests for empty repositories, normal repositories, gaps, duplicates, invalid IDs, numeric sorting, slug generation, and path conventions.
