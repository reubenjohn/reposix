---
phase: 79
plan: 03
title: "DVCS-ATTACH-02..04 (tests + close) — reconciliation behavior tests + idempotency + reject + Tainted integration + audit-row + docs"
wave: 3
depends_on: [79-02]
requirements: [DVCS-ATTACH-02, DVCS-ATTACH-03, DVCS-ATTACH-04]
files_modified:
  - crates/reposix-cli/tests/attach.rs
  - crates/reposix-cli/src/attach.rs
  - crates/reposix-cache/src/reconciliation.rs
  - quality/catalogs/agent-ux.json
  - CLAUDE.md
autonomous: true
mode: standard
---

# Phase 79 Plan 03 — `reposix attach` behavior tests + idempotency + close (DVCS-ATTACH-02..04)

<objective>
Land behavior coverage for the scaffold from 79-02: 6 reconciliation-case
integration tests, the re-attach idempotency + multi-SoT-reject tests,
the DVCS-ATTACH-04 reframed part-2 integration test (force one
materialization after attach, assert `Tainted<Vec<u8>>`), the OP-3
audit-row presence test, the CLAUDE.md update, and the per-phase push
that flips the catalog row from FAIL → PASS.

This plan exists per checker B1 — split from the original 79-02 6-task
shape to keep each plan ≤ 4 cargo-heavy tasks under context budget. It
runs in Wave 3, after 79-02's scaffold push lands.

What this plan delivers:

- `crates/reposix-cli/tests/attach.rs` (new) with 8 integration tests
  (T01: 6 reconciliation tests; T02: 2 idempotency/reject tests + 1
  Tainted-materialization test + 1 audit-row test).
- The catalog row `agent-ux/reposix-attach-against-vanilla-clone` is
  re-graded from `status: FAIL` (initial state from 79-02 T01) to
  `status: PASS` by the runner.
- `CLAUDE.md` § "Commands you'll actually use" gains a `reposix attach`
  example; § Architecture (or appropriate section) gains a
  `cache_reconciliation` table convention note.
- The plan terminates with `git push origin main` per CLAUDE.md push
  cadence.

What this plan does NOT deliver:

- New scaffold (any new clap arg, new public API, new module) — those
  belong in 79-02. If a test surfaces a defect, the executor MAY edit
  `crates/reposix-cli/src/attach.rs` or
  `crates/reposix-cache/src/reconciliation.rs` to fix-forward, but the
  scope of those edits is bounded to what the failing test requires.
  Larger scope drifts → file as SURPRISES-INTAKE entry.

This plan **must run cargo serially** per CLAUDE.md "Build memory budget".
After 79-03 pushes, the orchestrator dispatches the verifier subagent
per `quality/PROTOCOL.md` § "Verifier subagent prompt template" — that's
an orchestrator-level action, NOT part of this plan.
</objective>

