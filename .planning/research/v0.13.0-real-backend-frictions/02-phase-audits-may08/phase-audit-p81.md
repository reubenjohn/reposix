# Phase P81 Audit — L1 perf migration

**Auditor:** unbiased subagent (zero session context)
**Date:** 2026-05-08
**Phase:** P81 — `list_changed_since`-based conflict detection + `reposix sync --reconcile`
**Verdict at the time of close:** GREEN (with three advisory items) — `quality/reports/verdicts/p81/VERDICT.md`

## Verdict at a glance

- ALIGNED items: 3
- MISALIGNED items: 6
- SUSPECT items: 1

## Summary headline

P81 ships a real `precheck.rs` that's wired into both `handle_export` and the bus handler — DVCS-PERF-L1-01/03 are substantively delivered, and all four `BackendConnector` impls (sim, confluence, github, jira) override `list_changed_since` with native wire filters. The break is on the **escape hatch**: `reposix sync --reconcile` is documented as the on-demand recovery move for cache desync (DVCS-PERF-L1-02 — explicitly named in `docs/guides/troubleshooting.md` and `docs/concepts/dvcs-topology.md` as the fix the user types after a bus-remote `fetch first` rejection), but the implementation **rejects every non-sim backend with a hard error**. The user-facing recovery flow is unreachable on the only backends where the bus push (and therefore L1 desync recovery) actually matters — confluence, github, jira. The perf claim ("net REST cost: one call + actual writes") is also weakly verified: the only test that grades the catalog row drives a NO-OP push so `refresh_for_mirror_head` (which still calls `list_records`) is skipped; the write-path cost is not measured.

---

## ALIGNED items (load-bearing claim → real evidence)

### A1 — `BackendConnector::list_changed_since` is implemented natively for all four backends [ALIGNED]

