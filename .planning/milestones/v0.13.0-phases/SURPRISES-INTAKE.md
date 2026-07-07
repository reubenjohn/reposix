# v0.13.0 Surprises Intake (P96 source-of-truth)

> **CARRY-FORWARD BANNER — 2026-07-06 pre-v0.13.0-tag sweep.** v0.13.0 is **CLOSED-GREEN, tag imminent.** Every entry still marked **OPEN** below is a **live carry-forward** — it survives to the post-tag **v0.14.0 / v0.13.2 scoping session** for re-triage, NOT a v0.13.0 action item. A STATUS line that cites a now-closed **P9x** phase (P95, P97, …) means "deferred past that closed phase, now pending v0.14.0 re-triage" — the phase ref is historical, not a live target. Terminal (resolved / verified-clean) entries were DELETED this sweep — git is the archive (bound-to-live-state). Do NOT spin up a `v0.14.0-phases/` dir to hold these; they stay here until that scoping session ingests them.

> **Append-only intake for surprises discovered during P78-P96 execution.**
> Each entry is something the discovering phase chose NOT to fix eagerly because it was massively out-of-scope. **P96 (OP-8 Slot 1) drains this file** (was P87 in the original P78–P88 plan; renumbered when the milestone extended to P78–P97).
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering phase without doubling the phase's scope (rough heuristic: < 1 hour incremental work, no new dependency introduced, no new file created outside the phase's planned set), do it there. The intake file is for items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN, RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md` (P97, OP-8 Slot 2).

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: P<N> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for P<N>:** Why eager-resolution wasn't possible (scope, time, dependency).

**Sketched resolution:** One paragraph proposing how P87 should resolve.

**STATUS:** OPEN  (← P96 updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

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

## 2026-07-04 | discovered-by: P90 90-03 (confirmed live by 90-05) | severity: MEDIUM

**What:** All 4 subjective-rubrics rows' `verifier.args` pass bare rubric slugs (`cold-reader-hero-clarity`, `install-positioning`, `headline-numbers-sanity`, `dvcs-cold-reader`) to `--rubric`, but `.claude/skills/reposix-quality-review/dispatch.sh`'s case statement keys on the FULL `subjective/<slug>` id (`"subjective/cold-reader-hero-clarity")`, `"subjective/install-positioning"|"subjective/headline-numbers-sanity"`, `"subjective/dvcs-cold-reader"`). A bare-slug invocation path falls through to no matching case (and, upstream, a runner/catalog `find_row` lookup keyed on the full id would `KeyError`/miss for a bare slug) — the two spellings are inconsistent across the catalog-row/dispatcher boundary. Confirmed live (not hypothetical) by both 90-03 (which wired the `dvcs-cold-reader` dispatch case) and 90-05 (which renewed the 3 subjective waivers and re-read the same `verifier.args` while doing so).

**Why out-of-scope for P90:** P90's mandate is the honesty-rules framework fixes (RBF-FW-06..12); reconciling a pre-existing bare-slug-vs-full-id inconsistency in the dispatcher/catalog contract is a small but distinct wiring fix, not one of the chartered honesty rules, and touching the dispatcher script is outside 90-03/90-05's task envelopes.

**Sketched resolution:** Normalize on full row ids (`subjective/<slug>`) in every row's `verifier.args`, OR make `dispatch.sh`'s case statement accept both the bare slug and the full id (e.g. strip a leading `subjective/` before the case match). Either way, add a wiring smoke test that invokes each of the 4 rows' exact `verifier.script` + `verifier.args` and asserts the dispatcher recognizes the rubric (not just that the rubric name appears somewhere in the script). Home: P92 (or the next quality window that touches the dispatcher).

**STATUS:** OPEN

## 2026-07-04 | discovered-by: P91 91-03 (D91-09 token-scrub) | severity: LOW

**What:** `quality/gates/structure/banned-production-tokens.sh` catches only the suffixed phase-ID shape `\bP\d{2,3}-\d+\b` (e.g. `P79-02`, `P83-01`). The no-suffix `P\d{2,3}\+` shape (e.g. `P82+`, `P83+`) — used as a "lands in P82 and later" forward-reference — is NOT caught by any structure gate (framework research B(f); confirmed this pass: `P82+` in `sync.rs`/`reconciliation.rs` sailed past pre-push before P91-03 scrubbed it by grep + the deferral-pointer linter, never by the banned-token gate). Only the *deferral-pointer linter* (`lands? (alongside|in) P\d+`) covers a subset of these phrasings; a bare `action=FORK_AS_NEW (TODO P82+)` inside an eprintln string was covered by neither.

**Why out-of-scope for eager-resolution in P91-03:** extending the regex to `\bP\d{2,3}[-+]\d*\b` (or adding a `P\d{2,3}\+` alternative) is NOT cheap: existing production comments carry `P82+`/`P83+` forward-references across `reposix-remote/src/{main.rs:125, precheck.rs, bus_handler.rs}` (historical, some without `// banned-words: ok` markers). Turning the regex on now would newly BLOCK pre-push on legitimate historical comment lines in files outside P91-03's envelope — a cross-file marker sweep + regression risk, not a one-line change. Per D91-09 ("intake entry if not cheaply extendable").

**Sketched resolution:** In a structure-dimension window (P97 good-to-haves, or whenever `banned-production-tokens.sh` is next touched): (1) add the `P\d{2,3}\+` alternative to `PATTERN`, (2) sweep `crates/**/*.rs` for the now-caught historical hits and either reword them to drop the phase ID or add a per-line `// banned-words: ok` marker with rationale, (3) update the CLAUDE.md "Banned-token regex scope" section + the script header to document the widened shape. Grep target for the sweep: `grep -rnE 'P[0-9]{2,3}\+' crates --include='*.rs' | grep -v tests/`. Home: P97 (or the next structure-gate touch).

**STATUS:** OPEN

## 2026-07-04 21:00 | discovered-by: P91 91-05 (vision-litmus real-run) | severity: LOW

**What:** Two smaller findings from the real TokenWorld run. (a) `git-remote-reposix` emits a stray `git-remote-reposix: unknown command: feature` line after an egress-denied bus-push rejection — the helper doesn't gracefully consume git's post-rejection `feature` capability line; cosmetic but confusing next to the (good) teaching string. (b) `docs/reference/testing-targets.md` (D91-08) frames the durable fixtures as living in "the REPOSIX space ... same tenant as TokenWorld", implying two DISTINCT spaces — but both the `TokenWorld` and `REPOSIX` space keys resolve to the SAME Confluence space (id 360450, name "TokenWorld reposix demo space"). So the "never delete 7766017/7798785" constraint is load-bearing WITHIN the litmus's own mutation target, not a cross-space nicety — worth stating explicitly so a future cleanup sweep in "TokenWorld" understands it can hit the durable fixtures.

**Why out-of-scope for P91 91-05:** (a) is in `reposix-remote`; (b) is a `docs/reference/testing-targets.md` edit — both outside the litmus-only file envelope.

**Sketched resolution:** (a) have the helper consume/ignore the trailing `feature` line on the reject path (or map it to the same teaching context). (b) add one sentence to testing-targets.md: "The `TokenWorld` and `REPOSIX` space keys are two aliases for the SAME space (id 360450); the durable fixtures 7766017/7798785 live in it — any sweep of either key name must spare them." Home: P92/P97 docs+helper touch.

**STATUS:** OPEN

## 2026-07-04 22:15 | discovered-by: P91 litmus-REOPEN (repro setup + peer session) | severity: LOW

**What:** When `git-remote-reposix` is pointed at a dead origin (nothing listening on the host/port — e.g. a working tree whose `remote.origin.url` names a sim port with no live sim), a `git push`/`git fetch` HANGS indefinitely rather than failing fast with a teaching error. Observed live during the P91 mass-delete reproduction: a tree inited against `:7878` with the sim actually on `:7893` left the helper stuck for 2.5+ minutes (peer session confirmed via `ss -tlnp` that nothing listened on `:7878`). Likely cause: the helper's reqwest fetch/precheck path has no connect timeout, so a non-listening (or firewalled) origin blocks on TCP connect with no ceiling and no operator-readable diagnostic.

**Why out-of-scope for the litmus-REOPEN fix:** the fix envelope is the diff/fast_import no-commit guard; adding a connect timeout + teaching error touches the `reposix_core::http` client factory and every REST call site's error mapping — a separate robustness change.

**Sketched resolution:** set a bounded connect timeout (e.g. 10s) on `reposix_core::http::client()`, and map a connect failure to a teaching stderr line naming the unreachable origin + the likely fix ("is the sim/backend running at <host:port>? check `remote.origin.url`"). Add a test that points the helper at a closed port and asserts it exits non-zero within the timeout with the teaching string, rather than hanging. Home: a security/robustness or perf window that touches `reposix_core::http`.

