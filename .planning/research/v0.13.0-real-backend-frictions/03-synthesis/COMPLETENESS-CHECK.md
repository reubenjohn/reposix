# v0.13.1 Adversarial Completeness Check

**Auditor:** adversarial-completeness synthesis subagent (zero session context)
**Date:** 2026-05-08
**Inputs:** `01-dark-factory-may02/SUMMARY.md`; `02-phase-audits-may08/{phase-audit-p78..p88,vision-audit,AUDIT-BRIEF}.md` (sampled deeply on P83/P84/P85/P86/P87/P88 + vision; structural sample on P78–P82); `03-synthesis/{PATTERNS,REMEDIATION-PLAN,STRATEGIC-REFRAME}.md`; `CLAUDE.md` Operating Principles + Quality Gates section.

**My job:** find what the three synthesis subagents missed. The bar is candor over politeness. Confidence-graded so the orchestrator can route.

---

**DISPOSITION (2026-05-08):** S1 closed (cross-AI peer review approved per `_archive/SESSION-2026-05-08-HANDOFF.md`); S2/S3 ratified per `_archive/DECISIONS-NEEDED.md` Decisions 1+2 (now ratified — see top-level `README.md`); patch-vs-redesign tension resolved per Decision 3 (both required). MEDIUM/WEAK gaps deferred to P89 planning per top-level `README.md`.

**Note (2026-05-08, post-renumber + post-renaming):** Phase numbers in the body below reflect the pre-Decision-2 plan (6 work + 2 reservation = P89–P96). Post-Decision-2, P93 became "L2/L3 cache-coherence + SotPartialFail recovery" and the prior P93–P96 renumbered to P94–P97 (7 work + 2 reservation). For the current canonical phase shape, see `REMEDIATION-PLAN.md` § "Proposed v0.13.0 extension phase shape". This document also references the work as "v0.13.1" throughout the body — that was the pre-Path-A-ratification name; the milestone identity is now **v0.13.0 extension** (Path A: hold the v0.13.0 tag, extend with corrective phases P89–P97). v0.13.1 mentions in the body should be read as "the v0.13.0 extension work."

---

## Summary verdict

PATTERNS, REMEDIATION-PLAN, and STRATEGIC-REFRAME are individually strong: PATTERNS' meta-pattern (vertical slices with no horizontal probe) is exactly right; REMEDIATION's framework-fix-first ordering is exactly right; STRATEGIC's Q3 ("redesign with patch as concrete first commits") is exactly right.

But three of their **shared assumptions** are blind spots, and the v0.13.1 plan as currently shaped has structural risk of repeating v0.13.0's failure mode in a new register. The most concerning gap: **the plan that redesigns the framework is graded BY THAT SAME FRAMEWORK** — and the proposal does not name a non-self-referential validator for the redesign itself.

I find:
- 3 STRONG gaps (I am highly confident the synthesis missed these and they are load-bearing)
- 5 MEDIUM gaps (candidate misses; would benefit from owner validation before P89-01 ships)
- 4 WEAK gaps (nitpicks worth flagging but not load-bearing)

---

## STRONG-confidence gaps (load-bearing misses)

### S1 — The framework-fix phases (P89/P90) are graded by the very framework they're redesigning. Chicken-and-egg unaddressed.

**Where it bites:** REMEDIATION-PLAN P89/P90 say "Catalog-first commit mints 5 rows in `quality/catalogs/{agent-ux,framework}.json` with `status: NOT-VERIFIED` BEFORE implementation commits land." But the verifier subagent that grades P89 GREEN at phase close uses **the catalog rows P89 just minted** to certify P89 succeeded — and those rows are themselves the new framework rules. There is no external arbiter. Worse, P89 ships `cadence: pre-release-real-backend` + new `kind: shell-subprocess`; nothing in P89's *own* catalog rows is graded under the new cadence/kind because the new cadence/kind doesn't have established acceptance criteria yet. The phase will close GREEN against a tautology: "the new gates pass because the rows that define the new gates say they pass."

**Why STRONG:** This is the exact failure shape PATTERNS C7 ("Self-licensing-deferral-loop") names — but it lands on the v0.13.1 plan's own foundational phase, not on a deferred phase. The strategic-reframe agent's Q6 said "GSD optimizes for vertical-slice correctness and lacks a horizontal-composition gate" — but P89/P90 ARE the horizontal-composition gate, and they're graded vertically. The redesigner cannot self-certify.

