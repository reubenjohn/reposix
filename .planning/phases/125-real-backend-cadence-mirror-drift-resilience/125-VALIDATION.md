---
phase: 125
slug: real-backend-cadence-mirror-drift-resilience
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-18
finalized: 2026-07-18
---

# Phase 125 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> **Finalized by the planner** — the Per-Task Verification Map below now carries concrete
> `125-NN-TT` task IDs (mirroring the P116 precedent). Wave 0 gaps are scheduled as real
> tasks: the augmented-hint Rust test is the catalog-first RED first commit of Plan 01
> (task 125-01-01, committed BEFORE the write_loop.rs impl at 125-01-02); the DRAIN-02
> run-twice + backend-drift manual-verification procedures are documented in Plan 02
> (task 125-02-02) and the Manual-Only table below.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework (Rust)** | `cargo test` / `cargo nextest run`, per-crate (`reposix-remote`) |
| **Framework (bash gates)** | Hand-rolled `pass`/`fail` shell harness + `quality/gates/agent-ux/lib/transcript.sh`; no bats/shunit2 in this repo |
| **Config file** | none dedicated — `Cargo.toml` per crate; shell gates are catalog-driven (`quality/catalogs/agent-ux.json`) |
| **Quick run command** | `cargo test -p reposix-remote` (bus_precheck_a, bus_precheck_b, push_conflict — bin-target-adjacent, per `crates/CLAUDE.md`'s bin-vs-integration-target seam) |
| **Full suite command** | `python3 quality/runners/run.py --cadence pre-release-real-backend` (env-gated; needs real TokenWorld/GitHub creds) |
| **Estimated runtime** | Quick: <30s (no real backend). Full real-backend litmus run: minutes (network REST + git push round-trips); the DRAIN-02 regression proof requires running it TWICE back-to-back. |

---

## Sampling Rate

- **After every task commit:** `cargo test -p reposix-remote` for any Rust teaching-string
  edit (`write_loop.rs`, `push_conflict.rs`); `bash -n <script>` syntax check for any bash
  edit (`quality/gates/agent-ux/lib/litmus-flow.sh`, `.../lib/litmus-self-heal.sh`);
  `bash quality/gates/docs-alignment/walk.sh` + `bash quality/gates/docs-build/mkdocs-strict.sh`
  for any docs edit — a full litmus run needs real creds and is NOT a per-commit gate.
- **After every plan wave:** `bash scripts/preflight-real-backends.sh` (read-only
  reachability check, safe/cheap) before attempting a real litmus run; a real litmus run
  itself only at a wave boundary where real-backend budget is available.
- **Before `/gsd-verify-work` / phase close:** `python3 quality/runners/run.py --cadence
  pre-release-real-backend` exit 0, run **TWICE in immediate succession** — the second run
  is the actual DRAIN-02 regression proof (a stale mirror from the first run's own push
  must NOT false-negative the second run).
- **Max feedback latency:** ~30s for the Rust/bash/docs quick checks; real-backend litmus
  runs are wave-boundary-only, not per-commit.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 125-01-01 | 01 | 1 | DRAIN-12 | T-125-01 | Catalog-first RED: a new Rust test drives the mirror-lag-ref-populated branch (via `write_mirror_synced_at`) and asserts the augmented hint content (`reposix sync --reconcile` + Pattern-C remote-explicit line); committed BEFORE the impl | unit (Rust) | `cargo test -p reposix-remote mirror_lag_reject_hint_recommends_reconcile_and_remote_explicit_rebase -- --exact` (RED first, GREEN after 125-01-02) | ❌→✅ new test in `push_conflict.rs` | ⬜ pending |
| 125-01-02 | 01 | 1 | DRAIN-12 | T-125-01 | `write_loop.rs` recommends `reposix sync --reconcile` (not the no-op bare form) + ADDS the Pattern-C remote-explicit line; pinned `git pull --rebase` substring survives verbatim; no dynamic URL/remote interpolated (no credential can reach stderr) | unit (Rust) | `cargo test -p reposix-remote` full pass + `grep -c "reposix sync --reconcile" crates/reposix-remote/src/write_loop.rs` == 1 | ✅ `write_loop.rs` exists | ⬜ pending |
| 125-01-03 | 01 | 1 | DRAIN-12 | — | Fix-twice: additive Pattern-C clarification in `troubleshooting.md`; existing bare `git pull --rebase` recovery lines unchanged; any doc-alignment rebind minted (not hand-edited) in same commit | docs-alignment | `bash quality/gates/docs-alignment/walk.sh` exit 0 + `grep -q "Pattern-C" docs/guides/troubleshooting.md` | ✅ `troubleshooting.md` exists | ⬜ pending |
| 125-02-01 | 02 | 1 | DRAIN-12, DRAIN-02 | T-125-02 | Litmus self-heals BOTH backend drift (idempotent fixture restore/reparent pre-flight) AND mirror drift (bus-remote fetch + `git rm` + `checkout FETCH_HEAD` overlay before the marker edit); bounded backstop preserved; mirror reconcile does NOT use `reposix sync --reconcile` | shell-subprocess (real backend at phase close) / syntax per-commit | Per-commit: `bash -n quality/gates/agent-ux/lib/litmus-self-heal.sh && bash -n quality/gates/agent-ux/lib/litmus-flow.sh`. Phase close: `python3 quality/runners/run.py --cadence pre-release-real-backend` run twice, both exit 0 | ✅ `litmus-flow.sh` exists; `litmus-self-heal.sh` new | ⬜ pending |
| 125-02-02 | 02 | 1 | DRAIN-02, DRAIN-12 | — | DRAIN-02 run-twice regression proof + non-destructive backend-drift manual verification documented for the phase-close/milestone-close verifier | manual (documented procedure) | `grep -q "DRAIN-02 second-run mirror-drift regression" quality/dispatch/milestone-close-verdict.md` | ❌→✅ dispatch template | ⬜ pending |
| 125-03-01 | 03 | 2 | DRAIN-02 | T-125-03 | `testing-targets.md` names `scripts/refresh-tokenworld-mirror.sh` as the mirror-refresh pre-step (previously zero mentions) + cross-references the self-reconciling litmus; documented command carries no credential args | docs / docs-build | `grep -q "bash scripts/refresh-tokenworld-mirror.sh" docs/reference/testing-targets.md` + `bash quality/gates/docs-build/mkdocs-strict.sh` + `bash quality/gates/structure/banned-words.sh` | ✅ `testing-targets.md` exists | ⬜ pending |
| 125-03-02 | 03 | 2 | DRAIN-02 | T-125-03b | doc-alignment walk gate passes; any new binding minted via the tool, never hand-edited into `doc-alignment.json` | docs-alignment | `bash quality/gates/docs-alignment/walk.sh` exit 0 | ✅ `doc-alignment.json` exists | ⬜ pending |
| PHASE-G1 | all | close | DRAIN-02+DRAIN-12 (constraint) | — | Owner ruling honored: no product-code rewrite of the bus push's mirror fan-out (`GTH-V15-38` Option C) — phase stays in the test-harness/teaching-string/doc layer | gate | `git diff --stat main -- crates/reposix-remote/src/bus_handler.rs` reviewed: mirror fan-out / mirror-egress algorithm untouched (only precheck teaching-string in `write_loop.rs` changed) | n/a | ⬜ pending |
| PHASE-G2 | all | close | DRAIN-02+DRAIN-12 | — | No regression in existing pinned `git pull --rebase` substring tests / doc-alignment rows (≥12 hits, live grep 19) | gate | `cargo test -p reposix-remote` full pass + `bash quality/gates/docs-alignment/walk.sh` exit 0 | n/a | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Nyquist continuity: no 3 consecutive tasks lack an automated `<verify>` — every 125-NN-TT
task carries a per-commit automated check (cargo test / `bash -n` / grep / walk / mkdocs);
the two real-backend behaviors (run-twice second-run PASS; live backend-drift self-heal) are
wave-boundary/phase-close manual verifications by design (no scripted way to induce a trashed
fixture; matches the P124 `example-05` real-backend-only precedent), captured in Manual-Only
below rather than fabricated as synthetic automated tests.*

---

## Wave 0 Requirements

- [x] **"Run litmus twice back-to-back" proof procedure** — scheduled as task 125-02-02
  (documented manual verification in `quality/dispatch/milestone-close-verdict.md` +
  Manual-Only below), executed by the phase-close/milestone-close verifier against real
  TokenWorld. Not new CI wiring (cost/complexity of a fully automated "run twice" gate on
  top of the `shell-subprocess` kind).
- [x] **New Rust unit test asserting the augmented mirror-lag hint content** (not just the
  pinned base substring) — scheduled as the catalog-first RED first commit of Plan 01
  (task 125-01-01, sibling to `push_conflict.rs`'s existing
  `stale_base_push_emits_fetch_first_and_writes_no_rest`), committed before the impl at
  125-01-02.
- [x] **No scripted way to induce a trashed-fixture state** — acknowledged, NOT fabricated
  as a synthetic automated test. DRAIN-12's backend-drift half stays a documented
  manual/non-destructive verification (task 125-02-02 / Manual-Only), consistent with the
  fixture-restore pre-flight (125-02-01) being idempotent.

*Existing infrastructure (cargo test harness, quality/runners/run.py catalog machinery,
scripts/refresh-tokenworld-mirror.sh, scripts/confluence_tokenworld.py) covers composition
of the fix — no new test framework needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Task | Why Manual | Test Instructions |
|----------|-------------|------|------------|-------------------|
| Litmus self-heals a trashed protected page (backend drift) | DRAIN-12 | 125-02-01 | No scripted way to reliably induce the trashed-fixture precondition against real TokenWorld; inducing it destructively risks the protected pair (7766017/7798785) the tooling refuses to delete | Confirm fixture state via `python3 scripts/confluence_tokenworld.py list`; the `_litmus_fixture_preflight` restore/reparent sweep is idempotent — if a genuine drift is observed, run the litmus and confirm it self-heals before the marker push without a manual `restore` intervention |
| Second back-to-back litmus run does not false-negative on mirror drift | DRAIN-02 | 125-02-01 | Requires two sequential real-backend runs with real network/git round-trips; not a per-commit gate | Run `python3 quality/runners/run.py --cadence pre-release-real-backend` twice in immediate succession; confirm BOTH exit 0 (the second run is the regression proof — the first run's own push re-stales the mirror, and `_litmus_mirror_reconcile` must prevent a stale-base rebase conflict) |
| Corrected teaching string reads as a coherent, non-misleading fix | DRAIN-12 | 125-01-02 | Teaching-string clarity/UX quality is a Rust-compiler-grade-UX judgment call, not grep-verifiable | Read the corrected hint cold: does it point a stale-mirror operator at `reposix sync --reconcile` for the cache + a remote-explicit `git pull --rebase <bus-remote> main` for the tree, rather than a bare `git pull --rebase` that conflicts on divergent body content? |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or a documented Wave 0 / Manual-Only dependency
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (3 gaps scheduled as real tasks / manual procedures)
- [x] No watch-mode flags
- [x] Feedback latency < 30s (quick checks); real-backend litmus is wave-boundary-only
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** finalized — Per-Task Verification Map carries concrete `125-NN-TT` task IDs;
`status: ready`, `nyquist_compliant: true`, `wave_0_complete: true`.
