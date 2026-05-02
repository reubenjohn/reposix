---
phase: 83
plan: 02
title: "DVCS-BUS-WRITE-06 — Bus remote fault injection + audit completeness"
wave: 2
depends_on: [83-01]
requirements: [DVCS-BUS-WRITE-06]
files_modified:
  - crates/reposix-remote/tests/bus_write_mirror_fail.rs
  - crates/reposix-remote/tests/bus_write_sot_fail.rs
  - crates/reposix-remote/tests/bus_write_post_precheck_409.rs
  - crates/reposix-remote/tests/bus_write_audit_completeness.rs
  - quality/catalogs/agent-ux.json
  - quality/gates/agent-ux/bus-write-fault-injection-mirror-fail.sh
  - quality/gates/agent-ux/bus-write-fault-injection-sot-mid-stream.sh
  - quality/gates/agent-ux/bus-write-fault-injection-post-precheck-409.sh
  - quality/gates/agent-ux/bus-write-audit-completeness.sh
  - CLAUDE.md
autonomous: true
mode: standard
must_haves:
  truths:
    - "Mirror-push fault scenario produces correct end-state: head ref advances, synced-at frozen, helper_push_partial_fail_mirror_lag audit row written, helper exits zero with `ok refs/heads/main`, stderr WARN naming SoT-success-mirror-fail"
    - "SoT-write mid-stream fault scenario produces correct end-state: NO mirror push attempted, mirror baseline preserved, NO helper_push_accepted audit row, NO helper_push_partial_fail_mirror_lag audit row, helper exits non-zero with `error refs/heads/main some-actions-failed`"
    - "Post-precheck SoT 409 fault scenario produces correct end-state: NO mirror push attempted, helper exits non-zero, error names the failing record id, NO helper_push_accepted audit row"
    - "Audit-completeness happy-path scenario writes all expected rows to BOTH audit tables (audit_events_cache: helper_push_started + helper_push_accepted + mirror_sync_written; audit_events: per-record mutation rows)"
    - "Row 2 (`agent-ux/bus-write-mirror-fail-returns-ok`) flips FAIL → PASS during P83-02 T04 catalog flip BEFORE the per-phase push"
  artifacts:
    - path: "crates/reposix-remote/tests/bus_write_mirror_fail.rs"
      provides: "Fault-injection (a): mirror push fails between confluence-write and ack"
      contains: "bus_write_mirror_fail_returns_ok_with_lag_audit_row"
    - path: "crates/reposix-remote/tests/bus_write_sot_fail.rs"
      provides: "Fault-injection (b): confluence write fails mid-stream (5xx on second PATCH)"
      contains: "bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit"
    - path: "crates/reposix-remote/tests/bus_write_post_precheck_409.rs"
      provides: "Fault-injection (c): confluence 409 after PRECHECK B passed"
      contains: "bus_write_post_precheck_conflict_409_no_mirror_push"
    - path: "crates/reposix-remote/tests/bus_write_audit_completeness.rs"
      provides: "Dual-table audit-completeness assertion on the happy path"
      contains: "bus_write_audit_completeness_happy_path_writes_both_tables"
    - path: "quality/catalogs/agent-ux.json"
      provides: "4 new bus-write fault-injection / audit-completeness catalog rows for P83-02"
      contains: "agent-ux/bus-write-fault-injection-mirror-fail"
  key_links:
    - from: "crates/reposix-remote/tests/bus_write_mirror_fail.rs"
      to: "crates/reposix-remote/tests/common.rs::make_failing_mirror_fixture"
      via: "test imports cfg(unix) helper from P83-01 T05"
      pattern: "make_failing_mirror_fixture"
    - from: "crates/reposix-remote/tests/bus_write_audit_completeness.rs"
      to: "crates/reposix-remote/tests/common.rs::count_audit_cache_rows"
      via: "test queries audit_events_cache by op via P83-01 T05 helper"
      pattern: "count_audit_cache_rows"
    - from: "crates/reposix-remote/tests/bus_write_post_precheck_409.rs"
      to: "wiremock 409 PATCH route"
      via: "Mock::given(method(PATCH)).respond_with(ResponseTemplate::new(409))"
      pattern: "ResponseTemplate::new\\(409\\)"
