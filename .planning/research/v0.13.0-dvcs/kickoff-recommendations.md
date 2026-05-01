# v0.13.0 — Kickoff Recommendations

> **Audience.** The agent picking up v0.13.0 planning, after `/gsd-new-milestone v0.13.0` runs. Read **after** `vision-and-mental-model.md` (the thesis) and `architecture-sketch.md` (the technical design + open questions).
>
> **Status.** Advisory. Surfaced 2026-04-30 from the v0.12.1 milestone-close session, in response to the question "does v0.13.0 have the best chance now?" These recommendations convert "best chance" (~60%) toward "high chance" (~80%) by addressing readiness gaps the v0.12.1 cycle exposed.
>
> **Source.** v0.12.1 close-out session 2026-04-30. The recommendations follow from process learnings codified in `.planning/RETROSPECTIVE.md` § "Milestone: v0.12.1" and the new OP-9 ritual in `CLAUDE.md`.

## Why these matter

v0.13.0 is well-positioned: clean baseline (alignment_ratio = 1.0, retire-backlog drained, no carry-forward debt entering), substantive pre-research (408 lines across the sibling files with explicit open-questions / risks / success-gates / out-of-scope sections), a successful thesis-level precedent (v0.9.0 FUSE→git-native pivot), and process discipline that matured under v0.12.1 pressure (catalog-first, +2 phase practice, verifier-subagent dispatch, pre-commit fmt hook).

But DVCS is intrinsically harder than docs-alignment work. The architecture sketch already lists reconciliation cases that include hard errors; "Open questions for the planner" is not a placeholder. And two practices (the +2 phase eager-resolution heuristic, the autonomous-run push cadence) are untested on multi-day phases.

The four recommendations below address the gaps where v0.12.1 either struggled (push cadence) or didn't have to test (deep-phase eager-resolution).

---

## 1. Surface the architecture-sketch's "Open questions for the planner" verbatim before scoping phases

The sketch explicitly says *"specific algorithms below are starting points, not commitments."* Each open-question block is a forced decision the planner must make BEFORE writing PLAN.md, not during.

**Action.** Copy each "Open questions" block from `architecture-sketch.md` into v0.13.0's ROADMAP.md or a `.planning/research/v0.13.0-dvcs/decisions.md` companion file. For each question, record either:
- A decision (with one-line rationale), OR
- An explicit "deferred to phase N" with the trigger condition that forces the decision.

**Why.** Open questions that travel into phase execution become surprises. The v0.12.1 retrospective shows the +2 phase practice catches surprises but at the cost of intake-and-disposition overhead. Resolve what's resolvable before phases start; defer the rest with named triggers.

---

## 2. POC the bus-remote + reconcile path in `research/v0.13.0-dvcs/poc/` before committing to phases

The v0.9.0 FUSE→git-native pivot did this — a `poc/` subfolder under `research/v0.9-fuse-to-git-native/` proved the partial-clone helper end-to-end before phase decomposition. That POC caught issues that would have been mid-milestone surprises.

**Action.** Build a minimal end-to-end demo of the three innovations (`reposix attach`, mirror-lag refs, bus remote) against the simulator. Specifically exercise:
- `reposix attach` reconciliation against a working tree with mixed `id`-bearing + `id`-less files.
- A bus-remote push that observes mirror lag (SoT writes succeed, mirror trailing).
- The cheap-precheck path (refuse fast when SoT version mismatches local cache).

The POC ships as throwaway code in `poc/` — it is NOT the v0.13.0 implementation. Its job is to surface integration issues + algorithm-shape decisions that are cheaper to discover in a 1-day exploration than in phase-3 of a 6-phase milestone.

**Why.** v0.9.0's POC took ~1 day and shaved an estimated 3-4 days of mid-phase rework. ROI is strongly positive for thesis-level shifts.

---

## 3. Resolve push cadence (999.4) explicitly as v0.13.0's first decision, not a backlog item

v0.12.1's autonomous-run accumulated 115 commits between pushes. Pre-commit fmt hook (a25f6ff) closes the worst case (drift compounding) but the deeper feedback-loop delay — CI signal arrives at session-end — violates global OP #1 ("verify against reality"). DVCS phases will likely span days each; the same +N-stack pattern would compound 5-10× the v0.12.1 magnitude.

**Action.** As v0.13.0's first planning decision, commit to one of:
- **Per-phase push** — phase-close hook includes `git push origin main`; pre-push gate-passing is the close criterion. Tighter feedback loop; small overhead per phase.
- **Per-N-commits push** (e.g., every 10 atomic commits) — middle ground; needs a hook or operator discipline.
- **Session-end push + accept the gap** — current practice; cheapest but session-length feedback delay.

Document the decision in CLAUDE.md (per OP-9 the milestone-close ritual will record the outcome). Close 999.4 backlog row by referencing this decision.

**Why.** Deferring this question to "999.4 backlog" means it auto-rolls into v0.13.0 unchosen. Pick once, codify, move on.

---

## 4. Run `/gsd-review` on the v0.13.0 plan once drafted

Cross-AI peer review (the `/gsd-review` skill — peer review of phase plans from external AI CLIs) is cheap insurance for thesis-level milestones. It surfaces blind spots the local context didn't catch.

**Action.** After the v0.13.0 ROADMAP + first phase's PLAN.md are drafted but BEFORE execution starts, run `/gsd-review` on the bundle. Review feedback feeds back into the plan; the iteration cost is a few hours.

**Why.** Cross-AI review hasn't been used recently in this project. The v0.12.1 retrospective shows that internal verifier-subagent dispatch caught what it was designed to catch (catalog-row PASS, OP-8 honesty checks) but does not catch broader thesis-level "is this the right architecture?" feedback. External peer review is the right tool for that.

---

## Pre-kickoff checklist

Before `/gsd-execute-phase` runs on the first v0.13.0 phase:

- [ ] Open-questions resolved or explicitly deferred with named triggers (rec #1)
- [ ] POC in `research/v0.13.0-dvcs/poc/` shipped + reviewed (rec #2)
- [ ] Push cadence decided + codified in CLAUDE.md (rec #3)
- [ ] `/gsd-review` feedback integrated into the plan (rec #4)
- [ ] 3 WAIVED structure rows scheduled — verifier scripts land in v0.13.0 P0 or the waiver auto-renews past 2026-05-15 (defeats catalog-first principle; see `.planning/STATE.md` carry-forward note)
- [ ] ROADMAP.md progressive-disclosure if it crosses 80K (currently 60K; pre-commit hook warns above 20K)

## Out of scope for these recommendations

These are kickoff-readiness recommendations only. Specific phase decomposition, per-phase scope, and concrete algorithms are the planner's job — see `architecture-sketch.md` § "Phase decomposition (sketch — final shape decided by `/gsd-plan-phase`)" and § "What we're NOT building (and why)" for the boundaries the planner inherits.

---

## Lineage

- v0.12.1 RETROSPECTIVE (`.planning/RETROSPECTIVE.md`) — full retrospective on what the v0.12.1 close-out session learned
- OP-9 milestone-close ritual (`CLAUDE.md` § Operating Principles) — the rule this file's recommendations help enforce
- 999.4 push-cadence backlog (`.planning/ROADMAP.md` § Backlog) — the unresolved decision rec #3 closes
