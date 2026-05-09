# v0.13.0 Bird's-Eye Vision Audit — DVCS over REST

**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08
**Scope:** v0.13.0 milestone as a whole — does the shipped+graded milestone deliver the three-roles vision end-to-end?

## Verdict at a glance

- **Three roles delivered end-to-end on a real backend:** **0 of 3.**
- **Three roles testable in this build (sim or otherwise):** 1 of 3 (only the SoT-holder, and only on the simulator with copy-paste discrepancies).
- ALIGNED items: 1 (the architectural plumbing — bus URL parsing, mirror-lag refs writes on the sim wire path, fault-injection coverage in cargo tests).
- MISALIGNED items: 12 (see Findings).
- SUSPECT items: 1 (whether the cache.db OP-3 silence is a wrong-cwd helper bug or a deeper cache-discovery bug — would require a debugger).

The milestone delivered the *internal building blocks* (parsers, fault-injection in cargo tests, doc structure) but not the *vision* (a fresh dev runs the documented round-trip on a real backend). The verdict graded the building blocks. The vision is undelivered against any real backend, and broken against the simulator at the documented happy path.

---

## Findings

### F1 — `reposix attach` is sim-only; the milestone-defining subcommand cannot satisfy the vision against any real backend [SEVERITY: HIGH]

**Claim in plan / vision:** vision-and-mental-model.md line 89 names "`reposix attach <backend>::<project>` is implemented and tested" as success gate #1; the litmus test (lines 19–42) literally has Dev B run `cargo binstall reposix && reposix attach confluence::SPACE`. ROADMAP P79 goal: "implement `reposix attach <backend>::<project>` — builds a fresh cache from REST against an existing checkout".

**Reality:** `crates/reposix-cli/src/attach.rs:147–166` only matches `"sim"`; every other backend bails with `"attach: backend `{other}` not yet wired in P79-02 scaffold (sim only); github/confluence/jira land alongside the integration tests in P79-03"`. P79-03 in fact shipped — see `79-03-SUMMARY.md` — but its three commits (`a558d4a`, `791f7b9`, `dd3c801`) added integration tests on sim only. There is no commit anywhere in P78–P88 that wired confluence / github / jira into `attach.rs`.

**Evidence:** `crates/reposix-cli/src/attach.rs:162-165` (the bail!); `crates/reposix-cli/tests/attach.rs` contains zero references to "confluence", "github", or "jira" (`grep -n "confluence\|github\|jira" tests/attach.rs` → empty); P79 verdict `quality/reports/verdicts/p79/VERDICT.md:9-15` accepts "exists; subcommand wired" + sim integration test as PASS for DVCS-ATTACH-01..04 without distinguishing sim from real backends.

**Why it matters:** **the headline subcommand of v0.13.0 cannot satisfy the milestone's own litmus test.** Two of three vision-roles (mirror-only consumer round-tripping back via attach; round-tripper itself) are unreachable. T2 confirmed this against real Confluence; the dark-factory subagent was stopped on step 3 of 5. This is not a polish gap — it is the load-bearing user flow named on line 89 of the vision document, missing from the binary the milestone shipped.

---

### F2 — Production error message leaks GSD planning phase IDs to end users [SEVERITY: HIGH]

**Claim in plan:** Project conventions (CLAUDE.md OP-7 / verifier-subagent rules; project-wide error-message hygiene) require user-facing errors to be self-recoverable — the dark-factory thesis depends on stderr teaching the next move.

**Reality:** the `attach.rs:163` error literal contains `"P79-02 scaffold"` and `"P79-03"` — internal GSD phase IDs. A user has no way to look these up, no recovery hint, no tracking issue.

**Evidence:** `crates/reposix-cli/src/attach.rs:163-164` (literal). Same anti-pattern in `crates/reposix-cli/src/sync.rs:89` (`v0.13.0 (sim only)` — at least no phase ID, but same shape). T2-attach.md F7 documents the user-side observation.

**Why it matters:** violates the dark-factory teaching-string contract that the project markets as a security property (CLAUDE.md "agent UX is pure git, zero in-context learning"). The release went out the door with internal planning vocabulary visible in stderr — a canary that nothing in P78–P88 ever read this error from a user perspective.