**What's missing from the plan:** A *non-framework* arbiter for P89/P90. Candidates:
1. An out-of-band adversarial dispatch (Claude/Codex/Cursor cross-AI review of P89/P90 plans + verdicts; gsd-review skill exists for this and is unmentioned in the plan).
2. A literal pre-commitment: the orchestrator writes a "what would it take to convince me P89/P90 worked?" doc BEFORE P89-01 starts; a fresh subagent grades the artifacts against that doc (not against the rows P89 minted).
3. Re-grade ALL pre-existing v0.13.0 catalog rows under the new framework as P89's acceptance test; if N% flip from PASS → RAISE, the framework demonstrably has new teeth. This is a behavioral gate, not a structural one.

**Severity:** without this, P89 closes GREEN by construction; v0.13.1 ships and the next dark-factory exercise repeats the failure mode in a new shape.

---

### S2 — v0.13.1 has NO explicit phase-level owner for the v0.13.1 litmus test itself. The vision-drift failure repeats.

**Where it bites:** STRATEGIC-REFRAME Q6 nailed the v0.13.0 failure ("the vision-level claim has no phase-level owner"). REMEDIATION-PLAN P96 success criterion #4 says: "Re-running the dark-factory exercise produces ≤ 5 frictions (down from 37) and ZERO HIGH (down from 16)." But which phase OWNS running the dark-factory exercise mid-stream as a falsifier, not just at the end?

P89–P95 each ship internal plumbing. P96 is the milestone-close ritual. The dark-factory regression run that would falsify the milestone is named ONLY as a P96 success criterion — at the end of the chain, after 6 work phases of effort. If the dark-factory at P96 surfaces 8 HIGH frictions instead of 0, the entire v0.13.1 milestone has to either re-open (which v0.13.1 was supposed to fix the v0.13.0 anti-pattern of) or ship with known issues (which is the Path A vs. Path B failure all over again).

**Why STRONG:** v0.13.1's litmus test IS "rerun the 4-subagent dark-factory exercise post-fixes." That test should fire after P92 (push-flow correctness) at minimum, and ideally as a repeated checkpoint after P91, P92, P93. The plan does not schedule it. P96 is the ONLY firing.

**What's missing:** A "vision-litmus-test mid-checkpoint" between phases. Concretely:
- After P91 GREEN: run T2 (attach against real Confluence) — must pass before P92 starts.
- After P92 GREEN: run T1 + T4 (sim end-to-end + rebase recovery) — must pass before P93 starts.
- After P93 GREEN: run T3 (bus push against real Confluence + GH mirror) — must pass before P94 starts.
- After P94 GREEN: full 4-subagent dark-factory run; if ≥1 HIGH friction, P94 reopens or P95 absorbs.

This is the *executable* form of the strategic-reframe agent's "vision-litmus-test must become a runtime artifact." Without checkpoint firings, the litmus test is still a planning input, just one phase later.

**Severity:** P89–P94 = ~25 days of work. If P96 finds the round-trip is still broken (e.g., S1's tautology produced a framework that doesn't catch real failures), the milestone's trajectory is unrecoverable in a single absorption phase.

---

### S3 — The 11 deferred-to-v0.14.0 items contain at least 2 that are load-bearing for v0.13.1's vision; deferring them is a v0.13.0-style scope cut wearing v0.13.1 clothes.

The remediation plan's § 6 lists 11 deferrals. Two of them undermine v0.13.1's own thesis:

**(a) "L2/L3 cache-coherence redesign" — DEFERRED.** RBF-D-12 ships "honest-claim-with-asterisk" (test exercises a non-no-op push OR docs honestly state the asterisk). But P92's RBF-B-02 says `helper_push_*` rows MUST land for OP-3 compliance on every push. If the cache-coherence story has unaddressed L2/L3 desync today (`p81 F5` + dark-factory CLUSTER C cause hypothesis), then the audit-row contract P92 ships could itself be unstable on real backends. Deferring L2/L3 means RBF-B-02's "MUST land" is an aspiration, not a guarantee, on the very surfaces v0.13.1 promises to fix.

