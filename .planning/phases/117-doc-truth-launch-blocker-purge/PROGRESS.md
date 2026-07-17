# PROGRESS — Phase 117 "Doc-truth launch-blocker purge"

_A live phase-progress briefing. Refresh at every task/wave boundary in the SAME push;
every relief handover refreshes it. A stale progress file is worse than none. Structure
mirrors `.planning/PROGRESS.md` (SHIPPED / NOW / NEXT), scoped to this phase's 7 plan
files and their 5-wave dependency DAG._

## Waves at a glance

| Wave | Plans | Concern | Status |
|---|---|---|---|
| W1 | 117-01 | SC3 connection-refused errors + SC4 `attach.rs` reword | ✅ GREEN + banked |
| W2 | 117-02 ∥ 117-03 | docs-truth (SC1/SC2/SC5 + propagation) | ✅ GREEN + shipped (`c028d4c`, CI `29559891747`) |
| W3 | 117-04 ∥ 117-06 | (per ROADMAP DAG) | ✅ GREEN + shipped (`0a5f620`, CI `29563263605` success) |
| W4 | 117-05 | launch animation embed | ✅ GREEN + shipped (`644763a`, CI `29565137211` success) |
| W5 | 117-07 | phase close (animation `gh release upload` HELD-E1) | 🔄 in progress — close-pending-owner-E1 |

## SHIPPED

- 2026-07-16 — **W1 (117-01) — GREEN + banked** — SC3 sim/GitHub/Confluence/JIRA
  connection-refused errors now teach the recovery command (matching the `init.rs`
  exemplar); SC4 ratified as Option B — reworded `attach.rs`'s dangling `reposix detach`
  reference rather than building a real subcommand (Option A deferred as `GTH-V15-43`).
  Test-name-honesty marker on the multi-SoT recovery test independently verified genuine.
  CI green (`29550609095`). — `52092ad` / `4af2ece` / `56a222b`

## NOW

**W4 (117-05, launch animation embed) is COMPLETE and SHIPPED GREEN** at `644763a` — CI
run `29565137211` concluded success (post-push `code/ci-green-on-main` P0 probe PASS).
Poster WebP committed under `docs/assets/animation/`, click-to-play `<video>` embedded on
`docs/index.md` (poster + controls, no autoplay, no CDN Babel/React/unpkg), pinned to the
`docs-assets` release tag / `Reposix Launch Animation.mp4` filename so W5's upload matches
byte-for-byte. The 16-row `docs-alignment` line-shift cascade the embed caused was rebound
in the same wave (`644763a`); a catalog-first `docs-build/animation-renders` row is minted
`NOT-VERIFIED` (P2 — non-blocking on pre-push) pending W5's artifact generation against the
live release URL. Two stretch GTHs filed (`GTH-V15-54` interactive JSX embed, `GTH-V15-55`
hermetic-CDN `simpleicons` concern); a third (`GTH-V15-57`, doc-alignment line-number-bind
fragility — three full rebind cycles this phase) filed in the W4 rebind commit itself.

**Now in W5 — `117-07` (coordinator phase close) — IN PROGRESS, close-pending-owner-E1.**
Sub-lane 117-07's NON-BLOCKED portion is done this pass: folded the leftover
`code/ci-green-on-main` audit bump (chore commit); removed a stray, pre-mature
`.planning/verifications/playwright/index/animation.json` — a `run.py` "verifier not
found" stub written before `quality/gates/docs-build/animation-renders.sh` exists, not
real playwright evidence (the row's own `owner_hint` says both the artifact and the
verifier script are intentionally absent until W5 generates them against the live URL —
so "absent" is the correct pre-upload state, not a gap); confirmed `git status` clean;
re-ran the non-cargo gate sweep GREEN. Per the PLAN, the `animation-renders.sh` verifier
script itself is bundled with Task 2 (live artifact generation), which cannot run until
Task 1 (owner-approved `gh release upload`) lands — so the script stays unwritten, not a
gap, matching the row's own documented deferral.

**HELD-E1 (owner-gated, do not attempt):** Task 1 — `gh release create docs-assets` +
`gh release upload docs-assets 'Reposix Launch Animation.mp4'` — needs owner-named-target
approval per CLAUDE.md Non-negotiables; already RAISED to the top level. Task 2's live
playwright artifact generation + `animation-renders.sh` authoring are gated on Task 1's
asset existing. Until the owner approves, phase 117 is NOT fully closeable.

**Still coordinator/top-level-only after E1 clears:** Task 3 (cold-reader `/doc-clarity-
review` + `/reposix-quality-review --rubric` + badge-resolution sign-off — depth-2 fan-out
unreachable from inside a subagent tree-writer) and Task 4 (final pre-push, `git push
origin main`, post-push CI-green, unbiased verifier dispatch).

**Done:** W1 (117-01) GREEN + banked. W2 (117-02, 117-03) GREEN + shipped. W3 (117-04,
117-06) GREEN + shipped. W4 (117-05) GREEN + shipped.

## NEXT

**Pending, all owner/coordinator-only (per the phase dependency DAG):**

1. **Owner approval + upload (Task 1, HELD-E1)** — owner names/confirms the `docs-assets`
   tag, coordinator runs `gh release create` + `gh release upload` (external mutation,
   not self-authorizable).
2. **Coordinator Task 2** — once the asset is live: generate the playwright artifact at
   `.planning/verifications/playwright/index/animation.json` against the live URL, author
   `quality/gates/docs-build/animation-renders.sh` (mirrors `mermaid-renders.sh`'s
   source→artifact assertion shape), confirm `docs-build/animation-renders` flips PASS.
3. **Coordinator Task 3** — `/doc-clarity-review` + `/reposix-quality-review --rubric` +
   badge-resolution sweep (top-level slash commands, unreachable from a C1/subagent).
4. **Coordinator Task 4** — full pre-push, `git push origin main`, post-push CI-green
   confirmation, unbiased gsd-verifier dispatch grading SC1-SC5 + GTH-V15-36/37.

**Checkpoint protocol (all waves):** RAISE (do not attempt) any `STALE_DOCS_DRIFT` pre-push
BLOCK — L0 runs `/reposix-quality-refresh` at top level; depth-2 fan-out is unreachable
from inside a C1.
