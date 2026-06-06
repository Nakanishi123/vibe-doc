# Specs

Specs define what vibe-doc must do.

A spec should focus on externally observable behavior, user needs, constraints, API contracts, error cases, and acceptance criteria. It should avoid detailed implementation choices unless they are part of the product contract.

## Frontmatter

```yaml
---
id: 8
title: Example Spec
kind: spec
tags:
  - example
---
```

`status` is optional for specs. Existing specs are considered active unless explicitly marked deprecated.

## Current Specs

- `8`: vibe-doc product overview
- `9`: document model and repository layout
- `10`: CLI and validation behavior
- `11`: Web UI and API behavior
- `12`: Codex and agent integration
- `13`: delivery plan and MVP scope
- `14`: task model and lifecycle