**(b) "`SotPartialFail` + recovery-via-fetch-replan-push test" — DEFERRED.** This is the recovery shape after a bus write hits SoT-success + mirror-fail. v0.13.1's vision is "the round-trip works on real backends." The round-trip includes recovery from partial failure. Deferring this test to v0.14.0 means v0.13.1 ships a `git push reposix main` that works on the happy path against TokenWorld, and the failure path is ungated. The first `429 Too Many Requests` from real Confluence during a P92+P93 acceptance run will surface this.

**Why STRONG:** PATTERNS C8 ("Substrate-gap-deferred-but-row-passes-vacuously") is the exact taxonomic shape these two deferrals have. The synthesis cataloged the failure shape but proposes deferrals that instantiate it. C8's lesson should be applied to its own remediation: any v0.13.1 deferred item whose absence weakens v0.13.1's vision must be `WAIVED + until_date`, not merely "scheduled for v0.14.0" — with the implication that the "until" date is a hard release-blocker for v0.13.1's milestone-close VERDICT, not a roadmap entry.

**What's missing:** Each of the 11 deferrals needs a **vision-coverage delta** field: "if this is deferred, which v0.13.1 vision claim weakens?" Items where the delta is "none — purely v0.14.0 polish" stay deferred. Items where the delta is "v0.13.1 round-trip becomes happy-path-only" need either (a) promotion into v0.13.1 scope, or (b) explicit "v0.13.1 ships with happy-path-only round-trip; full recovery is v0.14.0" qualifier in CHANGELOG and CLAUDE.md.

---

## MEDIUM-confidence gaps (candidate misses worth owner validation)

### M1 — Cross-AI peer review (`gsd-review`) is unmentioned anywhere in the v0.13.1 plan.

The skill exists. It's literally designed for "we caught a quality miss; let's bring an external arbiter into the planning loop." S1's chicken-and-egg risk has a tool-shaped answer the synthesis didn't reach for. STRATEGIC-REFRAME Q6 names "subagent verifier dispatch (correct mechanism)" as one thing that survives, but doesn't escalate to cross-AI when self-grading is the suspect property. **Recommendation:** P89's plan-check should be a `gsd-review` call to Codex/Cursor/Gemini, not just a same-CLI subagent. **Confidence:** MEDIUM — depends on whether `gsd-review` is operationally available in this dev env; if so, this is a near-free risk reduction.

### M2 — The retroactive v0.13.0 verdict files question is unaddressed.

Strategic-reframe Q1 recommends "extend v0.13.0" (Option B), but there are 11 phase-verdict files at `quality/reports/verdicts/p7{8,9},p8{0..8}/VERDICT.md` already graded GREEN. If we extend, do we (a) leave them as historical artifacts (graded against the old framework), (b) re-grade them under the new P89/P90 framework rules and append amendments, or (c) issue retraction notices on the most-divergent ones (P79, P83, P86)? The plan doesn't say. **Why MEDIUM:** the plan implicitly assumes "no retroactive re-grade needed" (P95 RBF-S-02 is "retroactive intake entries," which is intake, not verdict re-grade). But the verdict files ARE the supply-chain trust artifact future planners read. Leaving them GREEN-without-asterisk is the same fossilization (PATTERNS C9) that v0.13.1 is trying to fix.

### M3 — Non-engineering stakeholders who saw the GREEN milestone notice are not addressed.

If v0.13.0 was graded GREEN on 2026-05-01 and announced (CHANGELOG draft, RETROSPECTIVE.md, possibly upstream notifications) before the dark-factory ran on 2026-05-02, *who outside the engineering team learned that v0.13.0 shipped*? The synthesis treats v0.13.1 as a pure-engineering concern. But:
- A pre-tag CHANGELOG draft might already be circulating in PR descriptions, blog drafts, social media.
- Downstream consumers of `crates.io` may have seen the per-package release-plz publishes.
- The `release-plz` per-package release problem CLAUDE.md flags ("zero assets, stole `releases/latest` pointer") may have shipped per-package tags for v0.13.0 even though the main `v0.13.0` tag is unpushed.

**Why MEDIUM:** I cannot verify any of this from the bundle alone. If true, "hold the tag" (Q1 Option B) is a partial-rollback scenario, not a clean hold. The plan should include a comms step: who has been told v0.13.0 shipped? What do they need told? **Recommendation:** P89-00 (pre-phase) checks crates.io publish status, GH releases page, recent issue comments, blog drafts; produces a "stakeholder rollback audit" before any code work starts.

