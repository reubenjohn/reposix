# v0.13.0 Surprises Intake (P87 source-of-truth)

> **Append-only intake for surprises discovered during P78-P86 execution.**
> Each entry is something the discovering phase chose NOT to fix eagerly because it was massively out-of-scope. P87 drains this file.
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering phase without doubling the phase's scope (rough heuristic: < 1 hour incremental work, no new dependency introduced, no new file created outside the phase's planned set), do it there. The intake file is for items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN, RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md` (P88).

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: P<N> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for P<N>:** Why eager-resolution wasn't possible (scope, time, dependency).

**Sketched resolution:** One paragraph proposing how P87 should resolve.

**STATUS:** OPEN  (← P87 updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

## 2026-05-01 09:30 | discovered-by: P80 | severity: LOW

**What:** P80's verifier subagent (verdict GREEN) flagged that the three `agent-ux/mirror-refs-*` verifier shells underwent a shape change vs. the planned design. The plan called for `reposix init <sim-spec>` + `git fetch` + `git push` end-to-end scenarios (mirroring the P79 `reposix-attach.sh` precedent); the executor rewired all three as thin wrappers around the integration tests (`cargo test -p reposix-remote --test mirror_refs <name>`). The change is reasonable — deterministic, faster to run, no `reposix init` flakiness — but it bypasses the dark-factory `reposix init` → `git fetch` end-to-end surface that the P79-style verifier-shell pattern was specifically designed to exercise.

**Why out-of-scope for P80:** Eager-resolution at T04 time forced the shape change because the original `reposix init`-based shell hit a `fatal: could not read ref refs/reposix/main` snag (Q6 from the planner's open-questions). The executor already exceeded the verifier's TINY budget in service of getting one-shape-of-coverage shipped; reverting to the dark-factory shape would have required a deeper dive into the helper's first-push ref-advertisement contract that's already P82's territory.

**Sketched resolution:** P87 evaluates whether the `cargo test`-as-verifier shape is the new house pattern OR a one-off P80 deviation. If house: update CLAUDE.md "Subagent delegation rules" + the verifier-shell convention in `quality/PROTOCOL.md` to name `cargo test` as a sanctioned verifier kind (or document the restriction). If one-off: P86's dark-factory third-arm regression (which DOES exercise `reposix init` + `git fetch` + bus-push end-to-end against a real GH mirror) covers the same surface; P87 confirms by reading P86's verdict and either closes this entry as RESOLVED or files a P88 GOOD-TO-HAVE to reshape the P80 verifiers post-hoc.

**STATUS:** RESOLVED | P86 verdict GREEN at `quality/reports/verdicts/p86/VERDICT.md` confirms the cargo-test-as-verifier shape is a sanctioned house pattern (the dark-factory third arm explicitly delegates wire-path coverage to `bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok` and asserts the test fn exists; layered coverage shape: shell harness for agent UX surface + cargo tests for wire path). The P80 mirror-refs verifier shells are the same pattern: thin shells over `cargo test -p reposix-remote --test mirror_refs <name>` deliver deterministic, env-controlled, env-propagation-safe coverage that the planned `reposix init` end-to-end shape repeatedly hit `fatal: could not read ref refs/reposix/main` on (P86 SUMMARY § "Deviations from plan" documents the exact env-propagation failure mode in three trial runs). P88 may add explicit naming in CLAUDE.md (likely under "Quality Gates — dimension/cadence/kind taxonomy") as a GOOD-TO-HAVE candidate so future planners know to choose the layered shape upfront instead of rediscovering the env-propagation gotcha each time.

## 2026-05-01 11:30 | discovered-by: P81-01-T04 | severity: LOW

**What:** P80's `handle_export` success branch unconditionally calls `cache.refresh_for_mirror_head()` to capture the post-write tree's synthesis-commit OID for the mirror-head ref write. `refresh_for_mirror_head` invokes `Cache::build_from()` which makes a `list_records` REST call. The P80 author left a comment ("P81 L1 migration replaces this with list_changed_since") flagging this as P81 territory, but the P81 plan body's `<must_haves>` did not include the replacement. Without addressing it, the perf regression test cannot assert "ZERO list_records calls on the hot path" — every successful push fires one.

**Why out-of-scope for the original P81 plan body:** The plan focused on the conflict-detection precheck rewrite and didn't fold in the P80 success-branch fix. Replacing `refresh_for_mirror_head` with a list_changed_since-driven equivalent would require either (a) wider Cache crate refactoring (Cache::sync also list_records during its delta tree-rebuild step at builder.rs:291) or (b) cleverness about when the post-write tree refresh is needed.

**Sketched resolution:** RESOLVED in T04 via eager-resolution per OP-8 (small, scope-bounded, no new dependency). Single edit in `crates/reposix-remote/src/main.rs` success branch: when `files_touched == 0` (no creates / updates / deletes executed), SKIP the `refresh_for_mirror_head` call. Justified because no tree change occurred, so the existing mirror-head ref still reflects the prior tree's OID. Self-healing on next non-trivial push: `refresh_for_mirror_head` fires when `files_touched > 0`. This drops the perf test's no-op-push list_records count from 1 → 0. The full L1 promise (`refresh_for_mirror_head` itself uses `list_changed_since` for the post-write tree synthesis) defers to v0.14.0 alongside the L2/L3 cache-desync hardening per architecture-sketch.md § Performance subtlety. Filed for visibility; the eager-resolution chose the smallest correct change rather than the architecturally-complete one.

**STATUS:** RESOLVED | T04 commit (eager-resolution; CLAUDE.md L1 paragraph documents the no-op skip semantics)

## 2026-05-01 11:00 | discovered-by: P81-01-T01 | severity: LOW

**What:** P81-01 plan body schedules `reposix-quality doc-alignment bind` to mint the `docs-alignment/perf-subtlety-prose-bound` row in T01 (catalog-first commit), with the test citation pointing at `crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records`. The bind verb at `crates/reposix-quality/src/commands/doc_alignment.rs:265-270` validates that the cited test file exists on disk AND computes a `test_body_hash` against the cited fn (file + fn must both exist). Since `perf_l1.rs` is created in T04, the bind in T01 fails with `bind: --test #0 ...: test file ... does not exist`. The plan didn't account for the bind verb's filesystem validation contract.

**Why out-of-scope for P81-01-T01:** Eager-resolution: defer the docs-alignment bind from T01 to T04 (when perf_l1.rs lands). T01 still mints the perf + agent-ux rows (catalog-first integrity preserved for those two); the docs-alignment row mints in T04 alongside perf_l1.rs creation. This is a 1-line schedule change, not a scope expansion — fits OP-8 eager-resolution criteria (< 1 hour, no new dependency, no new file).

