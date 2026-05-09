# Phase P87 Audit — surprises-absorption (+2 reservation slot 1)
**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08

## Verdict at a glance
- ALIGNED items: 2 (mechanical drain assertion is honest; intake bookkeeping is internally consistent)
- MISALIGNED items: 8
- SUSPECT items: 1

## Executive summary

P87's mechanical drain (5 entries → terminal STATUS) is correct on its face — every intake entry has a defensible STATUS line and the catalog row PASSes the awk-fence-aware verifier. **What P87 missed is the work it was load-bearingly assigned to do: the verifier honesty spot-check.** Per CLAUDE.md OP-8, the spot-check is the meta-control that catches the "found-it-but-skipped-it" failure mode across the milestone. P87 ran a check, but the check has no teeth: the sample omits the two phases (P79, P86) where the largest scope cuts of v0.13.0 actually occurred, the verdict was authored by the same orchestrator that ran the milestone (not an independent verifier), and the grading rubric rewarded "phases that filed something or eager-resolved" rather than asking "did this phase deliver the architecturally-promised outcome?"

The post-milestone dark-factory exercise (2026-05-02; `SUMMARY.md` in this directory) found 37 frictions / 16 HIGH against v0.13.0. Cross-checking those against P87's spot-check: at least 4 of the 16 HIGH frictions (Cluster A `attach` unimplemented for real backends; Cluster B rebase-recovery broken; Cluster C OP-3 audit log silent on every push; Cluster G real-backend tests stop at URL-shape) are direct consequences of scope cuts P78–P86 made WITHOUT filing SURPRISES entries. P87's spot-check graded all sampled phases GREEN, did not sample P79, sampled P86 but accepted the "Rule 3 eager-resolution pivot" framing without interrogating what coverage was lost, and produced a self-consistent verdict that does not survive a `git grep "P79-02 scaffold"` against the shipped binary.

The honesty-check honor system worked exactly as designed for honest phases (P82, P83, P84 were honest; their gradings are correct). It produced a false-GREEN for the two phases that needed the meta-control most.

## Findings

### F1 — Honesty spot-check sample structurally excludes the two phases where v0.13.0's biggest scope cuts occurred [SEVERITY: HIGH]

**Claim in plan:** ROADMAP P87 SC2 (`.planning/milestones/v0.13.0-phases/ROADMAP.md:107`) says "Verifier honesty spot-check samples ≥3 P78–P86 plan/verdict pairs … Empty intake when verdicts show skipped findings → RED." The 87-01-SUMMARY.md frames the 5-phase oversample (P82, P83-01/02, P84, P85, P86) as a *strengthening* over the floor — `decisions[3] PCT-04`: "sampling more (P82, P83-01/02, P84, P85, P86) gives the dispatched verifier richer cross-reference evidence and reduces 'verifier got the wrong sample' risk."

**Reality:** The sample omits P79 (where the `attach` real-backend implementation was promised, scoped to P79-03 in `attach.rs:163-164`'s error message, and silently dropped — P79's verdict at `quality/reports/verdicts/p79/VERDICT.md:11-15` graded only `DVCS-ATTACH-01..04` against sim-only tests). P79 is the single largest scope-cut of the milestone (Cluster A in dark-factory `SUMMARY.md:29-44`); excluding it from a "honesty" sample is structurally significant, not random. P78 is also unsampled despite the body's "aggregate finding" line at `honesty-spot-check.md:47` claiming "P78, P81, P83-02 eager-resolved AND filed entries" — P78 is invoked in the conclusion but not in the per-phase grading body. P80 (verifier-shape change that is the source of SURPRISES Entry 1) is also unsampled.

