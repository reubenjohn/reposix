# Project practices (long-form)

Moved out of CLAUDE.md per D-CONV-7 (quality/SURPRISES.md, 2026-07-04) —
CLAUDE.md keeps a 5-line summary of each practice; this file is the
authoritative long-form. Update BOTH when a practice changes.

## OP-8 — Plans accommodate surprises (the +2 phase practice)

Every milestone reserves its **last two phases** as absorption slots for
what reality surfaces during planned-phase execution:

- **Slot 1 — Surprises absorption.** Issues a planned phase discovered
  but couldn't fix without doubling its scope. The discovering phase
  appends to `.planning/milestones/<v>-phases/SURPRISES-INTAKE.md` (one
  entry per item: severity + what + why-out-of-scope + sketched
  resolution) instead of silently skipping or expanding scope. The
  surprises-absorption phase drains the file: each entry → RESOLVED |
  DEFERRED | WONTFIX, each with a commit SHA or rationale.
- **Slot 2 — Good-to-haves polish.** Improvements (clarity, perf,
  consistency, grounding) the planned phases observed but didn't fold
  in. Same intake mechanism, separate file (`GOOD-TO-HAVES.md`). Sized
  XS / S / M; XS items always close; M items default-defer to next
  milestone.

**Eager-resolution preference (load-bearing):** when a planned phase
observes a surprise or polish item, prefer fixing it inside the
discovering phase IF (a) < 1 hour incremental work, (b) no new
dependency introduced. The +2 reservation is for items that genuinely
don't fit the discovering phase. **What this practice prevents:** the
"found-it-but-skipped-it" failure mode where good signal gets dropped
to keep a phase tight; AND the "scope-creep-to-fit-the-finding"
failure mode where a phase grows to twice its planned size to absorb
every drift discovered. The intake split makes "I saw it, here's what
I think, P<last-2> will handle it" the default move.

**Verifier honesty check:** the surprises-absorption phase's verifier
subagent spot-checks the previous phases' plans + verdicts and asks
"did this phase honestly look for out-of-scope items?" An empty intake
is acceptable IF the running phases produced "Eager-resolution"
decisions in their plans; an empty intake when the verdicts show
skipped findings is a RED signal. This prevents the practice from
degrading into a no-op.

**F-K5 meta-rule (P90 RBF-FW-10, D90-08):** the above honesty check is
not discretionary prose — it is formalized, hash-bound, and
mechanically verified at
`quality/dispatch/absorption-honesty-spot-check.md` (four binding
clauses: sample EVERY no-intake phase; spot-check author ≠ milestone
orchestrator; rubric = "walk one critical example end-to-end mentally
— does it work?"; verifier content-hash-binds the report). Every
Slot-1 absorption phase MUST run that template, not reinvent this
section's prose from scratch. See `quality/PROTOCOL.md` § "Absorption
honesty spot-check dispatch" for the runtime contract.

The +2 reservation is in addition to whatever planned phases the
milestone scopes; if the milestone has 8 planned phases, it actually
has 10 (planned + 2 reservation). Roadmap entries for the reservation
phases name them explicitly so they're not omitted by accident.

## OP-9 — Milestone-close ritual: distill before archiving

Each milestone's `*-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` entries
AND the autonomous-run session findings get distilled into a new
section of `.planning/RETROSPECTIVE.md` BEFORE the milestone archives —
using the existing template (What Was Built / What Worked / What Was
Inefficient / Patterns Established / Key Lessons). Raw intake files
travel with the milestone archive into `*-phases/`; distilled lessons
live permanently and discoverably in `RETROSPECTIVE.md`.

**Why:** without this step, learnings get lost in milestone archives —
the +2 phase practice produces signal that's worth keeping
cross-milestone (failure modes, patterns, process gaps) but the raw
intake format is too granular for future readers to skim. The
ratification subagent for milestone-close should verify a
RETROSPECTIVE.md section exists for the milestone and grade RED if it
doesn't.