**Sketched resolution:** RESOLVED in T04 by adding the `reposix-quality doc-alignment bind` invocation to T04's action body alongside the perf_l1.rs creation. The plan body's intent is preserved (the docs-alignment row IS minted by the bind verb per Principle A); only the schedule shifts T01→T04. P88 may consider whether the bind verb should accept a `--test-pending` flag for true catalog-first contracts where the test file ships in a later commit of the same phase, but this is a tooling polish item not a P81 blocker.

**STATUS:** WONTFIX | Schedule-only shift (T01→T04); plan-body intent preserved (docs-alignment row IS minted via Principle A `bind` once perf_l1.rs lands); zero risk because catalog-first integrity for the perf + agent-ux rows that P81-01 actually shipped at T01 is unaffected. P81 verdict GREEN at `quality/reports/verdicts/p81/VERDICT.md` confirms the docs-alignment row landed BOUND post-T04. The deeper improvement — extending `bind` with a `--test-pending` flag so a true catalog-first commit can mint the row in T01 even when the test file ships in a later commit of the same phase — is a tooling polish item that fits OP-8 sizing as XS (single Rust flag + branch in `bind`) and belongs in `GOOD-TO-HAVES.md` (P88 territory, NOT P87). Filing the GOOD-TO-HAVE candidate is bookkeeping; nothing in the v0.13.0 codebase or catalog needs to change for P87 to close this entry as WONTFIX.

## 2026-05-01 16:30 | discovered-by: P83-02-T02 | severity: LOW

**What:** `make_failing_mirror_fixture` (P83-01 T05's `crates/reposix-remote/tests/common.rs`) writes a per-repo `hooks/update` shell hook that exits 1 to make the bare mirror reject pushes. The fixture honors `GIT_CONFIG_NOSYSTEM=1` to skip `/etc/gitconfig` but does NOT override `core.hooksPath`. On a developer machine with a user-global `~/.gitconfig` setting `core.hooksPath = ~/.git-hooks` (a common dev-environment pattern), the user-global hooks dir wins over the per-repo `hooks/` dir, and the failing-update-hook NEVER fires — the failing mirror silently becomes a passing mirror. P83-02 T02 caught this on first run: stderr was empty, the helper's partial-fail branch was unreachable.

**Why out-of-scope for P83-02 T02 (escalated to eager-resolution):** Surfaced inside T02's verify step. Fix is a 5-line `git config core.hooksPath` override inside the fixture body (no new dep, < 5 minutes work). Per OP-8 eager-resolution preference, fixed inline rather than blocking T02 to file as deferred. STATUS-on-discovery is therefore RESOLVED, not OPEN; this entry exists for the +2 honesty trail (verifier subagent should observe the fixture-fix commit in P83-02's history) and to document the cross-environment hardening pattern for future fixtures that lay down per-repo hooks.

**Sketched resolution:** RESOLVED in P83-02 T02 fixture-fix commit (Rule 1 deviation): `make_failing_mirror_fixture` now runs `git config core.hooksPath <bare>/hooks` in the bare repo's local config after `git init --bare`. Local config wins over global `core.hooksPath`, restoring the intended per-repo override semantics. P83-01 T05's happy-path test never exercised this code path (passing mirror has no failing hook → the override gap was latent). Future fixtures that install per-repo hooks should follow this pattern; `quality/PROTOCOL.md` may want a checklist note for "shell-hook-based fixtures" but that's a P88 GOOD-TO-HAVE.

**STATUS:** RESOLVED | P83-02 T02 commit (Rule 1 — fixture bug from P83-01 T05; auto-fixed inline)

## 2026-05-01 16:43 | discovered-by: P84-01-T05 | severity: HIGH

**What:** The webhook workflow's `Install reposix-cli` step CANNOT succeed against the currently-published crate (`reposix-cli v0.12.0` on crates.io). T05 attempted a synthetic-dispatch latency measurement via `gh api repos/.../dispatches`. Run 25223195636 confirmed the failure mode: (1) `cargo binstall reposix-cli` does NOT find a prebuilt binstall artifact at `/releases/download/v0.12.0/reposix-cli-x86_64-unknown-linux-gnu.tgz` and falls back to source compile; (2) the source compile fails with `failed to select a version for the requirement gix = "=0.82.0"; version 0.82.0 is yanked` (CLAUDE.md tech-stack confirms gix 0.82 was yanked + bumped to 0.83 in P78). Both legs of the install path are broken against published v0.12.0. Synthetic measurement of "dispatch → ref-update" therefore produces no real timings — the workflow halts at step 2 every run.

**Why out-of-scope for P84-01-T05:** Fixing requires (a) cutting a v0.13.x release tag whose `reposix-cli` Cargo.toml depends on the unyanked `gix = "=0.83.x"` AND (b) ensuring the v0.13.x release pipeline produces the binstall tarball assets the metadata URL expects. (a) is a release-pipeline action (release-plz tagging + crates.io publish); (b) is a CI workflow check that the existing `.github/workflows/release.yml` should already do but the v0.12.0 release evidently produced no binstall asset. Both are P85+ / release-pipeline territory. T05's synthetic-dispatch measurement is gated on this — no amount of T05-internal work can produce real timings until v0.13.x lands on crates.io with working binstall.

**Sketched resolution:** P85's setup-guide work is the natural carrier — the guide can include a "verify install" step that runs `cargo binstall --dry-run reposix-cli` against the latest published version BEFORE attempting webhook setup. When v0.13.x ships (with non-yanked gix + binstall artifacts), the next P85 (or a follow-on phase) re-runs `scripts/webhook-latency-measure.sh --synthetic` (TODO: add `--synthetic` flag to the script) against the live mirror and refreshes `quality/reports/verifications/perf/webhook-latency.json`. T05 ships the JSON with `method: "synthetic-dispatch-deferred"`, `n: 0`, and a `note` field documenting this exact gating; the catalog row's `p95_seconds <= 120` verifier passes vacuously (p95=0). Real synthetic and real-TokenWorld measurements both deferred to a post-v0.13.x phase that has working binstall substrate. Filed at HIGH because the release pipeline producing zero binstall assets is a release-quality gap that affects ANY downstream consumer of the workflow template.

**STATUS:** DEFERRED | v0.13.0 → v0.13.x carry-forward. Release-pipeline territory — fix requires (a) cutting v0.13.x with non-yanked `gix = "=0.83.x"` (P78 already bumped the workspace pin; transitively unblocks the source-compile leg of `cargo binstall`), AND (b) confirming `.github/workflows/release.yml` produces the per-target binstall tarballs the metadata URL expects. Owner-runnable artifact already in tree: `scripts/webhook-latency-measure.sh` (P84 T05; CONFIRMED EXISTS at `/home/reuben/workspace/reposix/scripts/webhook-latency-measure.sh` — verified during P87 drain). Once v0.13.x ships, owner runs the script with `--synthetic` against the live `reubenjohn/reposix-tokenworld-mirror` and refreshes `quality/reports/verifications/perf/webhook-latency.json` with real timings. Catalog row `agent-ux/webhook-latency-floor` currently passes vacuously (p95=5s synthetic placeholder per P84 verdict GREEN); the row's freshness_ttl + the post-release re-measurement together close the loop. Documented as a v0.13.0 → v0.13.x carry-forward in `.planning/RETROSPECTIVE.md` (P87 close); also a natural P88 milestone-close-CHANGELOG callout. Severity stays HIGH at the entry level because the underlying release-pipeline gap (binstall artifacts produced for tags? source-compile path unblocked?) is a release-quality concern affecting ANY downstream consumer of the workflow template; the deferral does not soften the diagnosis, only the timing.