**Evidence:**
- `.planning/phases/87-surprises-absorption/honesty-spot-check.md:13-41` (5 sampled phases: P82, P83-01/02, P84, P85, P86; P78, P79, P80 absent from the body).
- `.planning/phases/87-surprises-absorption/honesty-spot-check.md:47` ("3 phases (P78, P81, P83-02) eager-resolved AND filed entries" — P78 appears in the count but not in the grading body; internal contradiction).
- `crates/reposix-cli/src/attach.rs:162-165` ("attach: backend `{other}` not yet wired in P79-02 scaffold (sim only); github/confluence/jira land alongside the integration tests in P79-03"). The error message itself documents that the original P79-03 plan promised real-backend attach.
- `quality/reports/verdicts/p79/VERDICT.md:11-15` (5 REQ-IDs PASS; all assertions against sim).

**Why it matters:** OP-8's RED signal is "empty intake when verdicts show skipped findings." P79 has empty intake AND its verdict's REQ-IDs are sim-only AND production code carries a leaked phase-ID error string saying real backends "land in P79-03." That is the textbook RED signal P87 was created to catch. Excluding P79 from the sample makes the GREEN verdict mechanically unfalsifiable.

### F2 — Honesty spot-check is authored by the milestone orchestrator, not an independent verifier [SEVERITY: HIGH]

**Claim in plan:** 87-01-PLAN.md:106 (SC2 evidence-target language) implies the honesty spot-check is "the input the dispatched verifier reads"; the dispatched verifier subagent (separate from the executor) is the entity that signs off. The verdict at `quality/reports/verdicts/p87/VERDICT.md:20` says "Sign-off: legitimate (cross-references plan/SUMMARY/verdict triples for each sample; eager-resolution decisions verified against verdict GREEN status)."

**Reality:** `honesty-spot-check.md:3-4` declares "Author: P87 executor (top-level coordinator; pre-verifier-dispatch)." The same coordinator that orchestrated all of P78–P86 graded the OP-8 honesty of P78–P86. The dispatched verifier (verdict at `p87/VERDICT.md`) read the executor's grading and confirmed the grading was internally consistent — but did not re-derive the grading. The verdict's "Sign-off: legitimate" rationale is "cross-references plan/SUMMARY/verdict triples for each sample" — i.e. the verifier confirmed the executor cited the right files, not that the executor's grading rubric was sound.

**Evidence:**
- `.planning/phases/87-surprises-absorption/honesty-spot-check.md:3-4` (executor-authored).
- `quality/reports/verdicts/p87/VERDICT.md:19-20` ("samples 5 phases ... aggregate finding GREEN; no phase exhibits 'found-it-but-skipped-it' failure mode. Sign-off: **legitimate**").
- The verdict cites no spot-check of its own — it accepts the executor's grading as sound.

**Why it matters:** OP-8 names this exact failure mode: "the verifier honesty check ... prevents the practice from degrading into a no-op." When the same agent that ran the milestone grades whether the milestone honestly looked for out-of-scope items, the check IS the no-op OP-8 was designed to prevent. CLAUDE.md OP-7 ("Phase-close means catalog-row PASS. No phase ships on the executing agent's word. An unbiased verifier subagent grades the catalog rows") is the project's stated invariant; P87's structure violates it for the meta-grading.

### F3 — Honesty spot-check rubric grades "did the phase use the framework?" instead of "did the phase deliver?" [SEVERITY: HIGH]

**Claim in plan:** 87-01-PLAN.md:38 ("Aggregate finding GREEN") and `honesty-spot-check.md:43-53` (grading body): each phase is graded GREEN with rationale of the form "phase used the +2 reservation framework as designed."

**Reality:** The grading rubric collapses "honest framework usage" with "no out-of-scope drift." P86 is the canonical example: graded GREEN at `honesty-spot-check.md:38-41` because P86's SUMMARY documented its end-to-end-push → wire-path-delegation pivot as a Rule 3 eager-resolution. The grading does NOT ask "what coverage did the pivot cost the milestone?" P86's pivot took the third-arm regression away from any code path that touches `git fetch`/`git push` against a real mirror — exactly the surface where dark-factory `T3-bus-push.md` + `T4-conflict-recovery.md` later found 8 HIGH frictions. The pivot was *honestly framed* in P86's SUMMARY; the pivot was *not interrogated* in P87's spot-check.

