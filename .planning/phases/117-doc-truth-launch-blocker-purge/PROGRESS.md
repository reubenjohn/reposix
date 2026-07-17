# PROGRESS — Phase 117 "Doc-truth launch-blocker purge"

_A live phase-progress briefing. Refresh at every task/wave boundary in the SAME push;
every relief handover refreshes it. A stale progress file is worse than none. Structure
mirrors `.planning/PROGRESS.md` (SHIPPED / NOW / NEXT), scoped to this phase's 7 plan
files and their 5-wave dependency DAG._

## Waves at a glance

| Wave | Plans | Concern | Status |
|---|---|---|---|
| W1 | 117-01 | SC3 connection-refused errors + SC4 `attach.rs` reword | ✅ GREEN + banked |
| W2 | 117-02 ∥ 117-03 | docs-truth (SC1/SC2/SC5 + propagation) | 🔴 push BLOCKED by pre-push — wave RED |
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

**Wave 2 push BLOCKED by the pre-push hook — WAVE RED, nothing landed on origin/main.**
The nine W2 commits (`f42455a`..`c76b2c2`) are banked LOCALLY only; `git push origin main`
was rejected (exit 1, no `--no-verify`) with three FAILs. Two are genuine W2 content
regressions that must be fixed before any re-push; the third is the design-deferred
docs-alignment drift:

1. **`structure/banned-words` (P1 FAIL)** — `docs/guides/troubleshooting.md:334` (blamed to
   W2 commit `214f45e`) uses the banned plumbing word **"promisor"** in the recovery line
   `git config --unset extensions.partialClone  # clear the promisor binding to the old SoT`.
   Fix: reword to avoid "promisor" OR add `<!-- banned-words: ok -->` per `docs/.banned-words.toml`.
2. **`docs-repro/snippet-coverage` (P1 FAIL)** — the W2 troubleshooting reword pushed the
   fenced-code-block count to **51, over the threshold of 50** (`uncovered_count: 0`, no
   drift). Fix: switch to allow-list mode per `quality/gates/docs-repro/README.md` pivot rules.
3. **`docs-alignment/walk` (P0 FAIL)** — the BOUND doc-alignment row drift from the SC1/SC2
   edits. **Deferred by design to the W6 coordinator refresh** (RAISE, do NOT run `walk`/
   `bind`/`/reposix-quality-refresh` from inside a C1 — checkpoint protocol below).

**Next re-push is gated on:** (a) fix #1 + #2 in `troubleshooting.md` (117-02 lane), and
(b) L0 clearing the #3 STALE_DOCS_DRIFT via `/reposix-quality-refresh` at top level.

**Done:** W1 (117-01) GREEN + banked. W2 is committed-locally but NOT shipped.

**Deferred by design (do NOT chase in W2):** the BOUND doc-alignment row drifts these SC1/SC2
edits trip (index ×2, cli, git-remote, filesystem-layer `blob-lazy-first-cat` REWRITE) are
consolidated into the **W6 coordinator refresh** — no `bind`/`walk`/`/reposix-quality-refresh`
in this wave (`quality/CLAUDE.md`). Running `walk.sh` raw would mutate committed catalog counters.

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