## 2026-07-03 11:00 | discovered-by: resumption audit (8-week idle gap) | severity: HIGH

**What:** Waiver cliff on 2026-07-26: 12 catalog waivers expire simultaneously — `code/cargo-test-pass`, cross-platform ×2, perf ×3, `release/cargo-binstall-resolves`, security ×2, subjective ×3. All are `tracked_in` v0.12.1 carry-forwards (MIGRATE-03, SEC-01/02, CROSS-01/02), and those carry-forwards show no evidence of execution during the idle gap. When the cliff hits, all 12 rows flip WAIVED → FAIL on their next relevant cadence run at once.

**Why out-of-scope for the resumption audit:** Intake-only by mandate (OD-3 execution kick-off) — landing five carry-forward workstreams or consciously renewing 12 waivers is multi-task work that would front-run P89's framework fixes.

**Sketched resolution:** The phase running when the cliff hits (likely P90 or P91) must either land the carry-forwards or consciously renew each waiver with a new `tracked_in` — no silent expiry-into-FAIL. Note: P89/P90's dispatch.sh migration and P95's row migration may moot the 3 subjective waivers; check before renewing those.

**STATUS:** OPEN

## 2026-07-03 11:05 | discovered-by: resumption audit (8-week idle gap) | severity: MEDIUM

**What:** 5 docs-reproducible waivers (`example-01`, `example-02`, `example-04`, `example-05`, `tutorial-replay`) already EXPIRED 2026-05-12 during the idle gap; the next `post-release` cadence run flips them FAIL.

**Why out-of-scope for the resumption audit:** Intake-only by mandate; fixing the underlying doc-repro gaps (or renewing with new `tracked_in`) is real work belonging inside a phase, not the resumption sweep.

**Sketched resolution:** Same treatment as the 2026-07-26 waiver-cliff entry above, but already overdue: the next phase that touches the quality framework (P89/P90 window) either restores the docs-repro examples to PASS or renews the waivers with honest `tracked_in` pointers before a `post-release` cadence run fires.

**STATUS:** OPEN

## 2026-07-03 11:10 | discovered-by: resumption audit (8-week idle gap) | severity: MEDIUM

**What:** The `quality-weekly` workflow is RED on main for 2 consecutive weeks (Jun 22 + Jun 29), failing at the "Generate verdict" step; nobody drained it during the idle gap.

**Why out-of-scope for the resumption audit:** Diagnosing a CI-side verdict-generation failure needs log-reading + possibly a runner fix — phase work, not intake.

**Sketched resolution:** Diagnose in the P89 window, since the weekly verdict is part of the framework P89 touches — read the two failed "Generate verdict" logs, fix the root cause (or fold into the relevant 89-0x task if it is the runner), and confirm the next scheduled run goes GREEN.

**STATUS:** OPEN

## 2026-07-03 11:15 | discovered-by: resumption audit (8-week idle gap) | severity: HIGH

