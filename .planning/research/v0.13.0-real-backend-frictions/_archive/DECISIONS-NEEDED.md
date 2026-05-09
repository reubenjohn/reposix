# v0.13.1 — Decisions needed before P89-01

Sibling: `READY-TO-EXECUTE.md` is the cold-start entry point. This file is the owner's pre-P89 sign-off list.

## Decision 1: Test progress mid-stream, not just at the end

**Question:** After each fix-phase (P91 attach, P92 push/audit, P93 bus push) ships, should we re-run the relevant dark-factory test against the real backend BEFORE starting the next phase — and if it still has a HIGH-severity bug, REOPEN that phase rather than moving on?

**Recommendation:** Yes. Today the plan only re-runs the full dark-factory at P96 (the very end). If we wait until P96 to find out P91 didn't actually work, we're stuck — the milestone has to either reopen or ship broken (the same trap v0.13.0 fell into).

**Drill in:** `03-synthesis/COMPLETENESS-CHECK.md` § S2.

---

## Decision 2: Don't defer fixes that v0.13.1's vision needs

**Question:** The plan punts 11 items to v0.14.0. Two of them — "cache-coherence redesign" and "recovery from partial-failure pushes" — might be needed for the v0.13.0 round-trip to actually work end-to-end on real backends. Should we pull those two into v0.13.0 corrective scope (or block the v0.13.0 tag on them), instead of leaving them as v0.14.0 work?

**Recommendation:** Yes, pull them in or block the tag on them. The synthesis itself flags both as repeats of the "deferred-but-the-row-still-says-PASS" anti-pattern that v0.13.0 is being corrected for. Deferring them again ships the same failure shape one milestone later.

**Drill in:** `03-synthesis/COMPLETENESS-CHECK.md` § S3.

---

## Decision 3: Don't let P89/P90 declare the framework "fixed" by patches alone

**Question:** P89/P90 ship 8 small framework patches (new test kind, new cadence, new lint rules, etc). Should those 8 patches be enough to declare "framework redesign done" — OR should P89/P90 also be required to add a structural check that asks: "does each catalog row's verifier actually prove what the row's description claims?"

**Recommendation:** Require both — the 8 patches PLUS the structural check. The synthesis docs themselves disagree about whether 8 patches is enough. If you don't decide, the executor will pick the cheaper option (just the patches) and v0.13.1 ships a "redesign" that's actually still a patch.

**Drill in:** `SYNTHESIS-VERIFICATION.md` § "Cross-doc contradictions" item 1; `03-synthesis/COMPLETENESS-CHECK.md` § S1; `03-synthesis/STRATEGIC-REFRAME.md` Q3.

---

## Decision 4: What happens to the existing v0.13.0 GREEN verdict files

**Question:** v0.13.0 already wrote 12 verdict files (one per phase + one for the milestone) all marked GREEN on 2026-05-01. After P89–P96 fix the bugs those verdicts missed, which of these do we do?
- (a) Leave them GREEN as-is — historical record only.
- (b) Add a "this was extended on 2026-05-08, see P96 for final state" note to each.
- (c) Retract them — flip to RED, replace later.

And separately: when P96 closes the extended milestone, does its verdict overwrite `milestone-v0.13.0/VERDICT.md`, or write a new `milestone-v0.13.0-extended/VERDICT.md`?

**Recommendation:** (b) plus overwrite. Add a 2026-05-08 "extended-pending-P89–P96" note to each existing verdict; P96 overwrites `milestone-v0.13.0/VERDICT.md` when the extended milestone closes. Leaving them untouched (option a) is exactly the "old PASS row never re-graded" anti-pattern v0.13.0 is being corrected for.

**Drill in:** `03-synthesis/COMPLETENESS-CHECK.md:81-83` (M2); `COMPLETENESS-CHECK-2.md` § S-2.

---

Lower-priority items the orchestrator will surface during P89 planning, not requiring owner sign-off here: (a) the "v0.13.1" name vs. "v0.13.0 extension" cleanup throughout the synthesis docs; (b) the inventory headline says "51 HIGH" but only 43 are explicitly listed (the rest fold into clusters); (c) one timestamp drift (17m vs. cited 23m for commit `fd2e247`).
