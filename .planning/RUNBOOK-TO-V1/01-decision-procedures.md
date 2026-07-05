# RUNBOOK ch.01 — decision procedures + escalation valve, applied to this drive

**General doctrine moved (2026-07-05 permanence audit).** The DP-1..5 procedures, the
E1–E4 escalation valve, the fable consult-dispatch template, and the
`.planning/CONSULT-DECISIONS.md` ledger format are general orchestration doctrine, not
specific to the P92→v1.0 drive — they now live in
`.claude/skills/decision-procedures/SKILL.md` (`.planning/ORCHESTRATION.md` §11 is the
one-paragraph map that names them). **Load that skill when a judgment call actually
appears.** This chapter keeps only what's specific to this drive: the class exemplars
that motivated each procedure (so a successor understands WHY the procedure exists) and
this drive's own escalation history.

---

## Where each DP came from (class exemplars, 7e2a4cf2 engagement)

- **DP-1 — Coordinator-rot diagnosis.** The P91 Wave-5.5 polling-loop stall: rot was
  diagnosed from the OUTSIDE, from behavior alone (stop/watch cycles, no new commits),
  before the coordinator itself noticed.
- **DP-2 — Prove-before-fix on BLOCKERs.** A P90-window BLOCKER whose "repro" was a
  static code trace; fable forced an executable repro first — the trace had missed a
  guard, so the finding would have shipped a fix for the wrong mechanism.
- **DP-3 — Intake-design inversion.** Fable inverted an intake entry's sketched design —
  the sketch solved the symptom with a new mechanism; the fix used an existing one.
- **DP-4 — Executive resequencing.** OD-4's EXECUTIVE RESEQUENCE — the launch-readiness
  milestone pulled ahead of the v0.13.2 cross-link work, re-derived from the owner's
  stated intention ("puts this project on the global map"), not from phase order. See
  `03-road-to-v1.md` §A Portion 2 for where this landed.
- **DP-5 — Tangent-vs-charter classification.** The quality-gates framework ballooned
  from a tangent into the project's backbone — the right call, but it needed explicit
  classification at the time, not silent absorption.

## This drive's escalation history

Track live valve crossings in `.planning/CONSULT-DECISIONS.md`, not here (this file is
static reference, that ledger is the append-only record). Pre-framed decision points
this drive already knows will need a DP or the valve: P93's L2/L3 ADR decision (DP-4/E2
shape — see `03-road-to-v1.md` §C), and any P92/P94 litmus REOPEN cycles hitting two
failed iterations (E4 — see `02-loops-and-context.md` §A-L2).
