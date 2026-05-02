---
phase: 30
plan: 03
type: execute
wave: 1
depends_on: [30-01, 30-02]
files_modified:
  - docs/index.md
  - docs/mental-model.md
  - docs/vs-mcp-sdks.md
autonomous: true
requirements: [DOCS-01, DOCS-03]
must_haves:
  truths:
    - "A cold reader on docs/index.md can state reposix's value prop in one sentence within 10 seconds (validated by doc-clarity-review in Wave 4)."
    - "docs/index.md above the H2 'How to read this site' contains the V1 before/after code pair + the complement-line blockquote + a three-up grid-cards value-prop block."
    - "docs/index.md contains zero banned P1 tokens (replace*) and zero banned P2 tokens (FUSE/inode/daemon/kernel/mount/syscall) outside code fences."
    - "docs/mental-model.md is 300-400 words with each of the three locked H2s followed by 1-sentence equation + 1-2 sentence explanation + 3-5 line code snippet."
    - "docs/vs-mcp-sdks.md contains a comparison table, a P1-grounded paragraph using 'complement'/'absorb'/'subsume', and a footnote-style citation to the token-economy benchmark in why.md."
  artifacts:
    - path: "docs/index.md"
      provides: "Landing page with V1 before/after hero + complement line + three-up value props"
      min_lines: 80
      contains: "git commit -am"
    - path: "docs/mental-model.md"
      provides: "Mental-model page with three locked H2s and 300-400 words of prose + snippets"
      min_lines: 50
    - path: "docs/vs-mcp-sdks.md"
      provides: "Comparison page with table + P1 paragraph + benchmark citation"
      min_lines: 40
  key_links:
    - from: "docs/index.md"
      to: "docs/tutorial.md"
      via: "Where to go next grid-card link"
      pattern: "tutorial.md"
    - from: "docs/index.md"
      to: "docs/mental-model.md"
      via: "Where to go next grid-card link"
      pattern: "mental-model.md"
    - from: "docs/index.md"
      to: "docs/how-it-works/index.md"
      via: "Where to go next grid-card link"
      pattern: "how-it-works"
    - from: "docs/vs-mcp-sdks.md"
      to: "docs/why.md"
      via: "token-economy benchmark footnote"
      pattern: "why.md#token-economy-benchmark"
---

<objective>
Rewrite docs/index.md into a Layer-1 narrative hero that lands reposix's value proposition in 10 seconds for a cold reader, and fill the mental-model + vs-mcp-sdks skeletons from Wave 0 with publishable copy. After this plan lands:

- A cold reader arriving at `docs/index.md` sees (in order): a two-beat tagline, the V1 before/after code pair (curl/jq ceremony vs `sed` + `git push`), the mandatory complement-line blockquote, a three-up grid-cards value-prop block, and a "Where to go next" grid pointing at tutorial / mental-model / how-it-works / why.
- `docs/mental-model.md` is ~350 words — each of the three locked H2s (`mount = git working tree`, `frontmatter = schema`, `` `git push` = sync verb ``) gets: 1 sentence equation, 1-2 sentences explanation, 3-5 line code snippet. Ends with a "Now what" pointer.
- `docs/vs-mcp-sdks.md` contains a comparison table (reposix vs MCP vs REST SDK), a paragraph explicitly grounding P1 with the words "complement" / "absorb" / "subsume", and cites the token-economy benchmark from `why.md` via footnote.
- `vale --config=.vale.ini docs/index.md docs/mental-model.md docs/vs-mcp-sdks.md` exits 0 (P1 + P2 clean, per-file exception for mental-model.md respected).
- `mkdocs build --strict` still passes (no broken internal links introduced).

Purpose: DOCS-01 + DOCS-03 — the entire "value prop lands in 10 seconds" outcome hangs on this copy. Plan 30-04 wires the nav; plan 30-09 runs doc-clarity-review to validate the 10-second claim. This plan authors the prose.

Output: 3 rewritten markdown files (docs/index.md gets a full rewrite; mental-model + vs-mcp-sdks get content filled into existing skeletons).

**Locked decisions honored:**
- DOCS-01 hero vignette = **Vignette 1 "Close a Jira ticket"** (source-of-truth lines 109-181).
- Complement-line blockquote directly under the "after" block, EXACT TEXT from source-of-truth lines 173-177 (quoted in Task 1 action verbatim).
- DOCS-03 three conceptual keys — H2s verbatim, preserved from plan 30-02's skeleton.
- Competitor-pattern selection: **B (Fly.io two-beat cadence) + D (Val Town positioning-line) + G (Stripe "aha")** — RESEARCH.md §"Menu-style recap". NOT Linear, Warp, Raycast, Turso, Cloudflare.
- P1 banned word "replace" — zero occurrences in index.md.
- P2 banned above Layer 3 — zero occurrences in index.md (except in code fences which Vale exempts).
- Creative bans enforced: no stock photos, no "empower", no "revolutionize", no "next-generation", no feature-grid tables with check-mark icons, no marketing bullet points.
</objective>

- [context.md](./context.md) — execution context, interfaces, threat model
- [task-1.md](./task-1.md) — Task 1: rewrite docs/index.md
- [task-2.md](./task-2.md) — Task 2: fill docs/mental-model.md + docs/vs-mcp-sdks.md
- [verification.md](./verification.md) — verification, success criteria, output
