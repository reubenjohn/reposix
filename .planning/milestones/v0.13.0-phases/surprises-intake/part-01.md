# v0.13.0 Surprises Intake (P96 source-of-truth) — Part 1 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-07 | discovered-by: v0.13.1 CHECKOUT-BREAK lane | severity: MEDIUM

**What:** The headline pure-git promise — `git checkout origin/main && cat issues/<id>.md`
(CLAUDE.md § Architecture) — still does not work VERBATIM after `reposix init`. `init`
configures `remote.origin.fetch = +refs/heads/*:refs/reposix/origin/*`, so the synced tip
lands under `refs/reposix/origin/main` and NOT the standard `refs/remotes/origin/main`
that `git checkout origin/main` DWIM-resolves via — the command fails `error: pathspec
'origin/main' did not match any file(s)`. The v0.13.1 hotfix made the `init` success
banner + docs honest (they now print the verified-working `git checkout -B main
refs/reposix/origin/main`), but the pure-git ergonomic itself is deferred.

**Why out-of-scope for the hotfix:** The fix (verified PROMISING on this box) is an
ADDITIVE second fetch refspec `+refs/heads/*:refs/remotes/origin/*` alongside the existing
`refs/reposix/origin/*` mapping. On git 2.25.1 it cleanly populates BOTH refs and
`git checkout origin/main` resolves — BUT it lands in DETACHED HEAD (no local `main`
branch is created, unlike `git clone`), so the documented edit→commit→push loop then needs
`git push origin HEAD:main` or an auto-created tracking branch — an unshipped design
decision. Worse, the supported git floor is >= 2.34 (which fetches via `stateless-connect`,
NOT the import path this VM's git 2.25 exercises), so the additive refspec CANNOT be
verified against the real target transport here. Shipping an unverifiable ref-topology
change that also changes checkout ergonomics is not a hotfix-conservative move.

**Sketched resolution (v0.14.0):** Design the pure-git front door properly: either (a)
add the additive `+refs/heads/*:refs/remotes/origin/*` fetch refspec in `init.rs` AND have
`init` create a local `main` branch tracking `origin/main` (mirroring `git clone`, but
WITHOUT eagerly materializing blobs — set the ref + upstream via plumbing, leave the tree
lazy), or (b) keep the `refs/reposix/origin/*`-only topology and update the docs' 60-second
mental model to the fully-named checkout as the canonical command. Prefer (a) — it honours
the CLAUDE.md headline. Verify on git >= 2.34 (stateless-connect fetch path) with a
leaf-isolated test proving `init → git checkout origin/main → edit → commit → git push`
round-trips. Evidence that (a) resolves the checkout on 2.25 (both refs populated, push
round-trip green): CONSULT-DECISIONS.md 2026-07-07 CHECKOUT-BREAK entry.

**STATUS:** OPEN (deferred to v0.14.0 front-door design; docs/banner made honest in v0.13.1)

## 2026-07-03 11:10 | discovered-by: resumption audit (8-week idle gap) | severity: MEDIUM

**What:** The `quality-weekly` workflow is RED on main for 2 consecutive weeks (Jun 22 + Jun 29), failing at the "Generate verdict" step; nobody drained it during the idle gap.

**Why out-of-scope for the resumption audit:** Diagnosing a CI-side verdict-generation failure needs log-reading + possibly a runner fix — phase work, not intake.

**Sketched resolution:** Diagnose in the P89 window, since the weekly verdict is part of the framework P89 touches — read the two failed "Generate verdict" logs, fix the root cause (or fold into the relevant 89-0x task if it is the runner), and confirm the next scheduled run goes GREEN.

**STATUS:** DEFERRED-P95/P97 | Root cause diagnosed in P90 (90-05 RAISE LIST § 1, commit ab7078c): 2 `docs-repro` P2 rows (`benchmark-claim/8ms-cached-read`, `benchmark-claim/89.1-percent-token-reduction`) are `kind: manual` with `verifier.script: null` — structurally unable to PASS, so `weekly` renders yellow/RED every run regardless of everything else. North-star fix is writing the 2 real verifier scripts (tracked as GOOD-TO-HAVES-04, size M) — routed to P95/P97 per OD-4 §3, NOT closed by softening `verdict.py` (explicitly rejected; see the sibling 2026-07-04 05:30 entry below, same disposition).

## 2026-07-03 11:15 | discovered-by: resumption audit (8-week idle gap) | severity: HIGH

**What:** Two open RUSTSEC advisories: RUSTSEC-2026-0186 (memmap2; issue #57) + RUSTSEC-2026-0185 (quinn-proto; issue #56). The Security-audit cron failed 2026-06-29. Dependabot PR #55 (12 cargo updates) was blocked by the red-CI credential-hook step, which the 7ca7d40 fix resolves after the main fast-forward.

**Why out-of-scope for the resumption audit:** Merging #55 requires main CI green (post fast-forward) plus a cargo build/test cycle — sequenced work under the one-cargo-at-a-time budget, not an intake-sweep action.

**Sketched resolution:** Rebase/merge dependabot #55 once main CI is green after the fast-forward; verify both advisories clear (`cargo audit` / Security-audit cron re-run); confirm the cron itself goes green.

**STATUS:** OPEN | 2026-07-05 debt-drain triage: PARTIALLY RESOLVED / RE-SEQUENCED. Ground truth (verified via `gh`): PR #55 (the bundled 12-update group) is now **CLOSED**, superseded by individual dependabot PRs #64 (tower-http), #65 (gix), #66 (rusqlite), all OPEN + mergeable. The "Security audit" (cargo-audit) cron is GREEN on all three branches as of 2026-07-05. `memmap2 0.9.11` and `quinn-proto 0.11.15` are both present in `Cargo.lock`. The definitive re-confirmation that RUSTSEC-2026-0186 (memmap2) + RUSTSEC-2026-0185 (quinn-proto) are cleared requires a `cargo audit` run naming those IDs — DEFERRED to a cargo-holding lane (this debt-drain window runs under a no-cargo firewall). Net: the original "rebase/merge PR #55" action is now MOOT (PR dead); replacement action = land PRs #64/#65/#66 (owner-gated merges). Kept OPEN pending that cargo-audit confirmation.

## 2026-07-03 11:20 | discovered-by: resumption audit (8-week idle gap) | severity: LOW

**What:** 9 stacked weekly bench-refresh PRs (#40..#58) with CI stuck `action_required`; the cron produces PRs faster than they get merged.

**Why out-of-scope for the resumption audit:** Batch merge-or-close plus a policy decision (auto-merge for cron PRs) deserves a deliberate pass, not an intake-sweep side effect.

**Sketched resolution:** Merge-or-close the batch (newest-wins for bench data; close the superseded ones), then consider an auto-merge policy for the cron so the stack cannot rebuild.

**STATUS:** OPEN

## 2026-07-03 11:25 | discovered-by: resumption audit (8-week idle gap) | severity: LOW

**What:** 5 dependabot GH-Actions bumps (#35–#39) parked since 2026-05-02; Node-20 deprecation warnings already appearing in workflow runs.

**Why out-of-scope for the resumption audit:** Same PR-drain shape as the bench-refresh stack — needs CI-green main first (post fast-forward) and a merge pass.

**Sketched resolution:** Merge the 5 action-version bumps in the same PR-drain batch as the bench-refresh cleanup (low-risk, workflow-only diffs); verify the Node-20 deprecation warnings disappear from subsequent runs.

**STATUS:** OPEN

## 2026-07-03 11:30 | discovered-by: resumption audit (8-week idle gap) | severity: MEDIUM

**What:** Doc staleness cluster: `PROJECT.md:5` still says "git-backed FUSE filesystem" (pre-v0.9.0-pivot); PROJECT.md pins gix 0.82 and points Active requirements at v0.12.0; README "Project status" stops at v0.10.0; `.planning/MILESTONES.md` top entry is v0.10.0; CLAUDE.md tech-stack says axum 0.7 / rusqlite 0.32 (actual: axum 0.8 / rusqlite 0.39) and its workspace-layout omits the 10th crate `reposix-quality`.

**Why out-of-scope for the resumption audit:** A cross-file doc-refresh sweep with doc-alignment catalog implications — a phase-sized pass, not an intake fix.

**Sketched resolution:** Single doc-refresh sweep correcting all named locations in one commit series; natural home P95 (docs/UX phase) or P96 (surprises absorption). Run the doc-alignment walker after so drifted rows re-grade.

**STATUS:** OPEN

## 2026-07-03 11:45 | discovered-by: resumption audit (8-week idle gap) | severity: LOW

**What:** Owner-voiced quality controls with no gate yet: (a) a gitignore-hygiene check, and (b) a file-ownership / file-local-instructions check (e.g. what README owns vs CLAUDE.md vs code comments). Improvement-shaped (GOOD-TO-HAVES territory per this file's header) but filed here per owner routing during the resumption audit.

**Why out-of-scope for the resumption audit:** New gates are catalog-row + verifier work under the framework's extension contract — phase work, and the framework itself is mid-fix in P89/P90.

**Sketched resolution:** Mint as catalog rows + verifiers in a good-to-haves slot (P97) — one catalog row + one verifier each per the framework's own extension contract ("Adding a new gate is one catalog row + one verifier in the right dimension dir").

**STATUS:** OPEN

## 2026-07-03 21:00 | discovered-by: P89 orchestrator (CI triage) | severity: LOW

**What:** CI annotations noise: ENOENT opendir 'target/tests/target' in test + coverage jobs (jobs green; some uploader glob). Observed on run 28692818500.

**Why out-of-scope for P89:** Cosmetic CI-annotation noise from an uploader glob; zero functional impact; not framework-fix scope.

**Sketched resolution:** Locate the uploader step whose glob expands to `target/tests/target` and tighten the pattern (or create the dir). Home: P95 polish.

**STATUS:** OPEN

## 2026-07-03 21:00 | discovered-by: P89-01 | severity: LOW

**What:** `scripts/check-quality-catalogs.py` stale: ROW_REQUIRED demands legacy scalar `cadence`; VALID_KINDS lacks `shell-subprocess`; VALID_CADENCES lacks `pre-release-real-backend` + `pre-commit`. Invoked by no hook/CI (on-demand meta-helper), so nothing regresses. Source: 89-01 executor report.

**Why out-of-scope for P89-01:** The catalog-first commit's contract is minting rows, not refreshing an unwired meta-helper; the fix is sanctioned for fold-in only if 89-03/89-04's plans touch the script.

**Sketched resolution:** Update ROW_REQUIRED to `cadences: list`, add `shell-subprocess` to VALID_KINDS and `pre-release-real-backend` + `pre-commit` to VALID_CADENCES — fold into 89-03/89-04 if sanctioned, else P95.

**STATUS:** OPEN

## 2026-07-03 21:35 | discovered-by: 89-07 | severity: HIGH

**What:** RBF-FW-11's date-cutoff design (89-CONTEXT.md D-11c: "rows with `last_verified is None` OR `>= cutoff` are subject to the `claim_vs_assertion_audit` check; pre-cutoff legacy rows PASS unconditionally, so the 388 legacy rows keep validating without backfill") rests on an assumption that breaks in two independent ways once implemented: (1) the `doc-alignment.json` catalog (`dimension: docs-alignment`, ~393 rows) uses an entirely distinct per-row schema documented in `quality/catalogs/README.md` § "docs-alignment dimension" (`last_verdict`/`last_extracted`, no `last_verified` key at all) — treating "key absent" as "None → subject" would have demanded the audit field on all 393 legacy docs-alignment rows immediately, breaking runner load for every cadence. (2) Independent of docs-alignment, the runner's own `catalog_dirty()`/rollback logic (`run.py` main()) deliberately never persists `last_verified` back to the catalog file for rows whose status hasn't changed between runs (to avoid git-diff timestamp churn) — so a genuinely long-PASSing row like `structure/release-plz-disables-gh-releases` legitimately has `last_verified: null` forever by design, which the "null → subject" rule would misclassify as "brand new, needs the field." Five real (non-docs-alignment) rows were caught by this at the moment RBF-FW-11 landed: `docs-reproducible.json::benchmark-claim/{8ms-cached-read,89.1-percent-token-reduction}` (null, weekly cadence), `freshness-invariants.json::structure/release-plz-disables-gh-releases` (null, pre-push/pre-pr), `subjective-rubrics.json::subjective/dvcs-cold-reader` (null, pre-release), and `freshness-invariants.json::structure/file-size-limits` (real `last_verified: 2026-05-09T05:40:12Z`, i.e. genuinely post-cutoff and genuinely missing the field — a real process gap, not a schema-mismatch artifact). Caught by `pytest quality/runners/test_freshness_synth.py` going RED and by a direct `load_catalog()` sweep over all 11 discovered catalogs.

**Why out-of-scope for silent backfill vs. filing:** The fix for (1) — skip the validator entirely for `dimension == "docs-alignment"` — was applied in 89-07 itself (both `load_catalog()`'s call site and the README schema note) because it's a straightforward, well-justified 3-line guard consistent with the README's own documented schema split, and leaving it unfixed would have broken runner loading for every cadence, every catalog, forever. For (2), only 5 rows were affected (not the full 388-row legacy corpus P95 RBF-D-06 owns) — 89-07 backfilled accurate `claim_vs_assertion_audit` text on all 5 (reading each row's actual verifier/asserts to write faithful falsification text) rather than leave `main` pre-push-broken, since 89-07's own commit is what turns the enforcement on. This is filed for visibility rather than silently absorbed because it reveals the underlying assumption in 89-CONTEXT.md D-11c ("date-gating alone keeps the 388 legacy rows validating, no backfill needed") is false for any catalog using a schema without `last_verified`, and for any long-PASSing row whose timestamp the runner intentionally never persists — a structural gap in the date-cutoff design, not a one-off data-quality miss.

**Sketched resolution:** P95 RBF-D-06 should treat "grandfathered" as "predates the RBF-FW-11 commit" rather than "has an explicit pre-cutoff `last_verified` value" — e.g. by recording the git blob OID or commit SHA of the RBF-FW-11-landing commit and grandfathering any row present in the catalog at that SHA, OR by accepting that `docs-alignment`-dimension catalogs are permanently exempt (already applied) and that any FUTURE row surfaced by the null-check needs either (a) a real re-verification cycle that flips its status (persisting a fresh `last_verified`), or (b) manual backfill at the time it's noticed, as 89-07 did for these 5. Also worth a P95 read: whether `catalog_dirty()`'s "don't persist last_verified unless status changed" optimization should be revisited now that a schema check depends on that field being populated.

**STATUS:** OPEN

## 2026-07-04 05:30 | discovered-by: steward-window (post-P89, PR #58 triage) | severity: LOW

**What:** Weekly bench-cron PR #58 proposed a `docs/benchmarks/latency.md` refresh whose `reposix init cold` row looked column-shuffled: sim 27ms→343ms and confluence 508ms→26ms (a fast-local backend suddenly slower than a real-network one). Investigation **rules out a generator column-shuffle bug**: `quality/gates/perf/latency-bench/emit-markdown.sh` assembles the table from per-backend named variables (`SIM_*`/`GH_*`/`CF_*`/`JR_*`) into a fixed `sim|github|confluence|jira` column order — cross-column misalignment is structurally impossible, and the body rows (List/Get/PATCH) stayed correctly aligned (confluence's newly-populated cells hold network-plausible values). The real cause: the `init cold` cell is a **single sample** (`latency-bench/sim.sh:12` "init cold — single sample"), unlike the median-of-3 body rows. sim always runs first and eats one-time cold-process/disk/git-index warmup (343ms, still within the documented `<500ms` WARN band, `latency-bench.sh:34`); real-backend inits then run warm and, with `--filter=blob:none` (blobs=0, lazy), are network-cheap (~26ms). The prior run's 508ms confluence was the inverse warmup landing. So the numbers are **legitimate-but-misleading**, not a regression and not a bug.

**Why out-of-scope / why #58 was closed unmerged:** Not a generator bug, so the steward-window "fix-if-<1h-generator-bug" branch didn't apply. Merging would publish unverified cron numbers (cron PRs get no CI per the workflow's `GITHUB_TOKEN` trigger limitation) into the single most adoption-facing benchmark, with a credibility-denting sim-slower-than-confluence inversion in the headline row. #58 closed with an explanatory comment; the cron regenerates weekly and (after the steward-window `branch-suffix: timestamp` removal, commit 2df3d4a) now updates one stable `bench/refresh-latency` PR in place.

**Sketched resolution:** The underlying weakness is that `init cold` is single-sample + run-order-dependent, so it's an unreliable per-backend headline. Options for a future perf phase (P95/P97): (a) warm-up the first init (spawn a throwaway init before timing, mirroring the TLS warm-up the body rows already do) so column 1 doesn't eat global warmup; (b) take median-of-3 for init like the other rows; or (c) footnote the row as order-sensitive and drop it from any headline. Owner-verified real-backend re-measurement (the same north-star fix as the headline-numbers rows above) would settle whether confluence init is genuinely ~26ms or the 508ms was real.

**STATUS:** OPEN

## 2026-07-04 18:10 | discovered-by: quality-convergence connector re-audit | severity: HIGH

**What:** reposix-swarm has zero write-contention coverage for the one real backend it drives directly. `confluence_direct.rs` never calls `create_record`/`update_record` even though Confluence writes have been live for over a milestone — the harness's entire purpose is multi-agent contention testing, and the write-contention path (the risky one: version conflicts, push-time drift) is untested by it. (The stale "Phase 17 read-only by design — writes ship in Phase 21 / OP-7" comment is being fixed eagerly in the convergence fix wave; this entry tracks the missing workload.)

**Why out-of-scope for eager-resolution:** designing a write-contention workload (N agents racing update_record on shared records, asserting version-conflict handling + audit-row completeness) is M-sized test-harness work with real-backend etiquette concerns (TokenWorld mutation volume).

**Sketched resolution:** add a swarm write-contention scenario against the simulator first (default per OP-1), then an --ignored real-Confluence variant against TokenWorld per docs/reference/testing-targets.md cleanup conventions. Home: P91 (real-backend wiring) or P95.

**STATUS:** ROUTED-P95 | Per D91-12: swarm write-contention workload is explicitly routed OUT of P91 (do not absorb). P91's real-backend charter was `attach`/`sync` dispatch (D91-03), not the swarm harness; designing the N-agent write-contention scenario (sim-first, then an `--ignored` real-Confluence variant) remains M-sized test-harness work. Entry stays open at P95 with the home confirmed (not "P91 or P95" — P91 close makes it P95, full stop).

