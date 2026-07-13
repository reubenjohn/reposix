---
quick_id: 260712-oke
slug: land-v0-14-deferred-onto-v0-15
date: 2026-07-12
status: complete
---

# Quick Task 260712-oke: land v0.14.0 DEFERRED intake entries onto the v0.15.0 surface

## Goal

Owner ask (2026-07-12): every DEFERRED / DEFERRED-TO-v0.15.0 entry from the v0.14.0
surprises-intake (`part-01.md` + `part-02.md`) needs a **concrete landing spot** on the
v0.15.0 planning surface — *labels alone don't count*: each needs a real row/stub with
**severity + a concrete fix-sketch**, verbatim-faithful to the intake. Close the
roadmap-gap where the intake promised "v0.15.0 hardening phases" the roadmap did not list.

## Design (as implemented)

7 live DEFERRED entries confirmed by reading both part files (map matched):

| id | src | sev | what |
|----|-----|-----|------|
| GTH-V15-01 | part-01 | MED | concurrent `--persist` runners race-corrupt catalog JSON |
| GTH-V15-02 | part-01 | MED | shell-coverage 12.54% < 13% floor — **cross-ref to 999.5/999.6**, not duplicated |
| GTH-V15-03 | part-01 | MED | no gate checks `verifier.script` exists+executable |
| GTH-V15-04 | part-02 | HIGH | RBF-LR-03 fix unverified on git ≥ 2.34 stateless-connect (residual) |
| GTH-V15-05 | part-02 | MED | `resolve_import_parent()` degrades on ANY git error |
| GTH-V15-06 | part-02 | HIGH | subprocess-bypass — no binary-side self-safety refusal in `reposix init` |
| GTH-V15-07 | part-02 | MED | release-plz unwatched by phase-close probe (blocker now CLEARED) |
| GTH-V15-08 | hygiene | MED | `.planning/ORCHESTRATION.md` 26968 B > 20000 ceiling (WAIVED to 2026-08-08) |

## Tasks

1. `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` (NEW) — one row per entry
   (source part-file, severity, description, concrete fix-sketch). GTH-V15-02 recorded
   as a CROSS-REFERENCE to its existing home (phases 999.5/999.6), not a duplicate.
   GTH-V15-07 notes the code.json foreign-lock blocker is now cleared (tree clean).
   Plus GTH-V15-08 hygiene row.
2. `.planning/milestones/v0.15.0-phases/ROADMAP.md` — add § "Hardening candidates" with
   two `### Phase (candidate)` stubs for the 2 HIGH entries + a roadmap-gap
   reconciliation note. The two existing UX `Phase TBD` stubs left untouched.
3. Back-pointers into the part files: SKIPPED (both already over the 20000-char ceiling;
   noted in GOOD-TO-HAVES.md).
4. gsd-quick tracking record + STATE.md "Quick Tasks Completed" row.

## Verification

- `git status` clean at start → confirms GTH-V15-07's code.json foreign-lock is cleared.
- GOOD-TO-HAVES.md = 12863 B, ROADMAP.md = 8648 B — both under the 20000-char `.md` ceiling.
- `git diff --stat` before commit proves no `MANAGER-HANDOVER.md` staged.

## Constraints

No cargo. No `git push` (coordinator pushes). Explicit-path `git add` only (no `-A`/`.`).
Do NOT touch `.planning/MANAGER-HANDOVER.md`. End commit body with the Co-Authored-By line.