---

### F3 — OP-3 audit log is silently dark for every helper push (project-defined non-negotiable invariant violated) [SEVERITY: HIGH]

**Claim in plan:** CLAUDE.md OP-3 (load-bearing project invariant): *"Audit log is non-optional… either schema missing a row for a network-touching action means the feature isn't done."* ROADMAP recurring success criterion #8: *"Audit log non-optional (OP-3) — every bus-remote push writes audit rows to BOTH tables (cache audit + backend audit); mirror push writes a cache-audit row noting mirror-lag delta."*

**Reality:** every `git push` from a partial-clone working tree (sim AND real Confluence) prints `WARN cache unavailable for push audit: open reposix-cache: git: git config --add transfer.hideRefs failed: fatal: not in a git directory` and writes ZERO `helper_push_*` rows. `cache.db` is never created on the helper-push code path. SoT-side mutation succeeds; cache-side audit is permanently absent.

**Evidence:** T1-sim-baseline.md F6, T4-conflict-recovery.md MED-2 (verified via `sqlite3` against actual cache.db). Two independent dark-factory subagents reproduced this on the simulator. P83 verdict `quality/reports/verdicts/p83/VERDICT.md:51-52` "honesty spot-check 4" claimed the dual-table audit assertion is satisfied — but it asserted via wiremock + `count_audit_cache_rows` inside `assert_cmd`-controlled cargo tests where the cache path is explicit; it never tested the production helper invocation path that the actual `git push` shell subprocess uses.

**Why it matters:** OP-3 is the project's named non-negotiable. Eight phases (P79–P86) ran without anyone exercising `git push` end-to-end and inspecting `cache.db` afterward. The verifier framework's "audit completeness" row asserted what the test infrastructure controlled, not what the user experiences. The whole helper-push audit pipeline is dark — and the milestone shipped GREEN with P83's verdict explicitly citing OP-3 dual-table compliance as a "PASS" honesty spot-check.

---

### F4 — `git pull --rebase` recovery — the v0.9.0 architectural cornerstone — is broken on the simulator [SEVERITY: HIGH]

**Claim in plan / docs:** CLAUDE.md "Load-bearing behaviors": *"Push-time conflict detection. Helper rejects with the standard git 'fetch first' error on remote drift; agent recovers via `git pull --rebase && git push`."* `docs/guides/troubleshooting.md:227+` and `docs/index.md` § "What it looks like underneath" both promise this recovery.

**Reality:** rejection works (T4 step 5 — WIN). Recovery does not: every helper fetch mints a NEW root commit with no ancestry to the prior tip. `git fetch` after any successful push fails with `fatal: error while running fast-import`. `git pull --rebase` cascades the same error. `--force` does not help (error is internal to fast-import, not ref-update).

**Evidence:** T4-conflict-recovery.md HIGH-1 (verified across two checkouts A and B against sim::demo). P86 dark-factory third-arm scenario (`quality/gates/agent-ux/dark-factory.sh dvcs-third-arm`) does not exercise post-push fetch — its assertions stop at static teaching-string greps + `--help` token checks + bus URL composition + cache materialization (per catalog row `agent-ux/dvcs-third-arm` lines 1009–1017).

**Why it matters:** this is not a v0.13.0-introduced regression — it is the v0.9.0 architectural cornerstone, and it is silently broken on the default backend used in every CI run. The milestone graded GREEN with the dark-factory regression test passing because that test never tries to `git fetch` after a `git push`. Any two-writer flow on the sim is unrecoverable.

---

### F5 — Helper export validator collides architecturally with the documented mirror setup [SEVERITY: HIGH]

**Claim in plan / docs:** `docs/guides/dvcs-mirror-setup.md:42-46` instructs the user to `git commit` `.github/workflows/reposix-mirror-sync.yml` into the mirror repo. ROADMAP P82 + P83 promise the bus URL fans the SoT push out to the mirror so a single `git push` updates both. P86 dark-factory third-arm tests that flow.