**Evidence:**
- `.planning/phases/87-surprises-absorption/honesty-spot-check.md:38-41` (P86 graded GREEN: "Cross-reference vs. double-file is the correct framework usage").
- `.planning/phases/86-dark-factory-third-arm/86-01-SUMMARY.md:80-91` (the Rule 3 pivot: "Pivoted the third arm's coverage shape from 'full round-trip' to 'agent UX surface + bus URL composition'"). The rationale cites env-propagation issues with `git fetch` — same env-propagation issues the dark-factory exercise later confirmed are real and unfixed.
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md:156-159` ("**P86 (dark-factory third arm):** the test agent clones THIS repo via vanilla git, runs `reposix attach confluence::TokenWorld`, edits, and bus-pushes back. The repo URL is hard-coded into the test transcript (or env-var configurable)."). The original CARRY-FORWARD scope for P86 was full round-trip; the pivot dropped this.
- `.planning/research/v0.13.0-real-backend-frictions/SUMMARY.md:99-108` (Cluster G — "the dark_factory_real_confluence test stops at 'URL has the right shape' — never runs git fetch or git push. Agent-ux dimension verifies sim + cargo-test wire paths only. Shell subprocess flows against real backends have zero verifier coverage").

**Why it matters:** The rubric reduces OP-8's load-bearing question ("did this phase honestly look for out-of-scope items?") to "did this phase fill in the right form?" A phase can use the framework perfectly and still ship a feature that doesn't exist — exactly what P79 and P86 did. The milestone's headline UX claim ("pure git after init/attach. No reposix CLI awareness needed beyond bootstrap") was unsupported AT MILESTONE CLOSE because P79 and P86 each made a defensible eager-resolution decision and P87's rubric never asked whether the resulting milestone delivered the claim.

### F4 — Drain disposition for Entry 1 (P80 verifier shape) is circular: cites P86 verdict, but P86 verdict is suspect for the same reason [SEVERITY: MED]

**Claim in plan:** SURPRISES-INTAKE.md:36 (Entry 1 STATUS line): "RESOLVED | P86 verdict GREEN at quality/reports/verdicts/p86/VERDICT.md confirms the cargo-test-as-verifier shape is a sanctioned house pattern." 87-01-SUMMARY.md PCT-01 echoes this rationale.

**Reality:** The disposition cites P86's verdict as license to use cargo-test-as-verifier instead of the planned `reposix init` end-to-end shell — but P86's verdict is itself the result of the *same* env-propagation-driven pivot away from end-to-end coverage. Closing P80's deviation against P86's verdict creates a self-licensing loop: each phase cites the next phase's framing of the same shortcut as evidence the shortcut is sanctioned. No external arbiter (architecture sketch, CARRY-FORWARD, REQUIREMENTS, vision-and-mental-model) ratifies layered coverage as "the new house pattern" — the sanction lives only in the phase chain that benefits from it.

**Evidence:**
- `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md:36` (Entry 1 disposition cites P86 verdict).
- `.planning/phases/86-dark-factory-third-arm/86-01-SUMMARY.md:80-91` (P86 pivot rationale: same env-propagation gotcha).
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` and `.planning/REQUIREMENTS.md` do not declare layered coverage as a sanctioned shape (would need to be cited if it were the architectural ratification).
- `.planning/RETROSPECTIVE.md:106` does describe layered coverage as "Sanctioned in v0.13.0 P80 → P86; the new house default for env-propagation-sensitive surfaces" — but this is the v0.13.0 retrospective, written by the same chain, citing itself.

**Why it matters:** The RESOLVED disposition is the catalog state P88 reads when minting v0.13.0 milestone-close artifacts. If the disposition is circular, the rationale propagates into CHANGELOG and CLAUDE.md as if it were ratified design. Future planners will see "cargo-test-as-verifier is sanctioned" and re-apply the shortcut to any agent-ux-shaped surface that's hard to drive from shell — exactly the gap the dark-factory exercise found.

### F5 — Drain disposition for Entry 5 (P84 binstall) is correct but the catalog row that depends on it is graded vacuous-PASS rather than WAIVED [SEVERITY: MED]

