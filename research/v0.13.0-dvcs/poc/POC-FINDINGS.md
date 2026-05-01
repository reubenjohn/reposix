# POC-FINDINGS — v0.13.0 reposix attach + bus + precheck

**POC scope:** end-to-end exercise of the three v0.13.0 innovations
against the simulator. POC-01 / CARRY-FORWARD POC-DVCS-01.

**Simulator version:** `reposix-sim` from current workspace HEAD (P78
SHIPPED, gix 0.83.0).

**Date:** 2026-05-01 (in flight; updated by T05).

**Wall-clock:** TBD (filled in by T05).

---

## Path (a) — Reconciliation against mangled checkout

### What worked
- (placeholder; filled by T05 from `logs/path-a-reconciliation.log`)

### What surprised
- (placeholder)

### Design questions for 79-02
- (placeholder)

---

## Path (b) — Bus SoT-first observing mirror lag

### What worked
- (placeholder)

### What surprised
- (placeholder)

### Design questions for 79-02 / P82 / P83
- (placeholder)

---

## Path (c) — Cheap precheck on SoT mismatch

### What worked
- (placeholder)

### What surprised
- (placeholder)

### Design questions for 79-02 / P82
- (placeholder)

---

## Implications for 79-02

The orchestrator reads this section to decide whether to revise
`79-02-PLAN.md` before execution. Routing per
`79-PLAN-OVERVIEW.md` § "POC findings → planner re-engagement":

- **INFO** — informational, no plan revision needed.
- **REVISE** — 79-02 plan needs an in-place tweak.
- **SPLIT** — 79-02 scope exceeds budget; orchestrator surfaces split
  options to the owner.

| ID  | Tag | Finding | 79-02 task affected |
|-----|-----|---------|---------------------|
| TBD | TBD | (placeholder; filled by T05) | TBD |

---

## Time spent

- **Started:**  2026-05-01T06:20:31Z
- **Finished:** TBD
- **Total:**    TBD

Time-budget status (filled by T05):
- `< 1d`  → on budget (target).
- `1–2d`  → over target; not a SURPRISES item per CARRY-FORWARD POC-DVCS-01.
- `> 2d`  → SURPRISES-INTAKE entry filed at
            `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`.

---

## Cleanup

After this POC's findings are absorbed by 79-02:

- The directory `research/v0.13.0-dvcs/poc/` is RETAINED as historical
  research artifact (per CARRY-FORWARD POC-DVCS-01: "Throwaway code
  only; not v0.13.0 implementation"). No deletion in 79-02 or v0.13.0
  milestone close.
- The standalone scratch crate at `scratch/Cargo.toml` is NOT added to
  the workspace; production attach in 79-02 reuses the relevant code
  patterns by READING this POC, not by importing it.
