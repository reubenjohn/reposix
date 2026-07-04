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
