---
name: ✨ Feature request
about: Propose a new capability
title: "[FEATURE] "
labels: enhancement
---

## Use case (concrete)

What is the agent / human / pipeline trying to do that reposix doesn't make easy today?

## Proposed UX (preferred: pure git, no new CLI)

How would a fresh agent — with no reposix-specific knowledge — discover and use this feature?

## Why MCP / SDK isn't sufficient

(reposix's value prop is the dark-factory pattern — no schema discovery, no custom CLI surface. New features should preserve this. If the feature requires teaching the agent a new command, justify why.)

## Threat model impact

Does this feature touch:
- [ ] New egress endpoint
- [ ] New tainted-byte path (network input)
- [ ] New audit log op
- [ ] Cache-write semantics
- [ ] Frontmatter schema

If yes to any: propose mitigations.

## Acceptance criteria

(grep-checkable conditions for "done")