<must_haves>
- New file `crates/reposix-cli/tests/attach.rs` with 10 integration tests:
  1. `attach_against_vanilla_clone_sets_partial_clone` (DVCS-ATTACH-01) —
     bare `git init` checkout + `git remote add origin <https-url>` +
     1 fixture file, attach succeeds, post-conditions hold:
     `extensions.partialClone=reposix`, remote `reposix` URL has
     `reposix::` prefix and contains `?mirror=` (default `--bus`).
  2. `attach_match_records_by_id` (DVCS-ATTACH-02 case 1) — fixture has
     `id: 1` matching backend; post-attach `cache_reconciliation` has 1
     row with `record_id=1`.
  3. `attach_warns_on_backend_deleted` (DVCS-ATTACH-02 case 2) — fixture
     has `id: 99`; backend does NOT have it; attach succeeds with stderr
     containing `BACKEND_DELETED` (or "deleted on backend"); no row
     added for id=99.
  4. `attach_skips_no_id_files` (DVCS-ATTACH-02 case 3) — fixture has a
     `.md` with no frontmatter; stderr contains `NO_ID` (or "no id
     field"); no row added.
  5. `attach_errors_on_duplicate_id` (DVCS-ATTACH-02 case 4) — fixture
     has 2 files claiming `id: 42`; attach EXIT non-zero; stderr names
     both file paths; no rows committed (transactional).
  6. `attach_marks_mirror_lag_for_next_fetch` (DVCS-ATTACH-02 case 5) —
     backend has id=99 but no local file; attach succeeds; cache state
     shows id=99 known to the backend (visible via the existing
     tree-list machinery + `Cache::list_record_ids`).
  7. `re_attach_same_sot_is_idempotent` (DVCS-ATTACH-03 / Q1.3) —
     attach SoT1 → attach SoT1 again; second invocation exits 0;
     `cache_reconciliation` rows match the post-first-attach state
     (table re-populated from current backend; `INSERT OR REPLACE`
     leaves no stale rows).
  8. `re_attach_different_sot_is_rejected` (DVCS-ATTACH-03 / Q1.2) —
     attach SoT1 → attach SoT2; second invocation EXIT non-zero;
     stderr contains "already attached" AND "multi-SoT not supported in
     v0.13.0".
  9. `attach_then_read_blob_returns_tainted` (DVCS-ATTACH-04 reframed
     part 2) — after attach succeeds, the test forces ONE blob
     materialization via `Cache::read_blob` (the lazy materialization
     seam) and feeds the result into a function that ONLY accepts
     `Tainted<Vec<u8>>` — runtime evidence that OP-2 holds at the
     materialization site `attach` ultimately exposes via the cache.
 10. `attach_audit_log_records_walk_event` (OP-3) — after attach,
     SELECT from `audit_events_cache` for `event_type = 'attach_walk'`
     returns exactly 1 row. Asserts the
     `Cache::log_attach_walk` call landed in 79-02 T03 actually wrote.
- `cargo nextest run -p reposix-cli --tests attach` exits 0; all 10
  tests pass.
- `bash quality/gates/agent-ux/reposix-attach.sh` exits 0 (the verifier
  the catalog row binds to is now satisfied by the scaffold + tests).
- `python3 quality/runners/run.py --cadence pre-pr` exits 0 — confirms
  the `pre-pr` cadence accepts the row and re-grades to PASS.
  (Cadence verified at planning time against
  `quality/runners/run.py:54-56` `VALID_CADENCES`. Per checker W4: if
  the cadence rejects, fall back to `--cadence pre-push` which is
  verified working from P78. Reading the runner during execution
  confirms which cadence the row is graded under.)
- The catalog row `agent-ux/reposix-attach-against-vanilla-clone`
  status is re-grading from `FAIL` (set by 79-02 T01) to `PASS` after
  this plan's runner invocation.
- CLAUDE.md updated: § "Commands you'll actually use" gains an
  `attach` example block immediately after the existing `init` example;
  § Architecture (or a related section) gains a `cache_reconciliation`
  table convention note.
- Plan terminates with `git push origin main` (per CLAUDE.md push cadence)
  with pre-push GREEN. The catalog row's `status` field is updated by
  the runner BEFORE the push; the commit that pushes includes the
  updated catalog (row status PASS).
- All cargo invocations in this plan are SERIAL (one at a time per
  CLAUDE.md Build memory budget).
</must_haves>

<canonical_refs>
- `.planning/REQUIREMENTS.md` DVCS-ATTACH-02..04 — verbatim acceptance
  (DVCS-ATTACH-04 reframed by orchestrator BEFORE verifier dispatch;
  see OVERVIEW § "Reframe of DVCS-ATTACH-04").
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "1. `reposix attach <backend>::<project>`" — sketch + Q1.1, Q1.2, Q1.3.
- `.planning/research/v0.13.0-dvcs/architecture-sketch.md` § "Reconciliation cases" — the 5-row resolution table verbatim.
- `.planning/research/v0.13.0-dvcs/decisions.md` § "Phase-N (`reposix attach`) decisions" — Q1.1/1.2/1.3 ratifications.
- `.planning/phases/79-poc-reposix-attach-core/79-02-PLAN.md` — scaffold + APIs + audit hook this plan exercises.
- `crates/reposix-cli/tests/agent_flow.rs` — integration test pattern (`#[ignore]` real-backend, `#[test]` sim-backed; how to start a sim subprocess + run CLI binary against tempdir).
- `crates/reposix-cli/tests/cli.rs` (top 80 lines — basic clap surface tests; helpful for reference).
- `crates/reposix-sim/src/main.rs` (top 80 lines — sim CLI args, seed format).
- `crates/reposix-cache/src/reconciliation.rs` — the module from 79-02 T03 — to mirror its behavior in test fixture setup.
- `crates/reposix-cache/src/cache.rs` — `Cache::list_record_ids` / `find_oid_for_record` / `connection_mut` / `log_attach_walk` (added in 79-02 T03; this plan exercises them via integration tests).
- `crates/reposix-cache/src/builder.rs:436` — `Cache::read_blob` returns `Tainted<Vec<u8>>` — DVCS-ATTACH-04 part 2 forces one materialization through this API.
- `crates/reposix-core/src/taint.rs` — `Tainted<T>` shape; `_is_tainted(_: Tainted<Vec<u8>>) {}` pattern.
- `quality/catalogs/agent-ux.json` — row added by 79-02 T01; this plan flips status FAIL → PASS via runner.
- `quality/gates/agent-ux/reposix-attach.sh` — verifier authored in 79-02 T01; this plan's tests + scaffold make it exit 0.
- `quality/runners/run.py:54-56` — `VALID_CADENCES = ("pre-push", "pre-pr", "weekly", "pre-release", "post-release", "on-demand")`. Confirmed at planning time; `--cadence pre-pr` is valid.
- `quality/PROTOCOL.md` § "Verifier subagent prompt template" — phase-close verifier (orchestrator dispatches AFTER this plan pushes).
- `CLAUDE.md` § "Commands you'll actually use" — gains the `reposix attach` example.
- `CLAUDE.md` § "Build memory budget" — strict serial cargo.
- `CLAUDE.md` § "Push cadence — per-phase".
- `CLAUDE.md` § Operating Principles OP-1, OP-2, OP-3, OP-7, OP-8.

This plan introduces no new threat-model surface beyond what 79-02
established. Tests use the in-process simulator (`127.0.0.1:<dynamic>`)
which the existing allowlist permits. No new HTTP construction site, no
new shell-out, no new sanitization branch. No `<threat_model>` delta
required.
</canonical_refs>

---

## Chapters

- **[T01 — Reconciliation case integration tests](./T01.md)** — 6 integration tests covering DVCS-ATTACH-01 (extensions.partialClone post-condition) and DVCS-ATTACH-02 (5 reconciliation cases: match / no-id / backend-deleted / duplicate-id / mirror-lag). Includes the SimSubprocess helper, git_init helper, and test fixture patterns.

- **[T02 — Idempotency + multi-SoT reject + Tainted-materialization + audit-row](./T02.md)** — 4 additional integration tests: re-attach idempotency (Q1.3), different-SoT reject (Q1.2), Tainted blob materialization (DVCS-ATTACH-04 reframed part 2), and OP-3 audit-row presence. Catalog row flipped FAIL → PASS by runner at end of this task.

- **[T03 — CLAUDE.md update + per-phase push (terminal)](./T03.md)** — Docs edit adding `reposix attach` example + `cache_reconciliation` convention note to CLAUDE.md. Per-phase push per CLAUDE.md push-cadence rule. Pre-push gate failure modes and recovery procedure.