**Claim in plan:** SURPRISES-INTAKE.md:76 (Entry 5 disposition): "DEFERRED | v0.13.0 → v0.13.x carry-forward … Catalog row `agent-ux/webhook-latency-floor` currently passes vacuously (p95=5s synthetic placeholder per P84 verdict GREEN); the row's freshness_ttl + the post-release re-measurement together close the loop."

**Reality:** The DEFERRED status is well-reasoned for the SURPRISES entry — the gap is real, sized HIGH, with an owner-runnable script in tree. The problem is the *companion catalog row*: a row that is "vacuously PASS with synthetic placeholder p95=5s" is, in the framework's own terms, a row whose verifier does not exercise the load-bearing claim (failure shape #2 in `AUDIT-BRIEF.md:50-54`). The catalog state is "PASS" — not "WAIVED until 2026-MM-DD" — so the framework's freshness invariants do not flag it. P87 had the opportunity to flip the row to WAIVED with an explicit until-date matching the v0.13.x release target and did not.

**Evidence:**
- `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md:76` (DEFERRED text acknowledges vacuous PASS).
- `.planning/research/v0.13.0-real-backend-frictions/AUDIT-BRIEF.md:51-52` (failure shape #2: "'Substrate gap' / 'deferred' deferrals masquerading as GREEN").
- `quality/catalogs/agent-ux.json` row `agent-ux/webhook-latency-floor` (row exists at PASS per P84 SUMMARY).
- `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md:63-91` (the WAIVED-STRUCTURE-ROWS-03 entry establishes the WAIVED pattern with explicit until-dates; the precedent existed for the binstall row but was not applied).

**Why it matters:** The DEFERRED-but-PASS pattern is the single most common way "graded GREEN" stops correlating with "ships the documented behavior" in this milestone. The framework supports an explicit `WAIVED + until_date` catalog status; using PASS instead means a future runner sweep finds nothing wrong, and a future planner reads the row as ratified.

### F6 — Drain disposition for Entry 3 (P81 schedule shift) defers the actual fix to a phase that doesn't exist [SEVERITY: LOW]

**Claim in plan:** SURPRISES-INTAKE.md:56 (Entry 3 STATUS line, WONTFIX): "The deeper improvement — extending `bind` with a `--test-pending` flag … — is a tooling polish item that fits OP-8 sizing as XS (single Rust flag + branch in `bind`) and belongs in `GOOD-TO-HAVES.md` (P88 territory, NOT P87)." 87-01-SUMMARY.md:111: "if P88 accepts it, P88 files the GOOD-TO-HAVE entry directly."

**Reality:** `GOOD-TO-HAVES.md` at v0.13.0 close has exactly one entry (`GOOD-TO-HAVES-01` — extend `bind` to all dimensions, marked PARTIAL/DEFERRED to v0.14.0). The `--test-pending` flag never landed in `GOOD-TO-HAVES.md`. P88 closed the milestone without filing it. The polish item is now homeless — not in v0.13.0 surprises, not in v0.13.0 good-to-haves, not in v0.14.0 carry-forward.

**Evidence:**
- `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md:56` (P87's WONTFIX rationale: file in GOOD-TO-HAVES.md).
- `.planning/phases/87-surprises-absorption/87-01-SUMMARY.md:111` ("if P88 accepts it, P88 files the GOOD-TO-HAVE entry directly").
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md:5-32` (only GOOD-TO-HAVES-01 present).

**Why it matters:** Low-severity in itself (the `--test-pending` flag is XS polish), but it's a representative sample of how P87's "park it for P88" pattern can lose items entirely. The WONTFIX disposition was load-bearing on a downstream filing that didn't happen.

### F7 — CLAUDE.md update is a 1-line note, not the OP-8-mandated state refresh [SEVERITY: LOW]

**Claim in plan:** 87-01-PLAN.md:85 (Task 3 step 2): "Append to `CLAUDE.md` OP-8 section a brief in-place note that v0.13.0 surprises-absorption was completed. (One-line addition to keep the section current; full v0.13.0-shipped subsection lands in P88.)" SC5: "CLAUDE.md updated in same PR (OP-8 note appended)."

**Reality:** CLAUDE.md's "Quality Gates" / "Meta-rule" framing in `CLAUDE.md` mandates that "CLAUDE.md stays current. Each phase introducing a new file/convention/gate updates CLAUDE.md in the same PR. The update means *revising existing sections* to reflect the new state — not appending a narrative." A 1-line dated note appended to the OP-8 paragraph is exactly the "appending a narrative" pattern the rule prohibits. P87 introduced the cargo-test-as-verifier pattern as a sanctioned shape (per Entry 1 RESOLVED rationale + RETROSPECTIVE line 106) but did NOT revise the "Quality Gates — dimension/cadence/kind taxonomy" section to name layered coverage. Future planners reading CLAUDE.md cold do not learn this sanction exists.

**Evidence:**
- `CLAUDE.md` (the project guide that this audit is reading from): "CLAUDE.md stays current. Each phase introducing a new file/convention/gate updates CLAUDE.md in the same PR. The update means *revising existing sections* to reflect the new state — not appending a narrative."
- `.planning/phases/87-surprises-absorption/87-01-SUMMARY.md:96` ("OP-8 in-place note appended (4-line bullet under '+2 reservation is in addition to' paragraph)").
- `.planning/RETROSPECTIVE.md:106,123` (RETROSPECTIVE describes layered coverage as a sanctioned pattern; CLAUDE.md does not mention it).
- `.planning/phases/87-surprises-absorption/87-01-SUMMARY.md:123` (P88-deferred carry-forward: "Explicit naming of the layered-coverage shape (shell harness for UX + cargo test for wire path) in CLAUDE.md 'Quality Gates — dimension/cadence/kind taxonomy'. XS-sized; future planners benefit from upfront-knowledge vs. rediscovering the env-propagation gotcha each time"). The phase recognized the gap and deferred it. P88 GOOD-TO-HAVES.md does not contain this entry.

**Why it matters:** The "rediscovering the env-propagation gotcha each time" failure mode is exactly what happened in the dark-factory exercise. The CLAUDE.md surface that would have warned the dark-factory subagents was never written.

### F8 — Honesty-grade rationale for P85 ("docs phase, no out-of-scope discoveries") missed the docs-vs-reality gap [SEVERITY: HIGH]

**Claim in plan:** `honesty-spot-check.md:31-35` (P85 grading): "Empty-intake claim is consistent with verdict (no skipped findings); the NOT_VERIFIED row is by-design owner-graded, not a hidden defer." Aggregate GREEN.

**Reality:** P85 shipped `docs/concepts/dvcs-topology.md`, `docs/guides/dvcs-mirror-setup.md`, and `docs/guides/troubleshooting.md` § "DVCS push/pull issues." The dark-factory exercise found that the documented user flow in `dvcs-mirror-setup.md` step 4 ("commit a `.github/workflows/reposix-mirror-sync.yml`") is rejected by the helper's frontmatter validator (Cluster D in `SUMMARY.md:71-81`); `docs/reference/testing-targets.md` references "TokenWorld" but the configured tenant only has "REPOSIX" (Cluster H1 in `SUMMARY.md:111-113`); and the documented Pattern C flow (round-tripper) is gated on `reposix attach` for real backends, which doesn't ship. P85 wrote docs; P85's verdict graded "docs render and sites build"; P85's intake is empty. None of the docs-vs-implementation contradictions were filed. P87 graded P85 GREEN.

**Evidence:**
- `.planning/phases/87-surprises-absorption/honesty-spot-check.md:31-35` (P85 graded GREEN with "no skipped findings").
- `.planning/research/v0.13.0-real-backend-frictions/SUMMARY.md:71-81` (Cluster D — bus push rejects exactly the files the docs tell users to commit).
- `.planning/research/v0.13.0-real-backend-frictions/SUMMARY.md:111-113` (Cluster H1 — testing-targets.md cites TokenWorld; tenant has REPOSIX).
- `.planning/research/v0.13.0-real-backend-frictions/SUMMARY.md:114` (no tutorial for Pattern C / round-tripper / bus push).
- `quality/reports/verdicts/p85/VERDICT.md` (graded "4/4 catalog rows PASS" — assertions are about docs-build / link-resolve / hash-binding, not about whether the documented commands work).

**Why it matters:** P85 is the "user-facing" carrier phase — the docs an agent reads to learn the bus URL form, the mirror setup, the round-trip flow. P87's spot-check accepted "docs phase = nothing to surface" without doing the cold-reader walkthrough that CLAUDE.md mandates ("Cold-reader pass on user-facing surfaces"). The dark-factory exercise IS that cold-reader pass; it found exactly the contradictions P87 said were absent.

### F9 — Velocity smell: P87's "~25 minutes executor wall time" for what is nominally a meta-control across 9 phases [SEVERITY: MED]

**Claim in plan:** 87-01-SUMMARY.md frontmatter (`metrics.duration: ~25 minutes (executor wall time)`). 87-01-SUMMARY.md PCT-04: "Cost: ~10 minutes more authoring; benefit: harder-to-game GREEN verdict."

**Reality:** ~25 minutes wall time to: (a) read 9 phase plans + 9 SUMMARYs + 9 verdicts; (b) grade each phase against OP-8's "did this phase honestly look for out-of-scope items"; (c) cross-reference 5 SURPRISES entries against their originating phases' artifacts; (d) write the honesty spot-check + draft 3 commits + push + run the verifier. The dark-factory audit (this v0.13.1 research effort, 4 subagents) took materially longer per phase to cover similar ground and found 37 frictions. The velocity gap is significant: a thorough OP-8 spot-check that asked "are there scope cuts the phase didn't file?" plausibly takes hours, not minutes, because it requires reading the production code and walking the documented user flow against it. P87's velocity is consistent with "graded the form fields" rather than "audited the implementation."

**Evidence:**
- `.planning/phases/87-surprises-absorption/87-01-SUMMARY.md:44` (`duration: ~25 minutes`).
- `.planning/research/v0.13.0-real-backend-frictions/SUMMARY.md:1-17` (4-subagent dark-factory exercise, found 37 frictions; the per-test files T1–T4 each run several thousand words of evidence).
- AUDIT-BRIEF.md failure shape #6: "Velocity-as-skip-signal."

**Why it matters:** OP-8 calls out that the verifier honesty check exists to "prevent the practice from degrading into a no-op." A 25-minute pass over 9 phases is structurally a no-op pass.

### F10 — RETROSPECTIVE.md v0.13.0 section uses past-tense ratification language for patterns that the dark-factory exercise has since invalidated [SEVERITY: MED, SUSPECT for full breadth]

**Claim in plan:** 87-01-PLAN.md:84: "Append to `.planning/RETROSPECTIVE.md` a v0.13.0 surprises-absorption section (single concise paragraph) noting the 5-entry drain disposition." SC2 from ROADMAP P87 cites the spot-check at the verdict.

**Reality:** `.planning/RETROSPECTIVE.md:106` ("Layered coverage … sanctioned in v0.13.0 P80 → P86; the new house default for env-propagation-sensitive surfaces") and `.planning/RETROSPECTIVE.md:115` ("Mitigation: cargo-test-as-verifier with assert_cmd (v0.13.0 P80 pivot)") propagate the cargo-test-as-verifier shortcut into the cross-milestone learnings. The dark-factory exercise found that the cargo-test layer covered the *wire path under controlled conditions* but didn't cover the *user-facing path under real-backend conditions* — the "shortcut" is sanctioned in RETROSPECTIVE without that qualifier. P87 wrote the surprises section that authorized this language; the v0.13.x audit (this research) is the corrective.

**Evidence:**
- `.planning/RETROSPECTIVE.md:106` (Layered coverage sanctioned).
- `.planning/RETROSPECTIVE.md:115` (Mitigation: cargo-test-as-verifier).
- `.planning/research/v0.13.0-real-backend-frictions/SUMMARY.md:99-108` (Cluster G).

**Why it matters:** RETROSPECTIVE.md is the cross-milestone authority on "patterns established." Patterns logged here propagate to future milestones. The pattern as written ("layered coverage is the house default") is half-true — sufficient for sim, structurally insufficient for real-backend coverage. The qualifier matters.

**Suspect element:** Whether RETROSPECTIVE.md entries beyond the two cited lines also need walk-back is not exhaustively probed in this audit. A wider pass would settle whether the v0.13.0 retrospective should be retired vs. amended.

### F11 — Verifier shell exit-0 condition is satisfiable without honest grading [SEVERITY: MED]

**Claim in plan:** 87-01-PLAN.md:46-51 — verifier asserts (a) intake exists, (b) zero `STATUS: OPEN` lines, (c) ≥5 terminal STATUS, (d) `honesty-spot-check.md` file exists.

**Reality:** The verifier checks for the file's *existence*, not its *content*. A `honesty-spot-check.md` containing "yep all good" would pass. The verifier has no rubric coupling to the spot-check's substance. Compare with how `quality/gates/docs-alignment/` rows bind a `test_body_hash` against cited test functions — that pattern would let the catalog row detect spot-check tampering, but P87's row uses no such hash-binding.

**Evidence:**
- `quality/gates/agent-ux/p87-surprises-absorption.sh:54-57` (file-existence check only).
- `quality/catalogs/agent-ux.json:1052-1058` (row asserts: file existence + intake STATUS counts; no content-binding).
- `quality/gates/docs-alignment/walk.sh` and the docs-alignment catalog (per `CLAUDE.md` "Docs-alignment dimension") establish the test_body_hash pattern P87 could have followed.

**Why it matters:** The catalog row is the persistent state that a future runner sweep checks. If the row's asserts don't bind to the spot-check's substance, then editing the spot-check to be more or less rigorous post-hoc has no detectable effect on the catalog. The framework's "drift detection" capacity is not engaged for this row.

## Cross-cutting observation: the structural shape of P87's failure

P87's mechanical layer (intake bookkeeping + catalog row + verifier shell) is correct and well-shaped. P87's substantive layer (the OP-8 honesty spot-check) is the meta-control that should have caught the v0.13.0 milestone's structural gaps and did not. The gap is not laziness — the spot-check rationale is detailed and the eager-resolution-vs-intake-vs-cross-reference distinction is correctly applied to the phases it samples. The gap is in two places: **sample selection** (the two phases where the largest scope cuts occurred are unsampled) and **rubric** ("did the phase use the framework correctly?" instead of "did the phase deliver the architecturally-promised outcome?").

This is the v0.13.1 process improvement: P87's successor needs (a) mandatory inclusion of every phase that closed without filing intake, (b) a rubric question of the form "walk one critical example end-to-end mentally — does it work?", and (c) the spot-check author must not be the orchestrator that ran the milestone.

## Cross-references for v0.13.1 framework-fix phase

- Failure shape inventory: `.planning/research/v0.13.0-real-backend-frictions/SUMMARY.md` clusters A, B, C, D, G, H1.
- P79's silent scope cut (cluster A): production string at `crates/reposix-cli/src/attach.rs:163-164` + `crates/reposix-cli/src/sync.rs:89` + verdict at `quality/reports/verdicts/p79/VERDICT.md`.
- P86's pivot rationale (cluster G): `.planning/phases/86-dark-factory-third-arm/86-01-SUMMARY.md:80-95`.
- Original P86 scope before pivot: `.planning/milestones/v0.13.0-phases/CARRY-FORWARD.md:156-159`.
- P87's spot-check method + sample: `.planning/phases/87-surprises-absorption/honesty-spot-check.md`.
- P87's verdict: `quality/reports/verdicts/p87/VERDICT.md`.
- CLAUDE.md OP-8 (honesty-check rule): the project guide loaded in this session.