**STATUS:** OPEN

## 2026-07-04 | discovered-by: P91 91-02 (deferred-items.md), reconciled during 91-06 docs wave | severity: MEDIUM

**What:** `quality/catalogs/agent-ux.json`'s two +2-reservation drain-verifier rows — `agent-ux/p87-surprises-absorption` (`last_verified: 2026-05-01T22:00:00Z`) and `agent-ux/p88-good-to-haves-drained` (`last_verified: 2026-05-01T22:30:00Z`) — neither carries `claim_vs_assertion_audit`, and reproducing `python3 quality/runners/run.py --cadence on-demand` right now (2026-07-04) shows both FAIL with the exact `_audit_field.py` schema message: `row agent-ux/p87-surprises-absorption missing claim_vs_assertion_audit (>=50 chars required for rows minted on/after 2026-05-08T00:00:00Z)` (and the p88 sibling identically). This is surprising because both rows' `last_verified` predates the 2026-05-08 cutoff and neither carries `minted_at` — the legacy-exemption heuristic in `validate_row` (`is_new = lv is None or parse_rfc3339(lv) >= cutoff`) reads, on its face, like it should treat these as pre-cutoff legacy rows exempt from the field requirement. It doesn't, live, today. Root cause NOT diagnosed here (would require stepping through `_audit_field.py`'s date-comparison path or checking for a second code path this session didn't find) — flagging the discrepancy between the code's apparent intent and its observed behavior is the finding; full root-cause is P96/P97 scope. Confirmed NOT a catalog-load-crashing SystemExit in practice: `run.py` completes its full sweep (6 PASS, 4 FAIL, 0 PARTIAL, 1 WAIVED, 1 NOT-VERIFIED, exit=1) rather than hard-aborting — so this is a per-row FAIL, not the "hard-blocks catalog load" framing in the original 91-02 deferred-items.md note (worth correcting: it degrades gracefully to FAIL, it does not crash the runner).

**Why out-of-scope for eager-resolution (91-06):** 91-06 is a docs-only wave (no cargo, no catalog-mutation authority beyond what a phase's own rows require); root-causing `_audit_field.py`'s cutoff logic plus writing genuine >=50-char audit paragraphs for two pre-P90 legacy rows is a framework-honesty fix, not a doc fix, and belongs with the other P90-era catalog-honesty carryovers this project already routes to P95-and-later windows.

**Sketched resolution:** (a) root-cause why `is_new` evaluates true for these two rows despite `last_verified` predating the cutoff and no `minted_at` present — likely candidates to check first: a stray whitespace/format difference in the stored RFC3339 string tripping `parse_rfc3339`, or a second validation call site not read during this pass; (b) once understood, either fix the legacy-exemption logic (if the bug is in `_audit_field.py`) or backfill genuine `claim_vs_assertion_audit` prose for both rows (if the two rows are legitimately being held to the new-regime bar) explaining how the P87/P88 drain-verifier assertions would fail loud if their claims were false. Either fix is small once root-caused; the root-cause step itself is the M-sized unknown.

**STATUS:** DEFERRED-P96/P97 | Filed per 91-06 Task 5(i). Home: the next framework-fixes or catalog-honesty window (P96 or P97, whichever lands the next `quality/runners/_audit_field.py`-touching phase). Not blocking today's pre-push/pre-commit (both rows are `on-demand` cadence only, per their catalog entries), but the schema drift is real and reproducible right now — don't let a future `--cadence on-demand` run's FAIL surprise the next reader who assumes these two milestone-bounded historical rows are inert.

## 2026-07-05 | TokenWorld two-writer conflict verifier does not exist — SC1 real-backend arm cannot be verified until built | discovered-by: P92 SC1 adjudication (D-P92-03) | severity: HIGH

**What:** The P97 9th probe (`pre-release-real-backend`) MUST exercise a real-backend two-writer CONFLICT (reject → pull --rebase → push), not just single-writer push, to close SC1's real-backend arm. P92 SC1 accepted this gap by design (coverage_kind: real-backend, verified at P97 only). Both P92 executors independently identified building this as a genuine new artifact unwise to rush late-session: requires git>=2.34 container + a Confluence conflict fixture on the TokenWorld space + cleanup harness.

**Why out-of-scope for P92:** SC1's designed split (sim arm GREEN now via T4 litmus; real-backend arm NOT-VERIFIED by design) matches ROADMAP coverage-kind semantics. The two-writer conflict verifier is a P97 deliverable, not a P92 scope miss — filed to chain visibility into the P97 9th probe.

**Sketched resolution:** Build a replica of the `agent-ux/t4-conflict-rebase-ancestry-real-backend` verifier that drives a REAL two-writer scenario against TokenWorld: (1) init + sync against the Confluence backend, (2) writer A edits record X, pushes (succeeds), (3) writer B performs the same edit with a stale base, pushes (correctly rejected: version mismatch), (4) writer B runs `git pull --rebase origin main` (FULL round-trip), (5) conflict resolves cleanly (ordinary textual conflict from 3-way merge proves blob fetch succeeded), (6) writer B's recovery push succeeds. Verifier script template: `quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh` (sim arm) extended with a Confluence fixture create/cleanup arm + real-backend URL substitution. The fixture must be durable-safe (per `docs/reference/testing-targets.md` cleanup conventions) or self-seeding (create/assert/delete within the harness like the updated `contract_confluence_live_hierarchy` test).

**Why deferred:** both P92 executors noted the new-artifact risk (schedule+complexity, not a high-confidence one-liner). The sim arm is proven GREEN; deferring the real-backend verifier to P97's final probe (which mandates it) is the OP-8 eager-resolution principle applied to sequencing: do not rush low-confidence, high-stakes shipping checks.

**STATUS:** OPEN — P97 Wave A reconciliation, 2026-07-05: CONFIRMED KEPT OPEN. The two-writer
CONFLICT-replay verifier still does not exist. The P97 milestone-close 9th probe
(`pre-release-real-backend`) will read **NOT-VERIFIED this run** — the autonomous window has
no TokenWorld creds, and OD-2's fail-closed rule is creds-missing ⇒ NOT-VERIFIED (never
skip-as-pass). Full conflict-replay coverage (reject → `git pull --rebase` → push against a
real Confluence fixture) is to be verified on a **live owner run or v0.14.0**; building it
late-session against a live tenant remains the new-artifact risk both P92 executors flagged.

## 2026-07-05 debt-drain triage

