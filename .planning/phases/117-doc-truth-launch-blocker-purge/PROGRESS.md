# PROGRESS — Phase 117 "Doc-truth launch-blocker purge"

_A live phase-progress briefing. Refresh at every task/wave boundary in the SAME push;
every relief handover refreshes it. A stale progress file is worse than none. Structure
mirrors `.planning/PROGRESS.md` (SHIPPED / NOW / NEXT), scoped to this phase's 7 plan
files and their 5-wave dependency DAG._

## Waves at a glance

| Wave | Plans | Concern | Status |
|---|---|---|---|
| W1 | 117-01 | SC3 connection-refused errors + SC4 `attach.rs` reword | ✅ GREEN + banked |
| W2 | 117-02 ∥ 117-03 | docs-truth (SC1/SC2 + propagation) | 🔄 in progress |
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

**Wave 2 in progress (117-02 ∥ 117-03, docs-truth).** Two parallel docs-only lanes with
zero file-overlap:

- **117-02** (this lane) — SC1 (`docs/index.md:13` Confluence-as-issue-tracker +
  `git clone` bootstrap verb) + SC2 (the `cat`-secretly-networks lie in
  `filesystem-layer.md` and its real propagation set: glossary/cli/git-remote/confluence).
  FOLD-IN carried from the #55→#56 handover: `docs/guides/troubleshooting.md:329` still
  names the phantom `reposix detach` — the twin of the `attach.rs` reference W1 purged;
  reworded to manual recovery in this lane.
- **117-03** — the sibling docs-truth lane (parallel, non-overlapping files).

**Done:** W1 + 117-01 GREEN.

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
