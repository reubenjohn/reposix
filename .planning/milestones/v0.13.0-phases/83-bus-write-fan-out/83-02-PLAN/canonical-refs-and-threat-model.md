← [back to index](./index.md)

# Canonical refs + threat model

<canonical_refs>
**Spec sources:**
- `.planning/REQUIREMENTS.md` DVCS-BUS-WRITE-06 (line 79) —
  verbatim acceptance.
- `.planning/ROADMAP.md` § Phase 83 (lines 145-165) — phase goal +
  8 success criteria (#5 is the fault-injection coverage; #6 is
  audit completeness).
- `.planning/phases/83-bus-write-fan-out/83-RESEARCH.md` § "Test
  (a)/(b)/(c)/(d)" + § "Audit Completeness Contract" + §
  "Fault-Injection Test Infrastructure" — verbatim test contracts.
- `.planning/phases/83-bus-write-fan-out/83-PLAN-OVERVIEW.md` §
  "Decisions ratified at plan time" (D-01..D-10 — particularly
  D-02, D-04, D-09 for P83-02).
- `.planning/phases/83-bus-write-fan-out/83-01-PLAN.md` (sibling;
  test-helper donors in P83-01 T05's `tests/common.rs`).

**Test fixtures (T02–T04):**
- `crates/reposix-remote/tests/common.rs` (post-P83-01 T05) —
  `make_failing_mirror_fixture` (cfg(unix); D-04) +
  `count_audit_cache_rows` helpers.
- `crates/reposix-remote/tests/bus_precheck_b.rs` lines 60-100
  (`make_synced_mirror_fixture` — donor pattern for passing
  file:// mirror; copy to a sibling helper in `common.rs` if
  needed for cross-test reuse).
- `crates/reposix-remote/tests/perf_l1.rs` — wiremock fixture
  donor + `Mock::given(method("PATCH")).respond_with(ResponseTemplate::new(N))`
  idiom donor.
- `crates/reposix-remote/tests/push_conflict.rs::stale_base_push_emits_fetch_first_and_writes_no_rest`
  — 409 injection donor pattern (`ResponseTemplate::new(409)`
  with version-mismatch body).
- `crates/reposix-remote/tests/mirror_refs.rs` — helper-driver
  donor pattern (`drive_helper_export`, `render_with_overrides`).
- `crates/reposix-remote/tests/bus_write_happy.rs` (post-P83-01
  T05) — happy-path fixture pattern; T02 / T03 / T04 reuse the
  scaffolding shape.

**Quality Gates:**
- `quality/catalogs/agent-ux.json` — 12 P82 rows + 4 P83-01 rows
  pre-existing; the 4 new P83-02 rows join.
- `quality/gates/agent-ux/bus-write-sot-first-success.sh` (P83-01
  T01) — TINY verifier precedent.
- `quality/gates/agent-ux/bus-write-no-helper-retry.sh` (P83-01
  T01) — grep-based verifier alternative pattern (NOT used by
  P83-02 — all 4 P83-02 verifiers delegate to `cargo test`).
- `quality/PROTOCOL.md` § "Verifier subagent prompt template" + §
  "Principle A".

**Audit shape:**
- `crates/reposix-cache/src/audit.rs` — `audit_events_cache` ops
  enumerated by the schema CHECK list (P83-01 T03 added
  `helper_push_partial_fail_mirror_lag`).
- `crates/reposix-cache/src/cache.rs::Cache::log_helper_push_partial_fail_mirror_lag`
  — wrapper used by `bus_handler::handle_bus_export` on the
  partial-fail branch.
- `crates/reposix-core/src/audit.rs` — `audit_events` (backend
  audit table) — written by the sim/confluence/jira adapters
  inside their REST-mutation success paths.
- `crates/reposix-sim/` — sim adapter exposes its audit_events
  table for test inspection (confirm path during T04 read_first).

**Operating principles:**
- `CLAUDE.md` § "Build memory budget" — strict serial cargo,
  per-crate fallback.
- `CLAUDE.md` § "Push cadence — per-phase" — terminal push
  protocol (P83-02 T04's push closes the phase).
- `CLAUDE.md` § Operating Principles OP-1 (simulator-first), OP-3
  (audit log non-optional dual-table — the contract this plan
  enforces), OP-7 (verifier subagent), OP-8 (+2 reservation).
</canonical_refs>

<threat_model>
## Trust Boundaries

P83-02 introduces NO new trust boundaries. It exercises the
boundaries P83-01 introduced via fault-injection tests:

| Boundary (existing, exercised here) | What P83-02 changes                                                                                                                                                                                                                                                                                              |
|-------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `git push` shell-out (P83-01)        | Fault-injection (a) exercises the failure path: failing-update-hook bare repo → `git push` exits non-zero → `bus_handler::push_mirror` returns `MirrorResult::Failed`. T-83-01 / T-83-02 mitigations (D-08 plain push, 3-line stderr trim) preserved.                                                              |
| Cache audit row INSERT               | Fault-injection (a) exercises the new audit op `helper_push_partial_fail_mirror_lag`. The append-only triggers + DEFENSIVE flag (per `cache.rs::db.rs::open_cache_db`) preserved — no new INSERT site. Test asserts row presence via `count_audit_cache_rows`.                                                  |
| SoT REST writes (existing)           | Fault-injection (b) + (c) exercise the failure paths. NO new HTTP construction site. wiremock injects 5xx (b) or 409 (c) per the existing `BackendConnector` allowlist; the helper's existing failure handling in `apply_writes` (`SotPartialFail` outcome) is what's exercised.                                  |

## STRIDE Threat Register

P83-02 inherits P83-01's STRIDE register (T-83-01 through T-83-05)
without modification. The fault-injection tests EXERCISE the
mitigations rather than introducing new ones:

| Threat ID (inherited) | Disposition | What P83-02 verifies                                                                                                          |
|-----------------------|-------------|-------------------------------------------------------------------------------------------------------------------------------|
| T-83-01 (mirror_remote_name `-`-prefix reject) | mitigate (P83-01) | NOT exercised by P83-02 — `mirror_remote_name` is helper-resolved from valid `git config`; defensive reject is a code-only invariant verified by P83-01 T05 / T04 grep.                                                                |
| T-83-02 (stderr_tail trim to 3 lines) | mitigate (P83-01) | EXERCISED by T02 — assert the audit row's `reason` field's `tail=` portion is bounded; the failing-update-hook's stderr is trimmed.                                                                |
| T-83-03 (partial-fail repudiation) | mitigate (P83-01) | EXERCISED by T02 — assert the audit row exists with the SoT SHA; assert `head ≠ synced-at` post-fail (operator-detectable lag).                                                                |
| T-83-04 (SSH-agent prompt DoS) | accept (P83-01) | NOT exercised — tests use file:// fixtures exclusively per D-04; no SSH paths.                                                                |
| T-83-05 (Confluence non-atomicity) | accept (D-09) | EXERCISED by T03's mid-stream test — assert id=1 partial state observable on SoT (PATCH 200) AND mirror baseline preserved.                                                                |

No new threats introduced.
</threat_model>