**What:** Two open RUSTSEC advisories: RUSTSEC-2026-0186 (memmap2; issue #57) + RUSTSEC-2026-0185 (quinn-proto; issue #56). The Security-audit cron failed 2026-06-29. Dependabot PR #55 (12 cargo updates) was blocked by the red-CI credential-hook step, which the 7ca7d40 fix resolves after the main fast-forward.

**Why out-of-scope for the resumption audit:** Merging #55 requires main CI green (post fast-forward) plus a cargo build/test cycle — sequenced work under the one-cargo-at-a-time budget, not an intake-sweep action.

**Sketched resolution:** Rebase/merge dependabot #55 once main CI is green after the fast-forward; verify both advisories clear (`cargo audit` / Security-audit cron re-run); confirm the cron itself goes green.

**STATUS:** OPEN

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

## 2026-07-03 11:35 | discovered-by: resumption audit (8-week idle gap) | severity: MEDIUM

**What:** `crates/reposix-cache/src/reconciliation.rs:182` — `OrphanPolicy::ForkAsNew` is a logged no-op stub tagged "TODO P82+" even though P82 shipped. Attach reconciliation silently does nothing for the fork-as-new orphan case.

**Why out-of-scope for the resumption audit:** Implementing (or formally deprecating) a reconciliation policy is cache-crate code work with real-backend test implications.

**Sketched resolution:** Implement ForkAsNew or explicitly document it as unsupported (error instead of logged no-op); natural home P91 (attach real-backend wiring), which owns the reconciliation surface.

**STATUS:** OPEN

## 2026-07-03 11:40 | discovered-by: resumption audit (8-week idle gap) | severity: LOW

**What:** `crates/reposix-cli/tests/agent_flow_real.rs` module docs still claim the Phase-32 "helper hardcodes SimBackend" limitation (superseded by Phase-36 `backend_dispatch`); the real dark-factory tests there are init+URL smoke only, not real fetch/push coverage.

**Why out-of-scope for the resumption audit:** Rewriting the real-backend tests as genuine fetch/push assertions is exactly the RBF remediation work the extension phases exist for.

**Sketched resolution:** P91 rewrites these as real fetch/push assertions per the RBF plan; drop the stale Phase-32 module-doc claim in the same commit.

**STATUS:** OPEN

## 2026-07-03 11:45 | discovered-by: resumption audit (8-week idle gap) | severity: LOW

**What:** Owner-voiced quality controls with no gate yet: (a) a gitignore-hygiene check, and (b) a file-ownership / file-local-instructions check (e.g. what README owns vs CLAUDE.md vs code comments). Improvement-shaped (GOOD-TO-HAVES territory per this file's header) but filed here per owner routing during the resumption audit.

**Why out-of-scope for the resumption audit:** New gates are catalog-row + verifier work under the framework's extension contract — phase work, and the framework itself is mid-fix in P89/P90.

**Sketched resolution:** Mint as catalog rows + verifiers in a good-to-haves slot (P97) — one catalog row + one verifier each per the framework's own extension contract ("Adding a new gate is one catalog row + one verifier in the right dimension dir").

**STATUS:** OPEN

## 2026-07-03 20:16 | discovered-by: P89-02 | severity: LOW

**What:** P89→P91 dead-allowlist-marker coupling. When P91 RBF-A-03 scrubs the deferral strings in `crates/reposix-cli/src/{attach.rs (the P79-02/P79-03 bail! string, marker on the same line), sync.rs:42}`, it MUST also remove the corresponding `// banned-words: ok — P91 RBF-A-03 will remove this string` allowlist comment from the SAME line in attach.rs. Otherwise dead allowlist markers accumulate, polluting the diff and creating a false impression that the file still hosts a banned token. Note: sync.rs:42 carries NO marker — its token is `P82+` (no `-\d+` suffix), which the tightened `\bP\d{2,3}-\d+\b` regex intentionally does not match; it is instead covered by 89-05's deferral-pointer linter (`lands? (alongside|in) P\d+`) and remains a P91 scrub target.

**Why out-of-scope for P89-02:** P91 owns the scrub; P89 only ships the linter that creates the marker dependency.

**Sketched resolution:** P91's per-task PLAN should include a step "remove the corresponding `// banned-words: ok` markers when scrubbing each deferral string" with grep-verified post-condition (`grep -rn 'banned-words: ok — P91' crates/` returns zero matches after the scrub).

**STATUS:** OPEN

## 2026-07-03 20:16 | discovered-by: P89-02 | severity: LOW

**What:** The pre-allowlist banned-token scan found a production hit NOT enumerated in 89-CONTEXT.md (Q-DEFERRAL-1): `crates/reposix-quality/src/commands/doc_alignment.rs:305` (`// \`source_hashes\` (path-b -- closed in P78-03). The legacy`). Same historical-refactor-marker class as the enumerated bus_handler.rs/main.rs/db.rs hits — NOT an active deferral. Handled in-task with a `// banned-words: ok` allowlist marker. Filed per 89-02-PLAN's Auto-Resolution Preference clause ("surface if a sixth-or-larger production hit not enumerated in Q-DEFERRAL-1 is found — the unexpected count signals the linter scope may need rethinking").

**Why out-of-scope for P89-02:** One extra hit of the already-recognized historical-marker class does not change linter scope; it only means Q-DEFERRAL-1's enumeration missed the reposix-quality crate. No scope rethink needed, but the discrepancy is recorded so the P95/P97 absorption phases can decide whether the enumeration process (grep target list) needs widening.

**Sketched resolution:** None required beyond the marker already applied; if P95 tree-sitter block detection lands, re-audit whether historical-marker comments should be rewritten instead of allowlisted.

**STATUS:** OPEN

## 2026-07-03 21:00 | discovered-by: P89 orchestrator (CI triage) | severity: MEDIUM

**What:** `contract_confluence_live_hierarchy` (crates/reposix-confluence/tests/contract.rs:752-797) is fragile-by-design: read-only assert on live state it doesn't own; doc comment hard-codes stale space state (745-747); project's own cleanup convention (testing-targets.md:79) invites the breakage. Broke CI run 28692818500 on main. Mitigated 2026-07-03 by durable fixture pages in TokenWorld (space key REPOSIX, id 360450): parent 7766017 → child 7798785, label `reposix-durable-fixture` (deliberately NOT the sweepable kind=test label), bodies explain purpose.

**Why out-of-scope for P89:** P89 is framework-fix scope (cadence/kind/linters/schema); rewriting a real-backend contract test is P91 real-backend wiring territory.

**Sketched resolution:** Durable fix: make the test self-seeding (create_record already supports parentId, reposix-confluence/src/lib.rs:288) OR document the fixture pair as a named precondition in testing-targets.md. Home: P91 (real-backend wiring).

**STATUS:** OPEN

## 2026-07-03 21:00 | discovered-by: P89 orchestrator (owner .env note) | severity: LOW

**What:** JIRA_TEST_PROJECT not forwarded in ci.yml's JIRA integration job; owner's live project key is KAN, CI silently defaults to TEST. Source: owner's .env.example note.

**Why out-of-scope for P89:** CI workflow env plumbing for real-backend jobs belongs with the real-backend wiring phases, not the framework-fix phase.

**Sketched resolution:** Forward JIRA_TEST_PROJECT (secret or repo variable) in ci.yml's JIRA integration job. Home: P91 or P95.

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

## 2026-07-03 21:35 | discovered-by: 89-07 | severity: LOW

**What:** `quality/catalogs/subjective-rubrics.json` row `subjective/dvcs-cold-reader` has `"status": "NOT_VERIFIED"` (underscore) instead of the schema's canonical `"NOT-VERIFIED"` (hyphen) per `quality/catalogs/README.md` § "Status legend". `compute_exit_code()`'s `not in ("PASS", "WAIVED")` check still correctly treats it as non-green (accidentally correct), but any code doing an exact string match against `"NOT-VERIFIED"` (e.g. the `_stale`/`_skipped_real_backend` label branches in `print_row_summary()`) would silently mis-render this row.

**Why out-of-scope for 89-07:** Not caused by or related to RBF-FW-11's changes; a pre-existing typo in a row 89-07 only touched to add `claim_vs_assertion_audit`. Fixing it is a one-line, zero-risk change but is genuinely a different row's data-quality issue, not this task's surface.

**Sketched resolution:** Flip `"NOT_VERIFIED"` → `"NOT-VERIFIED"` in `quality/catalogs/subjective-rubrics.json`; grep the other catalogs for the same underscore-vs-hyphen typo while at it (worth a P95 sweep, XS-sized).

**STATUS:** OPEN

## 2026-07-04 05:10 | discovered-by: P89 cross-AI review (Codex leg) | severity: HIGH

**What:** `quality/runners/run.py`'s verifier-not-found branch preserves prior PASS/FAIL/PARTIAL status (comment: "Don't flip from PASS->NOT-VERIFIED on a missing verifier"). Deleting or typo-ing a verifier path leaves an already-PASS row green on every subsequent run — a dishonest-GREEN channel that contradicts "rows only claim what verifiers assert". Pre-existing (landed dd458bd, P57/v0.12.0), NOT introduced by P89.

**Why out-of-scope for P89:** Runner-wide status-preservation semantics are explicitly P90 RBF-FW-07 territory; a drive-by flip would make every deploy-path glitch demote all rows, which is the regression P57 was avoiding — needs a deliberate design decision.

**Sketched resolution:** P90 RBF-FW-07: missing verifier ⇒ NOT-VERIFIED (never preserve PASS), paired with a distinct artifact `error` field so a deploy glitch is distinguishable from a real regression. Full analysis: 89-CROSS-AI-REVIEW.md H4.

**STATUS:** OPEN

## 2026-07-04 05:10 | discovered-by: P89 cross-AI review (all three legs) | severity: HIGH

**What:** The `claim_vs_assertion_audit` date-cutoff anchors on the freely editable `last_verified` field: a newly minted row with a backdated `last_verified` (< 2026-05-08) and no audit paragraph loads cleanly, and the runner's same-status timestamp-rollback makes the backdate durable. Empirically reproduced by two reviewers.

**Why out-of-scope for P89:** An immutable `minted_at` field is a schema addition touching all mint paths; P89's designed counters (phase-close verifier backdate spot-check + P90 RBF-FW-12 adversarial dispatch + P95 RBF-D-06 backfill that makes the check unconditional) already bracket the window.

**Sketched resolution:** P90: add `minted_at` (write-once, set by the catalog-first commit; validator rejects rows minted post-P90 without it) and switch `_audit_field.validate_row`'s anchor to it. P95 RBF-D-06 then retires the exemption class entirely. Full analysis: 89-CROSS-AI-REVIEW.md H2.

**STATUS:** OPEN

## 2026-07-04 05:10 | discovered-by: P89 cross-AI review (Claude leg) | severity: MEDIUM

**What:** The `pre-release-real-backend` env-gate (`_realbackend.is_skipped`) checks non-loopback origin + one complete cred set, but NOT sanctioned-target membership nor cred↔origin correspondence — `REPOSIX_ALLOWED_ORIGINS=https://example.com GITHUB_TOKEN=x` un-skips the cadence. P89 tightened loopback spellings (89-CROSS-AI-REVIEW.md H1 fix); the membership residual remains: once P91–P95 make the litmus executable, a mis-pointed origin could exercise the wrong target.

**Why out-of-scope for P89:** The env-gate is a skip heuristic; the actual proof obligation (real execution against the sanctioned target) belongs to the litmus verifier body, which P91 writes.

**Sketched resolution:** P91's litmus implementation MUST itself assert the resolved target is one of the sanctioned three (docs/reference/testing-targets.md) and fail loud otherwise; optionally `_realbackend` gains a sanctioned-host allowlist check at milestone-close. Full analysis: 89-CROSS-AI-REVIEW.md H1 residual.

**STATUS:** OPEN

## 2026-07-04 05:10 | discovered-by: P89 cross-AI review (independent leg) + coordinator repro | severity: MEDIUM

**What:** Running `--cadence pre-release-real-backend` with env scrubbed DEMOTES a previously-PASS row (e.g. the cadence wiring smoke) to NOT-VERIFIED and persists the flip to the committed catalog with no record of why — a verification RE-RUN in a cred-less shell silently rewrites catalog ground truth. Reproduced live during 89-08 (coordinator reverted the churn).

**Why out-of-scope for P89:** Whether an env-gate skip should count as a re-grade event (demote) or a no-op (preserve last real grade + staleness) is a runner status-preservation design call — P90 RBF-FW-07's exact remit.

**Sketched resolution:** P90 RBF-FW-07: skip-events should not overwrite a prior real grade; instead mark staleness (e.g. `last_real_grade` + TTL) so honesty is preserved without ground-truth loss. Full analysis: 89-CROSS-AI-REVIEW.md M8.

**STATUS:** OPEN

## 2026-07-04 05:30 | discovered-by: steward-window (post-P89) | severity: MEDIUM

**What:** The `quality-weekly` GitHub Actions workflow has NEVER been green. `verdict.py` returns `1` on any non-`brightgreen` color (lines 272, 300: `return 0 if color == "brightgreen" else 1`), and `compute_color()` returns `"yellow"` whenever any P2 row is non-green (line 104-109). Two P2 `docs-repro` manual rows in `quality/catalogs/docs-reproducible.json` — `benchmark-claim/8ms-cached-read` (row @245) and `benchmark-claim/89.1-percent-token-reduction` (row @279) — are permanently `"status": "NOT-VERIFIED"` with `verifier.script: null` and `last_verified: null`. Both are `cadences: [weekly]`, so every weekly run computes yellow → exit 1 → red workflow badge. These two rows police the two most adoption-facing numbers in the project (headline latency + token-reduction in `docs/index.md` and `README.md`), so they are exactly the numbers that most deserve real verification, not verdict-softening.

**Why out-of-scope for the steward window:** The steward window is a merge/dep-hygiene batch, not a quality-gate authoring session. Writing the two verifier scripts is real work with real acceptance criteria (they must actually re-measure / re-grep and persist a fresh artifact + `last_verified`), and the 8ms row's own `owner_hint` already names an intended home: "v0.12.1 perf cross-check (`perf/headline-numbers-cross-check`) automates this row." Softening `verdict.py` to treat yellow as exit 0 was explicitly rejected as the anti-north-star move (it would hide real P2 drift, not surface it).

**Sketched resolution (north-star: VERIFY, don't soften):** Give both rows a real `verifier.script`. Their `expected.asserts` are already mechanical and cheap: (8ms row) assert `docs/benchmarks/latency.md` frontmatter `last_measured_at` is < 30 days old AND the cached-read p50 cell is within 8ms±2ms; (89.1% row) assert `89.1%` is greppable from `docs/benchmarks/token-economy.md` AND the referenced comparison fixture file exists. Both are a ~20-line python verifier apiece emitting the standard artifact JSON. Home: P95 (docs-alignment / headline-numbers automation) or P97 (release-polish), whichever the roadmap assigns `perf/headline-numbers-cross-check`. Until then, `quality-weekly` red is a known-yellow, not an acute failure.

**STATUS:** OPEN

## 2026-07-04 05:30 | discovered-by: steward-window (post-P89, PR #58 triage) | severity: LOW

**What:** Weekly bench-cron PR #58 proposed a `docs/benchmarks/latency.md` refresh whose `reposix init cold` row looked column-shuffled: sim 27ms→343ms and confluence 508ms→26ms (a fast-local backend suddenly slower than a real-network one). Investigation **rules out a generator column-shuffle bug**: `quality/gates/perf/latency-bench/emit-markdown.sh` assembles the table from per-backend named variables (`SIM_*`/`GH_*`/`CF_*`/`JR_*`) into a fixed `sim|github|confluence|jira` column order — cross-column misalignment is structurally impossible, and the body rows (List/Get/PATCH) stayed correctly aligned (confluence's newly-populated cells hold network-plausible values). The real cause: the `init cold` cell is a **single sample** (`latency-bench/sim.sh:12` "init cold — single sample"), unlike the median-of-3 body rows. sim always runs first and eats one-time cold-process/disk/git-index warmup (343ms, still within the documented `<500ms` WARN band, `latency-bench.sh:34`); real-backend inits then run warm and, with `--filter=blob:none` (blobs=0, lazy), are network-cheap (~26ms). The prior run's 508ms confluence was the inverse warmup landing. So the numbers are **legitimate-but-misleading**, not a regression and not a bug.

**Why out-of-scope / why #58 was closed unmerged:** Not a generator bug, so the steward-window "fix-if-<1h-generator-bug" branch didn't apply. Merging would publish unverified cron numbers (cron PRs get no CI per the workflow's `GITHUB_TOKEN` trigger limitation) into the single most adoption-facing benchmark, with a credibility-denting sim-slower-than-confluence inversion in the headline row. #58 closed with an explanatory comment; the cron regenerates weekly and (after the steward-window `branch-suffix: timestamp` removal, commit 2df3d4a) now updates one stable `bench/refresh-latency` PR in place.

**Sketched resolution:** The underlying weakness is that `init cold` is single-sample + run-order-dependent, so it's an unreliable per-backend headline. Options for a future perf phase (P95/P97): (a) warm-up the first init (spawn a throwaway init before timing, mirroring the TLS warm-up the body rows already do) so column 1 doesn't eat global warmup; (b) take median-of-3 for init like the other rows; or (c) footnote the row as order-sensitive and drop it from any headline. Owner-verified real-backend re-measurement (the same north-star fix as the headline-numbers rows above) would settle whether confluence init is genuinely ~26ms or the 508ms was real.

**STATUS:** OPEN

## 2026-07-04 06:20 | discovered-by: Stage-1 Lane-A (QL-001 empirical reproduction) | severity: BLOCKER

**What:** The push diff planner cannot round-trip a genuinely git-produced push from a real reposix working tree. VERIFIED by real end-to-end `git push` against the seeded sim (not a hand-crafted fast-export stream). Three compounding bugs, plus two environmental/consistency findings:

- **BUG-1 (path-shape mismatch — the ledger's QL-001).** `diff::plan` keys the prior record list on `format!("{:04}.md", id)` = bare, 4-zero-padded, NO `issues/` prefix (`crates/reposix-remote/src/diff.rs:104-107`). But every real on-disk tree uses `issues/<id>.md`. There are in fact FOUR code sites that spell the same issue's path THREE different ways:
  - cache / stateless-connect production read path → `issues/42.md` (unpadded, `issues/` subtree): `crates/reposix-cache/src/builder.rs:90` (`format!("{}.md", id)`) + `:135` (`issues` outer entry).
  - `reposix refresh` CLI → `issues/00000000042.md` (11-zero-pad): `crates/reposix-cli/src/refresh.rs:104,120` (`format!("{:011}.md", id)`).
  - import fallback (deprecated, git<2.34) → `0042.md` (bare 4-pad): `crates/reposix-remote/src/fast_import.rs:63` (`M 100644 :{} {:04}.md`).
  - push planner prior-key → `0042.md` (bare 4-pad): `diff.rs:106`.
  The planner agrees ONLY with the deprecated import fallback. Against the real cache/refresh shape, `prior_by_path.get(path)` misses for every record → each blob is a Create and each prior record is a Delete. **Reproduced:** unmodified push of a 6-issue `issues/<id>.md` tree vs a fresh 6-issue sim → `error: refusing to push (would delete 6 issues; cap is 5; commit message tag '[allow-bulk-delete]' overrides)` / `! [remote rejected] master -> main (bulk-delete)`. With ≤5 prior records the cap does NOT fire and the push silently deletes every existing record and re-creates them under fresh IDs (observed: ids 1-6 rotated to 7-11 in an earlier trial). Also: `issue_id_from_path` (`diff.rs:74-77` AND its duplicate `main.rs:432-435` — QL-157) does `strip_suffix(".md")` then `parse::<u64>()`; it returns `None` for any `issues/`-prefixed path, so the explicit `D issues/<id>.md` delete branch (`diff.rs:180`) is also silently dropped for real paths.

- **BUG-2 (planner parses non-issue blobs as issues — no path filter).** `plan()` runs `frontmatter::parse` on EVERY tree path. `reposix refresh` writes `.reposix/fetched_at.txt` + `.reposix/.gitignore` into the tree; any push carrying them fails with `error: invalid issue at .reposix/fetched_at.txt: ... missing frontmatter open fence; refusing push` / `(invalid-blob:.reposix/fetched_at.txt)`. There is no `issues/*.md`-only filter. Compounds with the fact that `parse_export_stream` inserts every `M`-added path into `out.tree` and never removes it on a later `D` (`fast_import.rs:181` insert vs `:185` deletes-vec append), so a file added then deleted across commits (e.g. a `.reposix/` blob dropped in a later commit) still reaches `plan()` and rejects the push.

- **BUG-3 (stream parser swallows a directive after commit-message data — silent data loss on the CORRECT path shape too).** `parse_export_stream`'s `data N` handler unconditionally consumes one whole line after `read_exact` (`fast_import.rs:156-157`: `let _ = r.read_line(&mut maybe_nl);`). The comment says "consume one [LF] if present" but it eats a full line whether or not an LF is present. Blob payloads are followed by an extra blank line (safe), but git fast-export emits the commit **message** payload immediately followed by the first `M` line with no extra LF — so that `read_line` swallows `M 100644 :1 <path>`. Result: the lowest-id issue (first M-line after the commit) is dropped from `parsed.tree` → classified as a spurious Delete. **Reproduced:** a NO-OP push (zero edits) of the planner's OWN native `0001.md..0006.md` shape against a fresh 6-issue sim returned exit 0 but DELETED issue 1 (`GET /projects/demo/issues/1` → HTTP 404; sim count 6→5, ids 2-6, no version bumps). This is a data-loss bug independent of BUG-1 and deterministic (always the first M-line).

- **FINDING-A (environment: production read path needs git ≥ 2.34; this box has 2.25.1).** With git 2.25, `git fetch` falls back to the deprecated `import` capability, which (a) emits the commit to `refs/reposix/origin/main` while git expects `refs/reposix/main` → `fatal: could not read ref refs/reposix/main`; and (b) a forced `-c protocol.version=2` fetch reaches stateless-connect but the cache open dies with `git: git config --add transfer.hideRefs failed: fatal: not in a git directory`. So the canonical `reposix init && git checkout origin/main && edit && git push` loop cannot run to completion on git < 2.34, and the failure messages do NOT teach "upgrade git" — they surface as cryptic ref/gitdir errors. The reproduction above therefore drove the real export→parse→plan code by constructing the real on-disk tree shapes directly and pushing them; the planner code path exercised is byte-identical to the git-2.34 path.

- **FINDING-B (test blind spot confirmed).** Every unit test in `diff.rs` (`five_deletes_passes_cap`, `unchanged_push_emits_no_patches`, etc.) hand-builds the `parsed.tree` map with the bare-padded `0001.md` shape, so they all pass while never touching the real path shape. `quality/gates/agent-ux/dark-factory/sim.sh` asserts only working-tree CONFIG (partialClone/promisor/filter) and helper stderr-teaching strings — it never performs a `git push`, so the round-trip is untested at the agent-ux gate too.

**Why out-of-scope for eager-resolution:** This is not a contained single-crate fix. A coherent fix must (1) pick ONE canonical on-disk path shape and align four code sites (`builder.rs`, `refresh.rs`, `fast_import.rs`, `diff.rs`) + the shared `issue_id_from_path` contract (consolidating the QL-157 duplicate as part of it — consolidating the two copies NOW would be churn, since the correct fix rewrites the function to strip the `issues/` prefix and tolerate any/zero padding); (2) add an `issues/*.md` path filter to the planner; (3) harden the stream parser to peek-and-consume the optional LF instead of eating a line, plus reconcile M-then-D within the stream; and (4) ship a regression test that performs a REAL `git push` (not a synthetic stream) whose green state depends on all of the above landing together — a parser-only fix cannot be verified against reality because the path-shape bug still fails the push. Attempting a partial fix would land code whose regression test cannot pass and risks half-fixing the planner semantics.

**Sketched resolution (P90/P91 MUST consume):** Decide the canonical path shape (recommend `issues/<id>.md` unpadded — the cache/stateless-connect production shape at `builder.rs:90`, which is also the CLAUDE.md-documented UX `issues/42.md`) and make `refresh.rs`, `fast_import.rs` (import + emit), and `diff.rs` all produce/consume exactly that, with a single shared `issue_id_from_path` that strips the `issues/` prefix. Add the non-issue-path filter. Fix the parser's trailing-LF handling (peek one byte; consume only if `\n`) and make `D` remove from `out.tree`. Then add the agent-ux `git push` round-trip.

**Sharpened acceptance criteria (verifiable, must be a REAL push):**
1. A real `git push` from a `reposix init`'d tree (or a tree in the canonical `issues/<id>.md` shape) round-trips exactly ONE edited record against the sim as a single PATCH, with **zero** manual fast-export and **misclassification count == 0** (no Creates, no Deletes).
2. A **no-op** push (pull, no edits, push) produces **zero** backend writes — no create/update/delete, no record deleted, no version bump on any record (directly falsifies BUG-3).
3. A push of the full seeded tree against a matching sim produces **zero** Delete actions (falsifies BUG-1's create/delete storm).
4. A tree containing `.reposix/` metadata (as `reposix refresh` writes) pushes without an `invalid-blob` rejection (falsifies BUG-2).
5. The regression runs a REAL `git push` end-to-end at the agent-ux gate; if the harness requires git ≥ 2.34, it asserts the version precondition and fails loud (not silently skips) on older git.
6. All four path-shape sites are grep-verifiable as producing the single canonical spelling for a given id.

**Repro recipe (self-contained; drove the real export→parse→plan path on git 2.25 by constructing the real tree shapes):**
```
# sim: target/debug/reposix-sim --bind 127.0.0.1:7878 --ephemeral \
#        --seed-file crates/reposix-sim/fixtures/seed.json   (6 issues, ids 1-6)
# PATH must include target/debug (for git-remote-reposix); export
#   REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:7878
# In a fresh git repo, remote.origin.url=reposix::http://127.0.0.1:7878/projects/demo:
#   BUG-1: commit issues/1.md..issues/6.md (rendered by `reposix refresh`), push
#          -> "refusing to push (would delete 6 issues; cap is 5)"
#   BUG-3: commit 0001.md..0006.md (bare shape), push unmodified
#          -> exit 0 but issue 1 GONE (GET /projects/demo/issues/1 -> 404)
#   BUG-2: keep `reposix refresh`'s .reposix/fetched_at.txt in the tree, push
#          -> "invalid issue at .reposix/fetched_at.txt ... refusing push"
```

**ADDENDUM 2026-07-04 (doc-alignment unblock lane) — the checkout claim is untested-and-unverifiable until this lands.** The docs-alignment grader flipped `docs/index/git-checkout-branch-command` (claim: "`git checkout -B main refs/reposix/origin/main` switches to main branch", cite `docs/index.md:129`) `BOUND → MISSING_TEST`: its prior binding `crates/reposix-cli/tests/agent_flow.rs::dark_factory_sim_happy_path` asserts only partial-clone CONFIG and never runs the checkout. The one test in the tree that DOES run the exact command — `quality/gates/docs-repro/tutorial-replay.sh` step 4 (`git checkout -B main refs/reposix/origin/main`, lines 76-82) — **HARD-FAILS (not skips)** when `refs/reposix/origin/main` fails to resolve, which is exactly the FINDING-A / BUG-1 failure on git < 2.34 and on the broken push/fetch path. So no test can be graded GREEN against this claim today without violating "don't declare green without seeing green": impossible on this git-2.25.1 box, and suspect on git ≥ 2.34 until BUG-1's canonical path shape lands. The row is therefore left `MISSING_TEST` (honest). **Walker impact (asked + answered):** `MISSING_TEST` is in `RowState::blocks_pre_push()` (`crates/reposix-quality/src/catalog.rs:362-369`), so this single row makes `quality/gates/docs-alignment/walk.sh` exit 1 and BLOCKS pre-push **independent of the floor/coverage ratios** — the block is per-row (each `blocks_pre_push()` state pushes a blocking line → `walk` returns `Ok(1)`), not a floor effect. When P90/P91 lands the real round-trip (acceptance criterion #5 above), bind this row to that regression; alternatively a human may soften/retire the `docs/index.md:129` checkout claim. NB: the sibling row `docs/tutorials/first-run/checkout-origin` (same checkout, claim "succeeds after reposix init") is *still* bound to the config-only `dark_factory_sim_happy_path` and was NOT flipped — an inconsistency the grader left; it has the same latent weakness.

**STATUS:** OPEN

## 2026-07-04 05:40 | discovered-by: steward-window (post-P89) | severity: LOW

**What:** Editing any file that a docs-alignment row binds via a **whole-file hash** flips that row `BOUND` → `STALE_TEST_DRIFT` even when the row's actual claim is untouched. The steward window's committed changes tripped 3 such rows (`claims_bound` 270→267, `alignment_ratio` 0.804→0.795, still well above the 0.5 floor so the pre-push walk stayed GREEN): one bound to `bench-latency-cron.yml` (whole-file hash; drifted by the item-4 `branch-suffix: timestamp` removal — the cited `line 32` persist-credentials claim is unchanged) and two bound to `release.yml` (aarch64-musl matrix + 5-target matrix whole-file hashes; drifted by the sanctioned dependabot `actions/checkout@v7` + `upload/download-artifact` version-ref bumps merged in #35/#36/#37). All 3 claims remain TRUE; only the whole-file hashes are stale. The pre-push docs-alignment walker also mutates `quality/catalogs/doc-alignment.json` in place (last_walked + verdict flips), so a chore that touches any hash-bound file leaves the tree dirty post-push — committed here per the established `catalog(...)` convention (e.g. a4ac3e4).

**Why out-of-scope for the steward window:** Re-binding is a deliberate docs-alignment operation (`reposix-quality doc-alignment bind` needs full per-row claim/source/test/grade/rationale re-specification; `/reposix-quality-refresh` is explicitly top-level-only deliberate work). Doing it via the stale May-1 `target/debug/reposix-quality` binary with hand-constructed args risks catalog corruption for a purely-mechanical rehash — higher risk than value inside a merge/dep-hygiene batch.

**Sketched resolution:** Run `/reposix-quality-refresh` (or `reposix-quality doc-alignment bind` per row) against the 3 STALE_TEST_DRIFT rows to rehash them back to BOUND — the claims are all still valid, so it's a clean re-bind that restores `claims_bound` to 270. Home: P90 (quality-framework) or a standalone refresh run. Separately worth a design look: whole-file-hash bindings are brittle — a version-ref bump on an unrelated line reddens a matrix-presence claim; a line-anchored or content-substring binding would be more robust for these "file contains X" claims.

**STATUS:** OPEN

## 2026-07-04 08:30 | discovered-by: Stage-2 catalog-row-minting | severity: MEDIUM

**What:** Adding `## \`reposix attach\`` and `## \`reposix sync\`` sections to `docs/reference/cli.md` (prerequisite doc fixes for the new `structure/cli-subcommand-parity` catalog row) shifted line numbers throughout the file, which flipped 7 line-anchored docs-alignment rows on `docs/reference/cli.md` to `STALE_DOCS_DRIFT` (plus 2 more on `docs/reference/exit-codes.md`, one via source drift and one via an unrelated test-body drift). Ran `/reposix-quality-refresh` on both docs — dispatched 10 Opus grader subagents (one per stale row) per the normal playbook. Result: 5 rows re-bound cleanly `BOUND` with corrected line citations (pure line-shift, claim + test both still valid) — `init_documented`, `list_query_backend`, `list_documented`, `mount_removed_v0.9.0`, `exit-codes/helper-exit-1-push-rejected`. But **5 rows the graders correctly refused to re-bind**, because the cited test was a loose name-match that never actually asserted the claim (a pre-existing weakness the whole-file/line-range hash had been silently papering over until this drift forced a re-grade):

- `docs/reference/cli.md/subcommands_exist` — claim "all CLI subcommands documented in help text"; test `help_lists_all_subcommands` only asserts `[sim, init, list, version]` appear, never checks `attach`/`sync`/the other 9.
- `docs/reference/cli.md/env_vars` — claim "7 env vars documented"; the cited test never references any env var at all.
- `docs/reference/cli.md/exit_codes` — claim "exit codes 0/1/2 documented"; the cited test (`mount_subcommand_is_removed`) only asserts a removed-subcommand error, not exit-code semantics.
- `docs/reference/cli.md/spaces_confluence_only` — claim "spaces lists Confluence spaces, Confluence-only"; cited test only checks `spaces` appears in top-level `--help`, never the Confluence-only constraint.
- `docs/decisions/009-stability-commitment/exit-codes-locked` — claim "exit codes locked under semver"; cited test (`gc_help_renders`) only asserts `gc --help` flag rendering, nothing about exit-code stability.

All 5 are now honestly `MISSING_TEST`, which per `RowState::blocks_pre_push()` (`crates/reposix-quality/src/catalog.rs:431-440`) blocks `quality/gates/docs-alignment/walk.sh` (and therefore `quality/runners/run.py --cadence pre-push`) independent of the alignment_ratio/coverage_ratio floors.

**Why out-of-scope for eager-resolution:** Fixing this properly means writing 5 new tests (or retargeting each to an existing test that genuinely asserts the claim) — real design work per claim (e.g., what does "all subcommands documented" even mean as an automatable assertion once there are 15 subcommands; what's the right test for "env vars are documented" as opposed to "env vars are enforced"). That's real per-claim design work, not a mechanical rebind — well over the 1-hour eager-fix budget and risks the exact false-BOUND failure mode the refresh playbook is designed to prevent if rushed.

**Sketched resolution:** For each of the 5 rows, either (a) write a dedicated test that actually asserts the claim (e.g. a `help_lists_all_subcommands`-style test parameterized over the FULL current subcommand list including `attach`/`sync`; a doctor/allowlist test that references the documented env-var names; an exit-code-table test that drives each subcommand to its documented code), or (b) soften/retire the claim if it's not worth a dedicated test (e.g. `spaces_confluence_only`'s "Confluence-only" half could fold into the existing `spaces.rs` bail-message tests instead of `help_lists_all_subcommands`). Whoever picks this up should re-run `/reposix-quality-refresh docs/reference/cli.md` and `/reposix-quality-refresh docs/reference/exit-codes.md` afterward to flip the rows back to `BOUND`. Until then, `pre-push` on this repo will show `docs-alignment/walk` RED for these 5 specific rows — this is a known, tracked, honest state, not a regression introduced by the catalog-row-minting work that triggered the re-grade.

**STATUS:** OPEN

## 2026-07-04 18:10 | discovered-by: quality-convergence connector re-audit | severity: HIGH

**What:** ~440 lines of dead Confluence API surface — `list_comments` (crates/reposix-confluence/src/lib.rs:501-570), `list_attachments` (644-705), `list_whiteboards` (720-803), `download_attachment` (819-844) plus their backing types in types.rs (~184-388: ConfComment/CommentKind/ConfCommentList, ConfAttachment/ConfAttachmentList, ConfWhiteboard/ConfDirectChildrenList/ConfDirectChild) have zero non-test callers workspace-wide. Nothing in reposix-cache or reposix-remote references comments/attachments at all, so none of this reaches a working tree. Each function is individually well-documented and well-tested — this is orphaned code built ahead of a cache-integration phase that never landed, not sloppy code. Related: `CommentSupport::SeparateApi` in the capability surface is backed only by this dead code, so any doc cell implying reposix delivers comment access is aspirational.

**Why out-of-scope for eager-resolution:** wire-vs-delete is a product decision (do comments/attachments become working-tree files in P91+ real-backend wiring, or do we shed the surface?), and either path is M-sized (materializer integration + record layout, or deletion + type cleanup + capability/doc reconciliation).

**Sketched resolution:** P91/P92 (real-backend charter) decides: (a) wire comments/attachments into the cache materializer as working-tree content, or (b) delete the four functions + backing types, downgrade `CommentSupport` claims, and update the docs capability matrix. Until decided, ensure no user-facing doc promises comment/attachment access.

**STATUS:** OPEN

## 2026-07-04 18:10 | discovered-by: quality-convergence connector re-audit | severity: HIGH

**What:** reposix-swarm has zero write-contention coverage for the one real backend it drives directly. `confluence_direct.rs` never calls `create_record`/`update_record` even though Confluence writes have been live for over a milestone — the harness's entire purpose is multi-agent contention testing, and the write-contention path (the risky one: version conflicts, push-time drift) is untested by it. (The stale "Phase 17 read-only by design — writes ship in Phase 21 / OP-7" comment is being fixed eagerly in the convergence fix wave; this entry tracks the missing workload.)

**Why out-of-scope for eager-resolution:** designing a write-contention workload (N agents racing update_record on shared records, asserting version-conflict handling + audit-row completeness) is M-sized test-harness work with real-backend etiquette concerns (TokenWorld mutation volume).

**Sketched resolution:** add a swarm write-contention scenario against the simulator first (default per OP-1), then an --ignored real-Confluence variant against TokenWorld per docs/reference/testing-targets.md cleanup conventions. Home: P91 (real-backend wiring) or P95.

**STATUS:** OPEN