### M4 — "Main agent never executes" is invoked but not enforced as a v0.13.1 rule.

CLAUDE.md says orchestration-shaped phases run at top-level (P89/P90/P95/P96 marked "top-level" in the remediation plan). But the plan doesn't say what happens if the orchestrator (top-level Claude session) writes code in those phases. The v0.13.0 P87 finding (F2) — that the orchestrator authored the honesty spot-check — is a process gap the synthesis correctly identified, but the v0.13.1 fix is just "spot-check author ≠ orchestrator." That's a property assertion, not a process enforcement. **What's missing:** a literal `quality/gates/structure/orchestrator-as-author.sh` that grep's commit author + commit subject lines for top-level phases and flags "did the same shell session both orchestrate and author?" This is the dishonest-test triage (F-K8) extended to phase orchestration. **Why MEDIUM:** this is a meta-rule about a meta-rule; risk is real but indirect.

### M5 — PATTERNS taxonomy may merge two distinct failure shapes into C2.

**The candidate split:** C2 ("Test-name-promises-more-than-assertion-delivers") covers two distinct shapes:
- **C2a — Vocabulary mismatch.** The test name describes a behavior, the assertion checks a structural property of the same surface. Example: `dark_factory_real_confluence` named for real-backend coverage, asserts URL shape (vision F7).
- **C2b — Layer-of-coverage substitution.** The test claims a behavior at one layer, satisfies it at another. Example: P83 "dual-table audit completeness" via wiremock + `audit_events_cache` count (PATTERNS classes this under C2 but it's structurally C7's self-licensing-deferral-loop applied at the row level — the assertion is honest about what it tests at *its* layer, the catalog description is dishonest about whether that layer is the right one).

These have different remediation surfaces. C2a is fixed by F-K8 dishonest-test triage (test name vs. body). C2b is fixed by F-K4 catalog-row honesty (description claim vs. assertion at the right layer). **Why MEDIUM:** the remediation plan happens to address both via different framework rules, so the merge isn't load-bearing — but a finer-grained taxonomy would let future verifier subagents grade rows more precisely. This is a polish, not a load-bearing miss.

---

## WEAK-confidence gaps (nitpicks)

### W1 — No mention of supply-chain trust risk in the catalog files.

Catalog rows are JSON-on-disk in `quality/catalogs/*.json`. They're not signed. They're hand-edited (per `_provenance_note` waiver fields, `phase-audit-p83 F8`). A future agent (or human, or compromised tool) could silently flip a row from FAIL → PASS with no audit trail beyond `git blame`. **Why WEAK:** v0.13.1 is not the venue for cryptographic-signing infrastructure. But the issue exists and PATTERNS C9 (Catalog-state-fossilized) is its near neighbor. Worth a one-line "deferred to v0.14.0+" entry in the deferral table.

### W2 — `pre-pr` cadence is wired into rows but not into any CI workflow.

`p80 F10` + `p84 F4` flagged this (rows tagged `cadences: ["pre-pr"]` are never executed; `pre-pr` cadence has no CI workflow invocation). The remediation plan's F-K1 adds a NEW cadence (`pre-release-real-backend`) without addressing the existing dead `pre-pr` cadence. RBF-D-06 (catalog migration) might catch it implicitly. **Why WEAK:** it's a low-impact cleanup. But adding a cadence while ignoring a documented-as-dead one is the kind of debris that compounds.

### W3 — REQ-ID prefix `RBF-` may collide with existing `RBF`-shaped tokens.

Minor naming hygiene. Could be `RBF-13.1-` or just `v131-` to disambiguate. **Why WEAK:** purely cosmetic.

### W4 — No "what if the framework redesign discovers ITSELF needs redesign?" branch.

The plan presumes P89's framework redesign converges on first attempt. If P89 writes shell-subprocess kind + real-backend cadence and P91's first integration test reveals "the new kind doesn't actually express what we needed," there's no P89.5 escape hatch. **Why WEAK:** speculative; the +2 reservation (P95) is supposed to absorb this. But naming it explicitly would prevent the eager-resolution-as-scope-cut failure mode (PATTERNS C6) at the framework level.

