---
phase: 83
plan: 01
title: "DVCS-BUS-WRITE-01..05 — Bus remote write fan-out core (apply_writes refactor + cache audit op + bus_handler write fan-out + happy-path tests)"
wave: 1
depends_on: [80, 82]
requirements: [DVCS-BUS-WRITE-01, DVCS-BUS-WRITE-02, DVCS-BUS-WRITE-03, DVCS-BUS-WRITE-04, DVCS-BUS-WRITE-05]
files_modified:
  - crates/reposix-remote/src/write_loop.rs
  - crates/reposix-remote/src/main.rs
  - crates/reposix-remote/src/bus_handler.rs
  - crates/reposix-cache/src/audit.rs
  - crates/reposix-cache/src/cache.rs
  - crates/reposix-cache/fixtures/cache_schema.sql
  - crates/reposix-remote/tests/common.rs
  - crates/reposix-remote/tests/bus_write_happy.rs
  - crates/reposix-remote/tests/bus_write_no_mirror_remote.rs
  - quality/catalogs/agent-ux.json
  - quality/gates/agent-ux/bus-write-sot-first-success.sh
  - quality/gates/agent-ux/bus-write-mirror-fail-returns-ok.sh
  - quality/gates/agent-ux/bus-write-no-helper-retry.sh
  - quality/gates/agent-ux/bus-write-no-mirror-remote-still-fails.sh
  - CLAUDE.md
autonomous: true
mode: standard
must_haves:
  truths:
    - "Bus URL push triggers apply_writes (lifted from handle_export); SoT REST writes succeed before any mirror push"
    - "On SoT-success + mirror-success: refs/mirrors/<sot>-head AND refs/mirrors/<sot>-synced-at advance; ok refs/heads/main returned"
    - "On SoT-success + mirror-fail: refs/mirrors/<sot>-head advances; synced-at FROZEN; helper_push_partial_fail_mirror_lag audit row written; ok refs/heads/main returned"
    - "On any SoT-fail outcome: mirror push NEVER attempted; mirror refs UNCHANGED"
    - "Helper does NOT retry on transient mirror failure (single push_mirror invocation; exit non-zero → audit + warn + return ok)"
    - "Bus URL with no local git remote configured for the mirror still fails with verbatim Q3.5 hint (P82 behavior preserved)"
    - "Existing single-backend handle_export integration tests (mirror_refs / push_conflict / bulk_delete_cap / perf_l1 / stateless_connect) all GREEN post-refactor"
  artifacts:
    - path: "crates/reposix-remote/src/write_loop.rs"
      provides: "Shared apply_writes function lifted from handle_export with narrow-deps signature"
      min_lines: 200
    - path: "crates/reposix-remote/src/bus_handler.rs"
      provides: "handle_bus_export with full write fan-out replacing emit_deferred_shipped_error stub; push_mirror helper"
      min_lines: 350
    - path: "crates/reposix-cache/src/audit.rs"
      provides: "log_helper_push_partial_fail_mirror_lag helper + unit test"
      contains: "log_helper_push_partial_fail_mirror_lag"
    - path: "crates/reposix-cache/fixtures/cache_schema.sql"
      provides: "extended op CHECK list with helper_push_partial_fail_mirror_lag"
      contains: "helper_push_partial_fail_mirror_lag"
    - path: "crates/reposix-remote/tests/bus_write_happy.rs"
      provides: "happy-path integration test asserting both refs + dual audit table on SoT+mirror success"
    - path: "crates/reposix-remote/tests/bus_write_no_mirror_remote.rs"
      provides: "regression test for SC4 / Q3.5 — no-mirror-remote hint preserved end-to-end"
    - path: "quality/catalogs/agent-ux.json"
      provides: "4 new bus-write catalog rows for P83-01"
      contains: "agent-ux/bus-write-sot-first-success"
  key_links:
    - from: "crates/reposix-remote/src/bus_handler.rs::handle_bus_export"
      to: "crates/reposix-remote/src/write_loop.rs::apply_writes"
      via: "post-PRECHECK-B branch reads stdin then calls apply_writes"
      pattern: "write_loop::apply_writes"
    - from: "crates/reposix-remote/src/bus_handler.rs::handle_bus_export"
      to: "crates/reposix-remote/src/bus_handler.rs::push_mirror"
      via: "post-apply_writes-SotOk branch invokes push_mirror"
      pattern: "push_mirror\\(&mirror_remote_name\\)"
    - from: "crates/reposix-remote/src/main.rs::handle_export"
      to: "crates/reposix-remote/src/write_loop.rs::apply_writes"
      via: "single-backend caller invokes the same shared function"
      pattern: "write_loop::apply_writes"
    - from: "crates/reposix-cache/fixtures/cache_schema.sql"
      to: "crates/reposix-cache/src/audit.rs::log_helper_push_partial_fail_mirror_lag"
      via: "schema CHECK list lists the op the helper writes"
      pattern: "helper_push_partial_fail_mirror_lag"