**Claim in plan:** Every backend ships a wire-level `list_changed_since` so the precheck's hot path is one REST call.
**Reality:** All four backends override the in-memory default impl with a native incremental query.
**Evidence:**
- `crates/reposix-core/src/backend/sim.rs:281-303` — sends `?since=<rfc3339>` query param.
- `crates/reposix-confluence/src/lib.rs:141-164` — sends CQL `lastModified > "<yyyy-MM-dd HH:mm>"`.
- `crates/reposix-github/src/lib.rs:472-510` — sends `?since=<rfc3339>` (GitHub's native filter) with pagination.
- `crates/reposix-jira/src/lib.rs:108-150` — sends JQL `updated >= "<yyyy-MM-dd HH:mm>"`.
- The trait's default impl at `crates/reposix-core/src/backend.rs:253-264` filters in memory (would defeat L1) — every adapter overrides it.

### A2 — `precheck.rs` is the single conflict-detection path; bus + single-backend share it [ALIGNED]

**Claim in plan:** DVCS-PERF-L1-03 — both `handle_export` and the future bus handler call the same precheck function; no path-specific copies.
**Reality:** `precheck_export_against_changed_set` lives at `crates/reposix-remote/src/precheck.rs:90-312` with narrow-deps signature `(cache, backend, project, rt, parsed)`. Both push paths reach it via `crate::write_loop::apply_writes`.
**Evidence:**
- `crates/reposix-remote/src/write_loop.rs:160` — single call site for `precheck_export_against_changed_set`.
- `crates/reposix-remote/src/main.rs:381` (single-backend) and `crates/reposix-remote/src/bus_handler.rs:247` (bus) — both go through `apply_writes`.
- Module doc at `precheck.rs:13-14` documents the dual consumption: "Both `handle_export` (P81) and the future bus handler (P82+) call this same function."

### A3 — Cache cursor wrappers + `read_blob_cached` ship [ALIGNED]

**Claim in plan (T02):** `Cache::read_last_fetched_at`, `Cache::write_last_fetched_at`, and `Cache::read_blob_cached` (sync, gix-only).
**Reality:**
- `crates/reposix-cache/src/cache.rs:476-484` — `read_last_fetched_at`.
- `crates/reposix-cache/src/cache.rs:513-527` — `write_last_fetched_at`.
- `crates/reposix-cache/src/cache.rs:509-527` (region) — `read_blob_cached`, sync, returns `Ok(None)` on miss.
- Cursor write fires after `log_helper_push_accepted` at `crates/reposix-remote/src/write_loop.rs:278`.
**Evidence:** PLAN-CHECK H1-H4 fixes verified at the verifier-cited lines; `precheck.rs:21-24` carries the explicit anti-pattern doc-comment naming `read_blob` as the wrong path.

---

## MISALIGNED items

### F1 — `reposix sync --reconcile` rejects every real backend [SEVERITY: HIGH]

**Claim in plan:** D-01 (RATIFIED) — "user recovery path is `reposix sync --reconcile` (T03) which does a full `list_records` walk and rebuilds the cache." DVCS-PERF-L1-02 (in REQUIREMENTS.md:76) calls this an "on-demand full `list_records` walk + cache reconciliation for users who suspect cache desync." Public docs (`docs/guides/troubleshooting.md:241, 249, 305` and `docs/concepts/dvcs-topology.md:79`) tell users to type the literal command `reposix sync --reconcile` after every bus-remote `fetch first` rejection or cache-desync symptom.

**Reality:** `crates/reposix-cli/src/sync.rs:79-92` switches on the backend slug and bails for everything except `sim`:
```rust
let backend: Arc<dyn BackendConnector> = match backend_slug.as_str() {
    "sim" => { /* construct SimBackend */ }
    other => bail!(
        "sync --reconcile: backend `{other}` not yet wired in v0.13.0 (sim only); \
         github/confluence/jira land alongside the bus-remote work in P82+"
    ),
};
```
P82 never expanded this dispatch — `git log crates/reposix-cli/src/sync.rs` shows only the original P81 commit `9321499`. P83-P88 never touched it.

**Evidence:**
- `crates/reposix-cli/src/sync.rs:79-92` — the bail.
- `git log` on the file: only `9321499 feat(cli): reposix sync --reconcile subcommand (DVCS-PERF-L1-02)`.
- `docs/guides/troubleshooting.md:241` — recovery instruction the user follows after a bus rejection: `hint: run \`reposix sync --reconcile\` to refresh your cache against the SoT, then \`git pull --rebase\``.
- `docs/guides/troubleshooting.md:296-310` — the "Cache-desync recovery via `reposix sync --reconcile`" subsection prescribes the command without backend-specific caveat.
- `docs/concepts/dvcs-topology.md:79` — same recovery hint embedded in topology doc.

**Why it matters:** The bus-remote work (P82–P83) targets confluence as the canonical SoT. Every documented recovery path that depends on rebuilding the cache against the SoT (the entire "Cache-desync recovery" troubleshooting section, the `fetch first` rejection recovery flow) hits the bail and instructs the user to file an upstream issue. This breaks the documented user contract on real backends — the only backends where the bus push exists. The same "scaffold sim-only" pattern bit P79-02/03 (cluster A in T2-attach.md) and migrated forward into P81 unmitigated.

### F2 — Catalog row `perf/handle-export-list-call-count` doesn't measure the load-bearing perf claim [SEVERITY: HIGH]

**Claim in plan:** Architecture-sketch (`innovations.md:182`) and CLAUDE.md update (commit `d21160c`) state the L1 success path is "ONE `list_changed_since` REST call plus ONE `get_record` per record in `changed_set ∩ push_set`" — i.e., the cost is bounded even when actual writes happen. The catalog row description claims: "with N=200 records seeded in wiremock and a one-record edit pushed, the helper makes >=1 list_changed_since REST call AND zero list_records REST calls."

**Reality:** The test pushes a NO-OP tree. `crates/reposix-remote/tests/perf_l1.rs:144-172` (`no_op_tree_export`) builds an export stream "IDENTICAL to the cache prior so plan() emits zero actions (no creates / no updates / no deletes)." When `files_touched == 0`, `crates/reposix-remote/src/write_loop.rs:285` skips `refresh_for_mirror_head` — the call site that DOES still fire `list_records` (via `Cache::build_from` at `crates/reposix-cache/src/builder.rs:60`). The test's `expect(0)` on `NoSinceQueryParam` therefore proves only the no-op-push case, not the steady-state-write case.

**Evidence:**
- `crates/reposix-remote/tests/perf_l1.rs:138-143` (comment): "Used by the perf test to push a tree IDENTICAL to the cache prior so plan() emits zero actions."
- `crates/reposix-remote/src/write_loop.rs:282-300` — `refresh_for_mirror_head` only fires when `files_touched > 0`.
- `crates/reposix-cache/src/mirror_refs.rs:297-299` — `refresh_for_mirror_head` delegates to `build_from`.
- `crates/reposix-cache/src/builder.rs:60` — `build_from` calls `self.backend.list_records(...)`.
- The catalog row's description (`quality/catalogs/perf-targets.json:139`) explicitly says "and a one-record edit pushed" — the test's no-op tree contradicts the description.
- `SURPRISES-INTAKE.md` 2026-05-01 entry 2 acknowledges this: "the full L1 promise (`refresh_for_mirror_head` itself uses `list_changed_since` for the post-write tree synthesis) defers to v0.14.0."

**Why it matters:** The headline perf claim — "DVCS push doesn't do 100+ REST calls; net cost is one + writes" — is the load-bearing motivator for the entire P81 phase (architecture-sketch.md:189: "Plain git's `git push` does ~3 REST round-trips. Bus-remote `git push` doing 100+ REST calls on every push violates that promise"). The verifier passes today by testing only the case where no actual writes happen. A clean push that creates one issue still incurs a `list_records` call via the surviving `refresh_for_mirror_head` site. The test misnamed the assertion ("a one-record edit pushed") to match the catalog row's wording, but the actual test body uses `no_op_tree_export`. This is a test-name-promises-more-than-the-assertion failure shape (failure shape #1 in AUDIT-BRIEF.md).

### F3 — `docs-alignment/perf-subtlety-prose-bound` rides the same vacuous test as F2 [SEVERITY: MED]

**Claim in plan:** `quality/catalogs/doc-alignment.json:9360-9382` binds the architecture-sketch's L1 prose ("Trades one safety property: today list_records would catch a record that exists on backend but missing from cache") to `perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records`.

**Reality:** The bound test is the same no-op test as F2. The prose claim is about a SAFETY trade (cache trusted as prior), not the perf trade — and the bound test doesn't exercise the safety claim either. It only checks "no `list_records` call on the no-op hot path." A drift in either direction (the prose getting weakened OR the L1 trade getting reverted) would not be caught by the bound test as long as `expect(0)`/`expect(1..)` still hold against a no-op push.

**Evidence:**
- `quality/catalogs/doc-alignment.json:9371-9377` — `tests: ["crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records"]`.
- The verdict's own honesty spot-check (`quality/reports/verdicts/p81/VERDICT.md:60-114`) graded the test "non-vacuous" purely by checking that the wiremock matchers attach to mocks with active expectations — but did not interrogate whether the matchers actually validate the prose's claim about cache desync surfacing as a 404. They don't: the prose's L1-strict trade is about a CACHE-DESYNC scenario (record in backend, missing from cache); the test seeds the cache fully populated.

**Why it matters:** Failure shape #1 (test name promises more than assertion delivers) recurs at the docs-alignment level. The walker stays GREEN even if the prose is rewritten or the L1 contract drifts. This is the same pattern AUDIT-BRIEF.md:50 calls out re. `dark_factory_real_confluence`.

### F4 — No verifier exercises the L1 path (or `sync --reconcile`) against any real backend [SEVERITY: MED]

**Claim in plan:** Architecture-sketch.md:189 names the bus-remote-on-confluence as the load-bearing workload ("DVCS thesis is 'DVCS at the same UX as plain git.' ... Bus-remote `git push` doing 100+ REST calls on every push violates that promise loudly enough that a cold reader will dismiss reposix as a toy. Fix the inefficiency as part of the DVCS milestone.").

**Reality:** Every P81 catalog row's verifier runs against wiremock. `crates/reposix-cli/tests/agent_flow_real.rs` (the only `--ignored` real-backend file) has zero references to `list_changed_since`, `sync --reconcile`, or any L1 code path.

**Evidence:**
- `grep "list_changed_since\|sync.*reconcile" crates/reposix-cli/tests/agent_flow_real.rs` — 0 matches.
- `quality/gates/perf/list-call-count.sh` and `quality/gates/agent-ux/sync-reconcile-subcommand.sh` — both shell to wiremock-backed cargo tests.
- No `cadence: pre-release` row exists for L1 in `quality/catalogs/perf-targets.json` or `agent-ux.json`.

**Why it matters:** The DVCS-PERF-L1-* requirements are explicitly transport-layer claims about what happens on real Confluence/JIRA/GitHub pushes. CLAUDE.md OP-6 rules: "simulator-only coverage does NOT satisfy acceptance for transport-layer or performance claims." The phase shipped with simulator-only coverage and was graded GREEN. This is failure shape #4 (project's own non-negotiable invariants violated silently): OP-6 is unambiguous, and the verifier didn't apply it.

### F5 — User-facing perf claim "one call + actual writes" overstates the cache's behavior on real pushes [SEVERITY: MED]

**Claim in plan:** CLAUDE.md update (commit `d21160c`, "L1 conflict detection (P81+)" paragraph) states: "On the cursor-present hot path, the precheck does ONE `list_changed_since` REST call plus ONE `get_record` per record in `changed_set ∩ push_set` (typically zero or one); the legacy unconditional `list_records` walk in `handle_export` is gone."

**Reality:** The "legacy unconditional `list_records` walk" is gone from the precheck path, but a different unconditional `list_records` walk (via `refresh_for_mirror_head` → `Cache::build_from`) is still present on every push that touches files. The CLAUDE.md text reads as if all `list_records` calls are gone on the steady-state hot path. They aren't — they're reshuffled to a different call site that the test happens to skip via the no-op carve-out (F2).

**Evidence:**
- `crates/reposix-remote/src/write_loop.rs:285-300` — calls `refresh_for_mirror_head` when `files_touched > 0`.
- `crates/reposix-cache/src/mirror_refs.rs:297-299` — `refresh_for_mirror_head` is `self.build_from()`.
- `crates/reposix-cache/src/builder.rs:60` — `build_from` calls `self.backend.list_records(&self.project)`.
- SURPRISES-INTAKE entry 2 (2026-05-01) explicitly acknowledges this: "Replacing `refresh_for_mirror_head` with a list_changed_since-driven equivalent would require either (a) wider Cache crate refactoring … or (b) cleverness about when the post-write tree refresh is needed. … the full L1 promise … defers to v0.14.0."

**Why it matters:** Failure shape #5 (documented user-facing flow rejected/contradicted by the implementation). The CLAUDE.md text bills L1 as cleaner than it actually is, and the no-op-skip workaround means the perf regression is silent on the steady-state-write case. A reader who trusts the docs will be surprised when their own measurement of a 1-issue PATCH push on a 5,000-issue confluence space shows a `list_records` call.

### F6 — Verifier subagent's "non-vacuity" spot-check stops at wiremock plumbing, not semantic alignment [SEVERITY: MED]

**Claim in plan:** Phase-close protocol (`81-PLAN-OVERVIEW/phase-close.md:105-126`) names the verifier criteria; the verifier's "Honesty spot-check" (VERDICT.md:60-114) reads the test body to confirm non-vacuity.

**Reality:** The verdict's spot-check confirms (a) wiremock matchers attach to active expectations, (b) priority-tier carve-out is in place, (c) positive control panics on flip, (d) N=200 records are seeded. Every check is about wiremock plumbing. The verifier never asks "does this test actually match the catalog row's description?" — the description says "a one-record edit pushed" but the test pushes a no-op tree.

**Evidence:**
- `quality/reports/verdicts/p81/VERDICT.md:60-114` — five "non-vacuity proofs," each about wiremock semantics.
- `quality/catalogs/perf-targets.json:139` — catalog row description: "with N=200 records seeded in wiremock and a one-record edit pushed."
- `crates/reposix-remote/tests/perf_l1.rs:138-143` — comment: "Used by the perf test to push a tree IDENTICAL to the cache prior so plan() emits zero actions."

**Why it matters:** Failure shape #1 (test name promises one thing, assertions deliver less) propagates to the verifier shape — when the verifier's honesty layer focuses on plumbing rather than semantic match, it can sign off a vacuous test. This is the framework integrity issue that motivates the v0.13.1 audit; documenting it concretely here gives the v0.13.1 framework-fix phase a target.

---

## SUSPECT items

### S1 — `Cache::sync` cursor advance for non-sim backends is unverified [SEVERITY: SUSPECT]

**Claim:** `Cache::sync` (which `sync --reconcile` calls when wired) supports any backend implementing `BackendConnector` and properly advances `last_fetched_at` for delta + seed paths.

**Reality:** The cache-side machinery (`crates/reposix-cache/src/builder.rs:225-440`) is backend-agnostic and would presumably work for confluence/github/jira. But because `sync.rs` bails on those backends (F1), no test exercises the cursor-advance + delta-rebuild for a non-sim adapter. If a future contributor wires real backends in `sync.rs`, they'd be relying on un-tested coupling between Cache::sync and the real-backend `list_changed_since` impls (which DO exist per A1).

**What would settle it:** Add a wiremock-driven test against each real-backend's HTTP shape (mirroring `confluence_list_changed_since_sends_cql_lastmodified` at `client.rs:937` but driven through `Cache::sync`), or land an `--ignored` real-backend test in `agent_flow_real.rs`.

**Why it matters:** Promotes F1's user-facing break from "blocked on CLI dispatch" to "blocked on CLI dispatch AND coupling untested" — the v0.13.1 fix won't be a one-line dispatch change; it needs verification.

---

## Cross-cutting observations

- **Velocity smell mitigated:** P81 shipped 5 commits across two days (`3a44a9e` 2026-04-30 → `4869545` 2026-05-01) — fast, but the SUMMARY records 4 sequential tasks with explicit catalog-first commit and post-hoc refresh, not skip-driven compression. The PLAN-CHECK process surfaced 4 HIGH issues before T02 landed; all four show up substantively fixed in the code (verdict Section "Plan-checker H1-H4 honesty spot-check"). Velocity is not the issue here.
- **Catalog-first contract honored at the row level:** T01 commit `3a44a9e` mints rows with `status: FAIL`; T04 commit `d21160c` flips them to `PASS`. The spec is respected. The integrity issue is not procedural — it's that the rows themselves don't fully exercise the load-bearing claim (F2/F3/F6).
- **CLAUDE.md was updated in-phase per QG-07** — both the § Architecture paragraph and the § Commands bullet landed in `d21160c`. The accuracy issue (F5) is content, not omission.

## What the v0.13.1 framework-fix phase should pick up

1. **F1 is the user-visible HIGH and the easiest to fix:** wire confluence/github/jira backends in `crates/reposix-cli/src/sync.rs` (mirror the dispatch from `attach.rs:147-166`, plus credential plumbing). Adds a real-backend `--ignored` test in `agent_flow_real.rs` for at least one backend (TokenWorld confluence is sanctioned per `docs/reference/testing-targets.md`).
2. **F2/F3/F5/F6 are the framework integrity HIGH-MED:** the perf catalog row should test a non-no-op push, OR the prose / CLAUDE.md should be honest that the L1-clean-push promise carries an asterisk (the post-write `refresh_for_mirror_head` `list_records` call) until v0.14.0. SURPRISES entry 2 already names the trade-off; the doc surfaces don't.
3. **F4 is a structural OP-6 violation:** every transport-layer / perf claim needs a real-backend verifier on at least the canonical sanctioned target (TokenWorld confluence). The current `agent_flow_real.rs` covers `attach` and `dark_factory`; it should also cover one bus-push round-trip end-to-end so the L1 path is tested against real Confluence at least once.