---

## Predictions: what the next dark-factory exercise will find after v0.13.1 ships

I asked myself: if P89–P96 ship as written, and the orchestrator runs the same 4-subagent dark-factory exercise the day v0.13.1 tags, what comes back? Five predictions, ordered by confidence:

**P1 — STRONG: A test name in the new framework will promise more than its assertion delivers.** Specifically: the v0.13.1 plan adds `kind: shell-subprocess` and writes verifiers for it. The first `shell-subprocess` verifier will assert something narrower than its name suggests — because the FIRST instance of a new artifact type tends to be the cleanest minimal example, and minimal examples never cover the messy reality. PATTERNS C2 in a new dimension.

**P2 — STRONG: The "milestone-close litmus test against TokenWorld" will pass on the day of P96 and break within 30 days.** Reason: catalog row `freshness_ttl` for the new probe will be set defensively long (90 days?), and TokenWorld's underlying state will drift (someone edits a page, an API token gets revoked, Confluence ships a non-breaking-but-edge-case API change). The probe runs, finds drift, but the freshness window says it doesn't need to re-run for 90 days. PATTERNS C9 (catalog-state-fossilized) in a new register.

**P3 — MEDIUM: Bus-push against a real GH mirror will succeed for `pages/*.md` and silently corrupt for paths with spaces or non-ASCII characters.** Reason: P93's RBF-C-01 ADR will likely choose path-prefix scope (`?records_root=pages/`) as the cleanest mental model; the implementation will not handle path encoding edge cases on the first pass. The dark-factory subagent's TokenWorld page titled "User Auth & Login" produces a path with `&` in it.

**P4 — MEDIUM: The `--set-upstream` UX nit (CLUSTER E F5) will be fixed for the documented happy-path command but break in the `git push -u reposix main` shape.** Reason: bare `git push` was the documented broken case; the fix lands; nobody remembers to test the explicit `-u` form, which goes through a different code path in the helper.

**P5 — MEDIUM: The `audit_events` SoT-side write will succeed on Confluence but fail silently on JIRA.** Reason: P92's RBF-B-03 wires `.with_audit(audit_conn)` for all three adapters; the JIRA adapter has a code path the other two don't (issue transitions). The first dark-factory T-JIRA run flags it.

If the v0.13.1 plan addresses any of these proactively in P91/P92/P93/P94, score reduces. As written, it doesn't.

---

## What the synthesis got right (worth preserving in the plan)

For balance:

- **PATTERNS C7's framing** ("self-licensing deferral loop") is the most precise diagnosis in the bundle. Any v0.13.1 phase that says "P<other> covers it" should be flagged as a C7 risk and require an external arbiter, not a phase-chain citation.
- **REMEDIATION-PLAN's framework-fix-first ordering** (P89/P90 before P91+) is correct and load-bearing. Reversing it would have the code/doc fixes ship into a still-broken framework.
- **STRATEGIC-REFRAME Q3's "redesign-class with patch-class as concrete first commits"** is exactly the right shape — neither pure patch nor pure redesign would have worked.
- **STRATEGIC-REFRAME Q6's "vision-litmus-test must become a runtime artifact"** is the meta-fix. S2 above just says it has to fire MORE THAN ONCE.

---

## Recommended sharpening of the v0.13.1 plan

Three concrete additions before P89-01 ships:

1. **P89-00 pre-phase: "Framework redesign external validation."** Output: a written commitment ("what would convince me the framework redesign worked?") + a `gsd-review` call to a non-Claude AI to audit the P89/P90 plan-overviews. (Addresses S1.)

2. **Inter-phase litmus checkpoints.** Add to ROADMAP between P91-P94: "After this phase GREEN, run dark-factory T<N> against real backend. If ≥1 HIGH friction, this phase REOPENS, not the next phase." (Addresses S2.)

3. **Vision-coverage-delta annotation on every deferred item.** Re-classify the 11 deferrals: those whose deferral weakens a v0.13.1 vision claim get `WAIVED + until_date = v0.13.1 release date`, blocking milestone close until promotion or scope qualifier lands. (Addresses S3.)

Doing these makes v0.13.1 honest about the same standard it's holding v0.13.0 to. Skipping them ships the redesign with the redesign's own anti-patterns embedded.

---

**End of completeness check.**
