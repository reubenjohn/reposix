# Phase P83 Audit — bus-write-fan-out
**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08

## Verdict at a glance
- ALIGNED items: 7 (the SoT-first algorithm, no-helper-retry contract, plain-push (no `--force-with-lease`), no-mirror-remote regression, mirror-lag head/synced-at semantics on partial-fail, fault-injection scenario assertions internally well-formed, audit_events_cache row counts on cache side)
- MISALIGNED items: 5 (audit-completeness false dual-table, fault-injection scope is sim-only, frontmatter validator collides with documented mirror tree, OP-3 audit-events backend table never written by helper, audit op `helper_push_partial_fail_mirror_lag` row content is unverified beyond count)
- SUSPECT items: 2 (force-warning surface + interaction with PRECHECK A; the `_provenance_note` hand-edit on every catalog row implying tooling gap)

## Findings

### F1 — Fault-injection coverage is 100% wiremock + file:// mirror; zero real-backend exercise [SEVERITY: HIGH]
**Claim in plan:** The phase tagline ("the riskiest in v0.13.0") explicitly carves out "fault injection" as the load-bearing deliverable; `83-PLAN-OVERVIEW/index.md:55` and ROADMAP §53 promise "kill-GH-push-between-confluence-write-and-ack / kill-confluence-write-mid-stream / simulate-confluence-409-after-precheck". `83-RESEARCH/08-fault-injection.md` enumerates Tests (a)/(b)/(c) and the `make_failing_mirror_fixture` (file:// bare repo + failing `update` hook).
**Reality:** All 6 integration tests in `crates/reposix-remote/tests/bus_write_*.rs` (1937 LOC) use `wiremock::MockServer` for the SoT side and a `file://` bare-repo with a fake `update` hook for the mirror side. Zero tests use the real `git push` to a real GitHub remote, real Confluence/JIRA, or even an in-process sim binary. The `make_failing_mirror_fixture` "kill-GH-push" scenario is a per-repo `hooks/update` shell that exits 1 — a synthetic local rejection that bypasses every real-world failure mode (network timeout, GitHub API rate limit, server-side push protection rules, OAuth token revocation, packfile rejection from receive.maxInputSize, hook output exceeding stderr buffer, etc.).
**Evidence:**
- `crates/reposix-remote/tests/bus_write_mirror_fail.rs:42-47` (wiremock imports), `:140-198` (`make_failing_mirror_fixture` use); fixture body at `crates/reposix-remote/tests/common.rs:145-198`.
- `crates/reposix-remote/tests/bus_write_happy.rs:46-47, :143-181` (file:// mirror fixture).
- `crates/reposix-remote/tests/bus_write_audit_completeness.rs:45-46, :165` (`MockServer::start()`).
- `quality/gates/agent-ux/bus-write-fault-injection-mirror-fail.sh:21-23` only invokes the cargo test (no real-backend variant).
- `agent_flow_real.rs` (`crates/reposix-cli/tests/agent_flow_real.rs`): zero references to bus push, mirror, or `?mirror=` URL construction (`grep -n "bus\|mirror\|reposix::"` returns only the existing single-backend URL-shape assertions).
**Why it matters:** P83 is named the milestone's riskiest phase precisely because cross-system partial-failure modes are real-backend phenomena. Shipping the catalog row `agent-ux/bus-write-fault-injection-mirror-fail` PASS while every assertion runs against a wiremock + local-hook fixture means the project's mirror-fail recovery contract has zero in-anger validation. This directly maps to the Failure Shape #1 from the AUDIT-BRIEF ("test name promises one thing, assertions deliver less"): the catalog `description` says *"mirror push fails between confluence-write and ack"* — a real-backend Atlassian-vs-GitHub atomicity story; the assertion is *"a local file:// bare-repo's update hook returns exit 1."*

### F2 — Frontmatter-only validator rejects the documented mirror tree (`.github/workflows/*.yml`, `README.md`, `.reposix/*`) [SEVERITY: HIGH]
**Claim in plan:** Bus push fans out SoT-first then mirror-best-effort; the user's working tree is the same tree the GH-Action workflow lives in (mirror setup walk-through at `docs/guides/dvcs-mirror-setup.md` Step 4 + `docs/guides/dvcs-mirror-setup-template.yml` enforce `.github/workflows/reposix-mirror-sync.yml` in the mirror repo). Pattern C in `docs/concepts/dvcs-topology.md` line 122-137 walks: vanilla `gh repo clone` of the mirror → `reposix attach` → edit → `git push`. The architecture-sketch (`.planning/research/v0.13.0-dvcs/architecture-sketch/webhook-sync.md:9`) literally states the mirror repo HOLDS `.github/workflows/reposix-mirror-sync.yml`.
**Reality:** `crates/reposix-remote/src/diff.rs:99-123` `plan()` walks every `(path, mark)` in `parsed.tree`, calls `frontmatter::parse(&text)` on each blob, and bails with `PlanError::InvalidBlob { path, source }` (which becomes `error refs/heads/main invalid-blob:<path>` per `write_loop.rs:223-230`). Any non-record blob in the working tree at push time hits this check. The frontmatter parser at `crates/reposix-core/src/record.rs:183` raises `"missing frontmatter open fence"` on any file missing `---\n`. So:
- `.github/workflows/reposix-mirror-sync.yml` → reject.
- `README.md` (`d224f47` initial commit on the mirror) → reject.
- `.reposix/.gitignore` + `.reposix/fetched_at.txt` (created by `reposix refresh`) → reject.

Bus push CANNOT succeed against a mirror that follows the documented setup. There is no path-prefix scope, no skip-list, no allowlist. The `plan()` function unconditionally walks every tree blob.
**Evidence:**
- Validator: `crates/reposix-remote/src/diff.rs:99-123`.
- Reject path: `crates/reposix-remote/src/write_loop.rs:223-230`.
- Frontmatter error: `crates/reposix-core/src/record.rs:183`.
- Mirror setup REQUIRES the workflow file: `.planning/research/v0.13.0-dvcs/architecture-sketch/webhook-sync.md:9`, `docs/guides/dvcs-mirror-setup-template.yml` (and the `agent-ux/webhook-trigger-dispatch` gate enforces byte-equal copies).
- Real-world reproduction: `T3-bus-push.md:158-186` (F12 + F13).
- ZERO mention in `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`, `GOOD-TO-HAVES.md`, or `CARRY-FORWARD.md` (verified via `grep -in "workflow\|frontmatter\|invalid-blob\|.reposix/" ...`).
**Why it matters:** This is an architectural contradiction *baked into the v0.13.0 design itself* — the same milestone ships the mirror-setup walk-through (P85) and the bus-push helper (P83) without either phase noticing the collision. The frontmatter validator is older code, but P83's job was to *integrate* it into the bus-push fan-out; the integration step never asked "what's actually in the mirror tree?" The dark-factory regression P86 was supposed to catch it; T3 confirms P86 didn't.

### F3 — `audit-completeness` row's "dual-table" assertion is a single-table assertion with a wiremock stand-in [SEVERITY: HIGH]
**Claim in plan:** `83-PLAN-OVERVIEW/index.md:53-56` lists T04 deliverable as *"audit-completeness test (`bus_write_audit_completeness.rs` queries BOTH audit tables on happy path)"*. Catalog row `agent-ux/bus-write-audit-completeness` `expected.asserts` (`quality/catalogs/agent-ux.json:748-753`) lists *"audit_events (wiremock request log proxy): per-record mutation requests (PATCH/POST/DELETE) match the executed actions."* CLAUDE.md OP-3 (line 37): *"`audit_events` … is written by the sim/confluence/jira adapters. A complete forensic query reads both. Either schema missing a row for a network-touching action means the feature isn't done."*
**Reality:** The test (`crates/reposix-remote/tests/bus_write_audit_completeness.rs:293-337`) does NOT open or query `audit_events`. It opens `audit_events_cache` directly via `count_audit_cache_rows` and treats `wiremock.received_requests().filter(method.is_mutation)` as "the byte-equivalent of `audit_events`." The test's own block comment at `:14-23` admits this: *"this test runs against an in-process `SimBackend` pointed at a `wiremock::MockServer`; there is no `audit_events` SQLite table."* Worse, in *production* the helper's bus push for a real Confluence backend NEVER writes `audit_events` either — `crates/reposix-remote/src/backend_dispatch.rs:242-260` builds `ConfluenceBackend::new_with_base_url(...)` but does NOT chain `.with_audit(audit_conn)` (the sole way to enable audit-events writes per `crates/reposix-confluence/src/client.rs:130-141`). Same for `instantiate_jira` (line 262), and `reposix-github` follows the same pattern. So OP-3's `audit_events` table is unwritten on every real-backend bus push.
**Evidence:**
- Test: `crates/reposix-remote/tests/bus_write_audit_completeness.rs:14-23, :293-337`.
- `with_audit` is the only switch: `crates/reposix-confluence/src/client.rs:130-141`.
- Helper-side instantiation skips `with_audit`: `crates/reposix-remote/src/backend_dispatch.rs:242-274` (confluence + jira + github paths).
- `crates/reposix-confluence/src/lib.rs:313-437` shows `audit_write` calls in `update_record`/`delete_record` — guarded internally by `if let Some(audit) = ...`. Without `with_audit`, the audit branch is silently skipped.
- Dark-factory T1+T4 corroborates (`SUMMARY.md:57-69` — Cluster C: "every `git push` writes ZERO `helper_push_*` rows"; T3-bus-push.md F16: dual cache.db locations, both empty/unwritten on real Confluence push).
**Why it matters:** The catalog row claims "dual-table" but exercises only one table; the production code path can't write the second table at all. This is OP-3 violated by construction *and* the verification layer hides it because the dual-table claim is checked against a test that doesn't have a second table. The verdict at `quality/reports/verdicts/p83/VERDICT.md:50-51` doubles down by calling this "honesty spot-check 4: PASS — both layers asserted." It's not.

### F4 — `helper_push_partial_fail_mirror_lag` row count asserted; row CONTENT (sot_sha, exit_code, stderr_tail) never queried [SEVERITY: MED]
**Claim in plan:** `83-PLAN-OVERVIEW/index.md:99-103` defines the partial-fail row's load-bearing fields: *"records the exit code + stderr tail (3-line tail, matches RESEARCH.md Pattern 2)"*. `83-RESEARCH/08-fault-injection.md:23` says *"audit_events_cache count where op = `helper_push_partial_fail_mirror_lag`: 1"*. CLAUDE.md (line 35 of v0.13.0 update) names the new audit op as a load-bearing deliverable.
**Reality:** `bus_write_mirror_fail.rs` checks the row exists (count == 1) but never SELECTs the row's payload columns. `count_audit_cache_rows` (`crates/reposix-remote/tests/common.rs:208-216`) is `SELECT COUNT(*) FROM audit_events_cache WHERE op = ?1` — only `op` is filtered; the `sot_sha`, `exit_code`, and `stderr_tail` payload columns (whatever shape `log_helper_push_partial_fail_mirror_lag` writes) are never read. The verifier at `bus_write_mirror_fail.rs:264` greps the test stderr for `"exit="` substring as a proxy — but stderr is the helper's WARN message, not the persisted audit row. So the audit row's persisted content is unverified.
**Evidence:**
- `crates/reposix-remote/tests/common.rs:208-216` (count-only query).
- `crates/reposix-remote/tests/bus_write_mirror_fail.rs:264` (stderr-only substring check, not DB query).
- The audit-row-content verification gap is invisible to the catalog row's `expected.asserts` at `quality/catalogs/agent-ux.json:622-630` because each assert is a behavior-level claim (*"refs/mirrors/<sot>-head advanced"*, *"ok refs/heads/main"*) and not a row-content claim.
**Why it matters:** The audit row is the forensic recovery handle. A row that exists but has corrupt or empty `stderr_tail` / `exit_code` / `sot_sha` is forensically useless — the entire OP-3 contract assumes the row's content is queryable to attribute the partial fail. The test cannot regress on a row-content bug.

### F5 — `helper_push_partial_fail_mirror_lag` row exists for the SoTPartialFail outcome too? Plan says no, code says no, but assertion is missing [SEVERITY: LOW]
**Claim in plan:** `83-PLAN-OVERVIEW/index.md:114-120` says *"On the SoT-fail path … Mirror push is NEVER attempted."* `83-RESEARCH/08-fault-injection.md:39` says *"audit_events_cache count where op = `helper_push_partial_fail_mirror_lag`: 0 (mirror never attempted)."* The decision is correct.
**Reality:** Test `bus_write_sot_fail.rs:300-305` does assert `partial == 0`; test `bus_write_post_precheck_409.rs` does likewise (`bus_write_post_precheck_409.rs:262`-ish per the verdict). But neither test exercises the `SotPartialFail` outcome where any_failure is set after a successful action — the mid-stream test specifically fails at PATCH 2 of 2, never crossing the success branch. There's no test for "1 update succeeds, 1 update fails, mirror push state on the partial outcome." This is the actual D-09 / Pitfall 3 surface area (Confluence non-atomicity).
**Evidence:**
- `crates/reposix-remote/src/write_loop.rs:253-268` — the `for action in actions { ... }` loop sets `any_failure = true` and continues; returns `WriteOutcome::SotPartialFail` only when the loop *completes* with at least one failure, but `bus_write_sot_fail.rs` configures the second PATCH to 500 so the loop bails at id=2 (likely also setting `any_failure`). However the existing test asserts `helper_push_accepted == 0` which is correct (the success branch is never reached); but it doesn't probe the case where one PATCH succeeded server-side. The test's stated D-09 / Pitfall 3 acknowledgement at the test source comment is informational, not asserted.
**Why it matters:** Low because the audit-row-count behavior is correct in code; the gap is that the regression assertion cannot catch a future bug where someone moves the partial-fail audit write into the SotPartialFail branch (a plausible refactor mistake). The decision is correctly implemented; the test doesn't fence it.

### F6 — `bus-write-no-helper-retry` verifier is grep-on-source, not behavior-on-failure [SEVERITY: MED]
**Claim in plan:** Q3.6 RATIFIED no helper-side retry. Catalog row's `expected.asserts` (`quality/catalogs/agent-ux.json:543-548`): *"crates/reposix-remote/src/bus_handler.rs contains NO retry constructs adjacent to push_mirror (no `for _ in 0..`, no `loop {`, no `tokio::time::sleep`)"*.
**Reality:** The verifier (`quality/gates/agent-ux/bus-write-no-helper-retry.sh`) greps `bus_handler.rs` for substrings. This catches "did someone wrap the call in a loop?" but does NOT catch:
- A retry loop *inside* `git push` itself (e.g. `[http]` config retry, `[transfer.retry]`, OAuth re-auth retry built into a future `gix push` substitution).
- A retry implemented *before* `push_mirror` is called (e.g. wrapping the whole `apply_writes` + `push_mirror` block in a loop).
- A retry implemented in a different file (e.g. a future `mirror_pusher.rs` extracted from `bus_handler`).
- A retry of the *whole* helper invocation triggered by a sibling crate.

The grep is fragile to renames; per `quality/reports/verdicts/p83/VERDICT.md:108` (M3) the executor specifically calls out "fixture rename risk" as a known fragility — same fragility applies to source-pattern verifiers.
**Evidence:**
- Verifier shell at `quality/gates/agent-ux/bus-write-no-helper-retry.sh` (referenced from catalog row).
- D-08 substring matches in `crates/reposix-remote/src/bus_handler.rs:478` are correct as of HEAD, but a no-retry contract that's only checked by source greps is structurally fragile.
**Why it matters:** "Behavioral" no-retry verification would observe an `Err` from a single mirror push, then assert the helper terminates rather than retries. That's testable: instrument `push_mirror` to record call count, drive a failing fixture, assert call count == 1. The current grep is a structural proxy that survives until someone moves the loop one layer up.

### F7 — Velocity smell: P83 marked as "milestone's riskiest phase," shipped in 11 commits with no real-backend exercise; eager-resolution carried unchecked weight [SEVERITY: MED]
**Claim in plan:** ROADMAP §53 calls P83 *"the riskiest part of the bus remote"*. Plan-overview at `83-PLAN-OVERVIEW/index.md:17` echoes *"the **riskiest** DVCS-substantive phase of milestone v0.13.0"*.
**Reality:** Per the verdict (`quality/reports/verdicts/p83/VERDICT.md:142-153`), 11 commits + 1 fix-forward over 2 days (4b7be9d…bd903bb spanning 2026-04-30 → 2026-05-01 from commit metadata in the tree). 1937 LOC of integration tests, all sim-only. Phase close ratified all 8 catalog rows PASS without any real-backend exercise. The single SURPRISES-INTAKE entry from P83 (`SURPRISES-INTAKE.md:58-66`) is a *fixture bug* (`core.hooksPath` override) — significant, but a test-infrastructure detail, not a load-bearing architecture finding. The ZERO entries about the `.github/workflows/*.yml` collision (F2) is the smell — the architecture-sketch's own webhook-sync chapter mentions the workflow file, P83 wired the export validator, P83's own fixture uses `git push` to a bare mirror, but no human or subagent in the loop noticed the contradiction. That's a velocity-as-skip-signal pattern: the phase shipped fast precisely because it didn't try the docs.
**Evidence:**
- `quality/reports/verdicts/p83/VERDICT.md:142-153` (commit table, 11 commits over 2 days).
- ZERO architecture-collision SURPRISES (`grep -in "workflow\|frontmatter\|invalid-blob" .planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` returns hits only for unrelated webhook-binstall issue at line 70+ and unrelated P80 fixture issue at 58).
- T3 dark-factory exercise (post-hoc, at the milestone-close boundary) caught both F2 and F3 within a 25-minute end-to-end attempt (`T3-bus-push.md`).
**Why it matters:** This is the failure-shape #6 from AUDIT-BRIEF. The phase was scoped as the milestone's riskiest yet shipped without exercising the most consequential failure mode (mirror-tree-vs-export-validator collision). Plan-time questions (Q-A through Q-F, then D-01 through D-10) covered narrow refactor mechanics, not user-facing topology end-to-end.

### F8 — Catalog rows hand-edited with `_provenance_note` apology; framework hasn't shipped the verb extension that would automate them [SEVERITY: LOW]
**Claim in plan:** `quality/catalogs/agent-ux.json:453, 494, 535, 571, 610, 653, 695, 737` (every P83 row) carries: *"Hand-edit per documented gap (NOT Principle A): reposix-quality bind only supports the docs-alignment dimension at v0.13.0; agent-ux dim mints stay hand-edited until GOOD-TO-HAVES-01 ships the verb extension."*
**Reality:** The hand-edit pattern is OK as a documented bridge, but `GOOD-TO-HAVES-01` hasn't shipped — every new agent-ux row in P83 (and P82, P80, P79 before it) is hand-edited. With 8 rows minted in P83 alone, the hand-edit cost compounds; per the meta-rule (CLAUDE.md "Quality Gates" section: *"Adding a new gate is one catalog row + one verifier in the right dimension dir"*), the agent-ux dimension diverges from this standard. Rows are also missing the `last_verified` JSON timestamp invariants used by docs-alignment (e.g., no freshness TTL).
**Evidence:**
- `quality/catalogs/agent-ux.json` rows have `freshness_ttl: null` for all 8 P83 rows (`grep -A1 "agent-ux/bus-write" quality/catalogs/agent-ux.json | grep freshness_ttl` returns null for every one).
- GOOD-TO-HAVES-01 referenced inline 8x but no commit reference signaling shipment in the verdict.
**Why it matters:** Cosmetic / future-work pointer. Doesn't block the phase but signals process drift: hand-editing 8 catalog rows per phase contradicts CLAUDE.md's "one catalog row + one verifier" pitch.

### F9 — `force` semantics divergence between PRECHECK A and helper protocol; user-facing surface unclear [SEVERITY: LOW / SUSPECT]
**Claim in plan:** D-08 RATIFIED: NO `--force-with-lease`, NO `--force` for mirror push (the helper's outbound git push). The plan never addresses the inbound side: what should the helper do when the user runs `git push --force origin main` to the *bus* URL?
**Reality:** Dark-factory T3 (F15) showed: helper prints `warning: helper reposix does not support 'force'` (a generic git remote-helper protocol message) but then proceeds to run the validator which rejects the push. The user perceives `--force` as discarded; PRECHECK A still gates the push; if PRECHECK A drifts the user has no escape hatch. There's no test for the inbound `force` surface — `bus_write_no_helper_retry.sh` only verifies absence of force flags in helper-internal call sites; doesn't probe inbound user-driven force semantics.
**Evidence:**
- T3-bus-push.md F15.
- No inbound-force test in `crates/reposix-remote/tests/bus_*.rs` (verified via `grep -n "force" crates/reposix-remote/tests/bus_*.rs`).
**Why it matters:** SUSPECT — would need a planning-level decision (does bus push support inbound force? force-with-lease? a `?force=ok` query param? user runs `reposix sync --reconcile` instead?) to settle. P83 plan didn't surface the question.

### F10 — `mirror-lag-ref` write on partial-fail: head ref advances inside `apply_writes` BEFORE the mirror push subprocess fires, not in a same-transaction commit [SEVERITY: LOW]
**Claim in plan:** `83-PLAN-OVERVIEW/index.md:97-100` *"On the SoT-succeed-mirror-fail path: `refs/mirrors/<sot>-head` IS updated to the new SoT SHA (head moved). `refs/mirrors/<sot>-synced-at` is NOT touched."*
**Reality:** The head ref advance happens at `crates/reposix-remote/src/write_loop.rs:286-291` — INSIDE `apply_writes`, BEFORE `push_mirror` is called. In the partial-fail path, `bus_handler.rs:301` then writes `helper_push_partial_fail_mirror_lag` and ack's `ok` to git. There is no atomicity guarantee — if the helper crashes BETWEEN `write_mirror_head` (line 288) and the `push_mirror` invocation in the bus path (`bus_handler.rs:264`), the head ref records the new SoT SHA but no mirror push was even attempted, no audit row was written. The state is observable only by absence of `helper_push_partial_fail_mirror_lag`. Recovery on next push works (PRECHECK B), but the audit forensic trail loses the "head-advanced-but-mirror-not-tried" interim.
**Evidence:**
- `crates/reposix-remote/src/write_loop.rs:286-291` (head ref write inside apply_writes).
- `crates/reposix-remote/src/bus_handler.rs:264` (push_mirror call site, after apply_writes returns).
- Same gap, same direction: cache-internal audit row writes happen via best-effort `tracing::warn!` if they fail (`write_mirror_head failed: {e:#}`); the helper still continues. There's no guarantee the head ref write is visible if the cache.db write failed silently.
**Why it matters:** Edge case under crash; not a behavioral defect on happy/sad paths. Documented in plan as "the head ref moves on partial fail," but the claim is mechanically more nuanced than the plan describes.

### F11 — Real-backend coverage gate is conspicuously absent from the catalog cadence list for 8 P83 rows [SEVERITY: MED]
**Claim in plan:** OP-1 + OP-6 + OP-7: simulator-only coverage does NOT satisfy acceptance for transport-layer claims; real backends are first-class test targets; phase-close means catalog-row PASS. P83's catalog rows ARE the load-bearing transport-layer claim for the bus remote.
**Reality:** All 8 P83 catalog rows have `cadences: ["pre-pr"]` (`grep -A2 "cadences" quality/catalogs/agent-ux.json | grep -B1 pre-pr` confirms). NONE have `pre-release` cadence (which per `quality/PROTOCOL.md` is the slot for real-backend gating). NO row in `quality/catalogs/agent-ux.json` is gated on `GITHUB_TOKEN`/`ATLASSIAN_API_KEY`/`JIRA_API_TOKEN` env-var presence (verified via `grep -rn "GITHUB_TOKEN\|ATLASSIAN_API_KEY" quality/catalogs/agent-ux.json` returns zero hits). The bus-write rows shipping with `pre-pr` cadence and zero real-backend gating means the load-bearing transport claim never gets exercised against a real backend in CI or release.
**Evidence:**
- `quality/catalogs/agent-ux.json:445-447, :486-488, :527-529, :602-604, :645-647, :687-689, :729-731` (all 8 rows).
- `agent_flow_real.rs` covers single-backend `attach`/`init` but never bus push (per F1 evidence).
**Why it matters:** This is the "framework structurally exempted real-backend flows" pattern (CLUSTER G in `SUMMARY.md`). P83 inherited that structural gap and didn't surface it as a phase-level concern. Combined with F1, this means P83's "GREEN" status is ratified entirely against synthetic substrate.

### F12 — `83-VALIDATION.md` lists exhaustive integration tests but doesn't include any real-backend acceptance bar [SEVERITY: LOW]
**Claim in plan:** Phase 83's Nyquist validation artifact (`.planning/phases/83-bus-write-fan-out/83-VALIDATION.md`) is the formal validation contract. It enumerates DVCS-BUS-WRITE-01..06 → test mappings.
**Reality:** Every entry in `83-VALIDATION.md` Phase Requirements → Test Map (lines 33-38) maps to a sim-only `bus_write_*` integration test. There is no row mapping any DVCS-BUS-WRITE-* requirement to a real-backend gate. The Nyquist contract is therefore inherently sim-bound. Per OP-1 + OP-6 ("real backends are first-class test targets"), this is acceptable for unit-coverage but does not satisfy "transport-layer claims" — yet the validation document is structured as if it does.
**Evidence:**
- `.planning/phases/83-bus-write-fan-out/83-VALIDATION.md:31-38`.
**Why it matters:** Process gap that compounds with F11. The Nyquist validation artifact is supposed to be the load-bearing acceptance contract; structuring it as sim-only locks in the framework's structural exemption.

### F13 — `apply_writes`'s mirror-head ref write fires even if the action loop had partial failures? [SEVERITY: SUSPECT]
**Claim in plan:** `WriteOutcome::SotPartialFail` is returned at `write_loop.rs:267` (after the loop sets `any_failure=true`); SotOk branch at line 270-309 only fires when `any_failure==false`. So mirror_head shouldn't write on partial fail.
**Reality:** The `if any_failure { ...; return Ok(WriteOutcome::SotPartialFail); }` at lines 263-268 is hit BEFORE the SotOk-only block (lines 272-303), so `write_mirror_head` is NOT called on partial fail. Code is correct.
However, *partial state at the SoT* IS observable (the documented Pitfall 3 / D-09 case): if id=1 PATCH succeeded server-side and id=2 PATCH failed, the SoT now has id=1's new version. The next push's PRECHECK B will see the drift, prompt fetch first. But during that interim window, the cache's `last_fetched_at` cursor (written at line 278 — only on SotOk branch — so NOT on partial fail) is stale. The partial server-side change is real but unrecorded in the local cache. The plan documents this as recoverable; no test exercises it.
**Evidence:**
- `write_loop.rs:263-268` (early return on `any_failure`).
- `write_loop.rs:272-303` (SotOk-only writes — cursor + mirror-head).
- `crates/reposix-remote/tests/bus_write_sot_fail.rs:285` (asserts `error refs/heads/main some-actions-failed`); no follow-up test that recovers via fetch + retry.
**Why it matters:** SUSPECT — would need an end-to-end "1 of 2 PATCHes succeeds, push fails, fetch + replan + push" recovery test to settle. The plan implies the architecture handles this; the test suite doesn't fence it.

## Summary spotlights for v0.13.1 framework-fix

The two highest-leverage findings are F2 (architectural collision: bus push rejects what the docs require in the mirror tree) and F3 (OP-3 audit_events table never written by the helper for any real-backend bus push). F1 (sim-only fault injection) is the meta-failure that explains why F2 + F3 shipped without detection. F11 (real-backend cadence gate absent from the catalog) is the framework hook that, if added, would have caught F1 + F2 + F3 at phase close. F4–F10 are individually small but cluster into a "tests assert structure, not behavior; verifiers grep source, not run scenarios" pattern that the v0.13.1 work could systematically address.