---

# Phase 83 Plan 01 — Bus remote: write fan-out core (DVCS-BUS-WRITE-01..05)

<objective>
Land the SoT-first-write + mirror-best-effort fan-out core for the
v0.13.0 bus remote. P82 shipped the read/dispatch surface (URL
parser, prechecks A + B, capability branching) and ends in
`emit_deferred_shipped_error` after both prechecks pass. P83-01
replaces that stub with the full algorithm:

1. **Read fast-import stream from stdin** (verbatim
   `parse_export_stream` from `handle_export`).
2. **Apply REST writes to SoT** via shared
   `write_loop::apply_writes` (lifted from `handle_export` lines
   360-606 in P83-01 T02 — single atomic refactor, narrow-deps
   signature mirroring P81's `precheck_export_against_changed_set`).
   On success, write `helper_push_accepted` to `audit_events_cache`,
   advance `last_fetched_at`, derive `sot_sha` via
   `cache.refresh_for_mirror_head()`, write
   `refs/mirrors/<sot>-head`. On any SoT-fail, return `WriteOutcome::<variant>`
   for the caller to map to `push_failed = true` + `error refs/heads/main <kind>`.
3. **Push to mirror** via `Command::new("git").args(["push",
   mirror_remote_name, "main"])` shell-out (no `--force-with-lease`
   per D-08; no retry per Q3.6).
4. **Branch on (`WriteOutcome`, `MirrorResult`):**
   - SoT-success + mirror-success → write `synced-at` ref + write
     `mirror_sync_written` cache audit row + emit `ok refs/heads/main`.
   - SoT-success + mirror-fail → DO NOT write `synced-at` (frozen
     at last successful mirror sync) + write
     `helper_push_partial_fail_mirror_lag` cache audit row + stderr
     WARN + emit `ok refs/heads/main` (Q3.6 SoT contract satisfied).
   - SoT-fail → mirror push NEVER attempted; reject lines + audit
     rows already emitted inside `apply_writes`; `state.push_failed
     = true`; return cleanly.

This is a **single plan, six sequential tasks** per RESEARCH.md
§ "Plan Splitting":

- **T01** — Catalog-first: 4 rows in `quality/catalogs/agent-ux.json` +
  4 TINY verifier shells (status FAIL).
- **T02** — `write_loop::apply_writes` refactor lift (single atomic
  commit; `handle_export` body shrinks to wrapper shape; existing
  single-backend integration tests all GREEN).
- **T03** — Cache audit op `helper_push_partial_fail_mirror_lag`:
  schema delta in `cache_schema.sql:28-48` op CHECK list + helper
  in `audit.rs` + `Cache::` wrapper in `cache.rs` + 1 unit test in
  `audit.rs::mod tests`. Single atomic commit (D-03).
- **T04** — `bus_handler.rs` write fan-out: replace
  `emit_deferred_shipped_error` stub with the full algorithm; add
  `push_mirror` helper + `MirrorResult` enum.
- **T05** — 2 integration tests: `bus_write_happy.rs` (SoT-success
  + mirror-success → both refs + dual audit table + `ok`),
  `bus_write_no_mirror_remote.rs` (SC4 / Q3.5 regression). Append
  `make_failing_mirror_fixture` + `count_audit_cache_rows` helpers
  to `tests/common.rs` (P83-02 will consume both).
- **T06** — Catalog flip FAIL → PASS + CLAUDE.md update + per-phase
  push.

Sequential (T01 → T02 → T03 → T04 → T05 → T06). Per CLAUDE.md
"Build memory budget" the executor holds the cargo lock sequentially
across T02 (`-p reposix-remote`), T03 (`-p reposix-cache`), T04
(`-p reposix-remote`), T05 (`-p reposix-remote`). T01 + T06 are
doc-or-shell-only; no cargo. NEVER `cargo --workspace`.

**Architecture (read BEFORE diving into tasks):**

The `apply_writes` function is a SHARED entry point — both
`handle_export` (single-backend) and
`bus_handler::handle_bus_export` (bus) call it. Per D-01,
`apply_writes` writes the head ref unconditionally on SoT-success
but DEFERS the `synced-at` ref + `mirror_sync_written` audit row
+ `log_token_cost` to the caller. Single-backend caller writes
synced-at unconditionally (single-backend has no separate mirror
leg — "SoT success" already means "mirror current"). Bus caller
defers synced-at to the `MirrorResult::Ok` branch.

The `push_mirror` helper lives in `bus_handler.rs` (NOT
`write_loop.rs`) because it's bus-specific. Plain `git push <name>
main` — NO `--force-with-lease` per D-08; NO retry per Q3.6 / D-08.

The new `helper_push_partial_fail_mirror_lag` audit op extends the
`audit_events_cache` op CHECK list at `cache_schema.sql:28-48`.
Stale cache.db files keep the legacy CHECK list (per the existing
comment); the audit helper is best-effort (returns `()`, WARN-logs
on INSERT failure), so stale caches WARN-log + the push still
succeeds. Fresh caches accept the row immediately. NO migration
script. Established P79 + P80 pattern (Pitfall 7).

**Key invariant on the partial-fail path:** `head` is written
BEFORE `push_mirror`; `synced-at` is written ONLY after
`push_mirror` returns Ok. If `head` were written AFTER `synced-at`,
an observer reading mid-write would see `synced-at > head` for a
moment — violating the load-bearing invariant *"synced-at is the
timestamp the head ref was last brought current."* Pitfall 1.

**No new error variants.** The remote crate uses `anyhow` throughout
(per `crates/reposix-remote/src/main.rs:18`). `apply_writes` +
`bus_handler` write fan-out + `push_mirror` all return
`anyhow::Result<...>`. Reject-path stderr strings are passed via
`.context("...")` annotations and emitted by the existing
`fail_push(diag, kind, detail)` shape (or its bus-handler-local
clone `bus_fail_push`).

**Best-effort vs hard-error semantics:**

- **`apply_writes` body — L1 precheck conflict:** soft error path
  (rejection); emit `error refs/heads/main fetch first` + hint
  citing mirror-lag refs (when populated); write
  `helper_push_rejected_conflict` cache audit row; return
  `WriteOutcome::Conflict`.
- **`apply_writes` body — L1 precheck REST unreachable:** soft
  error path; emit `error refs/heads/main backend-unreachable`;
  return `WriteOutcome::PrecheckBackendUnreachable`.
- **`apply_writes` body — `plan(...)` rejected:** soft error path;
  emit `error refs/heads/main bulk-delete | invalid-blob:<path>`;
  return `WriteOutcome::PlanRejected`.
- **`apply_writes` body — any `execute_action` returned Err:** soft
  error path; emit `error refs/heads/main some-actions-failed`;
  return `WriteOutcome::SotPartialFail`.
- **`apply_writes` body — all actions succeeded:** write
  `helper_push_accepted` + advance `last_fetched_at` + derive
  `sot_sha` via `refresh_for_mirror_head` + write
  `refs/mirrors/<sot>-head` (only when `sot_sha.is_some()`); return
  `WriteOutcome::SotOk { sot_sha, files_touched, summary }`. Caller
  decides `synced-at` + `mirror_sync_written` + `log_token_cost`.
- **`bus_handler` post-`apply_writes` — `push_mirror` Err
  (subprocess spawn failed):** propagate via `?` — the helper
  exits with a top-level error. Different from `MirrorResult::Failed`
  (which is non-zero exit from `git push` itself).
- **`bus_handler` post-`apply_writes` — `MirrorResult::Failed`:**
  write `helper_push_partial_fail_mirror_lag` audit row + stderr
  WARN + emit `ok refs/heads/main`. NO retry.
- **`bus_handler` post-`apply_writes` — `MirrorResult::Ok`:** write
  `synced-at` + write `mirror_sync_written` audit row + emit `ok
  refs/heads/main`.

This plan **must run cargo serially** per CLAUDE.md "Build memory
budget". Per-crate fallback (`cargo check -p reposix-remote`,
`cargo nextest run -p reposix-remote`, `cargo check -p reposix-cache`,
`cargo nextest run -p reposix-cache`) used instead of workspace-wide.

This plan terminates with `git push origin main` (per CLAUDE.md push
cadence) with pre-push GREEN. The catalog rows' initial FAIL status
is acceptable through T01–T05 because the rows are `pre-pr` cadence
(NOT `pre-push`); the runner re-grades to PASS during T06 BEFORE the
push commits. P83-02 is dispatched as a separate plan invocation
AFTER P83-01's terminal push lands; the phase verifier subagent
fires AFTER P83-02 closes.
</objective>

## Chapters

- **[context.md](./context.md)** — `<canonical_refs>` (spec sources, lift substrate, audit op patterns, test fixtures, quality gates, operating principles, shell-out introduction) + `<threat_model>` (trust boundaries + STRIDE threat register T-83-01..T-83-05). Read before any task.

- **[T01.md](./T01.md)** — Task 83-01-T01: Catalog-first — mint 4 catalog rows in `quality/catalogs/agent-ux.json` + author 4 TINY verifier shells. The GREEN contract lands before implementation. Row 2 stays FAIL through P83-01 close; flips at P83-02 T04.

- **[T02-step-1.md](./T02-step-1.md)** — Task 83-01-T02 (part 1 of 2): HARD-BLOCK analysis, `execute_action` narrow-deps refactor precondition, and `### 2a` — the new `write_loop.rs` module with `WriteOutcome` enum + `apply_writes` function body (full Rust source).

- **[T02-step-2.md](./T02-step-2.md)** — Task 83-01-T02 (part 2 of 2): `### 2b` through `### 2f` — refactor `execute_action`, replace `handle_export` body with wrapper shape, add `mod write_loop;`, cargo check + test run, atomic commit. Includes `<verify>` + `<done>` criteria.

- **[T03.md](./T03.md)** — Task 83-01-T03: Cache audit op `helper_push_partial_fail_mirror_lag` — schema delta in `cache_schema.sql`, helper fn in `audit.rs`, `Cache::` wrapper in `cache.rs`, unit test asserting INSERT round-trip + append-only trigger invariant. Single atomic commit (D-03).

- **[T04.md](./T04.md)** — Task 83-01-T04: `bus_handler` write fan-out — replace `emit_deferred_shipped_error` stub with the full algorithm; add `MirrorResult` enum + `push_mirror` helper (T-83-01 defensive reject + T-83-02 stderr tail trim); remove stub; update module-doc. Atomic commit.

- **[T05.md](./T05.md)** — Task 83-01-T05: 2 integration tests (`bus_write_happy.rs` happy-path + `bus_write_no_mirror_remote.rs` regression) + `tests/common.rs` helpers (`make_failing_mirror_fixture` + `count_audit_cache_rows` for P83-02 consumption).

- **[T06.md](./T06.md)** — Task 83-01-T06: Catalog flip rows 1/3/4 FAIL → PASS + CLAUDE.md update (Bus write fan-out paragraph + commands bullet) + per-phase `git push origin main`. Terminal task.

## Plan-internal close protocol

After T06 push lands, P83-01 transitions out of the executor's
hands. The orchestrator (top-level coordinator) dispatches P83-02
(sequential plan; uses the helpers added by P83-01 T05 to
`tests/common.rs`).

NONE of the following steps fire after P83-01 close (they fire
after P83-02 close):

1. Verifier subagent dispatch (per `quality/PROTOCOL.md § "Verifier
   subagent prompt template"`).
2. Verdict at `quality/reports/verdicts/p83/VERDICT.md`.
3. STATE.md cursor advance.
4. REQUIREMENTS.md DVCS-BUS-WRITE-01..06 checkboxes flipped.
5. SURPRISES-INTAKE / GOOD-TO-HAVES drain check.

P83-02 dispatches as a separate plan invocation. Its terminal
push (P83-02 T04) closes the phase and triggers the verifier
subagent dispatch.
