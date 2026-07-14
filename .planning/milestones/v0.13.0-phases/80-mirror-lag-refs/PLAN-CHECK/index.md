# Phase 80 — Plan Check

> **Revision status (2026-05-01).** The planner revised
> `80-01-PLAN.md` in response to this check. **All 3 HIGH issues**
> (H1 sim-port routing, H2 gix 0.83 tag invocation, H3 vacuous
> first-push test) and **4 of 5 MEDIUM issues** (M1 dev-deps, M2
> cache.db path, M3 Q2.2 verbatim phrase carrier, M4 fixed verifier
> ports) are addressed. **M5** (verifier-shell line-count
> watch-list) is left as guidance for the executor (not a code
> change). **L1–L5** are intentionally not addressed (stylistic
> per § Recommendation). See `80-01-PLAN.md` lines 26–37 for the
> embedded revision note + per-issue summary; revised plan is
> 2,443 lines (+184 net delta).

---

**Reviewer:** plan-checker subagent (goal-backward verification, pre-execution)
**Date:** 2026-05-01
**Plans reviewed:** `80-PLAN-OVERVIEW.md` (365 lines), `80-01-PLAN.md` (2,259 lines)
**Reference:** `80-RESEARCH.md`, ROADMAP § Phase 80 (lines 83–101), REQUIREMENTS DVCS-MIRROR-REFS-01..03, decisions Q2.1/Q2.2/Q2.3, `crates/reposix-cache/src/sync_tag.rs`, `crates/reposix-remote/src/main.rs::handle_export`, `CLAUDE.md`.

---

## Verdict: YELLOW

**Summary.** The plan is structurally sound, comprehensively researched, and respects every load-bearing CLAUDE.md operating principle (catalog-first, OP-3 audit, per-phase push, per-crate cargo, threat model). It maps each ROADMAP success criterion to a concrete artifact, and the donor-pattern citation discipline (`sync_tag.rs`, `log_sync_tag_written`, `reposix-attach.sh`) is exemplary.

However, there are **THREE HIGH-severity issues** that will fail execution as written:

1. **`reposix init` does NOT honor `REPOSIX_SIM_ORIGIN`** — verifier shells and integration tests will dial port 7878 (default) instead of the per-test port the sim is bound to. The plan inherits the env-var-based override pattern from P79's `reposix attach` verifier without verifying that `init` supports it.
2. **`gix::Repository::tag(...)` argument order is wrong** in the plan's code sample — the plan passes `(name, target, PreviousValue::Any, Some(committer), &message, true)` but the gix 0.83 signature is `(name, target, target_kind: gix_object::Kind, tagger: Option<SignatureRef>, message, constraint: PreviousValue)`. The bool `true` and the misplaced `PreviousValue::Any` will not compile. RESEARCH.md A1 hedged this; the plan should NOT have shipped a concrete invocation without confirming the signature.
3. **`reject_hint_first_push_omits_synced_at_line` integration test asserts nothing meaningful** — the "weaker form" the plan ships drives a SUCCESSFUL push (which never enters the conflict-reject branch) and asserts no "minutes ago" string in stderr. The success branch never composes the synced-at hint anyway, so this assertion is vacuously true regardless of correctness. SC4 first-push None-case behavior is not behaviorally tested at the helper layer.

A handful of MEDIUM issues (catalog test path bug, missing dev-dep declarations, ambiguous Q2.2 verbatim treatment) are listed below.

These are tractable to fix without splitting the phase. Verdict is YELLOW — minor revisions recommended before T01 lands.

---

## Chapters

- **[Per-question findings](./findings.md)** — Goal-backward verification across 10 questions: phase-goal delivery, ROADMAP success criteria coverage, catalog-first invariant, OP-3 audit, cargo discipline, threat model, plan size/context budget, open questions Q1–Q6, push cadence, and dependency chain. Includes the SC coverage table and Q-deferral table.

- **[Issues by severity](./issues.md)** — All findings graded HIGH (H1–H3), MEDIUM (M1–M5), and LOW (L1–L5), with root-cause analysis, impact statements, and concrete fix recommendations for each.

---

## Recommendation

**Verdict: YELLOW. Minor revisions recommended before T01 lands.**

Three HIGH issues are blocking but each has a bounded fix:

- **H1 (sim-port routing)** — add `git config remote.origin.url` re-pointing in 3 verifier shells + 1 Rust helper. Affects ~6 lines of plan body.
- **H2 (gix tag API shape)** — replace lines 1132–1142 with the correct invocation OR commit to the two-step `write_object` + `tag_reference` path up front. Affects ~15 lines.
- **H3 (vacuous integration assertion)** — engineer a real first-push conflict via sim seeding, OR move the assertion to a unit test in `main.rs` with a stub cache. Affects ~30 lines.

Combined revision footprint: ~50 plan lines. Estimated one revision loop (≤ 30 min planner time) before execution.

Medium issues (M1–M5) can be fixed inline during execution as eager-resolution items per OP-8 (each is < 1 hour, no new dep introduced) — they don't block T01.

Low issues (L1–L5) are stylistic / observational — no action required.

If the planner declines to revise H1–H3 before execution, the executor will (a) hit H2 on first `cargo check -p reposix-cache` and fix-forward, (b) hit H1 on first verifier-shell run and fix-forward, (c) miss H3 entirely (the test passes but proves nothing) — surfaced only at verifier-subagent grading time, where it may be downgraded to PASS-with-caveat or graded YELLOW.

---

**End of PLAN-CHECK.md**