**Reality:** the helper's export validator rejects `.github/workflows/*.yml` and `README.md` as "invalid records (no frontmatter)". `reposix refresh` itself writes `.reposix/.gitignore` and `.reposix/fetched_at.txt` into the working tree — files its own export then refuses on push.

**Evidence:** T3-bus-push.md F12, F13 (real Confluence + real GH mirror; reproduced repeatedly). The error messages are literal (`error: invalid issue at .github/workflows/reposix-mirror-sync.yml: invalid record file: missing frontmatter open fence; refusing push`).

**Why it matters:** **the bus push cannot succeed against any mirror that follows the documented setup.** This is not a doc-fix-able bug — the architecture has no path-prefix scope (e.g., `?records_root=pages/`) or skip-list. The two halves of the milestone (P83 bus-write + P84 mirror-setup-template) were built and tested in isolation but never composed end-to-end against the real shape they were meant to ship as. No phase-level verifier could catch this because both pieces individually pass.

---

### F6 — P85 cold-reader pass — the vision's success gate #5 — was deferred to "owner-runs-it" and never landed before milestone GREEN [SEVERITY: HIGH]

**Claim in plan:** vision-and-mental-model.md success gate #5: *"Cold-reader pass on the DVCS docs… passes `doc-clarity-review` against a reader who has read only `docs/index.md` and `docs/concepts/mental-model-in-60-seconds.md`."* ROADMAP P85 SC4 says the same.

**Reality:** the catalog row `subjective/dvcs-cold-reader` (DVCS-DOCS-04) is `NOT_VERIFIED` per P85 verdict line 14 ("owner-graded, by design… owner runs `/reposix-quality-review --rubric dvcs-cold-reader`"). The milestone-close verdict graded GREEN with this rubric still NOT_VERIFIED — the milestone-close VERDICT.md doesn't mention DVCS-DOCS-04 at all in its REQ counts (line 47 lists "1 rubric-pending-owner" but does not block).

**Evidence:** `quality/reports/verdicts/p85/VERDICT.md:14`, `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:47`. Dark-factory subagents in T1, T2, T3 — exactly the cold-reader profile the rubric was meant to grade — found 37 frictions including 16 HIGH. Had the rubric been run BEFORE the milestone-close verdict, every dark-factory finding in CLUSTER E (init UX broken on first contact) and CLUSTER F (tutorial output stale) would have been visible.

**Why it matters:** the project's stated quality gate against shipping illegible docs was deferred past the GREEN gate that was supposed to depend on it. The milestone shipped GREEN BEFORE the very check designed to catch what eventually shipped broken. This is a meta-failure: the framework has the right gate registered, but the milestone-close verifier did not block on it.

---

### F7 — `dark_factory_real_confluence` test name promises real-backend coverage; assertions stop at "URL has the right shape" [SEVERITY: HIGH]

**Claim in plan / catalog:** the test exists at `crates/reposix-cli/tests/agent_flow_real.rs:146-165` named `dark_factory_real_confluence`, gated behind real-backend env vars. The name and `--ignored` real-backend gate imply it tests the real Confluence dark-factory flow.

**Reality:** the test runs `reposix init confluence::<space>`, then asserts only that `git config remote.origin.url` starts with the expected `reposix::https://...` prefix and ends with `/confluence/projects/<space>`. It never runs `git fetch`, `git push`, `reposix attach`, or any push/pull against the real backend.

**Evidence:** `crates/reposix-cli/tests/agent_flow_real.rs:115-165`. The test's call site `run_init_and_assert` only checks the `remote.origin.url` shape (lines 118-122). Same shape for `dark_factory_real_github` and `dark_factory_real_jira`.

**Why it matters:** **the only test in the workspace that touches a real backend with the word "dark_factory" in its name does not exercise the dark-factory flow.** Combined with F1 (attach unwired) and F4 (rebase recovery broken), this means the framework had a misleading slot that *looked* like real-backend coverage but was a URL-shape smoke test. The verifier subagents who graded P86 GREEN cited this slot indirectly via "TokenWorld arm SUBSTRATE-GAP-DEFERRED", which is a different (also-deferred) test path; neither covers the actual round-trip.

---

