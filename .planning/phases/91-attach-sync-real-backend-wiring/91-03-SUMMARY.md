---
phase: 91-attach-sync-real-backend-wiring
plan: 03
subsystem: cli
tags: [attach, sync, backend-dispatch, real-backend, op-3, audit, reconciliation, confluence, jira, github]

requires:
  - phase: 91-02
    provides: canonical issues/<id>.md path fix (QL-001) — makes ForkAsNew work for free
  - phase: 91-01
    provides: attach-sync-real-backend catalog row (minted NOT-VERIFIED)
provides:
  - reposix attach/sync --reconcile construct real confluence/github/jira connectors via one shared factory
  - reposix-remote exposed as a [lib] (pub mod backend_dispatch)
  - OP-3 .with_audit wiring inherited by attach/sync by construction
  - ForkAsNew resolved (works for free) + honest Abort doc
  - attach_real_*/sync_real_* #[ignore] real-backend smokes + verifier script
  - ci.yml forwards JIRA_TEST_PROJECT
affects: [92-audit-log, 95-docs-drain, milestone-close-vision-litmus]

tech-stack:
  added: [reposix-remote path dep for reposix-cli (transitive parking_lot)]
  patterns:
    - "Single URL→connector dispatch factory reused by helper bin + CLI (no per-command copy)"
    - "Real-backend #[ignore] + skip_if_no_env! smoke idiom extended from init to attach/sync"

key-files:
  created:
    - crates/reposix-remote/src/lib.rs
    - quality/gates/agent-ux/attach-sync-real-backend.sh
  modified:
    - crates/reposix-remote/Cargo.toml
    - crates/reposix-remote/src/backend_dispatch.rs
    - crates/reposix-remote/src/main.rs
    - crates/reposix-remote/src/bus_url.rs
    - crates/reposix-cli/Cargo.toml
    - crates/reposix-cli/src/attach.rs
    - crates/reposix-cli/src/sync.rs
    - crates/reposix-cache/src/reconciliation.rs
    - crates/reposix-cli/tests/attach.rs
    - crates/reposix-cli/tests/agent_flow_real.rs
    - .github/workflows/ci.yml

key-decisions:
  - "Consolidation shape = reposix-remote [lib], not a new shared crate (D91-03 default); only prod dep dragged into CLI is parking_lot as anticipated"
  - "ForkAsNew disposition: works for free (no speculative machinery) — leaving the orphan file in place IS the mechanism; next push classifies it as a Create (diff.rs:177)"
  - "Abort naming: fix the doc, NOT rename — `--orphan-policy=abort` is a documented user-facing contract in cli.md + troubleshooting.md"
  - "banned-token P\\d+\\+ regex extension: NOT cheaply extendable (cross-file historical hits in helper internals) → filed intake, routed P97"

patterns-established:
  - "Attach/sync derive (kind, origin, project) from translate_spec_to_url + backend_dispatch::parse_remote_url, sim honours REPOSIX_SIM_ORIGIN override before instantiate"

requirements-completed: [RBF-A-01, RBF-A-02, RBF-A-03, RBF-A-04]

duration: 95min
completed: 2026-07-04
---

# Phase 91 Plan 03: Attach/Sync Real-Backend Wiring Summary

**`reposix attach confluence::TokenWorld` and `sync --reconcile` now construct real Confluence/GitHub/JIRA connectors through the git remote helper's one mature dispatch factory — inheriting OP-3 `.with_audit(...)` by construction — with the P79-era "not yet wired" stub, its phase-ID tokens, and the ForkAsNew TODO all retired; verified live against TokenWorld.**

## Performance

- **Duration:** ~95 min
- **Tasks:** 3/3
- **Files:** 11 modified + 2 created

## Accomplishments

