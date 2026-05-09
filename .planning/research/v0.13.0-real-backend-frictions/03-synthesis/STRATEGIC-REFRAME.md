# v0.13.1 — Strategic reframe

**Author:** strategic-reframe synthesis subagent
**Date:** 2026-05-08
**Inputs:** dark-factory SUMMARY.md (37 frictions / 16 HIGH), 12 phase audits + vision-audit (~160 findings / ~48 HIGH), milestone-v0.13.0 verdict (GREEN), CHANGELOG.md v0.13.0 entry, RETROSPECTIVE.md v0.13.0 section, v0.13.0 + v0.14.0 vision-and-mental-model docs, CLAUDE.md OP-1/3/6/7/8/9 + Quality Gates section.

**Read order for this file:** Q1 → Q2 → Q3 → Q4 → Q5 (CTO brief) → Q6 (meta-finding). Each section ends with a one-sentence answer suitable for the orchestrator TLDR.

---

## Q1 — Tag, retract, or extend v0.13.0?

### The three options and their costs

**Option A — push the v0.13.0 tag as-is and patch in v0.13.1.**
- Pros: maintains release cadence; CHANGELOG entry already drafted; tag-script already authored; downstream binstall metadata flows.
- Cons: every public-facing claim in CHANGELOG is **demonstrably false** for any real backend (CHANGELOG.md:11 — "Devs `git clone` ... install reposix only when they want to write back; `reposix attach` reconciles their existing checkout against the SoT, then `git push` via a bus remote fans out atomically." This claim does not hold on Confluence, GitHub, or JIRA — `attach` exits with the P79-02-scaffold error). Any user who reads the CHANGELOG and tries the documented flow will be misled. Downstream cost: blog posts, social media, and "what shipped in v0.13.0?" surfaces propagate the false claim before v0.13.1 ships.
- v0.13.1 must then ship as a **breaking-claim correction**, not a polish release. CHANGELOG.md:0.13.1 has to say "the headline UX in 0.13.0 didn't work; here's the version where it does." This is a credibility tax the project will pay for at least one release cycle.