### F8 — SURPRISES-INTAKE.md captured 5 LOW/HIGH polish items but missed every dark-factory-discovered HIGH issue [SEVERITY: HIGH]

**Claim in plan:** CLAUDE.md OP-8: *"+2 phase practice."* The SURPRISES intake is the safety valve for items the discovering phase chose not to fix eagerly. P87 verdict line 2 says: *"Verifier honesty spot-check sampled 5 phases (exceeded the >=3 floor); aggregate GREEN."*

**Reality:** the 5 SURPRISES-INTAKE entries are: (1) P80 verifier-shape pivot LOW; (2) P81 `refresh_for_mirror_head` no-op-skip LOW; (3) P81 docs-alignment T01→T04 schedule LOW; (4) P83-02 fixture `core.hooksPath` override LOW; (5) P84 binstall+yanked-gix substrate gap HIGH. **None** of the 16 HIGH dark-factory frictions (attach for real backends, OP-3 audit silence, rebase recovery broken, mirror-tree validator collision, init "Next:" hint contradicts itself, tutorial expected output stale, etc.) appears in the intake.

**Evidence:** `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (5 entries verified). `T1-T4-*.md` (37 frictions, 16 HIGH).

**Why it matters:** **OP-8 worked exactly as designed for the items the executing phases noticed; it never triggered for the items the executing phases never tried.** No phase ever ran `git push` from a partial-clone working tree and inspected the audit log. No phase ever ran `git pull --rebase` after a successful push on the sim. No phase ever copy-pasted the README quickstart on a fresh checkout. The +2 reservation is downstream of phase-level observation; if no phase observes the failure, the intake stays empty and P87's "honesty spot-check" passes. This is a structural blind spot, not a P87 failure.

---

### F9 — OP-1 says real-backend tests gate the milestone close. They didn't [SEVERITY: HIGH]

**Claim in plan:** ROADMAP recurring success criterion #6: *"Simulator-first (OP-1) — all phases run end-to-end against the simulator. Two simulator instances serve as 'confluence-shaped SoT' + 'GitHub-shaped mirror.' Real-backend tests gate milestone close, not individual phase closes."* vision-and-mental-model.md (OP-1 invariant): *"Real-backend tests (TokenWorld + reubenjohn/reposix) gate the milestone close, not individual phase closes."*

**Reality:** the milestone-close verdict at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` ran 8 re-verification probes (lines 13-23): grep REQUIREMENTS.md, count phase verdicts, run pre-push runner, run dark-factory.sh sim arm, check RETROSPECTIVE.md, stat tag-script, grep CHANGELOG, count SURPRISES OPEN. **Zero** of those 8 probes touched a real backend. The milestone-close verifier checked "are all phase verdicts GREEN" + "did the +2 ritual produce paper" — not "do the real-backend tests pass."

