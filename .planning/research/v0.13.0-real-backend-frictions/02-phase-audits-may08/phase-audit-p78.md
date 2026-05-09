# Phase P78 Audit — Pre-DVCS hygiene
**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08

## Verdict at a glance
- ALIGNED items: 12 (HYGIENE-01 deliverables; MULTI-SOURCE-WATCH-01 schema migration; load-bearing test; catalog row mint; CLAUDE.md updates; runner dispatch)
- MISALIGNED items: 6
- SUSPECT items: 2

P78 is the most "self-contained" of the v0.13.0 phases (cargo deps + shell + a Rust crate-internal schema migration), and most of its deliverables landed cleanly. The misalignments concentrate around (a) the HYGIENE-02 verifier scripts being implemented as "filename-substring grep" while the catalog row descriptions promise structural assertions, (b) the verifier subagent grading a workspace-gate claim it never independently re-ran, and (c) a couple of plan promises that quietly shrank in execution.

## Findings

### F1 — `no-pre-pivot-doc-stubs.sh` does a substring grep; catalog row promises structural assertion [SEVERITY: HIGH]
**Claim in plan / catalog:** `quality/catalogs/freshness-invariants.json:395-398` (`structure/no-pre-pivot-doc-stubs`) declares the assertion as: *"For each docs/*.md whose byte size <500 AND whose top-level slug appears as a key in mkdocs.yml plugins.redirects.redirect_maps, the redirect target resolves to a file present under docs/. Stubs without a redirect entry AND without a nav: entry are flagged."*
**Reality:** `quality/gates/structure/no-pre-pivot-doc-stubs.sh:18-23` simply runs `grep -qF "${base}" mkdocs.yml` for every <500-byte top-level docs/*.md. It does NOT (a) parse `plugins.redirects.redirect_maps` keys, (b) check that the redirect TARGET resolves to a file under `docs/`, or (c) distinguish "appears in nav:" from "appears in redirect_maps". A docs file whose basename appears anywhere — even inside a code block or unrelated comment in `mkdocs.yml` — counts as "referenced". This is one of the failure shapes the audit brief calls out: assertion stated as structural, implementation is `grep`-only.
**Evidence:**
- `quality/catalogs/freshness-invariants.json:395-398` (`expected.asserts` text)
- `quality/gates/structure/no-pre-pivot-doc-stubs.sh:11-30` (the grep -qF impl)
- `mkdocs.yml:69-72` (only two redirect_maps entries: `architecture.md` and `demo.md`; the row claims to check redirect-target resolution which the script never does)
**Why it matters:** This is a "WAIVED → PASS" flip — the entire point of HYGIENE-02 was to retire a waiver in exchange for a real verifier before 2026-05-15. The catalog now reads PASS at status level but the regression contract is materially weaker than what the row's `expected.asserts` advertises. Future attempts to fix a real pre-pivot stub (whose name happens to appear elsewhere in mkdocs.yml as a passing string) will get a false-PASS.

### F2 — `repo-org-audit-artifact-present.sh` checks for one vocabulary token; catalog row promises gap-mapping audit [SEVERITY: HIGH]
**Claim in plan / catalog:** `quality/catalogs/freshness-invariants.json:427-431` declares: *"File contains a top-section with audit table mapping every numbered gap from .planning/research/v0.11.1/repo-organization-gaps.md 'Top 10 cleanup recommendations' to a closure path (closed-by-catalog-row | closed-by-existing-gate | waived)."* The plan (`78-PLAN-OVERVIEW.md:30`) and the rationale comments inside the script (`repo-org-audit-artifact-present.sh:6-9`) state the same: every numbered gap must be mapped.
**Reality:** `repo-org-audit-artifact-present.sh:24` is a single `grep -qE 'closed-by-catalog-row|closed-by-existing-gate|waived'`. As long as the artifact file exists AND **at least one row anywhere** in it uses one of those three tokens, the script PASSES. It does not enumerate the gaps, does not count rows, does not assert "Top 10" coverage. The verifier passes trivially against any file containing the literal string `waived` once.
**Evidence:**
- `quality/catalogs/freshness-invariants.json:427-431`
- `quality/gates/structure/repo-org-audit-artifact-present.sh:22-27`
- `quality/reports/audits/repo-org-gaps.md:21` ("Top 10 recommendations" header — the structure the verifier should check but doesn't)
**Why it matters:** Same shape as F1. The catalog implies a structural audit; the verifier is a smoke-test for vocabulary. A future repo-org-gap file that drops half its rows but keeps the word `waived` somewhere would PASS. This was the plan's own MEDIUM risk (78-PLAN-OVERVIEW.md:111 "MULTI-SOURCE-WATCH-01 walker migration surfaces drift" — wrong row, but the same risk shape was anticipated for HYGIENE-02 as 78-02-PLAN T03's "TINY-shape ceiling"), and the deviation note in 78-02-SUMMARY.md:79-83 explicitly trims a 4-line block comment to make it fit "TINY shape" (≤30 lines). The line-budget pressure pushed the verifier toward a vocabulary smoke test instead of a row-by-row audit.

### F3 — Plan promised 5 regression tests in walk.rs; SUMMARY documents 3 new + 2 "carry-forward unchanged" [SEVERITY: MED]
**Claim in plan:** `78-03-PLAN/index.md:114-138` lists FIVE regression tests as "must_haves":
1. `walk_multi_source_non_first_drift_fires_stale` (load-bearing)
2. `walk_multi_source_first_drift_fires_stale`
3. `walk_multi_source_stable_no_false_drift`
4. `walk_legacy_catalog_backfills_source_hash_to_source_hashes`
5. `bind_multi_same_source_rebind_refreshes_just_that_index`

The plan acknowledges some tests already exist and instructs *"EXTEND or REWORK them; do NOT duplicate. … same test name, stronger assertion under the migrated walker."*
**Reality:** Per `78-03-SUMMARY.md:73-79` and the live test file:
- 3 NEW tests landed (#1, #4, #5 above) at `tests/walk.rs:535,621,704`.
- 2 PRE-EXISTING tests (#2, #3) "carry forward unchanged" — i.e. were NOT extended/reworked despite the plan's explicit instruction.

Cross-checking `walk_multi_source_first_drift_fires_stale` (tests/walk.rs:397-438): the test still only asserts `stderr.contains("STALE_DOCS_DRIFT")` — it does NOT assert the new `sources_drifted=[0]` diagnostic that plan 78-03 T02 promised "for forensic clarity" (acceptance line: *"Index of drift surfaces in the diagnostic line"*). The new diagnostic ships in the implementation (`commands/doc_alignment.rs:1043-1044`) but only ONE test (`walk_multi_source_non_first_drift_fires_stale:591`, asserting `sources_drifted=[1]`) verifies it.
**Evidence:**
- `78-03-PLAN/index.md:114-138` (must_haves: 5 tests)
- `78-03-SUMMARY.md:78-79` ("Existing P75 tests … carry forward unchanged. Total walk.rs: 6 → 9 tests.")
- `crates/reposix-quality/tests/walk.rs:397-438` (no `sources_drifted` assertion)
- `crates/reposix-quality/tests/walk.rs:591` (only test asserting the new diagnostic shape)
**Why it matters:** The plan's explicit instruction was to extend pre-existing tests under "stronger assertion" semantics. Two tests left unchanged is a quiet scope contraction. The forensic-clarity diagnostic for the FIRST-source case is implemented but unverified — if a refactor accidentally drops the index-0 path of the diagnostic loop, only the path-(b) test would catch it (and only because index 1 is exercised, not index 0).

### F4 — Verifier verdict claims `cargo check -p reposix-cache` clean but never re-ran the workspace gate [SEVERITY: MED]
**Claim in plan:** `78-PLAN-OVERVIEW.md:159` (verifier criteria): *"`cargo metadata` shows non-yanked gix; `gh issue view 29 30` returns CLOSED (or human-action note); CLAUDE.md cites new version."*
The plan's success contract (78-01-PLAN.md:42-50) names `cargo check --workspace`, `cargo clippy --workspace`, `cargo nextest run --workspace` as the GREEN gates.
**Reality:** The verdict (`p78/VERDICT.md:11,27,38`) reports it ran `cargo check -p reposix-cache` (per-crate spot check) and a single test run (`cargo test -p reposix-quality --test walk walk_multi_source_non_first_drift_fires_stale`). The verdict explicitly disclaims: *"workspace-wide gate covered by pre-push hook & confirmed by SUMMARY"* (line 27). i.e. the verifier subagent **trusted the executor's SUMMARY** for the workspace-wide cargo claim. This is exactly the failure mode CLAUDE.md OP-7 is meant to prevent ("the executing agent does NOT grade itself").
**Evidence:**
- `quality/reports/verdicts/p78/VERDICT.md:27` ("workspace-wide gate covered by pre-push hook & confirmed by SUMMARY")
- `quality/reports/verdicts/p78/VERDICT.md:38` (Method line names only `cargo check -p reposix-cache` + the single `walk` test)
- `78-01-SUMMARY.md:53-56` (the workspace-gate claim the verdict relied on — but the SUMMARY itself notes `cargo nextest` was substituted for `cargo test --workspace`, deviation §1)
**Why it matters:** The verifier's job is to grade catalog rows from artifacts with zero session context. Running `cargo check -p reposix-cache` does not actually verify HYGIENE-01's plan-stated workspace gate. If the gix bump broke a downstream crate (say `reposix-remote`), the verifier subagent wouldn't have caught it; only the executor's self-report would. The workspace gate IS covered by pre-push, which the verifier reports as 25 PASS — but pre-push runs the runner gates, not the cargo workspace gates (the cargo workspace gates run only at the pre-push hook layer outside the runner). This is a small but real audit-trust gap.

### F5 — `cargo nextest` plan acceptance silently downgraded to `cargo test`; flagged as "Eager-resolution" but not as deviation [SEVERITY: LOW]
**Claim in plan:** `78-01-PLAN.md:46-47` and `:273-274` both name `cargo nextest run --workspace` as the canonical gate. The plan even discusses memory-pressure fallback as `cargo nextest`-flavored (per-crate `cargo nextest run -p ...`).
**Reality:** Per `78-01-SUMMARY.md:62-74`: *"`cargo nextest` not installed on host. … Fell back to `cargo test --workspace --no-fail-fast` … No SURPRISES entry needed (this is local-host tooling drift, not a phase finding)."* The verifier (`p78/VERDICT.md`) does not mention this fallback; the workspace-gate claim is propagated unchanged.
**Evidence:**
- `78-01-PLAN.md:46-47, 273-274`
- `78-01-SUMMARY.md:62-74` (the deviation; characterized as "local-host tooling drift")
- `quality/reports/verdicts/p78/VERDICT.md` (no mention of the substitution)
**Why it matters:** Low because `cargo test --workspace` IS a substantively similar gate. But CLAUDE.md OP-3 (audit log) and the "find-it-but-skipped-it" anti-pattern in OP-8 both push the other way: a deviation from the plan's named tool is exactly the kind of thing that goes in SURPRISES-INTAKE.md, not in a SUMMARY footnote labeled "no SURPRISES entry needed". `cargo nextest` was promoted to a CLAUDE.md soft-rule precisely because of memory-budget concerns ("`cargo nextest run` materializes test binaries one at a time vs `cargo test`'s parallel link"); silently dropping it is a process drift, not pure local-tooling drift.

### F6 — Live-catalog walker found 33 STALE_TEST_DRIFT rows; verdict file does not acknowledge them [SEVERITY: MED]
**Claim in plan:** `78-PLAN-OVERVIEW.md:111` (Risk + Mitigation): *"MULTI-SOURCE-WATCH-01 walker migration surfaces drift on the live 388-row catalog … drift here is the CORRECT behavior (path-(a) was hiding it). Refresh affected rows via `/reposix-quality-refresh <doc>`. If >5 rows surface, defer the rest to P87 SURPRISES."*
**Reality:** `78-03-SUMMARY.md:103, 130-131, 187-191` reports 33 STALE_TEST_DRIFT rows on the live 388-row catalog post-migration. That is far above the plan's 5-row threshold for SURPRISES deferral. The summary characterizes them as *"pre-existing, out of scope for P78-03 (orchestrator-flagged)"* — but pre-existing or not, the plan said >5 drift rows go to SURPRISES-INTAKE.md. They didn't (verified: SURPRISES-INTAKE.md grep for "P78" returns no matches; no `2026-04-30 | discovered-by: P78` entry).

The phase verdict (`p78/VERDICT.md`) makes no mention of these 33 rows. It reports `Pre-push gate GREEN at last push` as 25 PASS / 0 FAIL, which is consistent — but only because the alignment_ratio dropped from 1.0 to 0.9006 (still above the 0.5 floor). The 33 STALE_TEST_DRIFT rows are visible only inside the executor's SUMMARY.
**Evidence:**
- `78-PLAN-OVERVIEW.md:111` (>5 drift rows → SURPRISES)
- `78-03-SUMMARY.md:103, 130-131` (33 STALE_TEST_DRIFT rows on live catalog)
- `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (no P78 entry)
- `quality/reports/verdicts/p78/VERDICT.md` (no mention of live-catalog drift)
- (Current state: `grep -c STALE_TEST_DRIFT quality/catalogs/doc-alignment.json` = 71; later commits 0805644/98058e4/34f9f45 in May reduced/refreshed many but the P78-time count of 33 is what the plan's threshold should have triggered on)
**Why it matters:** "Found-it-but-skipped-it" is the failure mode OP-8 is explicitly designed to surface. The plan named a numeric threshold (5 rows); execution found 33 (~7×); zero SURPRISES entries got written. The justification ("orchestrator-flagged as out-of-scope; pre-existing") may be reasonable but the plan's threshold was the contract. The verifier's "honesty check" (see ROADMAP P87 SC: *"Empty intake when verdicts show skipped findings → RED"*) should arguably have failed here — but P78 close happens before P87, so the check would only have fired retroactively.

### F7 — Multi-source legacy backfill silently skips 14 rows in live catalog (path-(a) tradeoff partially preserved) [SEVERITY: LOW]
**Claim in plan:** `78-03-PLAN/index.md:69-82` says: *"After deserialization, a one-time backfill copies `source_hash` into `source_hashes[0]` when `source_hashes.is_empty() && source_hash.is_some()`. The backfill runs inside `Catalog::load` so every read path enters the new world."* This is the plan's load-time contract — every row enters the migrated world.
**Reality:** Per `78-03-SUMMARY.md:142-149` Deviation #1, the implementation refines the backfill to skip multi-source legacy rows (14 in the live catalog at P78 time) because backfilling 1 hash for an N-cite row would violate the parallel-array invariant. Those rows keep `source_hashes: []` — a "no-hash-recorded-yet" state that the walker treats as **skip drift compare** (see `catalog.rs:401-419`). Effectively, for those 14 rows, the path-(a) false-negative window is NOT closed; they continue to behave as before until they re-bind.
**Evidence:**
- `78-03-PLAN/index.md:69-82` (plan: every row enters the new world via backfill)
- `78-03-SUMMARY.md:142-149` (deviation: multi-source legacy rows skip backfill)
- `crates/reposix-quality/src/catalog.rs:401-432` (the actual skip logic)
- `crates/reposix-quality/src/commands/doc_alignment.rs:933-937` (`if row.source_hashes.is_empty()` → `source_drift = None` skip)
**Why it matters:** This is partial closure of MULTI-SOURCE-WATCH-01 dressed as full closure. The verdict (`p78/VERDICT.md`) reports MULTI-SOURCE-WATCH-01 as PASS without disclosing that 14 rows in the live catalog are still in the path-(a) tradeoff state. The plan's deviation note in the SUMMARY justifies the deviation reasonably (the alternative would slow `Catalog::load` and rehash on every read), but the partial-closure character is not reflected in CARRY-FORWARD's "shipped" status (REQUIREMENTS.md:128 marks MULTI-SOURCE-WATCH-01 `shipped`) or in the verdict. A complete shipping would either (a) backfill those 14 rows by rehashing, (b) re-bind them at migration time, or (c) re-classify their state as a new tracked condition.

### F8 — `walk_legacy_catalog_backfills_source_hash_to_source_hashes` covers the single-source backfill but NOT the multi-source skip [SEVERITY: LOW]
**Claim in plan:** `78-03-PLAN/index.md:130-134`: *"`walk_legacy_catalog_backfills_source_hash_to_source_hashes` — load a catalog written by the pre-migration binary … Assert that after `Catalog::load`, `row.source_hashes == vec!["<hex>"]`."* — single-source case only.
**Reality:** The plan only spec'd the single-source backfill. After execution, deviation #1 (`78-03-SUMMARY.md:142-149`) introduced a new behavior: multi-source legacy rows skip backfill entirely. There is NO regression test for the multi-source-skip path — only the single-source backfill test (`tests/walk.rs:621`).
**Evidence:**
- `78-03-PLAN/index.md:130-134` (test scope: single-source only)
- `crates/reposix-quality/tests/walk.rs:621-702` (the test as shipped)
- `crates/reposix-quality/src/catalog.rs:420-432` (the skip-multi branch with no test)
**Why it matters:** When a deviation introduces behavior that wasn't in the plan, the test surface should grow with it. As shipped, a future refactor that flips "skip backfill on multi-source legacy" to "backfill anyway" (re-introducing the invariant violation) would not be caught by any test in the workspace. The walker would then refuse to load the catalog. This is a SUSPECT-borderline finding because the catalog `validate_parallel_arrays` would still fire — but there's no test that exercises the migration path itself for the multi-source legacy shape.

### F9 — `78-02-SUMMARY.md` line-count trim hint suggests TINY-shape ceiling shaped the verifier coverage [SEVERITY: MED]
**Claim in plan:** `78-PLAN-OVERVIEW.md:27-32` insists on TINY shape (≤30 lines) for the 3 verifiers, *"mirroring `quality/gates/docs-alignment/jira-adapter-shipped.sh`"*.
**Reality:** `78-02-SUMMARY.md:79-83` Deviation 1 documents the trim: *"Initial draft of `repo-org-audit-artifact-present.sh` was 31 lines, exceeding the 30-line TINY-shape ceiling … Collapsed a 4-line block comment ('Sanity: ... categories; require at least one row using that vocabulary.') to a 2-line summary"*. The pruning was at the comment level only — but combined with F2, the pattern is consistent: the TINY ceiling pushed the verifier into smoke-test shape, and the SUMMARY characterizes the result as "no semantic change". F2 shows the semantic IS changed: the row's claimed assertion vs. what the verifier actually checks.
**Evidence:**
- `78-PLAN-OVERVIEW.md:27-32` (TINY-shape ceiling)
- `78-02-SUMMARY.md:79-83` (the trim)
- `quality/gates/structure/repo-org-audit-artifact-present.sh:22-27` (the actual assertion)
- (Comparable file: `quality/gates/docs-alignment/jira-adapter-shipped.sh` — for context)
**Why it matters:** A line-count budget should not be the constraint that decides whether a verifier is "row-by-row structural" vs. "vocabulary smoke test". The TINY-shape rule was meant to ensure verifiers stay reviewable and grep-friendly; what it accidentally enforced here was "the assertion is whatever fits in 30 lines, regardless of what the catalog row promised". This is a v0.13.1-relevant framework finding: the catalog-row-defines-contract principle (PROTOCOL.md Principle A) is violated when verifier line-budget < contract-richness.

### F10 — `cargo check -p reposix-cache` per-crate gate doesn't cover gix-using crates [SEVERITY: SUSPECT]
**Claim in plan / verdict:** Verifier disclosure (`p78/VERDICT.md:27,38`): *"`cargo check -p reposix-cache` actually ran clean (per-crate spot-check per CLAUDE.md memory-budget rules)"*.
**Reality:** `gix` is also a direct dependency of `reposix-remote` (per `78-01-PLAN.md:71`: *"`crates/reposix-remote/Cargo.toml:49` — consumers inheriting the workspace pin"*). The per-crate spot check used by the verifier covered ONE of the two direct gix consumers. If the gix 0.83 API broke `reposix-remote` but not `reposix-cache`, the verifier's per-crate check would not have caught it. The pre-push runner (which the verifier ran and reported 25 PASS) does NOT include a workspace-wide cargo check — pre-push hook does (via `scripts/end-state.py` + the cargo-check pre-push step). This is SUSPECT because settling it requires running the workspace-wide gate or reading the pre-push hook implementation; a clean run during P78 close is the most likely state, but the verifier's *trust path* skipped one of the two direct consumers.
**Evidence:**
- `78-01-PLAN.md:71` (two direct gix consumers: reposix-cache + reposix-remote)
- `quality/reports/verdicts/p78/VERDICT.md:27,38` (verifier ran reposix-cache only)
**Settled by:** running `cargo check -p reposix-remote` against the gix 0.83 baseline (currently still in tree); or reading the pre-push hook to confirm `cargo check --workspace` is invoked.

### F11 — Verdict's "SURPRISES.md unchanged" claim is technically true but wrong-shaped [SEVERITY: LOW]
**Claim in plan:** `78-PLAN-OVERVIEW.md:118-131` "out-of-scope candidates" lists multiple things to flag if discovered: gix multi-version jump, runner-doesn't-dispatch-.sh, walker-migration-surfaces-real-drift (>5 rows = SURPRISES).
**Reality:** `p78/VERDICT.md:24` reports: *"SURPRISES.md unchanged for P78 | PASS | no `2026-04-30 P78` entries in tail; executor reported 'none' — file matches"*.

But per F6, the >5-rows trigger DID fire (33 rows, ~7×); the executor self-reported "none" and the verifier's check was "executor reported 'none' — file matches". This is the verifier confirming the executor's self-report rather than re-evaluating the trigger conditions independently. The verdict's verbiage in line 24 (*"executor reported 'none' — file matches"*) is honest about the trust path, but the OP-8 honesty principle (CLAUDE.md OP-8 *"Verifier honesty check: the surprises-absorption phase's verifier subagent spot-checks the previous phases' plans + verdicts and asks 'did this phase honestly look for out-of-scope items?'"*) is exactly what was sidestepped here.
**Evidence:**
- `quality/reports/verdicts/p78/VERDICT.md:24`
- `78-03-SUMMARY.md:130-131` (the 33 rows that should have triggered an entry per the plan's >5 threshold)
- CLAUDE.md OP-8 (*"empty intake when verdicts show skipped findings is a RED signal"*)
**Why it matters:** The verifier subagent took the executor's "none" at face value. The trigger conditions in the plan were specific and numeric; nobody re-evaluated them independently against the SUMMARY's text. This compounds the v0.13.0-wide "verifier ratifies what executor wrote" pattern.

### F12 — "Velocity smell": all 3 plans atomically committed within 25 minutes (10:17pm → 10:41pm Pacific) [SEVERITY: SUSPECT]
**Claim in plan:** Per `78-PLAN-OVERVIEW.md:40-49`, the wave plan reserves Wave 2 (78-03 cargo) to run AFTER Wave 1 completes (78-01 cargo serial; 78-02 shell parallel). Each plan was scoped as a substantive piece of work (gix workspace-gate run; 3 shell verifiers; schema migration with 3 new tests across 4-touched-rs-files).
**Reality:** From `git log` timestamps:
- `ba4b4f2` (HYGIENE-01) — Apr 30 22:17:07 -0700
- `2bc4dc7` (HYGIENE-02) — Apr 30 22:22:32 -0700 (5 min after)
- `28ed9be` (MULTI-SOURCE-WATCH-01) — Apr 30 22:38:59 -0700 (16 min after)
- `ef81546` (CLAUDE.md SHA cite) — Apr 30 22:39:13 -0700 (14s after)
- `086422e` (P78 close) — Apr 30 22:46:23 -0700

Total elapsed between first commit and phase-close: 29 minutes 16 seconds. This includes (per the SUMMARY) `cargo check --workspace` (7s), `cargo clippy --workspace` (12.6s), `cargo test --workspace` (618 tests), AND the schema migration with 3 new tests AND a single-test subset cargo run, AND the verifier subagent's audit. cargo build of `reposix-cache` from cold can take 4-5 minutes alone; a clean workspace test run is several minutes more.

This is fast for the scope. The audit brief calls this out as a failure-shape ("Velocity smell — phases that shipped faster than scoped → suspect scope was cut"). The cuts I can identify:
- F3: 2 of 5 promised tests left unchanged (saves ~10 min of work)
- F2 / F9: TINY-line-budget collapsed structural assertions to vocabulary checks (saves ~30 min of work per verifier)
- F4 / F10: Verifier ran ONE per-crate check, not workspace (saves a few minutes)
- F6: 33-row drift threshold trigger ignored, no SURPRISES entry written (saves the entry-writing time)

The cumulative saved-effort matches the velocity gap.
**Evidence:**
- `git log --oneline` for ba4b4f2..086422e (timestamps above)
- F2, F3, F4, F6, F9 above (the cuts)
**Settled by:** comparing P78 elapsed time to similar past phases' elapsed times; reading `gsd-executor` timestamps if recorded.

### F13 — CLAUDE.md edit is technically correct but doesn't reflect the partial closure of MULTI-SOURCE-WATCH-01 [SEVERITY: LOW]
**Claim in plan:** `78-03-PLAN/index.md:167-173` instructs CLAUDE.md to be updated with: *"'Path (b) closed in v0.13.0 P78-03 via the parallel-array `source_hashes: Vec<String>` schema migration; non-first-source drift now fires `STALE_DOCS_DRIFT`.' Cite the commit SHA in the edit."*
**Reality:** Per F7, multi-source legacy rows (14 in the live catalog at P78 time) keep `source_hashes: []` and the walker SKIPS them. The CLAUDE.md edit (`CLAUDE.md:426` per verdict) cites the closure as complete; the partial-closure semantic for legacy multi-source rows is not surfaced anywhere except `78-03-SUMMARY.md:142-149`. CLAUDE.md is the canonical agent-facing artifact; the partial closure is a footgun for the next agent who reads "P78-03 closes path-(b)" and assumes the entire catalog is in the path-(b) world.
**Evidence:**
- `78-03-PLAN/index.md:167-173` (edit text)
- `CLAUDE.md` (currently mentions P78 in the gix line at :151; further P78-03 cite at the path-(a) tradeoff paragraph mentioned in 78-03-SUMMARY.md:218)
- `78-03-SUMMARY.md:142-149` (the partial-closure deviation)
**Why it matters:** Low because the deviation is documented in SUMMARY; but per CLAUDE.md OP-7 ("CLAUDE.md stays current… revising existing sections to reflect the new state — not appending a narrative"), the path-(a) tradeoff paragraph should mention the residual legacy-multi-source case. As-is, an agent reading CLAUDE.md will see "closed" without the asterisk.

### F14 — Plan's load-bearing test asserts `sources_drifted=[1]` and `doc_b.md`; the FIRST-source case has no equivalent assertion [SEVERITY: LOW]
**Claim in plan:** `78-03-PLAN/index.md:115-122` (load-bearing test): *"Build a Multi row with 2 sources where the SECOND source's bytes change post-bind. Assert `walk` exits non-zero, prints `STALE_DOCS_DRIFT` for that row, and the diagnostic names the second-source index (or file path) so the operator knows which source drifted."*
**Reality:** The test (`tests/walk.rs:535-613`) does this correctly for the second-source case. But by symmetry, when the FIRST source drifts (a regression from path-(a)), the operator should see `sources_drifted=[0]` and the doc_a.md path. There is no test that asserts this. Per F3, `walk_multi_source_first_drift_fires_stale` (tests/walk.rs:397) only asserts `STALE_DOCS_DRIFT` is in stderr; it does not assert `sources_drifted=[0]` or `doc_a.md`.
**Evidence:**
- `78-03-PLAN/index.md:115-122` (the load-bearing test spec)
- `crates/reposix-quality/tests/walk.rs:585-598` (path-(b) test asserts diagnostic shape)
- `crates/reposix-quality/tests/walk.rs:432-435` (path-(a) test does NOT assert diagnostic shape)
**Why it matters:** Asymmetric coverage of the diagnostic. A regression that breaks the index-0 path of the diagnostic loop in `commands/doc_alignment.rs:1043-1044` would only be caught if the loop also breaks for index 1. Symmetric tests are cheap (extend `walk_multi_source_first_drift_fires_stale` with two assertions).

## Cross-cutting observations

1. **Catalog row description vs. verifier mismatch is the v0.13.1 framework signal here.** F1, F2, F9 all show the same shape: catalog row's `expected.asserts` describes a richer assertion than the shipped verifier delivers. The TINY-shape line-budget for shell verifiers is structurally biased toward smoke-tests; the catalog rows are not. This is exactly the failure shape the audit brief named ("URL-shape only / exists-only / structure-only assertions where the description implies functional verification").

2. **Verifier subagent trust path leans on executor SUMMARY.** F4, F5, F6, F11 share this pattern: the verdict's evidence is "executor said X; SUMMARY confirms X". This is the inverse of OP-7's "zero session context" intent. P78 is mostly cheap to reverify (cargo cheap; shell verifiers TINY) — but the verifier ran ONE per-crate cargo check + ONE single-test cargo run + the runner. The bigger cargo gates and the live walker output were trusted.

3. **Eager-resolution preference papers over plan-vs-ship gaps.** Several deviations (F3 left tests unchanged; F5 substituted `cargo test`; F7 partial backfill; F9 trimmed comment for line budget) were classified as "no SURPRISES needed". The plan's named numeric thresholds (5 drift rows; 30-line ceiling; 5 tests) were each violated, and each was justified post-hoc as in-spirit. This is fine when each deviation is small; the cumulative effect is the velocity gap in F12.

4. **MULTI-SOURCE-WATCH-01 `shipped` status overstates closure.** Per F7, F8, F13, the migration's coverage of LEGACY multi-source rows is "skip drift compare" rather than full path-(b). The REQUIREMENTS.md shipping label, the CLAUDE.md edit, and the verdict's PASS verdict all communicate full closure; the partial-closure residue lives only in 78-03-SUMMARY.md deviations. v0.13.1 framework note: "shipped" should require closing the entire stated acceptance, not the happy path of it.