**Option B — hold the tag, extend v0.13.0 with corrective phases (P89–P92 + +2 reservation), then tag.**
- Pros: the tag, when cut, ships a milestone whose vision is demonstrably true. CHANGELOG entry remains accurate. The +2 reservation slots already absorbed P87/P88 — extending with 4 more functional phases requires owner sign-off but is the OP-8 spirit (the milestone reserves last 2 phases as absorption slots; we're saying the two slots are insufficient because the dark-factory exercise found load-bearing scope was missed by the original phase chain).
- Cons: violates the "milestone scope is fixed at planning time" convention. Sets precedent that a graded-GREEN milestone can re-open. Conflates "v0.13.0 vision works" with "v0.13.0 ships in May 2026" — if the four corrective phases take 2 weeks each, v0.13.0 slips into July. The retrospective + tag-script + CHANGELOG entry all need rewriting (cost: ~2–3 hours of close-ritual work, repeated).

**Option C — retract the GREEN verdict, redo the milestone close honestly, then tag.**
- Pros: maximally honest with the framework's own rules. The milestone-v0.13.0/VERDICT.md says "Tag-ready" based on 8 probes that **did not check the vision**. Retracting it means the verifier subagent system gets a real case study in "GREEN that wasn't"; the next milestone close-ritual can pre-bind a real-backend probe to its checklist.
- Cons: process trauma. Retracting a verdict is precedent-setting for the project. The verifier subagent did its job (graded artifacts honestly); retracting attacks the tooling rather than the gap (the gap is that the verifier had no probe wired to the vision-litmus-test, not that the verifier graded the wired probes badly).

### Recommendation: **Option B — hold the tag, extend v0.13.0.**

**Reasoning chain:**

1. **The CHANGELOG is the load-bearing public surface.** CHANGELOG.md is what binstall users, blog readers, and `gh release view v0.13.0` consumers see. It currently makes claims (CHANGELOG.md:11, :17 — `reposix attach` reconciles existing checkouts; bus remote fans out atomically) that are factually wrong for the only backends that matter (real Confluence/GitHub/JIRA — sim does not exist on `crates.io` or in any user's reality). Shipping the tag with this CHANGELOG is a one-way door: the credibility tax compounds over time, and v0.13.1 cannot fully unwind it because the v0.13.0 release page and the `[v0.13.0]` CHANGELOG entry remain the canonical narrative for that version.

2. **Option A's cost is asymmetric in the wrong direction.** Tagging now and patching in v0.13.1 saves ~2 weeks of process work; it costs the project the public claim that the dark-factory third arm works on real backends, and it costs the CHANGELOG its truth value for one full release cycle. The asymmetry: a 2-week delay is recoverable; a falsified CHANGELOG entry is permanent.

3. **Option C is right in spirit but too expensive in process.** The verifier subagent did its job — it graded the catalog rows that existed against the artifacts that existed. The gap is upstream of the verifier (no probe was wired to the vision-litmus-test). Retracting the verdict signals "the framework failed"; the more honest signal is "the milestone scope didn't include a real-backend litmus probe, and the next milestone close adds one." Option B operationalizes that lesson without the trauma of retraction.

4. **The +2 reservation precedent is exactly what justifies Option B.** OP-8 explicitly carves out the last two phases of every milestone as absorption slots for "what reality surfaces during planned-phase execution." The dark-factory exercise IS reality surfacing — 16 HIGH frictions a planned phase missed. The OP-8 spirit says: this is precisely what +2 is for. The letter of OP-8 says +2 is fixed at planning; the spirit says +2 expands when reality says it must. Recommend the spirit; document the case in RETROSPECTIVE.md so the next milestone's planner knows the +2 slot count is a floor, not a ceiling.

5. **Retract-then-extend is achievable in commits.** P88 produced a tag-script (`tag-v0.13.0.sh`) but the tag has not been pushed (CHANGELOG.md:7 says "Release status: PENDING owner tag-cut"). The owner has not yet run the tag script. This means **no v0.13.0 tag exists publicly** — the milestone is in a "ready-to-tag" state but not tagged. Extending the milestone with P89–P94 (4 functional + 2 reservation) before the owner runs `tag-v0.13.0.sh` is the lowest-friction path: no retraction needed, no public release amended.

### TLDR sentence

**Hold the tag; extend v0.13.0 with 4 functional + 2 reservation corrective phases (P89–P94); cut v0.13.0 honestly when the dark-factory regression passes against real Confluence end-to-end.**

---

## Q2 — v0.13.1 as a new milestone, or extend v0.13.0?

If Option B from Q1 holds (extend v0.13.0), then v0.13.1 as a separate milestone is **not needed** — the corrective work absorbs into v0.13.0 itself, and v0.13.1 becomes a future patch release for whatever surfaces post-tag.

But the user pre-approved (per dark-factory SUMMARY.md § "Recommended v0.13.1 phase shape") the framing where these are P89–P94 in v0.13.0's phase numbering. The phase numbers don't conflict with milestone identity; what matters is whether the milestone tag is `v0.13.0` (extended) or `v0.13.1` (separate).

### Argument FOR extending v0.13.0 (consistent with Q1 Option B)

- The vision-and-mental-model.md `litmus test` is the v0.13.0 acceptance criterion. If the litmus test doesn't pass, v0.13.0 didn't ship the vision. There is no neutral interpretation of "milestone done" that lets the litmus test fail.
- CHANGELOG.md:11 + RETROSPECTIVE.md:7 already promise the litmus test. Splitting into v0.13.1 means CHANGELOG.md v0.13.0 ships with claims that were not verified, then v0.13.1 quietly fixes them. The "what shipped in v0.13.0?" surface is permanently misleading.
- The +2 reservation IS the mechanism for absorbing in-milestone surprises. The dark-factory exercise is exactly the kind of "what reality surfaces" OP-8 anticipates. Treating the dark-factory findings as v0.13.1 work means OP-8 is admitting it can't absorb at-scale surprises — which weakens the reservation as a tool.

### Argument FOR new v0.13.1 milestone (the inverse case I'm asked to argue against my own answer)

- **Velocity-vs-correctness tradeoff is real.** Extending v0.13.0 with 4–6 more phases means the milestone runs ~3 weeks longer. If real-backend coverage is a complex problem (e.g., the cache.db OP-3 silence is actually a deeper architectural bug, not a wrong-cwd fix), the corrective phases can balloon. v0.13.1 as a separate milestone caps the scope: ship v0.13.0 with explicit known-issues, then v0.13.1 with the fixes, with each milestone's tag pointing to a self-consistent state.
- **Per-milestone retrospectives produce cleaner cross-milestone learnings.** OP-9's RETROSPECTIVE.md distillation is per-milestone. A v0.13.0 retrospective that lists "16 HIGH frictions missed at close" is structurally important — it captures the failure mode in the place future planners read. Folding the dark-factory findings INTO v0.13.0's retrospective dilutes the signal: the retrospective then has to say "we missed these and then we fixed them in P89–P94," which is a self-resolving narrative. A v0.13.1 milestone with its own retrospective produces a sharper "v0.13.0 missed; v0.13.1 caught" story across two retrospective sections — the archaeological record is clearer.
- **The "milestone closes when graded GREEN" convention is itself an invariant.** Re-opening a graded milestone is the precedent that erodes confidence in the GREEN verdict overall. If a future milestone closes GREEN and someone discovers a load-bearing gap 24h later, the project wants the same response (re-open vs. patch-release). Saying "v0.13.0 stays closed; v0.13.1 absorbs the work" preserves the rule that GREEN is final.
- **Public surfaces want a release with the correction visible in its name.** If a user installs v0.13.0 and hits the `attach` real-backend bug, the fix being labeled `v0.13.1` is more discoverable than "we extended v0.13.0 silently before tagging." A binstall-equipped user types `cargo binstall reposix-cli@0.13.1` — that string is more honest about the correction than a never-published v0.13.0 that quietly grew 4 more phases.

### Synthesizing

The strongest counter-argument is the third one (preserving GREEN-is-final as an invariant). It says: even if v0.13.0 fails the vision-litmus-test, the framework needs to honor its own grade or the grade means nothing. Re-opening creates a perverse incentive structure where future milestones can be re-opened on owner whim.

**My answer to the inversion:** the GREEN-is-final rule is sound at steady state, but this is the FIRST observed instance of the failure mode (a verifier-honestly-graded milestone whose vision-litmus is broken). The right framework move is to (a) extend v0.13.0 once, with explicit RETROSPECTIVE.md acknowledgment that this is a one-time exception, and (b) ratify a milestone-close real-backend probe as the structural fix that prevents needing to re-open future milestones. If the framework absorbs the lesson, the GREEN-is-final invariant strengthens (next time, the litmus probe blocks GREEN). If the framework treats this as v0.13.1, the lesson is partially lost (the v0.13.0 close-ritual is recorded as having shipped GREEN despite the failure, which is the audit trail future planners read).

### Recommendation

**Extend v0.13.0 (Option B from Q1).** The strongest counter-argument has merit but is outweighed by the public-surface honesty argument: CHANGELOG.md and RETROSPECTIVE.md already promise the litmus test; making them true is cheaper than making them false-then-fixed.

### TLDR sentence

**Extend v0.13.0 with P89–P94, treating the dark-factory exercise as a one-time precedent that justifies expanding the +2 reservation; v0.13.1 becomes the next patch release for whatever surfaces after the corrected v0.13.0 tag.**

---

## Q3 — Quality framework: patch or redesign?

### What the dark-factory + audit findings say about the framework

The framework graded 11 phases GREEN, then a milestone GREEN, while:
- **OP-3 was violated on every push** (no cache audit row written; helper opens cache.db with wrong cwd) — and three phase verdicts (P79, P83, milestone-close) cited dual-table audit completeness as PASS via assertions made under `assert_cmd`-controlled cargo tests where the cache path was explicit.
- **OP-1 was violated** ("real-backend tests gate milestone close" — the milestone-close VERDICT.md ran 8 probes; zero touched a real backend).
- **The headline subcommand (`reposix attach`) was sim-only** — its production error string leaked GSD phase IDs to end users, a canary that no human read the stderr from a user perspective during P78–P88.
- **The architectural cornerstone of v0.9.0 (`git pull --rebase` recovery)** was broken on the simulator and went undetected.
- **The bus push** could not succeed against any mirror that followed the documented setup.
- **The cold-reader rubric** (DVCS-DOCS-04) that was supposed to catch this was deferred to "owner runs it post-phase" and was still NOT_VERIFIED at milestone GREEN.
- **The verifier honesty spot-check (P87)** sampled 5 phases but excluded the two phases (P79, P86) where the largest scope cuts occurred; was authored by the same coordinator that ran the milestone (OP-7 violation for the meta-grading).
- **The milestone-close verdict (P88)** silently dropped SC5's "TokenWorld arm GREEN" clause without a deferral note.

### Patch-class vs. redesign-class — the question

**Patch-class**: add `kind: shell-subprocess`, add `cadence: pre-release` enforcement on the milestone-close real-backend probe, add an adversarial deferral verifier (catalog row WAIVED-with-until-date instead of vacuous PASS), add a content-binding hash for honesty-spot-check.md so its substance is grading-resistant.

**Redesign-class**: the dimension/cadence/kind taxonomy itself misses the load-bearing axis — "is the verifier's assertion congruent with the catalog row's claim?" In failure-mode language: the framework grades **assertion-passes-against-artifact**, not **assertion-actually-tests-the-claim**. F1 of P88 (4 catalog rows are 100% file-presence/structural) and F1/F2/F3/F4 of P86 (ROADMAP SCs not exercised) and F8 of P85 (docs phase verifier checks docs-build, not docs-vs-implementation truth) all share the same shape: the verifier is HONEST about what it asserts, and the description claims something larger.

### Argument: this is BOTH, but the redesign axis is the load-bearing one

The patch-class fixes are necessary (the missing `kind: shell-subprocess`, the WAIVED-vs-vacuous-PASS distinction, the content-binding for honesty-spot-checks). All three are tractable and would close some of the failure shapes.

But every patch-class fix sits inside the current framework, which optimizes for "verifier ran, exit code 0, row PASS." That optimization target is the structural defect.

The load-bearing missing axis is **claim-vs-assertion congruence**:

> For every catalog row, the verifier's assertion must demonstrably falsify the row's *description* claim if the description claim is false.

The current framework allows:
- A row described "GOOD-TO-HAVES drained" whose assertion is "≥1 STATUS terminal token grep-able" (P88 F1).
- A row described "End-to-end push success on confluence + GH mirror" whose assertion is "shell stub agent did `reposix attach`; URL has the right shape" (P86 F1).
- A row described "OP-3 dual-table audit forensic completeness" whose assertion is "audit_events_cache count ≥ N inside an assert_cmd-controlled cargo test" (P83 F5; vision-audit F3).

The catalog README (`quality/catalogs/README.md`) does not mandate this congruence test. The runner does not check it. The verifier subagent grades artifacts, not the description-vs-assertion delta. The honesty spot-check (P87) grades phase-vs-process, not row-vs-claim.

### What a redesign looks like (sketch)

Two ratifying changes:

1. **Catalog-row contract upgrade — add `claim_vs_assertion_audit` to every row**. A short paragraph the row's author writes explaining: "The description says X. The assertion tests Y. The bridge from Y to X is Z." If Z is "by construction" (e.g., the description is a tautology of the assertion), say so. If Z is "delegated to row R-2 in dimension D" (layered coverage), name R-2 and verify R-2 actually covers the delta. If Z is "best-effort substrate gap, see SURPRISES-INTAKE entry K," name K. If Z is empty, the row is misclassified.

2. **Milestone-close adversarial pass — a fresh subagent reads catalog descriptions only (no implementation context) and asks "what would I need to assert to falsify this?"** The subagent grades each row's `claim_vs_assertion_audit` against its own first-principles answer. If the audit's assertion would not falsify the description, the row is RED. This is the layer P87's honesty-spot-check should have been.

These two changes together would have caught all 12 misalignment items in the vision-audit. The patch-class fixes are subsumable inside this redesign as concrete `kind`s and `cadence`s.

### Recommendation: **redesign-class problem; patch-class fixes are necessary but not sufficient**

The dimension/cadence/kind taxonomy is correct and useful — keep it. The redesign is to add a fourth axis (claim-vs-assertion congruence) and operationalize it through a milestone-close adversarial pass. The patch-class fixes (shell-subprocess kind, WAIVED-with-until-date enforcement, content-binding for honesty-spot-check) implement the new axis at specific failure-shape points.

This is consistent with the user's framing in the question: "is this patch-class or redesign-class?" — the load-bearing answer is redesign, with patch-class as the concrete first step.

### TLDR sentence

**Redesign-class — add a "claim-vs-assertion congruence" axis to every catalog row and a milestone-close adversarial pass that grades it; the patch-class fixes (shell-subprocess kind, WAIVED-with-until-date, content-binding hashes) are the concrete first commits that operationalize the new axis.**

---

## Q4 — v0.14.0 readiness

### What v0.14.0 presumes

`.planning/research/v0.14.0-observability-and-multi-repo/vision-and-mental-model/index.md` opens with: "v0.13.0 ships a thesis-level shift (DVCS over REST); v0.14.0 makes the resulting system observable, multi-tenant, and self-correcting at scale." The dependency chain is explicit:

- v0.14.0 Phase 54+ adds OTel spans on **the helper hot paths**. The helper hot paths include push (OP-3 is dark today) and the bus URL fan-out (broken against documented mirror setup today).
- v0.14.0 origin-of-truth scope (Phases ~58–59) extends "the per-record version-check that origin-of-truth extends" — the version-check is in `handle_export`. v0.13.0 P83 ships fan-out coverage at `bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok`, which is wiremock-only and never exercises a real backend (P83 F1).
- v0.14.0 L2/L3 cache-desync hardening (Phases ~60–62) presumes "we have OTel telemetry on actual desync incidence." But desync incidence today is invisible because every helper push silently fails to write a cache audit row (vision-audit F3). Without that row, desync is unmeasurable; without measurement, the L2-vs-L3 decision rule has no input.

### The dependency chain in one sentence

**v0.14.0's "OTel on hot paths" + "origin-of-truth extends version-check" + "L2/L3 desync telemetry" all presume v0.13.0's helper-push path actually works on a real backend. It doesn't.**

### What re-scoping looks like

Three options:

1. **Status quo: v0.14.0 starts after v0.13.0 ships extended (P89–P94)**. v0.14.0 inherits a working v0.13.0 vision and the v0.14.0 plan executes as written. Cost: v0.14.0 starts ~2–4 weeks later. Benefit: every v0.14.0 phase has a working substrate to build on.

2. **Re-sequence: v0.14.0 absorbs the "v0.13.0 corrective phases" wholesale** (i.e., v0.13.0 ships as-is, v0.14.0 opens with the dark-factory remediation). Cost: v0.14.0's roadmap becomes "fix v0.13.0 first, then OTel"; the milestone identity blurs. Benefit: faster v0.13.0 tag.

3. **Re-scope v0.14.0: drop L2/L3 hardening (which depends on telemetry from a working push path) and ship only OTel + reposix tail in v0.14.0**. Cost: v0.14.0 becomes a smaller milestone. Benefit: v0.14.0 doesn't depend on v0.13.0 corrections.

### Recommendation: **Option 1 (status quo, gate on extended v0.13.0)**

Reasoning:

- v0.14.0's Phase 54 (OTel on helper push) is **only useful if the helper push works**. Tracing a broken hot path produces broken traces. The v0.14.0 vision document literally says "we now have multiple writers, multiple backends, and a cache that's load-bearing in places we couldn't see before" — this presumes the cache IS load-bearing today. If OP-3 is dark, the cache is silent today, not load-bearing. Phase 54 against a silent cache produces no signal.
- v0.14.0's L2/L3 decision rule (Phase ~60–62) is data-driven. The data comes from desync incidence telemetry. Today's incidence is unmeasured because OP-3 is dark. Shipping L2/L3 in v0.14.0 without first fixing the OP-3 darkness is shooting blind.
- Option 2 (absorption) blurs milestone identity in a way the project has so far avoided. Each milestone has a thesis (v0.13.0 = DVCS shift; v0.14.0 = operational maturity). Absorbing v0.13.0's correction into v0.14.0 means v0.14.0's thesis grows to include "and also fix DVCS." That dilution makes the next retrospective harder to write.
- Option 3 (re-scope down) is reasonable but premature — the right time to drop L2/L3 from v0.14.0 is after seeing whether v0.13.0's extended close produces telemetry usable for the L2/L3 decision. SPECULATIVE: my guess is L2/L3 will need at least 2–4 weeks of real-backend telemetry before the data settles enough to make L2-vs-L3 a confident call. v0.14.0 may need to ship OTel + tail first, then telemetry-data-collection-only, then L2/L3 in v0.15.0. But that's a v0.14.0 planning decision, not a v0.13.1 strategic decision.

### Specific re-sequencing recommendations

- **Block v0.14.0 on extended-v0.13.0 GREEN.** The new v0.13.0 close-ritual MUST include a real-backend dark-factory run; v0.14.0 cannot start until that probe passes.
- **v0.14.0's first phase (Phase 54 OTel) should be preceded by a v0.14.0-Phase-0 "validate v0.13.0 substrate"**: run the dark-factory exercise again, confirm OP-3 lights up, confirm bus push works against documented mirror setup. If any fails, v0.14.0 pauses and the gap goes back into v0.13.0 patch territory.
- **L2/L3 should explicitly carry an "evidence prerequisite": >= N weeks of real-backend desync telemetry before the decision rule fires.** SPECULATIVE: this likely defers L3 (transactional cache writes) to v0.15.0, since the data-collection window is the gating constraint.

### TLDR sentence

**v0.14.0 cannot start until extended-v0.13.0 ships, because every v0.14.0 scope (OTel hot-paths, origin-of-truth version-check extension, L2/L3 telemetry) presumes the v0.13.0 helper-push and cache audit paths actually work on real backends — which they don't today; recommend re-sequencing v0.14.0 behind extended-v0.13.0 and adding a v0.14.0-Phase-0 substrate-validation gate.**

---

## Q5 — The CTO brief (5 minutes)

### Headline paragraph

**v0.13.0 was graded GREEN by the verifier subagent on 2026-05-01; the milestone is in a "ready-to-tag" state but the tag has not been pushed. A 4-subagent dark-factory exercise the next day, plus a 12-subagent codebase audit a week later, found that the milestone delivered its internal building blocks (URL parsers, fault-injection cargo tests, doc structure, mirror-lag refs, `attach` on the simulator) but not its vision: zero of the three roles the milestone was named for (SoT-holder, mirror-only consumer, round-tripper) work end-to-end on any real backend, and only one (SoT-holder) is testable on the simulator with three of six documented steps requiring the user to ignore the docs. The headline subcommand `reposix attach` literally bails on real backends with an error string that leaks internal GSD phase IDs (`P79-02 scaffold`, `P79-03`) to end users. The architectural cornerstone of v0.9.0 (`git push` rejected → `git pull --rebase` recovery) is broken on the simulator. The project's named non-negotiable invariant (OP-3, audit log non-optional) is dark for every push from a real working tree. None of these appeared in the surprises intake or the milestone retrospective because no phase ever ran the documented user flow end-to-end against a real backend; the milestone-close gate (which the vision document and ROADMAP both said gates real-backend tests) ran 8 probes, zero of which touched a real backend.**

### The single decision the CTO needs to make this week

**Authorize one of two paths:**

- **Path A (recommended):** hold the v0.13.0 tag; extend the milestone with 4 corrective phases (P89: real-backend `attach`; P90: push-flow correctness — rebase + OP-3 audit; P91: bus push compatibility with documented mirror setup; P92: framework upgrade — claim-vs-assertion congruence + real-backend milestone-close probe) plus 2 reservation phases; tag v0.13.0 only when the dark-factory subagent can complete the vision-document litmus test against real Confluence end-to-end. Estimated wall-clock: 2–4 weeks. RETROSPECTIVE.md captures this as a one-time precedent and ratifies a milestone-close real-backend probe as the structural fix.

- **Path B:** push v0.13.0 as-is with a CHANGELOG amendment naming the known issues; ship v0.13.1 within 2 weeks as the user-facing release with the four corrections. Estimated wall-clock: 1 week to v0.13.0 tag + 2–3 weeks to v0.13.1. CHANGELOG.md v0.13.0 ships with explicit "known-issues" section; users instructed to wait for v0.13.1 if they want the documented vision.

**The decision pivots on one question:** is the CHANGELOG entry's truth value worth a 2–3 week tag delay? Path A says yes (the public claim that "`reposix attach` reconciles existing checkouts on Confluence/GitHub/JIRA" should be true when v0.13.0 ships). Path B says no (release cadence is more important than CHANGELOG fidelity; v0.13.1 fixes the gap).

### Subordinate framing for the CTO

If the CTO picks Path A, also authorize: (1) the framework redesign in Q3 (claim-vs-assertion congruence axis on every catalog row + milestone-close adversarial pass); (2) the v0.14.0 sequencing in Q4 (v0.14.0 blocks on extended-v0.13.0 GREEN; v0.14.0-Phase-0 substrate-validation gate). Without (1), the next milestone repeats the failure shape; without (2), v0.14.0 builds on a substrate that doesn't work.

### TLDR sentence

**Headline: v0.13.0 graded GREEN but its three-role vision works on zero real backends; recommend hold-the-tag and extend with 4 corrective + 2 reservation phases; the load-bearing decision is "CHANGELOG fidelity vs. 2-3 week tag delay."**

---

## Q6 — Meta-finding: what does this say about how the project does GSD?

### What worked exactly as designed

- **Vertical-slice verification.** Every phase had a catalog row, a verifier, an unbiased subagent grade, a verdict file, a SUMMARY.md, a CHANGELOG-segment-worth of shipped surface. Each phase's vertical slice IS internally consistent and verifiable. The framework graded what was in front of it honestly. The misalignments are not "the verifier missed something within scope" — they are "the verifier wasn't asked about something out-of-scope."
- **Catalog-first rule.** Every phase minted catalog rows BEFORE writing implementation. Verifier subagents read rows that existed pre-implementation. This is a load-bearing trust property and it held.
- **+2 reservation slots (OP-8) for items the discovering phase observed.** P81's `refresh_for_mirror_head` no-op-skip eager-resolution, P83-02's `core.hooksPath` override eager-resolution, P84's binstall+yanked-gix DEFERRED — all handled correctly.
- **Per-phase push cadence (codified mid-v0.13.0).** Closed the v0.12.1 115-commit unpushed-stack failure mode. This rule held.
- **Hooks > prose for enforcement.** The pre-commit fmt hook, the freshness invariants in `scripts/end-state.py`, the docs-alignment walker — all worked as designed.

### What didn't work — the structural lesson

The GSD process — research → plan → execute → verify → close — has **no horizontal-composition step**. Every layer in the chain trusts an earlier layer to have validated the cross-cutting claim:

1. The vision document writes the litmus test (lines 19–42 of v0.13.0 vision-and-mental-model.md).
2. The ROADMAP decomposes the litmus test into phases. Each phase covers a slice.
3. Each phase's PLAN-OVERVIEW + per-task PLAN files specify the slice's catalog rows and verifiers.
4. Each phase executes; verifier subagent grades the slice; verdict GREEN.
5. The milestone-close (P88) aggregates per-phase verdicts; verdict GREEN if all phases GREEN + close-ritual artifacts exist.

**No step in the chain explicitly tests "the litmus test from the vision document, end to end."** The litmus test is the input to step 2 and the input to step 5 (in the form of a SC like "TokenWorld arm GREEN"). It is never executed. Every phase trusts step 5 to have validated the composition; step 5 trusts step 2 to have decomposed completely; step 2 trusts step 1 to have specified the test correctly. The composition fall-through is the bug.

The framework has the right gates (catalog-first, unbiased verifier, +2 reservation, OP-9 retrospective) for validating slices. It does not have a gate for validating the composition. The dark-factory exercise IS that gate, run post-hoc by humans noticing things are broken.

### What this says about GSD itself

**GSD optimizes for vertical-slice correctness and assumes the human (or the planner subagent) decomposed the milestone correctly.** When the decomposition is sound and each slice individually delivers, the milestone delivers. When the decomposition has a gap (e.g., "real-backend coverage" is named in the vision but isn't a phase), GSD has no mechanism to catch the gap because GSD doesn't know what the vision expected — the vision document is a planning input, not a runtime artifact.

**Three GSD-process changes that would have caught this:**

1. **Vision-document citation is mandatory in every phase plan and verdict.** Each phase's PLAN.md and VERDICT.md names the vision-litmus-test step(s) the phase covers. The milestone-close VERDICT.md cross-references vision-litmus-test steps to phase verdicts; uncovered steps are RED. This makes "we missed step 5 of 6" structurally visible.

2. **Milestone-close runs the vision-litmus-test verbatim.** Not a probe ("does step 5 produce stderr matching pattern X"); the LITERAL litmus test ("a fresh subprocess agent given only the vision-doc commands completes the round-trip"). If the litmus test isn't runnable as a script, the vision document hasn't been written executably enough.

3. **Cold-reader rubric runs BEFORE milestone-close GREEN, not after.** The DVCS-DOCS-04 cold-reader rubric was deferred to "owner runs it post-phase" and was still NOT_VERIFIED at milestone GREEN. The framework treats subagent-graded rubrics as freshness-checkable but doesn't block GREEN on them. Cold-reader is exactly the rubric that would have caught the docs-vs-implementation drift; making it a milestone-close blocker (not a TTL-graded "owner runs it") would have caught Cluster E (init UX broken) and Cluster F (tutorial output stale) before tag.

### What survives unchanged

- The dimension/cadence/kind taxonomy (correct primitive; just needs the fourth axis from Q3).
- Per-phase push cadence (load-bearing; survives).
- +2 reservation (correct mechanism; needs spirit-not-letter interpretation when reality demands more than 2 phases).
- Catalog-first rule (load-bearing; survives).
- Subagent verifier dispatch (correct mechanism; needs the meta-grading-by-different-subagent rule from F2 of P87 to extend to honesty-spot-check authorship).

### TLDR sentence

**GSD optimizes for vertical-slice correctness and lacks a horizontal-composition gate; the vision-litmus-test must become a runtime artifact (executable end-to-end at milestone close), not a planning input — every layer of the GSD chain trusted an earlier layer to have validated the composition, and no layer actually had.**

---

## One-page summary (mirror to TLDR)

1. **Tag/retract/extend?** Hold the tag; extend v0.13.0 with P89–P94 (4 corrective + 2 reservation); tag honestly when dark-factory passes against real Confluence end-to-end.
2. **New milestone vs. extend?** Extend v0.13.0 (counter-argument: v0.13.1-as-separate preserves GREEN-is-final invariant — but outweighed by CHANGELOG/RETROSPECTIVE truth-value cost).
3. **Framework patch or redesign?** Redesign-class: add "claim-vs-assertion congruence" axis to every catalog row + milestone-close adversarial pass that grades it; patch-class fixes (shell-subprocess kind, WAIVED-with-until-date, content-binding hashes) are the concrete first commits.
4. **v0.14.0 readiness?** Block v0.14.0 on extended-v0.13.0 GREEN; add a v0.14.0-Phase-0 substrate-validation gate; SPECULATIVE: L2/L3 likely defers to v0.15.0 because the data-collection window is the gating constraint.
5. **CTO brief?** v0.13.0 GREEN-but-vision-broken; the load-bearing decision is "CHANGELOG fidelity vs. 2–3 week tag delay" — Path A (hold + extend, recommended) vs. Path B (push tag with known-issues + ship v0.13.1).
6. **Meta-finding?** GSD optimizes for vertical-slice correctness and lacks a horizontal-composition gate; the vision-litmus-test must become a runtime artifact (executable end-to-end at milestone close), not a planning input.