**Evidence:** `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md:13-23`; the dark-factory.sh sim-arm (probe #4) is the only end-to-end shell flow exercised — and even that does not test real Confluence per the deferral framing in the catalog comment.

**Why it matters:** **the milestone-close gate that was supposed to enforce real-backend coverage didn't enforce real-backend coverage.** The vision document and ROADMAP both explicitly cite this as the milestone-level gate. The gate fired without firing the check. Combined with F7 (the only real-backend tests are URL-shape only) and F8 (intake never received the HIGH frictions), this is a circular dependency: every layer trusted that an earlier layer had checked, and no layer actually had.

---

### F10 — RETROSPECTIVE.md v0.13.0 section reports zero of the 16 HIGH dark-factory frictions; the OP-9 distillation captures phase-internal lessons but not vision-level outcomes [SEVERITY: MED]

**Claim in plan:** CLAUDE.md OP-9 mandates RETROSPECTIVE.md distillation BEFORE archive: *"What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons. Source: SURPRISES-INTAKE + GOOD-TO-HAVES + per-phase verdicts + autonomous-run findings."*

**Reality:** `.planning/RETROSPECTIVE.md:14-50` (v0.13.0 section) cites 5 milestone-specific lessons + 4 v0.13.0-specific patterns + 4 inefficiencies — every one of them about internal process (cargo-test-as-verifier, narrow-deps refactors, eager-resolution carve-out, CI fix-forward churn, env-var races, CHANGELOG length). **None** of the 16 dark-factory HIGH frictions appears as a Lesson. The "What Worked" list does not flag that the litmus test in the vision document was never demonstrably executed against a real backend.

**Evidence:** `.planning/RETROSPECTIVE.md:14-50`. Compare T1-T4-*.md and SUMMARY.md (37 frictions, 16 HIGH, root cause "the quality framework structurally exempts real-backend end-to-end flows" — a statement that should have appeared in OP-9 distillation if the milestone-close ritual had ground-truth visibility).

**Why it matters:** OP-9 is the cross-milestone learning channel. If the 16-HIGH dark-factory finding from 2026-05-02 is not reflected in the v0.13.0 retrospective, the next milestone's planner will read RETROSPECTIVE.md and see "v0.13.0 worked." This is the failure mode OP-9 was designed to prevent ("learnings get lost in milestone archives"). Tactically: the RETROSPECTIVE was finalized BEFORE the dark-factory exercise — chronologically defensible. Strategically: the milestone-close ritual produced the distillation BEFORE the test that would have invalidated it. The ordering is the bug.

---

### F11 — Verdict honesty: P83 + P86 + milestone-v0.13.0 verdicts cite "honesty spot-checks" that never tested production paths [SEVERITY: MED]

**Claim in plan:** verifier subagent prompt (per `quality/PROTOCOL.md`) requires unbiased grading from artifacts. P83 verdict (`quality/reports/verdicts/p83/VERDICT.md:159-167`) lists "5 hard stress-tests"; P86 verdict (lines 35-50) cites "Wire-path delegation legitimacy — DEFENSIBLE".

**Reality:** every P83 honesty spot-check was a `grep` against source files (`apply_writes &ParsedExport`, `bus_handler.rs:478` plain push, `#[cfg(unix)] mod common`, etc.) or a count against an `assert_cmd`-controlled cargo test's audit-table query. None invoked `git push` as a subprocess and inspected the resulting cache state. P86 verdict's defensibility argument explicitly accepts that "literal `git push` end-to-end at shell scope is documented best-effort and brittle to env propagation" — which is the project saying *we know we can't test the production path, so we'll test the parts we can.* That's a reasonable engineering tradeoff, but it must NOT be hidden inside a "GREEN" verdict that the milestone-close gate then trusts.

**Evidence:** P83 VERDICT.md:43-52, 161-166; P86 VERDICT.md:34-50; the layered-coverage CLAUDE.md sanction (post-v0.13.0 P86 trail).

**Why it matters:** the verifier subagent system did its job — it graded the artifacts in front of it. But the artifacts in front of it deliberately substituted cargo-test wire-path coverage for shell-subprocess production-path coverage, with both verdicts saying "this is defensible" rather than "this is a known-coverage-gap; the milestone-close gate must verify the production path." The milestone-close verdict then did not verify the production path. The framework correctly graded the substitution as legitimate; nothing in the framework owned the original obligation.

---

### F12 — Cross-phase coherence: P78–P88 stack as 11 vertical slices that each pass their own verifier; no horizontal probe ever validates the vision composition [SEVERITY: HIGH]

**Claim in plan:** ROADMAP frames P78–P88 as a coherent build-up to the three roles. Vision document's "litmus test" (lines 19-42) sequences `reposix init` → mirror push → vanilla `git clone` Dev B → edit → `reposix attach` → bus push → confluence webhook → mirror catches up.

**Reality:** P79 ships attach-on-sim (passes); P80 ships mirror-refs-on-sim (passes); P81 ships L1 perf-on-sim (passes); P82 ships URL parsing (passes); P83 ships bus-write fault-injection in cargo tests (passes); P84 ships GH Action template byte-equality (passes); P85 ships docs that describe Pattern C (passes the structural docs-alignment binding rows); P86 ships shell-greps + cargo wire-path (passes the catalog asserts). **No phase, and no milestone-close gate, ever sequences the litmus test from line 19 to line 42 of the vision doc against any backend, sim or otherwise.** The dark-factory subagent's T2 (real Confluence) and T3 (real Confluence + real mirror) are the first time anyone tries — and they fail at step 3 (T2) and step 5 (T3).

**Evidence:** Reading the 11 phase verdicts in sequence (`quality/reports/verdicts/p7{8,9},p8{0..8}/VERDICT.md`) — each is internally consistent, none claims to verify the litmus test, the milestone-close verdict aggregates them without claiming to verify the litmus test either.

**Why it matters:** **the vision-level claim has no phase-level owner.** Every individual phase passes against its own scope; the vision composition is unverified. This is the structural reason the dark-factory exercise found 37 frictions against a "GREEN" milestone — the GREEN was for the vertical slices, the frictions live in the horizontal composition.

---

## Cross-cutting story (what no individual phase auditor would catch)

The five compounding gaps:

1. The vision's litmus test (the round-trip on a real backend) is composed of pieces that each pass their own verifier in isolation.
2. The simulator stand-in for real backends is exempt from the production-path test (helper subprocess + cache.db inspection) by the layered-coverage decision sanctioned in P80→P86; cargo tests cover the wire path under `assert_cmd` env control.
3. The real-backend tests that DO exist (`agent_flow_real.rs`) are URL-shape smoke tests, not flow tests — the names suggest more than the assertions deliver.
4. The cold-reader rubric that was supposed to catch this kind of failure was deferred to "owner runs it post-phase" and was still NOT_VERIFIED at milestone close.
5. The milestone-close verifier's 8 probes do not include any real-backend flow probe; OP-1's "real-backend tests gate milestone close" is asserted in the ROADMAP but never operationalized into the verifier's checklist.

The result: every layer trusts an earlier layer to have checked, no layer actually has, the milestone ships GREEN, and the dark-factory exercise (which is just a fresh dev typing the documented commands) finds 37 frictions in 4 hours.

---

## Exec brief — 5 minutes to a CTO

**v0.13.0 graded GREEN. The milestone shipped its internal building blocks (URL parsers, fault-injection cargo tests, doc structure, mirror-lag refs, `attach` on the simulator) and graded them honestly. It did not ship its vision: a developer cannot complete the documented round-trip on any real backend, and cannot complete it on the simulator either without ignoring the docs at three of six steps.**

The headline subcommand `reposix attach` is sim-only — its production error message literally tells users `not yet wired in P79-02 scaffold; github/confluence/jira land alongside the integration tests in P79-03`, leaking internal planning vocabulary because no human ever read this stderr from a user perspective during P78–P88. The architectural cornerstone of v0.9.0 (`git push` rejected → `git pull --rebase` recovery) is broken on the simulator because every helper fetch mints a fresh root commit. The bus push (P83's headline feature) cannot succeed against any mirror that follows the documented mirror-setup guide because the helper's frontmatter validator rejects the workflow YAML the guide tells you to commit. OP-3 ("audit log non-optional," named as a release blocker in the threat model) is dark for every push from a real working tree because the helper fails to open the cache. None of these appear in the SURPRISES intake — every one of them was missed because no phase tried the user flow end-to-end.

The framework worked exactly as designed at the layer it was designed for: vertical-slice verification with unbiased subagent grading. The framework has a structural exemption for real-backend, end-to-end, shell-subprocess flows — the layered-coverage decision (P80 → P86) substitutes cargo tests under `assert_cmd` env control for the production path, and the milestone-close verifier's 8 probes do not include a horizontal litmus-test probe. The cold-reader rubric (DVCS-DOCS-04) that would have caught the documentation-vs-reality drift was deferred to "owner runs it post-phase" and was still NOT_VERIFIED at milestone GREEN.

**The fix isn't bigger phases or more verifiers; it's a milestone-close gate that runs the vision document's litmus test verbatim against a real backend before the milestone tag.** v0.13.1 (planned in this directory's SUMMARY.md) ships the four functional fixes (P89–P92). The framework fix — promoting "real-backend dark-factory subagent runs the litmus test" to a non-skippable milestone-close probe — is the load-bearing follow-on, because without it, v0.13.1 will close GREEN with the same blind spot.