- **Task 1 (RBF-A-01/A-02, D91-03):** Added `crates/reposix-remote/src/lib.rs` + `[lib]` target exposing `pub mod backend_dispatch`; bumped `parse_remote_url`/`instantiate`/`BackendKind`/`ParsedRemote`/`sanitize_project_for_cache` to `pub`; thinned `main.rs`/`bus_url.rs` to `use` the lib. Added the `reposix-cli → reposix-remote` path dep. `attach.rs` + `sync.rs` now parse the translated SoT URL and call `backend_dispatch::instantiate` — deleting the sim-only bail arms.
- **Task 2 (D91-04, RBF-A-03):** Resolved `OrphanPolicy::ForkAsNew` as **works-for-free** (honest "kept; next push creates it" message + a proving integration test); fixed the misleading `OrphanPolicy::Abort` doc (it never aborts); scrubbed the last `P82+` tokens from `reconciliation.rs`. Flipped ForkAsNew + dead-marker intakes RESOLVED; filed the `P\d+\+` banned-token gap.
- **Task 3 (RBF-A-04, D91-09):** Added `attach_real_{confluence,github,jira}` + `sync_real_{confluence,github,jira}` #[ignore] real-round-trip smokes; corrected the stale "helper hardcodes SimBackend" module doc; wrote `quality/gates/agent-ux/attach-sync-real-backend.sh` (env-gated exit-75, OD-2 hard-FAIL, transcript via lib/transcript.sh); forwarded `JIRA_TEST_PROJECT` in both ci.yml JIRA env blocks.

## Dependency diff outcome

`reposix-cli` gains one new path dependency: `reposix-remote = { path = ... }`. The only new **transitive prod dep** pulled into the CLI is `parking_lot` (confirmed absent from the CLI tree before this plan) — exactly what Research Warning #3 anticipated. Everything else `reposix-remote` links (`reposix-core/cache/github/confluence/jira`, `gix`, `rusqlite`, `tokio`, `chrono`, `thiserror`, `serde_json`, `tracing*`) was already in the CLI's dep tree via `reposix-cache`/the backend crates. **No checkpoint required** — the diff is within the anticipated envelope.

## ForkAsNew disposition: FREE (no machinery built)

Per D91-04's investigate-first mandate, confirmed against `reposix-remote/src/diff.rs:177`: once 91-02's canonical-path fix landed, an orphan file present in the working tree but absent from the pushed prior state is already classified `PlannedAction::Create` by the push planner. So `ForkAsNew`'s only job is to **not delete and not abort** — which the arm already did. No `create_record`-at-attach machinery was built (that would have been speculative). The `// TODO P82+` no-op message became `action=FORK_AS_NEW (kept; next push creates it)`, and `attach_fork_as_new_keeps_orphan_for_next_push` proves the file is kept and no TODO is advertised.

## OP-3 evidence (RED-line preserved)

Attach/sync construct connectors **only** through `backend_dispatch::instantiate` — grep-confirmed no `SimBackend::new`/`with_audit`/`Backend::new` bypass survives in `attach.rs`/`sync.rs`:
```
sync.rs:92:  let backend = backend_dispatch::instantiate(&parsed)
attach.rs:156: let connector: Arc<dyn BackendConnector> = backend_dispatch::instantiate(&parsed)
```
`instantiate` routes Confluence/JIRA through `build_confluence`/`build_jira` → `.with_audit(audit)` (backend_dispatch.rs:303,322). The `connector_audit_wired_confluence`/`connector_audit_wired_jira` tests (20 backend_dispatch tests total) pass after the `pub` visibility change. Because attach/sync inherit the factory, OP-3 is satisfied **by construction** for the P92 write path — not "if the next phase remembers".

## Phase-ID token scrub (D91-09) — grep proof

Zero `P\d{2,3}[-+]` tokens remain in the three target files:
```
$ grep -rnE 'P[0-9]{2,3}[-+]' crates/reposix-cli/src/{attach,sync}.rs crates/reposix-cache/src/reconciliation.rs
CLEAN
$ grep -rn 'banned-words: ok — P91' crates/    → zero
```
SC-3 confirmed: `reposix attach confluence::REPOSIX` (no creds) emits a doc-linked missing-env error with **no** phase-ID token. Remaining `P79|P82` hits in `crates/**/*.rs` live only in untouched helper internals (`main.rs`, `bus_handler.rs`, `precheck.rs`) — historical-refactor-class comments, covered by the new `P\d+\+`-banned-token intake entry (routed P97).

