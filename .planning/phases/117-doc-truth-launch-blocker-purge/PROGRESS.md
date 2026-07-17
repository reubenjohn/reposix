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
| W4 | 117-05 | launch animation embed | 🔄 in progress |
| W5 | 117-07 | phase close (animation `gh release upload` HELD-E1) | ⬜ queued |

## SHIPPED

- 2026-07-16 — **W1 (117-01) — GREEN + banked** — SC3 sim/GitHub/Confluence/JIRA
  connection-refused errors now teach the recovery command (matching the `init.rs`
  exemplar); SC4 ratified as Option B — reworded `attach.rs`'s dangling `reposix detach`
  reference rather than building a real subcommand (Option A deferred as `GTH-V15-43`).
  Test-name-honesty marker on the multi-SoT recovery test independently verified genuine.
  CI green (`29550609095`). — `52092ad` / `4af2ece` / `56a222b`

## NOW

**W3 (117-04 ∥ 117-06) is COMPLETE and SHIPPED GREEN** at `0a5f620` — CI run
`29563263605` concluded success (post-push `code/ci-green-on-main` P0 probe PASS).

**Now in W4 — `117-05` (launch animation embed)**, sub-lane owner executing against the
dedicated `117-05-PLAN.md`: poster WebP committed under the new `docs/assets/animation/`
convention, click-to-play `<video>` embedded on `docs/index.md` (poster + controls, no
autoplay, no CDN Babel/React/unpkg), pinned to the `docs-assets` release tag / `Reposix
Launch Animation.mp4` filename so W5's upload matches byte-for-byte, and a catalog-first
`docs-build/animation-renders` row minted `NOT-VERIFIED` (P2 — confirmed non-blocking on
pre-push; `run.py`'s `compute_exit_code` only fails P0/P1 rows) pending the coordinator's
117-07 artifact generation against the live release URL. Two stretch GTHs filed
(`GTH-V15-54` interactive JSX embed, `GTH-V15-55` hermetic-CDN `simpleicons` concern).

**Then:** W5 = `117-07` (coordinator close; the animation `gh release upload` is HELD
pending owner approval — E1, external mutation, do not attempt).

**Done:** W1 (117-01) GREEN + banked. W2 (117-02, 117-03) GREEN + shipped. W3 (117-04,
117-06) GREEN + shipped.

## NEXT

**Pending waves (per the phase dependency DAG):**

1. **W4 — 117-05** — in progress (this session). NOTE for the coordinator: the
   `docs/index.md` edit shifted line numbers for every doc-alignment row bound to that
   file — `bash quality/gates/docs-alignment/walk.sh` reports 16 `STALE_DOCS_DRIFT` rows
   on `docs/index.md` (same class as the W3 push-blocker, commit `3ed2f21`). Does NOT
   block local commits (pre-commit cadence doesn't run `docs-alignment/walk`), but WILL
   block the W4 `pre-push` run — RAISE, do not hand-fix (catalog is mint-only); resolve
   via `/reposix-quality-refresh docs/index.md` at top level before pushing, per the
   checkpoint protocol below.
2. **W5 — 117-07 (phase close)** — not started. Carries the GTH-V15-37 launch-animation
   embed; the mp4 `gh release upload` is an owner-gated external mutation (RAISE, do not
   attempt — `ORCHESTRATION.md` §9).

**Checkpoint protocol (all waves):** RAISE (do not attempt) any `STALE_DOCS_DRIFT` pre-push
BLOCK — L0 runs `/reposix-quality-refresh` at top level; depth-2 fan-out is unreachable
from inside a C1.
