# SESSION-HANDOVER.md — v0.13.0 tag-halted, 2026-07-06

For the incoming top-level orchestrator (L0). Replaces the P92-era handover (all closed).
This is the map, not the territory — detail lives in git and the linked files.

## 1. Current state

- **v0.13.0 autonomously GREEN** — P78–P97, 20/20 phases shipped.
- **Tag HALTED** on one owner decision (§2), not on any failing phase.
- **Git:** published `origin/main = 8a39efc`. Local is **UNPUSHED** ahead (chain:
  `3109fbb` doctrine → `1b37350` prune → wind-down + review commits; run `git log`).
  **Next session must coordinate the push** — flag it before any new work.

## 2. #1 tag gate (E4) — the decision the tag waits on

Create-partial-fail reconciliation **cannot converge on id-reassigning backends**
(GitHub / JIRA / Confluence): the placeholder-id → backend-id map has no home, because
cursor / `oid_map` is deliberately NOT advanced on `SotPartialFail`. Surfaced by the
real-backend 9th probe (P93) — a P0 the SIM backend HID.

→ **Owner design decision required:** revise or supersede
`docs/decisions/010-l2-l3-cache-coherence.md` §3 (RBF-LR-03), deciding where the mapping
lives. **Nothing tags until this resolves.**

## 3. t4-real — unimplemented

Real-backend sibling of sim's T4 litmus (conflict → rebase root-commit ancestry
invariant). **Option B** (recommended, `.planning/milestones/v0.13.0-phases/97-HANDOVER.md`):
an `#[ignore]` Rust smoke in `crates/reposix-cli/tests/agent_flow_real.rs` (~1.5h, no
schema change) + a shell wrapper mirroring the sim T4 assertions.

## 4. Live deferred backlog

The pruned intakes ARE the live registry (open-only; resolved items are DELETED — git is
the archive):

- `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` — **28 open**
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` — **40 open** (incl. the 2
  doctrine follow-ups)

**Git-only relocated items** — deleted with the SURPRISES archive during the prune, NOT
proven resolved; carry forward as pointers. Full text:
`git show 3109fbb:.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE-ARCHIVE-P89-P97.md`

- **P84-01-T05** (HIGH)
- **P89 cross-AI Claude-leg**
- **steward-window** (MEDIUM)
- **quality-convergence** (HIGH)
- **P91 T2-REOPEN** (MEDIUM)
- **Entry-27** — walker forward pre-audit (post-v0.14.0 gate)

## 5. Known brittle gate — p94 badges misfire

`quality/gates/docs-build/p94-badges-real-vs-transient.sh:78` greps `GOOD-TO-HAVES.md` for
an h2 heading that the OP-8 archive-drain relocated → regex fails → **false pre-push
FAIL**. Pre-existing, not session-introduced. Fix by asserting the invariant, not the
heading (see RETROSPECTIVE calibration § "Gates assert invariants"). A known brittle gate
to fix or replace.

## 6. Doctrine

C2 / relief-threshold doctrine finalized this session — see `.planning/ORCHESTRATION.md`
(pointer only; do not restate or edit here).

---

History lives in git — `git log` / `git show`, not restated here.