**Scope:** A tree-writer session running interleaved with the active P93 phase lane (no cargo, disjoint files from P93's `crates/`+`quality/` code) reviewed the intake backlog and recorded dispositions below. No items were silently skipped; each was either resolved, re-sequenced, or confirmed correctly routed.

- **Dependabot PR #55 + RUSTSEC** (2026-07-03 11:15 entry above): disposition updated in place on its own STATUS line — PARTIALLY RESOLVED / RE-SEQUENCED (PR #55 closed/superseded by #64/#65/#66; cargo-audit re-confirmation of the two RUSTSEC IDs deferred to a cargo-holding lane).
- **Catalog date-cutoff schema gap** (2026-07-03 21:35 / 89-07 entry, "OPEN" above): LEFT AS-IS. Already correctly routed to P95 RBF-D-06 per that entry's own Sketched resolution paragraph; no debt-drain action needed.
- **git 2.43.0 breaks single-backend push** (2026-07-05 / P92 T4 entry above): LEFT AS-IS. Already tagged for P94 (bus-push compatibility); no debt-drain action needed.
- **TokenWorld two-writer conflict verifier missing** (2026-07-05 / P92 SC1 entry above): LEFT AS-IS. Already routed to the P97 9th probe by design; no debt-drain action needed.

See the companion `GOOD-TO-HAVES.md` for the same-window disposition of the P92-filed security-gate good-to-haves (one resolved this window, one deferred, one left as-is) and the follow-on "branch hygiene + PR triage" entry appended below for owner-gated repo housekeeping.

## 2026-07-05 debt-drain: branch hygiene + PR triage (staged for owner)

**What:** A tree-writer session (same window as the triage above) ran a remote-branch + open-PR inventory as housekeeping. Findings, transcribed as verified:

- **Remote-branch inventory:** 9 branches total (NOT the ~32 previously assumed). Safe-to-delete list = EXACTLY ONE: `release-plz-2026-05-01T03-32-29Z` (its PR #32 is CLOSED/superseded by #61; 1 orphaned release-plz commit, 2 months stale). All others KEEP: `main`, `gh-pages`, the 4 dependabot branches backing OPEN PRs #62/#64/#65/#66, `release-plz-2026-07-04T05-03-11Z` (backs OPEN PR #61), and `workstream/v0.13.2` (holds 2 UNMERGED commits `cf79fd4` + `c4ed713` = P98-CONTEXT.md + P98-DISCUSSION-LOG.md, 285 lines, NOT on main — do NOT sweep; decide at P98 kickoff whether to cherry-pick or regenerate).
- **Deletion is OWNER-GATED** (external mutation per CLAUDE.md's "Uncommitted = didn't happen... External mutations need owner-named-target approval") — STAGED, not executed. Owner action: `git push origin --delete release-plz-2026-05-01T03-32-29Z` (only after confirming PR #61 supersedes #32's intent).
- **PR #62** (codecov-action 6→7): all 16 checks green, mergeable — merge proposal STAGED for owner: `gh pr merge 62 --squash --delete-branch`. Owner-gated, not executed.
- **Note:** the "quality gates (pre-pr)" FAILURE currently showing on PRs #64/#65/#66 is NOT a defect in those dependency bumps — it's the parallel P93 lane's own catalog-first rows (commit `543bfb4` added 3 `agent-ux/p93-*` catalog rows whose verifiers don't exist yet → NOT-VERIFIED → exit=1). Self-resolves once P93 ships its verifiers. PR #62 predates `543bfb4` so it reads green.
- **Note (cosmetic debt, filed below as a new GOOD-TO-HAVE):** `.git/hooks/pre-push` is a BROKEN symlink → nonexistent `scripts/hooks/pre-push` (confirmed: `ls` on the target errors ENOENT), but it's INERT because `core.hooksPath=.githooks` overrides it — the real active hook is `.githooks/pre-push`. Follow-up filed as a new low-sev entry in `GOOD-TO-HAVES.md` to delete the dead symlink for tidiness.

**Why staged, not executed:** branch deletion and PR merges are external mutations against the shared remote (`origin`) outside this window's file-editing charter — owner-named-target approval required per CLAUDE.md's dark-factory guardrails.

**STATUS:** OPEN (staged for owner action)

## 2026-07-05 | `quality/gates/docs-build/mkdocs-strict.sh` under-reports broken internal anchors (swallowed at INFO log level) | discovered-by: P93 Wave 2a executor | severity: MEDIUM

**What:** `quality/gates/docs-build/mkdocs-strict.sh` runs `mkdocs build --strict` and
then greps the build log + rendered HTML for the literal string `"Syntax error in
text"` (the mermaid HTML-entity failure mode) — but does nothing to catch broken
internal anchor/fragment links (e.g. `[text](page.md#stale-heading-slug)`). mkdocs's own
link-validation machinery logs anchor-resolution misses at `INFO` level, not
`WARNING`, so `mkdocs build --strict` (which only promotes `WARNING`-and-above to a
build failure) does not fail on them and the wrapper script's own grep never looks for
them either. Net effect: the gate is named `mkdocs-strict.sh` and is wired into
`pre-push`/`pre-pr` as the docs-build authority, but a broken `#anchor` fragment inside
`docs/**` can land on `main` with a fully GREEN `docs-build` dimension — the strict gate
silently under-reports exactly the class of link rot it implies it catches.

**Why out-of-scope for P93 (Wave 2a):** confirming and fixing this needs (a) a
reproduction — a synthetic doc with a genuinely broken internal anchor, run through the
real `mkdocs build --strict` to confirm the INFO-vs-WARNING behavior empirically rather
than from documentation of mkdocs's log-level defaults, and (b) a considered change to
either the script (parse `mkdocs build`'s INFO-level output for anchor misses and fail
on them) or the mkdocs config (a plugin/log-level override that promotes anchor-miss
INFO records to WARNING so `--strict` already catches them). Both are `docs-build`-gate
surgery requiring a real `mkdocs build` run to verify the fix actually changes the
gate's behavior — orthogonal to Wave 2a's push-unblock + de-risk charter, and this
executor was the sole cargo/tree-writer for a different, narrower fix (Task A).

**Sketched resolution:** Promote broken-anchor detection to a real FAIL (preferred: grep
the build log for mkdocs's own anchor-miss message pattern — e.g. text containing
"contains a link to ... which is not found" or the specific phrasing mkdocs emits for
unresolved fragments — and `exit` non-zero when found, mirroring the existing
`Syntax error in text` grep pattern already in the script) or, at minimum, a `WARN` line
the runner surfaces in its summary output rather than a fully silent INFO log line
nobody reads. Add a synthetic-fixture regression test (a throwaway doc page with a
`[text](other.md#does-not-exist)` link) proving the tightened gate actually catches it,
mirroring the mermaid-regression fixture pattern already used for POLISH-03.

**Default disposition:** MEDIUM — fold into the P94–P97 debt-drain window or the next
`docs-build`-gate-touching phase; natural pairing with the already-filed `badges-resolve`
flake investigation (same dimension, same debt window).

**STATUS:** OPEN

---

## 2026-07-05 | Pagination-truncation safety of sync's `prune_oid_map` — a truncated `list_records()` can DELETE oid_map rows for LIVE records beyond the cap | discovered-by: P93 DP-2 REOPEN re-review (relayed via coordinator, independently re-verified) | severity: HIGH

**What:** D-P93-02's shipped fix (`meta::prune_oid_map`, commit `272882c`/`e246e84`)
DELETEs `oid_map` rows whose `issue_id` is absent from a `keep_ids` set built from
`self.backend.list_records(&self.project)`. But `list_records()` on the GitHub, JIRA, and
Confluence connectors can silently return a **truncated** `Ok(partial_list)` at a
pagination/size cap (`github/lib.rs` `MAX_ISSUES_PER_LIST=500` / `MAX_RAW_ITEMS_PER_LIST`;
`jira/lib.rs` non-strict `list_issues_impl`; the Confluence equivalent) — the caller has no
way to distinguish "the project only has 40 records" from "the project has 4000 records and
we truncated at 500." Feeding a truncated `keep_ids` set into the prune's DELETE wipes
`oid_map` rows for **live records that exist beyond the cap** — a real record now looks
ghost/deleted to `Cache::list_record_ids()`, and it recurs on EVERY sync, not once. The
sim backend (the project default per OP-1) never truncates, so every sim-run gate —
including all of P93's own GREEN test runs — is structurally blind to this. Completeness is
known-but-DROPPED inside all three real connectors: `BackendConnector::list_records`
returns a bare `Result<Vec<Record>>` with no completeness/has-more signal in its type.
Before `272882c` a truncated list only under-populated the working tree (an accepted,
documented HARD-02 tradeoff); the prune fix turned that same truncation into an active
data-loss operation. No existing test guards this (all P93 tests are sim-backed with small
record counts). Full analysis + a 4-option decision tree already drafted:
`.planning/phases/93-cache-coherence/93-RELIEF-HANDOFF.md` §4-6.

**Why out-of-scope for P93:** P93's charter (mint the missing verification artifacts +
close the phase) is not the place to make an architectural connector-contract decision.
The safe fix genuinely forks: either (a) an **E2 connector-contract change** — add a
completeness/`has_more` signal to `BackendConnector::list_records`'s return type (or a
sibling method) so the prune can be gated on "listing is known-complete" — or (b) a
**truncation-safe prune redesign** that restricts pruning to a dedicated full-paginated
reconcile path (e.g. `reposix sync --reconcile`, which already forces a full rebuild) and
skips pruning on the normal delta path entirely, or reverts to the pre-`272882c` Strategy 2
(reclassify delete-time `NotFound` as idempotent success — already filed separately above
as a deliberate, NOT-chosen defense-in-depth alternative). Both forks are real API-surface /
architectural decisions (Rule 4 territory), not a same-phase mechanical fix.

**Sketched resolution:** P94 should (1) build a mock/capped-backend test proving the data-
loss reproduction (not just a code-read assertion) and check whether any connector already
exposes a completeness signal a truncation-safe fix could key off, then (2) package an E2
consult with the four options already sketched in the RELIEF-HANDOFF (§6 step 2): **A** —
add a completeness signal to the `list_records` contract and gate the prune on it; **B** —
revert `272882c`, ship Strategy 2 instead; **C** — restrict the prune to the dedicated
full-paginated reconcile path only; **D** — a cap-count heuristic that skips the prune when
`keep_ids.len()` is suspiciously close to a known cap. Plan for an E2 consult in P94 rather
than self-deciding.

**Default disposition:** HIGH — real data-loss hazard against live upstream records once a
real-backend project exceeds its connector's pagination cap; P94 (real-backend frictions)
is the natural next phase to prove + package this for an E2 decision.

**STATUS:** OPEN

---

## 2026-07-05 | ROADMAP.md § "Phase 94"–"Phase 97" prose is STALE/orphaned vs the LIVE STATE.md cursor | discovered-by: P94 catalog-first planning lane | severity: MEDIUM

**What:** `.planning/milestones/v0.13.0-phases/ROADMAP.md` § "Phase 94: Bus-push
compatibility with documented mirror setup (Cluster D)" (RBF-C-01..07) and its
downstream P95–P97 prose describe work that no longer matches what those phases deliver.
STATE.md frontmatter — the LIVE machine-readable cursor — says `next_phase: P94 …
real-backend frictions (pagination-truncation E2-fork + git-2.43 fallback-sentinel)`. The
bus-push / mirror-setup / Cluster-D work the ROADMAP prose describes already shipped in
P82–P86. A future planner grepping the ROADMAP for "Phase 94" would plan against
orphaned prose (RBF-C-* requirement IDs that no longer map to P94's true scope). P94's
PLAN.md carries an explicit `<scope-correction>` block flagging this so THIS phase does
not mis-execute, but the ROADMAP itself is still uncorrected.

**Why deferred (not fixed here):** the P94 catalog-first lane is a SPEC/mint lane
(markdown + catalog JSON only, no re-authoring of milestone ROADMAP prose). Reconciling
the ROADMAP's P94–P97 phase descriptions against the delivered reality is a milestone-close
docs-reconciliation task (freshness/structure dimension), which is exactly what the P96/P97
absorption + milestone-close slots exist for. Fixing it mid-P94 would be scope-creep on a
mint lane.

**Sketched resolution:** During the P96/P97 milestone-close docs reconciliation, rewrite
ROADMAP.md § Phase 94–97 to reflect delivered scope (P94 = real-backend frictions:
pagination-prune fix + git-2.43 fallback-sentinel + badge determination + freshness
sweep), retire or remap the orphaned RBF-C-* requirement IDs (confirm they were satisfied
in P82–P86 or move them to their true home), and reconcile the P95–P97 prose against
STATE. Cross-check REQUIREMENTS.md traceability so no RBF-C-* row points at a phase that
never touched it.

**Default disposition:** MEDIUM — no runtime impact, but a real mis-direction hazard for
the next planner; route to the P96/P97 milestone-close docs reconciliation (freshness /
structure dimension).

**STATUS:** OPEN

---

## 2026-07-05 | STATE.md frontmatter has no strict-YAML parseability guard — a bare `: ` regresses it silently | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW

**What:** `.planning/STATE.md` opens with a `---`-fenced YAML frontmatter block that is the
LIVE machine-readable cursor (`status`, `last_updated`, `workstreams.workstream_a.next_phase`,
`phases_completed`) consumed by `gsd-sdk` state handlers and by ad-hoc probes. Nothing in the
pre-commit / pre-push gate set asserts that block still parses as strict YAML, so a hand-edit
that introduces a bare `: ` (unquoted colon-space inside a scalar), a tab, or a mis-indented
key silently produces an unparseable block — the failure is invisible until a downstream
`yaml.safe_load` consumer chokes mid-session. This class of breakage has bitten before in the
P94 era (a bare `: ` around `eea309f`, since repaired). **Verified against reality this window:**
STATE.md frontmatter parses CLEAN on HEAD `889c922` (`status: executing-p95-post-close-drain`,
`next_phase: P96`, `phases_completed: 18`) — so this is a PREVENTIVE guard against silent
regression, not a live break.

**Why out-of-scope for P96 Wave 3a:** this window is planning-artifact hygiene (no new gate
wiring / catalog rows — those are Wave 3b + the verifier's territory, and adding a gate touches
`quality/`). Filing the guard sketch keeps the footgun visible for the next `quality/gates/`
window.

**Sketched resolution:** add a tiny `scripts/check-state-yaml.py` (or a `structure`-dimension
pre-commit row) that `yaml.safe_load`s the STATE.md frontmatter block and exits non-zero on any
parse error, naming the offending line. Cheap, deterministic, no cargo. Pairs naturally with the
STATE-vs-ROADMAP staleness entry (`ROADMAP.md § Phase 94–97 prose is STALE` above) — one is
semantic drift, this is syntactic breakage; both harden the single most load-bearing planning
cursor.

**STATUS:** OPEN

---

## 2026-07-05 | Committed catalog `status` lags the live grade between explicit `--persist` mints (by P96 design) | discovered-by: P96 Wave 3a (OP-8 Slot 1 hygiene) | severity: LOW

**What:** The P96 grade/persist split (D-P96-01 / CONSULT-DECISIONS `36dad20`) makes a bare
`run.py --cadence <c>` validate-only: it computes status in-memory + writes per-row artifacts
under `quality/reports/verifications/` (gitignored) but does NOT call `save_catalog`. This is
the correct fix for the recurring self-mutation bug. The DESIGNED consequence: any consumer that
reads the COMMITTED catalog `status` field — `verdict.py`'s badge rollup, dashboards, and
load-time phantom-green checks — sees the LAST-MINTED value, not the live grade, until the next
explicit `--persist` mint. Between mints the committed status can be stale relative to what the
runner just measured (related to the `Catalog-freshness sweep needed` entry above, which
observed the same staleness pre-P96 for a different root cause).

**Why out-of-scope for P96 Wave 3a:** filing only — the operational rule below is a runner-docs
+ milestone-close-checklist note, and this window does not touch `quality/` runners or catalogs.

**Sketched resolution:** (a) a one-line note in the runner docs (`quality/PROTOCOL.md` runner
section) stating that committed `status` is authoritative only as of the last `--persist` mint;
(b) an operational rule — milestone-close MUST run an explicit `run.py --cadence <c> --persist`
mint BEFORE reading the milestone verdict / regenerating badges, so the committed status the
verdict rolls up is fresh. LOW: no correctness hazard (gate integrity is preserved in-memory);
purely a "don't trust a stale committed badge between mints" reporting-freshness note.

**STATUS:** OPEN

---

## 2026-07-05 | `--persist` mint path should refuse to write a row it would reject at load (write-path load-refusal hardening) | discovered-by: P96 Wave 3a (residual split out of the D-P96-01 self-mutation fix) | severity: MEDIUM

**What:** The P96 grade/persist split (RESOLVED entry `Recurring quality-runner self-mutation
bug` above) fixed core requirement #2 (cadence runs never persist phase-row grades). Its named
residual — requirement #1, the `--persist` MINT path should REFUSE to write a row it would itself
reject at LOAD (e.g. a `minted_at`-less legacy row that `_audit_field.validate_row` would fail on
load) — is still OPEN and is filed here as its own item (the RESOLVED entry explicitly invited
"file as its own item if pursued"). Filed standalone so it stays visible in the working intake
rather than buried inside a now-terminal entry.

**Why out-of-scope for P96 Wave 3a:** this is a `quality/runners/run.py` write-path code change
(add a pre-`save_catalog` validation pass) with its own test obligation — a runner-touching fix,
orthogonal to this no-cargo hygiene window.

**Sketched resolution:** in the `--persist` branch, before `save_catalog`, run each to-be-written
row through the same `validate_row` predicate the loader applies; abort the mint (loud, naming
the offending row + reason) rather than persisting a row that the very next load would reject.
Closes the "mint writes an un-loadable row" asymmetry. Add a regression: `run.py --cadence <c>
--persist` over a catalog whose in-memory grade would produce a `minted_at`-less row must exit
non-zero and leave the file byte-identical. Route to the next `run.py`-touching quality-framework
window (P97 or v0.14.0).

**STATUS:** OPEN

---

## 2026-07-05 | `--persist` mint re-flips `subjective/*` rows off a STALE rubric artifact on every mint (pre-release cadence collateral churn) | discovered-by: P96 phase-close (post-close drain / verdict NOTICED review) | severity: MEDIUM

**What:** The P96 grade/persist split fixed the *validate-only* leak — cadence runs no longer
persist status flips. But the legitimate `--persist` MINT path still re-grades every in-scope row
in-memory, INCLUDING the `subjective`-kind rows (`subjective/dvcs-cold-reader` and its `pre-release`
siblings) whose status comes from a subagent RUBRIC artifact, not a mechanical verifier. That rubric
artifact is only refreshed by an explicit `/reposix-quality-review` dispatch (30-day TTL), so every
UNRELATED `--persist` mint recomputes the subjective row off the STALE artifact, flips its status,
and dirties `subjective-rubrics.json` with a spurious change the mint author must then hand-restore.
This bit the P96 mint and — left unfixed — will bite the **upcoming P97 milestone mint**: the
non-skippable `pre-release-real-backend` 9th probe runs `--persist` in exactly the cadence these
subjective rows live in. Distinct from the RESOLVED self-mutation bug (that was the validate-only
path) and from the `--persist` load-refusal entry above (that is about un-loadable `minted_at`-less
rows).

**Why out-of-scope for P96:** the clean fix is a `quality/runners/run.py` change (drop subjective
rows from the `pre-release` mint scope, OR make `--persist` treat manual/subagent-graded rows as
no-ops that preserve their prior status) carrying its own test obligation — a runner-touching change
orthogonal to the P96 no-cargo hygiene window, which deliberately left the mint path untouched beyond
the grade/persist split.

**Sketched resolution:** make `--persist` a **no-op for `kind: subjective`/`kind: manual` rows** —
never overwrite a subagent-graded status/`last_verified` from a mechanical mint; only the
rubric-dispatch path (which actually re-ran the rubric) may write them. Alternatively drop the
subjective rows from `pre-release` cadence membership so the milestone mint never touches them. Pairs
with GOOD-TO-HAVES-03's per-row `--row` filter (a scoped mint avoids fanning across subjective rows
at all). **Explicit P97 note:** until this lands, P97's milestone `--persist` mint MUST restore the
subjective-row collateral (git-checkout the spurious flips on `subjective-rubrics.json`) as a known,
expected step — do NOT let it ride into the tagged tree.

**STATUS:** OPEN — P97 Wave A reconciliation, 2026-07-05: CONFIRMED KEPT OPEN. The clean fix is a
`run.py` change (runner-touching, cargo) → **DEFERRED-v0.14.0**. **Load-bearing hand-off to Wave B:
the P97 milestone `--persist` mint MUST restore the `subjective-rubrics.json` collateral** —
git-checkout the spurious `subjective/*` status flips off the STALE (unrefreshed) rubric artifact so
they do NOT ride into the tagged tree. Wave A is planning-only and does not run the mint; this note
is the explicit instruction for whichever agent runs the Wave-B 9th-probe `--persist`.

---

## S-260706-rbf-01 — ADR-010 Consequences item-4 note is itself stale (LOW / cosmetic)

**Found during:** quick 260706-rbf (RBF-LR-03 known-limitation docs).
**Severity:** low / cosmetic.
**Issue:** `docs/decisions/010-l2-l3-cache-coherence.md` Consequences item 4 (~line 319) says
`docs/guides/troubleshooting.md:352` points at a dvcs-topology "Out of scope" anchor and says L3
"defers to v0.14.0" — but troubleshooting.md:352 now correctly reads "L3 … is shipped … Only L2 …
remains deferred." The P93 fix wave already fixed that doc line; the ADR's item-4 note now describes
a defect that no longer exists (the note is stale, not the doc).
**Sketch:** trim/annotate the item-4 note to reflect that the troubleshooting cross-ref was fixed.
NOT eager-fixed: editing a ratified ADR's Consequences section for a cosmetic staleness risks
rewriting decision history; belongs in an OP-8 drain, not a docs quick.

## S-260706-rbf-02 — CONSULT-DECISIONS T1 entry has an empty Commit field (LOW / cosmetic)

**Found during:** quick 260706-rbf.
**Severity:** low / cosmetic.
**Issue:** `.planning/CONSULT-DECISIONS.md` T1 tag-timing entry (~line 75) has `**Commit:** (this
entry; handover encodes the sequencing)` — no real SHA for a load-bearing sequencing decision.
**Sketch:** backfill the SHA of the commit that recorded the T1 decision. Ledger hygiene.

## S-260707-rbf-01 — `crlf_blob_body_round_trips_byte_for_byte` intermittent red on PR #61's `quality-pre-pr` job (HIGH / unresolved, non-timeout assertion failure — upgraded from MEDIUM 2026-07-07)

**Found during:** release-gating investigation for PR #61 (v0.13.0 release-plz branch), CI run
28819166220 / rerun job 85521914911.

**What was found:** `crates/reposix-remote/tests/protocol.rs::crlf_blob_body_round_trips_byte_for_byte`
failed inside the `quality gates (pre-pr)` job (via `quality/gates/agent-ux/p94-git243-fallback-sentinel.sh`
Arm 2's `CARGO_BUILD_JOBS=2 cargo test -p reposix-remote --test stateless_connect_e2e --test protocol`),
while the separate `test` job (`cargo test --workspace --locked`, full/unthrottled) passed on the same
commit. The test is a pure in-memory byte-assertion (ephemeral `wiremock::MockServer`, no real git
checkout/autocrlf path) — provably immune to the checkout-level causes CRLF bugs normally come from.

**Reproduction attempted (6 runs, all GREEN — mechanism NOT confirmed):**
1. `cargo test -p reposix-remote --test protocol` alone x3 — all green.
2. The gate's exact invocation (`CARGO_BUILD_JOBS=2 cargo test -p reposix-remote --test
   stateless_connect_e2e --test protocol`, warm `cargo build --workspace --bins --quiet` first) x3 —
   all green.
3. Same exact invocation with the full CI env matched (`CARGO_TERM_COLOR=always RUST_BACKTRACE=1
   CARGO_INCREMENTAL=0 RUSTFLAGS="-D warnings"`) x3 — all green.
4. Manual source review of `protocol.rs`: the CRLF test and its neighbors share no static/global
   state, no fixed port (each uses `MockServer::start()` — ephemeral loopback port), no shared temp
   dir. No plausible in-file race identified.
5. `--locked` divergence ruled out: the gate's unlocked, throttled, warm-build invocation reproduces
   cleanly every time locally; no dependency-resolution drift observed.

**Real root cause NOT confirmed** — could not reproduce the failure in this sandbox under any of the
above conditions. Leading (unconfirmed) hypothesis: CI-runner resource contention. The failing test is
the only one in the file that combines a real ephemeral TCP `wiremock` server + `tokio::task::spawn_blocking`
+ a real subprocess spawn, all under an `assert_cmd` `Command::timeout` (was 15s) — on a shared 2-vCPU
GH Actions runner running many tests concurrently (default thread-parallel `cargo test` across 9+3
tests), an occasional CPU-starved subprocess could plausibly blow a 15s wall-clock budget without any
actual byte-level regression.

**Why out-of-scope for eager full resolution:** the actual CI panic/assertion message for the failing
run was itself lost — `p94-git243-fallback-sentinel.sh` piped the full cargo-test log through
`tail -25` before printing it, and the tail window landed entirely inside the backtrace footer,
never reaching the panic line or assertion diff. Confirming the true mechanism requires catching a
live recurrence with full diagnostics, which this session could not force to reproduce.

**Eager-fixed in place (commit — see below), no new dependency:**
1. `quality/gates/agent-ux/p94-git243-fallback-sentinel.sh` — the cargo-test failure branch now
   archives the *full* log to `quality/reports/verifications/agent-ux/p94-git243-fallback-sentinel-cargo-test-failure.log`
   (gitignored) before truncating, and greps for `panicked at` / `failures:` / `assertion...failed`
   context plus a 60-line tail (previously 25, and *only* the tail) — the next occurrence will carry
   an actual diagnosable message instead of a bare backtrace footer.
2. `crates/reposix-remote/tests/protocol.rs` — bumped the 4 `wiremock`-backed `Command::timeout`
   calls (the CRLF test + the two H-03 500-response tests + the H-02 non-UTF-8 test) from 15s to 30s
   as a defensive headroom increase against shared-runner contention. This does NOT touch any
   assertion or expected value — only the wall-clock budget for a real subprocess+network round trip.

**Sketched resolution (if it recurs):** with the new full-log archive + panic-line grep in place, the
next CI recurrence will surface the actual panic/assertion text. If it turns out to be a genuine
timeout (subprocess killed mid-test), the 30s bump already applied may fully resolve it; if the fuller
log instead reveals a genuine logic bug, re-open with that evidence. Consider adding
`--test-threads=1` to the gate's cargo invocation if the doubled timeout still flakes, to remove
thread-parallelism as a contention vector entirely (narrower fix pending confirmed evidence, not
applied here since no shared in-file resource was identified to justify it up front).

**STATUS:** OPEN — mitigated (log capture + timeout headroom) but root cause unconfirmed; needs a
live recurrence with the new diagnostics to close definitively.

---

**2026-07-07 follow-up (RBF investigation, commit `aa2b33d`'s timeout-bump fix retested):**

**(a) Timeout-bump fix proven ineffective.** PR #61 (now head `deee8fd`, run `28837407948`, job
`85523987205`) hit the SAME failure again on `quality gates (pre-pr)` AFTER the 15s→30s timeout
bump from `aa2b33d` landed. `test result: FAILED. 8 passed; 1 failed; ... finished in 0.14s` —
the whole `protocol.rs` test binary (all 9 tests) finished in 140ms, nowhere near either the old
15s or new 30s timeout. **The timeout theory is dead.**

**(b) Confirmed a real, non-timeout assertion failure, not a race.** `timed_out: False` in the
verifier's own JSON, and the failing test's backtrace (frame 24/25) points straight at
`tests/protocol.rs:219:6` inside `crlf_blob_body_round_trips_byte_for_byte` (called from its
`{{closure}}` at line 154, the `tokio::test` body) — i.e. one of the two `assert!` calls after the
push succeeds (`stdout.contains("ok refs/heads/main")` at :203-206, or the CRLF-preservation check
at :216-219) is false. 0.14s wall-clock rules out any subprocess-hang / contention-timeout
explanation for either the original 15s or the bumped 30s budget.

**(c) New diagnostic finding: the actual panic/assertion message is STILL invisible in CI, and
now we know exactly why.** `quality/runners/dump_verifications.py` had `_TAIL_LINES = 40` —
it prints only the LAST 40 lines of each verifier's captured `stderr`. But
`p94-git243-fallback-sentinel.sh`'s failure branch (the "fix" from `aa2b33d`) prints
`grep -n -B2 -A15 'panicked at|failures:|assertion.*failed'` (containing the real panic text)
FOLLOWED BY a separate `tail -60` of the raw log. A 40-line tail-window falls entirely inside
that trailing 60-line block, so the grep context — the only place the actual panic message and
`body=...` diff lives — is discarded before it ever reaches the CI log. Confirmed by grepping the
full raw job log for `panicked`/`thread '`/`CRLF`/`body=` — zero matches anywhere in the printed
output, even though the gate script's own grep command (if its match had survived truncation)
would have caught it. **Fixed in commit `fbe5bee`** (pushed to `main`): bumped `_TAIL_LINES` to
200 with a comment explaining the truncation-window bug. This does not fix the underlying test
failure, but the NEXT CI recurrence (on any branch rebased past `fbe5bee`) will finally surface
the real assertion text.

**(d) Local reproduction still elusive.** Repeated the CI job's exact sequence again in this
session (`cargo build --workspace --bins --quiet` then `CARGO_BUILD_JOBS=2 cargo test -p
reposix-remote --test stateless_connect_e2e --test protocol`, 1x combined + 5x isolated single-test
re-runs) — 6/6 GREEN locally, consistent with the prior investigation's 6/6 GREEN. The failure
appears to require the actual GitHub Actions runner environment (2-vCPU `ubuntu-latest`,
`Swatinem/rust-cache`-restored `target/`) to manifest; it has never reproduced in this sandbox
despite matching every documented aspect of the CI sequence, including env vars.

**Updated hypothesis:** given (b) rules out contention/timeout and the failure is fast and
deterministic-looking within a given CI run, the leading candidate shifts to either (i) a genuine
environment-dependent behavior difference (e.g. JSON string-escaping of `\r`/`\n` differing by
`serde_json` version, or a `wiremock`/`http`-stack difference) between whatever dependency
versions the CI runner's `Swatinem/rust-cache`-restored lockfile-driven build resolves vs. this
sandbox's already-built `target/`, or (ii) an assertion race specific to `stdout.contains("ok
refs/heads/main")` (line 203-206) rather than the CRLF assertion at line 219 itself — the
backtrace only proves the panic unwound through the `#[tokio::test]` body, not which specific
`assert!` fired first, and dump_verifications' truncation swallowed the disambiguating line.
**This distinction is now resolvable** on the next CI recurrence once `fbe5bee` is on the branch
under test.

**Severity raised to HIGH:** this blocks PR #61 (v0.13.0 release, `quality gates (pre-pr)` is a
required check) and is release-blocking per the settled GO/NO-GO cadence until the real assertion
text is captured and either fixed or the test is proven backend/environment-specific.

**STATUS:** OPEN — HIGH. Timeout theory dead; root cause still unconfirmed but now diagnosable
(log-truncation bug fixed in `fbe5bee`). Next action: re-run `quality gates (pre-pr)` on a branch
rebased past `fbe5bee` and read the now-uncapped verifier stderr for the actual panic/assertion
message.

---

**2026-07-07 follow-up #2 (post-`fbe5bee` re-run on PR #68, non-reproduction):**

**Context:** PR #68 (branch `release-plz-2026-07-07T02-37-20Z`, head `14bb5e43d7ff9552245dae6f3b47caeaece4ea1f`,
already includes `fbe5bee`'s 200-line tail-window fix) had its `CI`/`Security audit`/`quality gates
(pre-pr)` workflows re-triggered via a real-actor `gh pr close 68` / `gh pr reopen 68` (the
`pull_request`-trigger silently not re-firing after a release-plz branch regen is itself now filed
as a new process good-to-have, see `GOOD-TO-HAVES.md` 2026-07-07). Watched CI run `28838198234`
(the `CI` workflow) to completion; the `quality gates (pre-pr)` job runs as run `28838198234` /
job `85526336500`.

**Result: the flaky test did NOT reproduce this run.** The `quality gates (pre-pr cadence)` step's
full output shows `[PASS ] agent-ux/p94-git243-fallback-sentinel (P1, 15.49s)` — the verifier that
drives `CARGO_BUILD_JOBS=2 cargo test -p reposix-remote --test stateless_connect_e2e --test
protocol` (which includes `crlf_blob_body_round_trips_byte_for_byte`) completed GREEN in 15.49s.
Overall job summary: `70 PASS, 1 FAIL, 0 PARTIAL, 1 WAIVED, 0 NOT-VERIFIED -> exit=0` — the single
FAIL is the pre-existing, unrelated, non-blocking P2 row `docs-build/p94-badges-real-vs-transient`
(already tracked separately in `GOOD-TO-HAVES.md`'s "badges real-vs-transient flap" note), not the
CRLF test. Grepped the full job log for `panicked`, `assert.*failed`, `crlf_blob`,
`protocol.rs:2`, `body=`, `FAILED`, `failures:` — zero matches anywhere in the archived output,
because the test simply passed; there was no failure branch to trigger the `fbe5bee` full-log
archive or the panic-line grep this time.

**What this means:** the `fbe5bee` log-capture fix is still unverified in the one way that matters
(catching a live recurrence) because this CI run happened to be green. Hypothesis A vs. B
(CRLF-preservation assert at `protocol.rs:216-219` vs. the unrelated `stdout.contains("ok
refs/heads/main")` check at `protocol.rs:203-206`) remains **UNRESOLVED** — no new evidence either
way. Source-line mapping re-confirmed by direct read of `crates/reposix-remote/tests/protocol.rs`
lines 195-220 on this same commit: the `stdout.contains("ok refs/heads/main")` assert is at lines
203-206 (`"stdout missing ok: {stdout}"`), and the CRLF-preservation assert is at lines 216-219
(`body_str.contains("line-one\\r\\nline-two\\r\\n")`, `"POST body did not preserve CRLF — raw-bytes
path stripped \\r; body={body_str}"`) — both spans unchanged since the original filing, so the
2026-07-07 follow-up #1's line-number-to-source mapping (`protocol.rs:219:6` in a *previous* CI
backtrace) still points at the CRLF assert specifically, not the earlier `ok refs/heads/main` check,
IF that previous backtrace frame is accurate. This run adds no independent confirmation of that
frame; it only re-verifies the source stayed put and that the log-capture fix has not yet been
exercised against a real failure.

**STATUS:** OPEN — HIGH. Root cause still unconfirmed; `fbe5bee`'s diagnostic fix remains
unverified-in-anger (this run was green, not a recurrence). Per prove-before-fix on BLOCKERs, no
fix attempted. Next action: keep watching subsequent CI runs on this PR (or the next release-plz
regen) for a genuine recurrence, then immediately pull the job log before any further truncation
regressions can hide it again.

---

**2026-07-07 wrap-up (release-gate closure on PR #68's green run, no new diagnostic evidence):**

**Context:** all required checks on PR #68 (head `14bb5e43d7ff9552245dae6f3b47caeaece4ea1f`) went
GREEN, including `quality gates (pre-pr)`
(https://github.com/reubenjohn/reposix/actions/runs/28838198234/job/85526336500 — the same run
documented in the follow-up #2 entry above: 70 PASS / 1 unrelated pre-existing FAIL
(`docs-build/p94-badges-real-vs-transient`) / 1 WAIVED cadence, exit=0). Re-verified via
`gh pr checks 68` at wrap-up time — full 22-check matrix PASS, no regressions.

**What this confirms and does NOT confirm:** the release is unblocked ON THIS RUN'S EVIDENCE —
`crlf_blob_body_round_trips_byte_for_byte` did not fail, so there is no required-check red gating
PR #68. This is NOT a root-cause fix and NOT new diagnostic evidence: hypothesis A vs. B (CRLF
assert at `protocol.rs:216-219` vs. the `stdout.contains("ok refs/heads/main")` check at
`protocol.rs:203-206`) remains exactly as unresolved as the prior follow-up #2 left it, and local
reproduction is still 0/7+ attempts across two sessions. The underlying intermittent
CI-environment-specific flake is unresolved and could recur on a future CI run for this or any
other PR that exercises the same `quality gates (pre-pr)` job.

**Recommendation:** keep **STATUS: OPEN**, severity unchanged at **HIGH** (no rationale to
downgrade the severity itself — the failure mode, when it does recur, is still an unexplained
required-check red). Reframe the urgency posture only: this entry is **monitor — not
release-blocking on a green run; revisit if it recurs.** Do not close this entry on the strength of
a single non-reproducing green run; closing requires either (a) a live recurrence captured with the
`fbe5bee` full-log diagnostics and a confirmed root cause, or (b) enough consecutive green runs
across independent PRs to justify a deliberate re-classification as environment-noise, which has not
been attempted yet.

**STATUS:** OPEN — HIGH — monitor (not release-blocking on a green run; revisit if it recurs). Root
cause still unconfirmed; hypotheses A/B both still open; next action unchanged from follow-up #2
(catch a live recurrence with `fbe5bee`'s uncapped log before any further truncation regression can
hide it again).

---

**2026-07-07 deep-dive (opus RBF debugging session — Hyp A CONFIRMED from the real CI backtrace; A/B disambiguated):**

**(A) Hypothesis A is CONFIRMED — it is the CRLF-preservation assert at `protocol.rs:216-219`, NOT the `stdout.contains("ok refs/heads/main")` check at :203-206.** Pulled the actual failing job log via `gh api repos/reubenjohn/reposix/actions/jobs/85523987205/logs` (run `28837407948`, the confirmed `quality gates (pre-pr)` FAILURE — note run `28819166220`'s pre-pr job actually SUCCEEDED; its failure was a rerun `85521914911`). The backtrace frame is unambiguous: `24: protocol::crlf_blob_body_round_trips_byte_for_byte at ./tests/protocol.rs:219:6` → the closing `);` of the `assert!(body_str.contains("line-one\\r\\nline-two\\r\\n"), ...)` block. The push SUCCEEDED (`ok` assert :204 passed) and a POST WAS issued (`.expect("a POST was issued")` :213 passed, else the panic would name :213). So: a Create POST fired, returned 201, but its captured body lacked the JSON-escaped CRLF substring. The actual `body=` diff is STILL not in this log — this job predates `fbe5bee`, so `dump_verifications.py`'s 40-line tail-window landed mid-backtrace and discarded the panic line. `fbe5bee` (now on main) fixes that for the NEXT recurrence.

**(B) Three prior hypotheses RULED OUT with evidence:**
1. *Timeout* — already dead (0.14s), re-confirmed.
2. *Dependency-version drift (serde_json / wiremock / http)* — DEAD. `git show deee8fd:Cargo.lock` (the failing PR #61 head) vs HEAD `Cargo.lock` are BYTE-identical for wiremock 0.6.5, hyper 1.9.0, http 1.4.0, serde_json 1.0.150, tokio 1.52.3, h2 0.4.13, want 0.3.1. Same code, same deps.
3. *Shared on-disk cache race corrupts the POST body* — DEAD by code analysis. The four wiremock export tests DO collide on one cache (`resolve_cache_path` keys only on `<backend-slug>-demo.git`; none set `REPOSIX_CACHE_DIR`), BUT the Create's body bytes come DIRECTLY from in-memory `parsed.blobs[mark]` (diff.rs:228-236 → main.rs:484-495), never re-read from the shared bare repo / cache.db. `frontmatter::parse` slices the body by byte offset (record.rs:187/213-218 — no `.replace`/`.lines()`) and `render` pushes it verbatim (record.rs:148); `parse_export_stream` captures the blob via `read_exact(len)` (fast_import.rs:183-184). Every deterministic path preserves CRLF byte-exact. Concurrent cache access CANNOT alter the POSTed body. (The shared cache IS a real test-isolation / OP-4 "no hidden state" defect that precheck reads — but precheck poisoning would fail at :199/:204/:213, never :219, so it does not explain THIS failure.)

**(C) Local reproduction: ~87 GREEN runs across three sessions, zero failures.** This session added: 30× single-core (`taskset -c 0`, `--test-threads=4`, shared cache, cleaned per iter) + 20× full gate-replica (build both `--test stateless_connect_e2e --test protocol` binaries, run in gate order sharing one cache, pinned to 2 cores `taskset -c 0-1` to mimic the 2-vCPU runner). 0/50 this session, 0/37 prior.

**(D) Leading hypothesis (now the ONLY one consistent with all evidence): intermittent INCOMPLETE POST-body capture by `wiremock`/`hyper`'s `received_requests()` under CI CPU starvation — a TEST-HARNESS artifact, not a reposix byte-handling bug.** If, under 2-vCPU contention, the recorded request body is occasionally empty/truncated, `std::str::from_utf8(&post.body)` still succeeds (empty/partial is valid UTF-8), `body_str` is empty/partial, and `.contains(CRLF)` fails at exactly :219 — matching the backtrace. This is consistent with: body is provably byte-exact in-memory, deps identical, and total local non-reproduction. The `test` job never fails because `cargo test --workspace` schedules protocol.rs among hundreds of tests (different load profile) whereas the gate runs only these two files.

**(E) DATA-INTEGRITY VERDICT for shipped v0.13.0: NO corruption risk, no urgent point-release needed.** The failure requires the parallel-test harness. In production, one `git push` = one `git-remote-reposix` process per `(backend, project)`; the body is built from in-memory parsed fast-import bytes with zero line-ending normalization anywhere in the write path. A real push does NOT mangle blob bytes. The only real cost is release-gate flakiness.

**Next experiment (single most valuable, NOT yet done — budget-gated):** push a throwaway `debug/crlf-capture` branch that loops the CRLF push ~60× per run with UNCONDITIONAL `eprintln!("DEBUG POST body = {body_str:?} len={}", post.body.len())`, open a PR to fire `quality gates (pre-pr)`, and read the captured body on the first failing iteration. If `body=` is empty/short → confirms (D), fix = poll `received_requests()` until the POST body is non-empty (wiremock timing), a test-harness fix. If `body=` shows `line-one\nline-two\n` (LF only) → a genuine reposix `\r`-stripping bug, re-open at BLOCKER. Prior sessions never ran this because they hadn't disambiguated A/B; that is now done.

**STATUS:** OPEN — HIGH — monitor. Hyp A confirmed (`:219`, CRLF assert). Timeout + dep-drift + shared-cache-body-corruption all ruled out with artifacts. Leading cause: wiremock/hyper incomplete-body capture under CI load (test-harness-only; no shipped-data risk). Definitive close still requires the `debug/crlf-capture` loop to capture the real `body=` string.

## S-260707-pr-01 — `reposix-sim` ships in NO prebuilt distribution; documented 3-crate binstall installs ZERO binaries (BLOCKER)

**Found during:** v0.13.0 post-release verification (zero-shot docs-following fleet).
**Severity:** BLOCKER.
**Issue:** The release tarball/zip contains only `reposix` + `git-remote-reposix`
(`tar tzf` confirmed against the published release asset); the Homebrew formula installs
only those two binaries (`.github/workflows/release.yml:465-466`); `cargo binstall ...
reposix-sim` fails with no prebuilt archive available (exit 94). Because `cargo binstall`
is all-or-nothing across its argument list, the documented tutorial command `cargo
binstall reposix-cli reposix-remote reposix-sim` installs **zero** binaries when any one
crate lacks a prebuilt archive. The CLI resolves `reposix-sim` as a sibling of
`current_exe()`, so the default simulator backend — the entire first-run tutorial path —
is unreachable for every non-source install band (curl installer, PowerShell installer,
Homebrew, `cargo binstall`).
**Sketch:** Either (a) add `reposix-sim` to the release build's distributed binaries
(release.yml dist-binaries list + installer scripts + Homebrew formula's `install` block),
or (b) stop advertising sim-backed onboarding to binary-install users and gate the
sim-tutorial path behind a from-source checkout, with docs updated accordingly. Option (a)
is the north-star fix — the sim is OP-1's designated first-run default.

## S-260707-pr-02 — `reposix init` hides a fatal fetch failure behind exit 0, then hands out a broken "Next:" hint (HIGH)

**Found during:** v0.13.0 post-release verification (zero-shot docs-following fleet).
**Severity:** HIGH.
**Issue:** When the sim backend is unreachable, `reposix init sim::demo <path>` prints a
`fast-import` / `backend-unreachable` crash and leaves a `.git/fast_import_crash_*` file
behind, yet the process exits **0** and still prints `Next: git checkout origin/main` — a
command that then fails with `pathspec 'origin/main' did not match any file(s) known to
git` because the fetch never populated `origin/main`. A zero-shot user (or agent) reading
only the exit code and the "Next:" line has no signal that init actually failed.
**Sketch:** Exit non-zero when the underlying fetch/import fails; suppress (or caveat) the
"Next:" hint on the failure path; clean up (or explicitly reference) the
`fast_import_crash_*` file so the user isn't left with a silent turd in `.git/`.

## S-260707-pr-03 — `reposix sim` fallback shells out to `cargo run -p reposix-sim` from a release binary (HIGH)

**Found during:** v0.13.0 post-release verification (zero-shot docs-following fleet).
**Severity:** HIGH.
**Issue:** When `reposix-sim` isn't found on `PATH`, `reposix sim` falls back to invoking
`cargo run -p reposix-sim`, which fails with `could not find Cargo.toml` because a
packaged release binary has no workspace to build from. A shipped binary must never reach
for `cargo`/`Cargo.toml` — that fallback only makes sense in a source checkout.
**Sketch:** In release builds, replace the cargo fallback with a teaching error (e.g.
`reposix-sim not found on PATH; install it via <x>`, once S-260707-pr-01 ships a prebuilt
`reposix-sim`) rather than a bare cargo-not-found failure.

## S-260707-pr-04 — reposix-swarm integration tests share one on-disk cache + sqlite path across parallel threads/binaries (MEDIUM)

**Found during:** v0.13.0 post-release verification (crlf-investigation lane, contributing
factor surfaced while chasing S-260707-rbf-01).
**Severity:** MEDIUM.
**Issue:** Integration tests share one on-disk `~/.cache/reposix/<slug>-demo.git` +
sqlite path across parallel test threads AND across the gate's two separate test
binaries — an OP-4 (no-hidden-state) violation and a latent flake source; it was a
contributing factor the crlf lane surfaced (not itself the crlf root cause).
**Sketch:** Give each test a unique `REPOSIX_CACHE_DIR` (e.g. a `tempdir` per test) so
concurrent test runs never share cache/sqlite state.

## S-260707-pr-05 — `reposix init`/`doctor` write cache state to `~/.cache/reposix/` unconditionally, with no documented override (MEDIUM)

**Found during:** v0.13.0 post-release verification (zero-shot docs-following fleet).
**Severity:** MEDIUM.
**Issue:** `reposix init`/`doctor` write cache state to `~/.cache/reposix/` unconditionally
with no CWD scoping, causing collisions between concurrent runs or testers on a shared
machine. `REPOSIX_CACHE_DIR` exists as an override but is undocumented — it doesn't appear
in `--help` output or in the docs site.
**Sketch:** Document (and confirm honoring of) a per-invocation cache-dir override; surface
`REPOSIX_CACHE_DIR` in `--help` text and in the relevant docs/reference page.

## S-260707-pr-06 — `git-remote-reposix` has no `--version`; installer.sh has no arg validation; `REPOSIX_INSTALL_DIR` undocumented (LOW)

**Found during:** v0.13.0 post-release verification (zero-shot docs-following fleet).
**Severity:** LOW.
**Issue:** (a) `git-remote-reposix` has no `--version` flag, inconsistent with
`reposix --version`. (b) `installer.sh` has no `--help`/argument parsing — it silently
runs a full install on any argument passed to it. (c) `REPOSIX_INSTALL_DIR` works as an
install-location override but is undocumented.
**Sketch:** Add `--version` to `git-remote-reposix`; add basic arg validation (and a
`--help`) to `installer.sh`; document `REPOSIX_INSTALL_DIR` alongside the install
instructions.

## S-260707-pr-07 — `docs-build/p94-badges-real-vs-transient` catalog row reported stale-NOT-VERIFIED while the underlying gate needs live re-verification (LOW)

**Found during:** v0.13.0 post-release verification, Task D (catalog-row spot-check).
**Severity:** LOW.
**Issue:** The row `p94-badges-real-vs-transient` in `quality/catalogs/docs-build.json`
was reported as stale/NOT-VERIFIED at hand-off time; the report claimed "the gate runs
green live." Actually running `bash quality/gates/docs-build/p94-badges-real-vs-transient.sh`
in this verification session gives **exit=1** (FAIL), not green:
```
PASS: badges-resolve.py re-run on >=2 spaced occasions; pass/fail pattern recorded (3 runs)
FAIL (docs-build/p94-badges-real-vs-transient): GOOD-TO-HAVES badges-resolve entry is not RESOLVED (still OPEN or missing)
```
The gate's own logic requires the `GOOD-TO-HAVES.md` `badges-resolve` entry to carry
STATUS: RESOLVED before it will pass; that entry has not been resolved, so the "runs
green live" claim does not hold as of this session. Per the task's own guard ("never
fabricate a PASS the gate didn't produce"), the catalog row was **not** flipped.
**Sketch:** Either resolve the `badges-resolve` GOOD-TO-HAVES entry (determine
real-vs-transient, record the finding, flip its STATUS to RESOLVED) so the gate's own
precondition is satisfied, or correct whoever/whatever asserted "runs green live" — the
claim was stale/wrong at verification time. Only then update the catalog row's status +
`last_verified` with the dated evidence.

## S-260707-pr-08 — agent "worktrees" are NOT isolated; a sim-seed leaf corrupted the shared repo (`t <t@t>` flipped `core.bare=true`) (HIGH)

**Found during:** 2026-07-07 orchestration session — a dispatched sim/seed leaf corrupted
the shared local repo `/home/reuben/workspace/reposix` (repaired twice this session).
**Severity:** HIGH.
**Issue:** Agent "worktrees" share the coordinator's `.git/config` + object store — they
are NOT isolated from the shared repo — and a leaf's cwd resets to the repo root between
Bash calls. A sim-seed leaf whose `reposix init` / seed / `git commit` / `git config`
did not `cd` into its `/tmp` target dir *within the same Bash invocation* therefore ran
against the real shared repo: it committed under the sim-fixture identity `t <t@t>` and
flipped `core.bare=true`, breaking the shared checkout for every concurrent agent. This
is systemic — any future setup leaf that assumes worktree isolation or durable cwd can
reproduce it. A prose hard-stop now lives in `.planning/ORCHESTRATION.md` § Leaf
isolation, but there is no *enforced* guardrail yet.
**Sketch:** Enforce isolation, don't just document it: (a) each setup leaf gets an
isolated `/tmp` clone + a distinct `REPOSIX_CACHE_DIR`; (b) a `.claude/hook` (pre-commit
or the existing stop-uncommitted family) that REJECTS any commit authored by `t <t@t>`
(or any known test-fixture identity) against the shared repo; and/or (c) a guard that
fails a leaf's `reposix init`/seed when `$PWD` is inside the shared repo tree. Route to
the enforcement map once shipped (`ORCHESTRATION.md` § Enforcement map).

## S-260707-rbf-lr03-external-write-crash | 2026-07-07 | discovered-by: v0.13.1 B5 TRIAGE | severity: HIGH | tag: v0.14.0 RBF-LR-03 pivot

**What:** The documented post-conflict recovery CRASHES when the SoT moved via an
**external REST write** (web UI / direct `PATCH`) rather than a git-side push. Reproduced
leaf-isolated in `/tmp` (sim `--ephemeral` seeded, git 2.25.1, `git-remote-reposix` on
PATH): after a local commit + an external `PATCH /projects/demo/issues/1`, the FULL
documented sequence `reposix sync --reconcile` → `git pull --rebase` → `git push` aborts
at the pull with:
```
warning: Not updating refs/reposix/origin/main (new tip 764e1a70… does not contain bd848c1…)
fatal: error while running fast-import
```
`reposix sync --reconcile` (exit 0) does NOT help — it is the trigger: it mints a fresh
"Sync from REST snapshot" synthesis commit that is NOT a descendant of the tracking tip,
so git's fast-import refuses the ref update. The bare `git pull --rebase` (no reconcile)
crashes identically. Data-safety kicker: the follow-on `git push` returns exit 0 and
would push the tree that never absorbed the external edit — silently overwriting the
external writer (the exact overwrite reconcile was documented to prevent). The only
current recovery is a fresh `reposix init` into a new dir (verified: fresh tree shows the
external edit), losing unpushed local commits.

**Why out-of-scope for v0.13.1 hotfix:** No `<1h`/no-new-dependency fix. A correct fix
must make the cache build snapshot commits as *descendants* of the prior synthesis tip
(lineage + dedup + push-conflict semantics) — precisely the RBF-LR-03 reconciliation
redesign already ratified as the v0.14.0 owner pivot (CONSULT-DECISIONS 2026-07-06). A
point patch here re-entrenches the placeholder-synthesis design the pivot exists to
replace. HOTFIX-conservative bias → docs made honest in v0.13.1, deep fix deferred.

**Sketched resolution (v0.14.0 RBF-LR-03):** In the reconciliation redesign, the
cache-side "Sync from REST snapshot" commit MUST be parented on the prior
`refs/reposix/origin/main` tip so `git pull --rebase` sees a fast-forwardable /
rebaseable lineage. Model the external-write reconciliation as a commit *appended* to the
existing history (owner's commit-sequence model) rather than a fresh-root snapshot. Add a
leaf-isolated regression proving `external PATCH → pull --rebase` recovers without
`fatal: error while running fast-import`. Repro scripts + full transcript:
CONSULT-DECISIONS.md 2026-07-07 entry.

**STATUS:** OPEN (deferred to v0.14.0 RBF-LR-03 pivot; docs-honesty spec handed to the
v0.13.1 doc-truth lane)
