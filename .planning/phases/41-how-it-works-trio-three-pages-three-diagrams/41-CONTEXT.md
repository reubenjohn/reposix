---
phase: 41
name: How-it-works trio — three pages, three diagrams (filesystem-layer / git-layer / trust-model)
milestone: v0.10.0
status: in-progress
date: 2026-04-24
skip_discuss: true
requirements: [DOCS-02]
---

# Phase 41 CONTEXT — How-it-works trio (filesystem-layer · git-layer · trust-model)

## Phase boundary (from ROADMAP.md)

> **Goal:** Carve `docs/how-it-works/{filesystem-layer,git-layer,trust-model}.md` from the v0.9.0 architecture-pivot summary + the existing `docs/architecture.md` + `docs/security.md`. Each page has one mermaid diagram. P2 progressive disclosure: the new banned-above-Layer-3 list (`partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2`) is permitted on these three pages but nowhere above them. The word "replace" is banned per P1.

Phase 41 ships:

- `docs/how-it-works/filesystem-layer.md` — cache flow + working tree + frontmatter; partial-clone is allowed here and FUSE may be mentioned only as the prior art that was superseded.
- `docs/how-it-works/git-layer.md` — helper protocol, push round-trip sequence diagram, push-time conflict detection, blob limit guardrail.
- `docs/how-it-works/trust-model.md` — lethal-trifecta cuts: tainted-by-default, egress allowlist, audit log, frontmatter field allowlist.

Each page has exactly one ` ```mermaid` fenced block.

## Inputs (already-committed artifacts)

- `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` — canonical architecture; reference for every diagram and prose claim.
- `docs/concepts/mental-model-in-60-seconds.md` (Phase 40) — three-keys baseline; each how-it-works page elaborates one key.
- `docs/concepts/reposix-vs-mcp-and-sdks.md` (Phase 40) — referenced for cross-links (not duplicated).
- `docs/research/agentic-engineering-reference.md` — lethal-trifecta source for trust-model.
- `crates/reposix-cache/src/audit.rs` + `src/cache.rs` — actual audit ops vocabulary (`materialize`, `egress_denied`, `helper_fetch`, `helper_push_started/accepted/rejected_conflict/sanitized_field`, `delta_sync`, `blob_limit_exceeded`).
- `mkdocs.yml` — has `pymdownx.superfences` with the `mermaid` custom_fences entry already configured.

## Claude's discretion (skip-discuss decisions)

- **Inline runner instead of `gsd-planner`/`gsd-executor` hop.** Three small docs files; mirroring Phase 40's pattern. Atomic commits per file.
- **Mermaid source only.** No PNG pre-rendering this phase. The mkdocs-mermaid plugin renders at build time. Phase 45 owns the playwright screenshot pass.
- **No `mkdocs build --strict`.** Phase 45 finalizes that. Phase 41 keeps cross-links relative and consistent so the strict build is one-shot in Phase 45.
- **Trust-model closing link target.** `docs/security.md` still exists (carve-out is Phase 43). Cross-link to it; Phase 43 will redirect the stub.
- **Refspec namespace + `git-remote-reposix` capability names.** These are Layer 4 jargon from the banned-above-Layer-3 list, so they appear ONLY on these three Layer 3 pages and may be mentioned by name. Reference page (`docs/reference/git-remote.md`) is where the wire-level detail lives; how-it-works summarises.

## Wave sizing — single-pass, no planner hop

Three atomic-commit waves, one file per commit:

- **41-01:** `docs/how-it-works/filesystem-layer.md` — cache + working tree.
- **41-02:** `docs/how-it-works/git-layer.md` — helper protocol + push.
- **41-03:** `docs/how-it-works/trust-model.md` — trifecta cuts.

Verification (`41-VERIFICATION.md`) follows the runner's goal-backward checks.

## Success criteria (from ROADMAP.md, condensed for assertable subset)

1. Three pages exist at the expected paths.
2. Each page has exactly 1 ` ```mermaid` fenced block (`grep -c '```mermaid' <file>` returns 1).
3. The word `replace` appears 0 times across the three pages (`grep -ic replace docs/how-it-works/*.md` returns 0 per file).
4. Each page contains a relative link to `concepts/mental-model-in-60-seconds.md`.
5. Layer-3 jargon (`partial-clone`, `stateless-connect`, etc.) appears ONLY on these three pages — verified during Phase 43 nav restructure when the banned-words linter goes live.

Out of scope (deferred):

- mkdocs.yml nav restructure — Phase 43.
- Banned-words linter wiring — Phase 43.
- mermaid PNG renders + playwright screenshots — Phase 45.
- Carving `docs/architecture.md` + `docs/security.md` into redirect stubs — Phase 43.
- doc-clarity-review release gate — Phase 44.