---

# Phase 83 Plan 02 — Bus remote: fault injection + audit completeness (DVCS-BUS-WRITE-06)

<objective>
Close DVCS-BUS-WRITE-06 — the requirement that *"fault-injection
tests cover every documented failure case — kill GH push between
confluence-write and ack; kill confluence-write mid-stream;
simulate confluence 409 after precheck passed. Each produces
correct audit + recoverable state."* P83-01 shipped the write
fan-out core + the happy-path / no-mirror-remote tests; P83-02
ships the three fault-injection scenarios + the dual-table
audit-completeness verification.

Per the phase-close ritual (CLAUDE.md OP-7 + REQUIREMENTS.md
§ "Recurring success criteria"): P83-02's terminal push (T04)
closes the entire phase. The phase verifier subagent dispatch
runs AFTER P83-02 T04's push lands — grading all 8 catalog rows
(4 from P83-01 T01 + 4 from P83-02 T01).

This is a **single plan, four sequential tasks** per RESEARCH.md
§ "Plan Splitting":

- **T01** — Catalog-first: 4 rows in `quality/catalogs/agent-ux.json` +
  4 TINY verifier shells (status FAIL).
- **T02** — Mirror-fail integration test (`bus_write_mirror_fail.rs`)
  using `make_failing_mirror_fixture` from P83-01 T05's
  `tests/common.rs` (gated `#[cfg(unix)]` per D-04). Asserts:
  helper exits zero with `ok refs/heads/main`, audit op
  `helper_push_partial_fail_mirror_lag` written, head advances,
  synced-at frozen, mirror baseline preserved (the failing-update-hook
  rejects the push), stderr WARN naming SoT-success-mirror-fail.
- **T03** — SoT-fail tests:
  - `bus_write_sot_fail.rs`: wiremock returns 200 on PATCH id=1,
    500 on PATCH id=2 (mid-stream fail). Asserts: helper exits
    non-zero with `error refs/heads/main some-actions-failed`,
    NO mirror push attempted, NO `helper_push_accepted` row, NO
    `helper_push_partial_fail_mirror_lag` row, mirror baseline
    preserved.
  - `bus_write_post_precheck_409.rs`: wiremock returns `[]` on
    `list_changed_since` (PRECHECK B Stable) but 409 on PATCH.
    Asserts: helper exits non-zero, NO mirror push, error names
    the failing record id (D-09 / Pitfall 3 documented behavior).
- **T04** — Audit-completeness test + catalog flip + CLAUDE.md
  addendum + per-phase push (terminal):
  - `bus_write_audit_completeness.rs`: queries BOTH
    `audit_events_cache` (cache.db) AND `audit_events` (sim's
    audit table — exposed via the SimBackend's `audit_events`
    table or via a test-side helper). Asserts the row counts
    match RESEARCH.md § "Audit Completeness Contract".
  - Flip 4 P83-02 rows + the 1 lingering P83-01 row
    (`agent-ux/bus-write-mirror-fail-returns-ok`) FAIL → PASS.
  - CLAUDE.md addendum: name the four shipped fault-injection
    tests + the dual-table audit-completeness contract in the
    P83-01-introduced "Bus write fan-out" paragraph.
  - `git push origin main` (terminal — closes the phase).

Sequential (T01 → T02 → T03 → T04). Per CLAUDE.md "Build memory
budget" the executor holds the cargo lock sequentially across T02
(`-p reposix-remote`), T03 (`-p reposix-remote`), T04 (`-p
reposix-remote`). T01 is doc-or-shell-only; no cargo. NEVER
`cargo --workspace`.

**Architecture (read BEFORE diving into tasks):**

P83-02's tests exercise the helper end-to-end via
`assert_cmd::Command::cargo_bin("git-remote-reposix")`. Each test
constructs its own:

1. **Wiremock SoT** (per P81's `tests/perf_l1.rs` pattern) — mocks
   `list_records`, `list_changed_since`, and PATCH/POST/DELETE
   routes.
2. **File:// bare-repo mirror** with passing OR failing update hook
   (P83-01 T05's `tests/common.rs::make_failing_mirror_fixture`
   for the failing variant; the existing
   `bus_precheck_b.rs::make_synced_mirror_fixture` shape — copied
   to a public helper if needed — for the passing variant).
3. **Working tree** with the bare mirror configured as a local
   `git remote` so P82's STEP 0 finds it.

The four tests share the same scaffolding pattern (helper-driver
from P80's `mirror_refs.rs`); only the wiremock + mirror fixture
configuration differs. T02 + T03 + T04 each create one test file;
the executor implementing them should follow the donor pattern
exactly (no creative deviations — RESEARCH.md § "Don't Hand-Roll"
covers every piece of test infrastructure needed).

**Key invariants per RESEARCH.md § "Audit Completeness Contract":**

| End-state                            | `audit_events_cache` rows                                                                                                             | `audit_events` rows (backend)                                          |
|--------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|
| Bus push, SoT ok, mirror ok          | `helper_backend_instantiated` + `helper_push_started` + `helper_push_accepted` + `mirror_sync_written` (+ optional sanitize rows)    | One row per executed `create_record` / `update_record` / `delete_or_close` |
| Bus push, SoT ok, mirror FAIL        | `helper_backend_instantiated` + `helper_push_started` + `helper_push_accepted` + `helper_push_partial_fail_mirror_lag` (NO `mirror_sync_written`) | One row per executed REST mutation                                       |
| Bus push, SoT precheck conflict      | `helper_backend_instantiated` + `helper_push_started` + `helper_push_rejected_conflict`                                              | None                                                                   |
| Bus push, SoT 409 post-precheck      | `helper_backend_instantiated` + `helper_push_started` (no accepted/conflict)                                                          | One row for any record whose PATCH succeeded BEFORE the 409             |
| Bus push, mirror remote not configured | `helper_backend_instantiated` only                                                                                                   | None                                                                   |

T04's `bus_write_audit_completeness.rs` exercises the first row
(SoT-ok-mirror-ok) and asserts the exact row counts. The other
end-states are asserted by the row-2 test (T02), the row-3 test
(T03 mid-stream), and the row-4 test (T03 post-precheck).

**No new error variants.** The remote crate uses `anyhow` throughout.
All test failures map to `assert!` panics that surface via
`cargo test`'s output.

This plan **must run cargo serially** per CLAUDE.md "Build memory
budget". Per-crate fallback (`cargo nextest run -p reposix-remote
--test <name>`) used for each individual test invocation; final
sweep is `cargo nextest run -p reposix-remote` (per-crate, NOT
workspace).

This plan terminates with `git push origin main` (per CLAUDE.md
push cadence) with pre-push GREEN. The catalog rows' initial FAIL
status is acceptable through T01–T03 because the rows are `pre-pr`
cadence (NOT `pre-push`); the runner re-grades to PASS during T04
BEFORE the push commits. T04 is the phase's terminal task — the
verifier subagent dispatch follows.
</objective>

## Chapters

- **[Canonical refs + threat model](./canonical-refs-and-threat-model.md)** — Spec sources, fixtures, gates, audit shape, principles, STRIDE register.
- **[T01 — Catalog-first: 4 rows + 4 verifier shells](./t01-catalog-first.md)** — 4 TINY verifiers + hand-edit `agent-ux.json`; catalog-first commit, status FAIL.
- **[T02 — Mirror-fail test (`bus_write_mirror_fail.rs`)](./t02-mirror-fail.md)** — Fault-injection (a): failing mirror fixture; zero exit, lag audit row, head advances, synced-at frozen.
- **[T03 — SoT-fail tests (`bus_write_sot_fail.rs` + `bus_write_post_precheck_409.rs`)](./t03-sot-fail.md)** — Fault-injection (b)+(c): mid-stream 5xx + post-precheck 409; no mirror push, non-zero exit.
- **[T04 — Audit-completeness + catalog flip + CLAUDE.md addendum + per-phase push](./t04-audit-completeness-and-close.md)** — `bus_write_audit_completeness.rs`, flip 5 rows FAIL→PASS, CLAUDE.md addendum, terminal push. Close protocol.