## Real-Confluence smoke result (verified against reality)

Ran the verifier live against the TokenWorld tenant (REPOSIX space) with `.env` creds:
```
running 2 tests
test attach_real_confluence ... ok
test sync_real_confluence ... ok
test result: ok. 2 passed; 0 failed; 0 ignored
PASS: real attach + sync --reconcile round-trip against Confluence TokenWorld (transcript emitted)
```
Read-only (attach + sync issue `list_records` only) — no page mutation, no cleanup needed; durable fixture pages 7766017/7798785 untouched. Env-gate confirmed: verifier returns exit 75 with creds unset.

## Per-crate test results

| Command | Result |
| --- | --- |
| `cargo check/clippy -p reposix-remote --all-targets -D warnings` | clean |
| `cargo check/clippy -p reposix-cli --all-targets -D warnings` | clean |
| `cargo clippy -p reposix-cache --all-targets -D warnings` | clean |
| `cargo test -p reposix-remote backend_dispatch` | 20 passed (incl. connector_audit_wired_*) |
| `cargo test -p reposix-cache reconciliation` | passed |
| `cargo test -p reposix-cli --test attach -- --ignored` | 11 passed |
| `cargo test -p reposix-cli --test sync` | 3 passed |
| `cargo test -p reposix-cli --test attach ... attach_fork_as_new...` | 1 passed |
| `cargo test -p reposix-cli --test agent_flow_real` (no creds) | 1 passed, 9 ignored (clean skip) |
| real Confluence attach_real/sync_real (live) | 2 passed |

## Deviations from Plan

- **[Rule 3 - blocking] Verifier sanctioned-space set widened to {TokenWorld, REPOSIX}.** The verifier first hard-coded only `TokenWorld`; the owner's `.env` sets `REPOSIX_CONFLUENCE_SPACE=REPOSIX` (the durable-fixture space, D91-08). Both are owner-owned, documented sanctioned spaces, and the smoke is read-only, so I sanctioned both (still hard-FAIL on any other space, preserving OD-2). Without this the live verification would have hard-failed on the owner's own config.
- No architectural (Rule 4) changes. ForkAsNew stayed free (never turned M+), so no checkpoint.

## NOTICED near this work

- **refresh.rs `fetch_issues` is a THIRD read-only copy** of the env-check/connector pattern (research §2.2). Deliberately NOT absorbed into P91 (read-only, no `with_audit`). Retargeting it at the shared factory is a GOOD-TO-HAVE — noted here, not filed as a blocker.
- **`main.rs`/`bus_handler.rs`/`precheck.rs` carry `P82+` in comments** without `// banned-words: ok` markers. Not caught by the current gate (no `-\d` suffix). Filed the `P\d+\+`-extension intake rather than sweeping files outside this plan's envelope.
- **`OrphanPolicy::Abort` and `ForkAsNew` are behaviourally identical at reconcile time** (both keep the file; the difference is only the operator's stated intent surfaced in the warning). Documented this honestly in the enum doc rather than pretending they diverge.
- **Confluence live-hierarchy self-seeding (D91-08)** is NOT in this plan's envelope (contract.rs untouched) — its intake stays OPEN for the plan/wave that owns it.

## Known Stubs

None introduced. The one pre-existing stub in the plan's scope (ForkAsNew "TODO P82+") was resolved.

## Intake filed / flipped

- RESOLVED: ForkAsNew (cf7a37d), dead-allowlist-marker coupling (1f7fff3), agent_flow_real stale-doc (3f67c0e), JIRA_TEST_PROJECT (3f67c0e).
- NEW (OPEN, routed P97): `banned-production-tokens.sh` misses the no-suffix `P\d+\+` shape; not cheaply extendable.

## Self-Check: PASSED

All 3 created files exist on disk; all 5 commits (1f7fff3, cf7a37d, 9af1d69, 3f67c0e, bd5e115) present in git log.
