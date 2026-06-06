---
id: 18
title: Implement global ID allocation and filename generation
kind: task
type: feature
status: planned
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

- [ ] Next-ID utility exists.
- [ ] Duplicate ID detection exists.
- [ ] Slug generation handles common title input.
- [ ] Filename generation follows the documented directory conventions.

## Done Criteria

- [ ] Tests cover empty repositories, normal repositories, gaps, and duplicates.
- [ ] IDs are sorted numerically, not lexicographically.
- [ ] Related specs remain accurate.

## Result

Not implemented.
