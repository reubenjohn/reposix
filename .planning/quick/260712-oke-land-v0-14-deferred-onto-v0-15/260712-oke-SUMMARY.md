---
quick_id: 260712-oke
slug: land-v0-14-deferred-onto-v0-15
date: 2026-07-12
status: complete
---

# Quick Task 260712-oke — Summary

Gave every DEFERRED / DEFERRED-TO-v0.15.0 entry from the v0.14.0 surprises-intake a concrete
landing spot on the v0.15.0 surface — a real row with severity + fix-sketch, not just a label.
Closed the roadmap-gap (the intake cited "v0.15.0 hardening phases" the roadmap didn't list).

## Files changed

- **`.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` (NEW, 12863 B)** — 8 rows:
  GTH-V15-01..07 (the 7 carried-forward intake entries, verbatim-faithful) + GTH-V15-08
  (ORCHESTRATION.md file-size hygiene). Each row: source part-file + discovered-by + date,
  severity, what, concrete fix-sketch. GTH-V15-02 (shell-coverage) is a CROSS-REFERENCE to
  its existing home (phases 999.5/999.6), explicitly NOT duplicated. GTH-V15-07 (release-plz)
  records that its old "code.json foreign-locked" blocker is now CLEARED (tree clean at
  landing). A back-pointer note documents why the reverse pointers into the part files were
  skipped.
- **`.planning/milestones/v0.15.0-phases/ROADMAP.md` (edited, 8648 B)** — new
  § "Hardening candidates" with a roadmap-gap reconciliation note + two `### Phase (candidate)`
  stubs for the 2 HIGH entries (GTH-V15-04 modern-git RBF-LR-03 verification + GTH-V15-05
  helper-hardening folded in; GTH-V15-06 binary-side self-safety refusal). The two existing
  UX `Phase TBD` stubs are byte-untouched.
- gsd-quick tracking record (PLAN.md + this SUMMARY) + STATE.md "Quick Tasks Completed" row.

## Entry-by-entry landing (8/8)

1. GTH-V15-01 (part-01, MED, catalog race) → GOOD-TO-HAVES row + flock/single-persist-lane sketch.
2. GTH-V15-02 (part-01, MED, shell-coverage) → CROSS-REFERENCE to phases 999.5/999.6 (not duplicated).
3. GTH-V15-03 (part-01, MED, verifier-script-exists) → GOOD-TO-HAVES row + `structure/verifier-script-exists.sh` sketch.
4. GTH-V15-04 (part-02, HIGH, RBF-LR-03 modern-git) → GOOD-TO-HAVES row + ROADMAP `### Phase (candidate)` stub.
5. GTH-V15-05 (part-02, MED, resolve_import_parent) → GOOD-TO-HAVES row + folded into the helper-hardening ROADMAP stub.
6. GTH-V15-06 (part-02, HIGH, subprocess-bypass) → GOOD-TO-HAVES row + ROADMAP `### Phase (candidate)` stub.
7. GTH-V15-07 (part-02, MED, release-plz unwatched, blocker CLEARED) → GOOD-TO-HAVES row + owner-gate questions.
8. GTH-V15-08 (hygiene, MED, ORCHESTRATION.md 26968 B > 20000) → GOOD-TO-HAVES hygiene row + split sketch.

## Verify-against-reality

- Both intake part files read; the 7-entry map (ids/wording/severity) confirmed. The 2 HIGH
  entries are exactly GTH-V15-04 (RBF-LR-03 modern-git) and GTH-V15-06 (subprocess-bypass).
- `git status` was clean at task start → independently confirms GTH-V15-07's code.json
  foreign-lock is cleared (code.json unmodified).
- New/edited `.md` files verified under the 20000-char ceiling (12863 / 8648 B).
- `git diff --cached --stat` inspected before commit — no `MANAGER-HANDOVER.md` staged.

## Noticing (OD-3)

- **Both intake part files are ALREADY over the 20000-char `.md` ceiling** (part-01 = 21516 B,
  part-02 = 21574 B) — the OP-8 split that created them didn't bring either under budget, only
  under the pre-split monolith. Filed as the reason the reverse back-pointers were skipped;
  a future progressive-disclosure re-split of the intake parts is the real fix.
- The gitStatus snapshot in the session header showed `code.json` modified + several untracked
  dirs, but the live tree was clean when work started — the manager/other lane committed or
  reverted between snapshot and dispatch. Worth noting for anyone trusting the stale snapshot.
- GTH-V15-04's severity is HIGH per the intake header, but its BODY is explicit that the
  residual is "verification-only, NOT a live bug" (parent fix RESOLVED-in-P105 `bd5b9cb`,
  gate GREEN on 2.25.1). Preserved both facts in the row so the planner doesn't over-scope it
  as an unfixed defect.
