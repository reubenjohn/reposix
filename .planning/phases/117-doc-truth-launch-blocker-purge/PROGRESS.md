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
| W3 | 117-04 ∥ 117-06 | (per ROADMAP DAG) | ⬜ not started |
| W4 | 117-05 | (per ROADMAP DAG) | ⬜ not started |
| W5 | 117-07 | phase close | ⬜ not started |

## SHIPPED

- 2026-07-16 — **W1 (117-01) — GREEN + banked** — SC3 sim/GitHub/Confluence/JIRA
  connection-refused errors now teach the recovery command (matching the `init.rs`
  exemplar); SC4 ratified as Option B — reworded `attach.rs`'s dangling `reposix detach`
  reference rather than building a real subcommand (Option A deferred as `GTH-V15-43`).
  Test-name-honesty marker on the multi-SoT recovery test independently verified genuine.
  CI green (`29550609095`). — `52092ad` / `4af2ece` / `56a222b`

## NOW

**W2 (docs-truth + launch-blocker purge) is COMPLETE and SHIPPED.** The docs-alignment
refresh + the docs-repro pivot fix (GTH-V15-49 Option B: count uncovered snippets, not
raw fenced-block totals) are both green. Pushed at `c028d4c`; CI run `29559891747`
concluded success; the post-push `code/ci-green-on-main` P0 probe is PASS
(`last_verified` refreshed in `quality/catalogs/code.json`).

**Opening W3 — 117-04 ∥ 117-06** (per the phase dependency DAG), serialized:
1. **117-04** — docs IA/cold-reader polish.
2. **117-06** — CLAUDE.md fix-twice sweep + docs/social freshness gate + dead-code removal.

**Then:** W4 = `117-05` (launch animation embed), W5 = `117-07` (coordinator close; the
animation `gh release upload` is HELD pending owner approval — E1, external mutation,
do not attempt).

**Done:** W1 (117-01) GREEN + banked. W2 (117-02, 117-03) GREEN + shipped.

## NEXT

**Pending waves (per the phase dependency DAG):**

1. **W3 — 117-04 ∥ 117-06** — not started.
2. **W4 — 117-05** — not started.
3. **W5 — 117-07 (phase close)** — not started. Carries the GTH-V15-37 launch-animation
   embed; the mp4 `gh release upload` is an owner-gated external mutation (RAISE, do not
   attempt — `ORCHESTRATION.md` §9).

**Checkpoint protocol (all waves):** RAISE (do not attempt) any `STALE_DOCS_DRIFT` pre-push
BLOCK — L0 runs `/reposix-quality-refresh` at top level; depth-2 fan-out is unreachable
from inside a C1.
