---
phase: 30
title: "Docs IA and narrative overhaul — landing page aha moment and progressive-disclosure architecture reveal"
depends_on: [phase-29]
status: awaiting-plan
created: 2026-04-17
---

# Phase 30 — Docs IA and narrative overhaul

## Source of truth

The narrative intent, framing principles, hero vignette, supporting vignettes,
IA sketch, and scope fence for this phase are captured in a dedicated
exploration note. **The planner must read this note in full before producing
`30-NN-PLAN.md` files.**

> **Primary input:** `.planning/notes/phase-30-narrative-vignettes.md` (committed
> at `1ba0479`, renamed from `phase-27-*` at Phase 30 addition).

## One-line scope

Rewrite the landing page and restructure the MkDocs nav so reposix's value
proposition lands hard within 10 seconds of a cold reader arriving, with the
technical architecture (FUSE, remote helper, simulator) progressively revealed
in a "How it works" section rather than leaked above the fold.

## Non-negotiable framing principles

1. **Complement, not replace.** reposix does not replace REST APIs; it absorbs
   the ceremony around the 80% of common operations. The word "replace" is
   banned from hero and value-prop copy.
2. **Progressive disclosure — phenomenology before implementation.** Layer 1
   (hero) describes what the user *experiences*; Layer 3 (how-it-works) is
   where FUSE/daemon/helper first appear. Banned terms above layer 3: FUSE,
   inode, daemon, helper, kernel, mount, syscall.

Full layer model, tonal rules, and rationale: see source-of-truth note
§"Framing principles".

## In scope (from source-of-truth note)

- **Hero rewrite** — landing page, above-fold copy, one before/after code
  block (V1), three-up value props.
- **"How it works" section** — three new pages (filesystem layer, git layer,
  trust model), each with one mcp-mermaid diagram (playwright-screenshot
  verified). Content carved from `docs/architecture.md` + `docs/security.md`.
- **Home-adjacent pages** — "Mental model in 60 seconds" (three conceptual
  keys: mount = git working tree, frontmatter = schema, `git push` = sync
  verb); "reposix vs MCP / SDKs" (comparison grounding P1).
- **New Guides** — "Write your own connector" (BackendConnector
  walkthrough); "Integrate with your agent" (Claude Code / Cursor / SDK
  patterns); "Troubleshooting" (stub that grows post-launch).
- **Simulator page relocated** — from How it works to Reference.
- **Tutorial** — 5-minute first-run experience against the simulator.
- **Nav restructure** — `mkdocs.yml` changes implementing the IA sketch.
- **mkdocs-material theme tuning** — palette, hero features, social cards.
- **Banned-word linter** enforcing progressive-disclosure layer rules.

## Out of scope

- New features, new CLI surface, new backend connectors.
- Changes to `REQUIREMENTS.md` beyond the phase itself.
- Rewrites of `docs/reference/` or `docs/decisions/` trees — Phase 26 already
  made those correct.

## Verification (per user OP#1: close the feedback loop)

- `mkdocs build --strict` green.
- `playwright` screenshots of landing + how-it-works + tutorial pages at
  desktop and mobile widths — attached to the phase SUMMARY.
- `gh run view` green on CI after push.
- Banned-word linter passes on every doc commit.

## Suggested subagent fan-out (planner to finalize)

1. **Explore** — competitor narrative scan (Linear, Turso, Fly.io, Tailscale,
   Warp, Val Town, Raycast, Stripe): extract one pattern per site that fits
   the hero vignette style.
2. **Copy** — hero + three value props, constrained by the P1 banned-word list.
3. **IA** — two competing nav structures against the sketch, scored against
   Diátaxis + the three personas in the note.
4. **Diagrams** (mcp-mermaid) — three architecture diagrams (filesystem layer,
   git layer, trust model), rendered and playwright-screenshotted.
5. **Tutorial** — authors the 5-minute getting-started path, runs it end-to-end
   against the simulator, screenshots each step.

## Milestone status

Phase 30 is deferred to v0.10.0. The v0.9.0 milestone was repurposed as the
"Architecture Pivot — Git-Native Partial Clone" milestone (2026-04-24), which
deletes FUSE and replaces it with git partial clone. Phase 30 docs MUST describe
the new architecture, not FUSE. The existing 9 plans (30-01 through 30-09) need
revision before execution.

## v0.9.0 architecture changes that affect Phase 30 narrative

> **Added 2026-04-24.** These notes capture design decisions from the architecture
> pivot that the Phase 30 planner MUST incorporate when revising plans. Read
> `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` for the full design.

### What changed in the product

- **FUSE is deleted.** There is no mount, no daemon, no fusermount3. The agent
  works with a real git repo on disk. `reposix init` replaces `reposix mount`.
- **Agent UX is pure git.** `git clone`, `cat`, `git push` — zero reposix CLI
  awareness. Agents learn from git error messages, not documentation.
- **Lazy loading via partial clone.** Files appear in the directory tree but
  content is fetched on first access. Sparse-checkout controls what materializes.
- **Push-time conflict detection.** No polling or refresh needed. At `git push`,
  the helper checks backend state and rejects with standard git errors if
  conflicts exist. Agent does normal `git pull --rebase` + `git push`.
- **Blob limit as teaching mechanism.** If an agent tries to fetch too many files,
  the helper refuses with an actionable error: "narrow your scope with
  sparse-checkout." No prompt engineering or system prompt needed.
- **Delta sync.** `git fetch` calls `?since=<timestamp>` on the backend — one
  API call regardless of repo size. Tree always fully synced; blobs lazy.

### Impact on Phase 30 plans

- **P2 banned terms list needs revision.** "FUSE" and "mount" no longer appear
  anywhere in the product. The progressive-disclosure layer model still applies
  but the banned terms shift: git internals (partial clone, promisor remote,
  sparse-checkout, stateless-connect) are the new Layer 3+ terms. Layer 1/2
  should say "files and folders" and "git."
- **Hero vignette V1 still works** but "mount" language in the after block must
  change to "clone" or "init." The before/after structure (curl vs cat+git) is
  even stronger now — it's literally just git.
- **"How it works" pages** should describe: (1) the directory tree = a git repo,
  (2) lazy-loading (files appear but content fetched on demand), (3) push =
  sync back to the service, (4) error messages teach scope narrowing.
- **Tutorial** runs against the simulator via `reposix init sim://demo /tmp/workspace`
  instead of `reposix mount`. The 5-minute first-run flow is simpler.
- **"reposix vs MCP / SDKs" page** gets a much stronger pitch: agents use
  standard git commands they already know, not custom tool schemas.
- **Architecture diagrams** show the helper binary, backing cache, and REST
  backends — no FUSE layer. Reference: mermaid diagram in
  `.planning/research/architecture-pivot-summary.md`.
- **Interesting features to highlight in docs:**
  - Error-as-teaching: "the helper teaches agents to use sparse-checkout via
    error messages — no documentation needed"
  - Push-time conflict detection: "like pushing to any git remote — no new
    concepts"
  - Delta sync: "one API call tells you what changed, regardless of repo size"
  - Platform support: "works everywhere git works — no FUSE, no kernel modules"
