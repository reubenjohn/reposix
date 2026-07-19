---
phase: 125
slug: real-backend-cadence-mirror-drift-resilience
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-18
---

# Phase 125 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> **Skeleton, per RESEARCH.md § Validation Architecture** — the Per-Task Verification Map
> below uses placeholder task IDs; the planner finalizes it with concrete `125-NN-TT` task
> IDs (mirroring the P116 precedent — `116-VALIDATION.md` skeleton → planner fills in real
> task IDs when writing PLAN.md) and flips `status: ready` / `nyquist_compliant: true` once
> Wave 0 gaps below are scheduled as real tasks.

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
  edit (`write_loop.rs` / `bus_handler.rs`); `bash -n <script>` syntax check for any bash
  edit (`quality/gates/agent-ux/lib/litmus-flow.sh`, `scripts/refresh-tokenworld-mirror.sh`)
  — a full litmus run needs real creds and is NOT a per-commit gate.
- **After every plan wave:** `bash scripts/preflight-real-backends.sh` (read-only
  reachability check, safe/cheap) before attempting a real litmus run; a real litmus run
  itself only at a wave boundary where real-backend budget is available.
- **Before `/gsd-verify-work` / phase close:** `python3 quality/runners/run.py --cadence
  pre-release-real-backend` exit 0, run **TWICE in immediate succession** — the second run
  is the actual DRAIN-02 regression proof (a stale mirror from the first run's own push
  must NOT false-negative the second run).
- **Max feedback latency:** ~30s for the Rust/bash quick checks; real-backend litmus runs
  are wave-boundary-only, not per-commit.

---

## Per-Task Verification Map

*Skeleton — planner assigns concrete `125-NN-TT` task IDs when PLAN.md is written.*

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | DRAIN-02 | T-125-01 | mirror-refresh pre-step documented/enforced OR litmus self-reconciles; second back-to-back litmus run PASSES (no false-negative from the first run's own push re-staling the mirror) | shell-subprocess (real backend) | `bash quality/gates/agent-ux/milestone-close-vision-litmus.sh` run twice; 2nd run exit 0 | ✅ litmus script exists; "run twice" proof procedure is Wave 0 new | ⬜ pending |
| TBD | TBD | TBD | DRAIN-02 | — | pre-step referenced in `docs/reference/testing-targets.md` (if SC1 goes the doc route) | docs-alignment / manual | `bash quality/gates/docs-alignment/walk.sh`; manual read of `docs/reference/testing-targets.md` for `refresh-tokenworld-mirror.sh` mention | ❌ Wave 0 — no existing binding for this claim | ⬜ pending |
| TBD | TBD | TBD | DRAIN-12 | T-125-02 | litmus self-heals a trashed protected page (backend drift) | shell-subprocess (real backend) / manual | no scripted way to induce trashed-fixture state repeatably — likely a documented manual verification step, not an automated regression test (see Wave 0 gap) | ❌ Wave 0 gap — flag, do not fabricate a synthetic test | ⬜ pending |
| TBD | TBD | TBD | DRAIN-12 | T-125-02 | litmus self-heals GitHub mirror drift, reconciling through the reposix bus remote before the marker push | shell-subprocess (real backend) | run litmus twice back-to-back (same proof as DRAIN-02 row above) | ✅ once self-heal lands | ⬜ pending |
| TBD | TBD | TBD | DRAIN-12 | T-125-03 | corrected mirror-drift teaching string present (augmenting, never removing/replacing the pinned `"git pull --rebase"` substring — ≥12 existing regression tests + doc-alignment rows pin it verbatim) | unit (Rust) | `cargo test -p reposix-remote` (existing pinned-substring tests) + a NEW test asserting the augmented hint content | ❌ Wave 0 — no existing test asserts the augmented hint text | ⬜ pending |
| TBD | TBD | TBD | DRAIN-12 (security) | — | any newly-interpolated remote/URL string in a corrected hint is redacted via `reposix_core::http::strip_url_userinfo` or `reposix_remote::backend_dispatch::redact_userinfo` — never echo raw credentials | unit (Rust) | `cargo test -p reposix-remote` (redaction test, pattern per existing `precheck_mirror_drift_redacts_credentials_and_teaches_on_ls_remote_failure`) | ❌ Wave 0 — write alongside the teaching-string edit | ⬜ pending |
| PHASE-G1 | all | close | DRAIN-02+DRAIN-12 (constraint) | — | manager ruling honored: no product-code rewrite of the bus push's mirror fan-out (`GTH-V15-38` Option C) — this phase stays in the test-harness/teaching-string/doc layer | gate | `git diff --stat -- crates/reposix-remote/src/bus_handler.rs crates/reposix-remote/src/mirror_egress.rs` reviewed manually for scope (fan-out algorithm untouched; only precheck teaching-string / litmus-flow changes) | n/a | ⬜ pending |
| PHASE-G2 | all | close | DRAIN-02+DRAIN-12 | — | no regression in existing pinned `"git pull --rebase"` substring tests/doc-alignment rows (≥12 hits) | gate | `cargo test -p reposix-remote` full pass + `bash quality/gates/docs-alignment/walk.sh` | n/a | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] **"Run litmus twice back-to-back" proof procedure** — the core DRAIN-02 regression
  test does not exist as a scripted artifact yet. Likely a documented manual verification
  step for the verifier subagent to execute against real TokenWorld (cost/complexity of a
  fully automated "run twice" gate on top of the existing `shell-subprocess` kind), rather
  than new CI wiring. First Wave 0 item the planner should schedule.
- [ ] **New Rust unit test asserting the augmented mirror-lag hint content** (not just the
  pinned base substring) — sibling to the existing `bus_precheck_b.rs` / `write_loop.rs`
  test pattern (`stderr.contains(...)`).
- [ ] **No scripted way to induce a trashed-fixture state** for repeatable backend-drift
  testing — DRAIN-12's backend-drift half may need to stay a manual/documented
  verification step. Flag explicitly; do not fabricate a synthetic automated test.

*Existing infrastructure (cargo test harness, quality/runners/run.py catalog machinery,
scripts/refresh-tokenworld-mirror.sh, scripts/confluence_tokenworld.py) covers composition
of the fix — no new test framework needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Litmus self-heals a trashed protected page (backend drift) | DRAIN-12 | No scripted way to reliably induce the trashed-fixture precondition against the real TokenWorld backend; inducing it destructively risks the protected fixture pair (7766017/7798785) that the tooling explicitly refuses to delete | Coordinate with `scripts/confluence_tokenworld.py list` to confirm current fixture state; if a genuine drift is observed (or can be safely simulated via `restore`/`reparent` round-trip), run the litmus and confirm it self-heals before the marker push, without manual `confluence_tokenworld.py restore` intervention |
| Second back-to-back litmus run does not false-negative on mirror drift | DRAIN-02 | Requires two sequential real-backend runs with actual network/git round-trips; not a per-commit gate | Run `python3 quality/runners/run.py --cadence pre-release-real-backend` twice in immediate succession; confirm both exit 0 |
| Corrected teaching string reads as a coherent, non-misleading fix (not just literally distinct from the pinned substring) | DRAIN-12 | Teaching-string clarity/UX quality is a Rust-compiler-grade-UX judgment call, not grep-verifiable | Read the corrected hint cold: does it correctly point a stale-mirror-clone user at `git fetch <bus-remote> && git rebase && git push` (or `scripts/refresh-tokenworld-mirror.sh`) rather than a bare `git pull --rebase` that will conflict on divergent body content? |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies (pending planner task assignment)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (3 gaps listed above)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s (quick checks); real-backend litmus is wave-boundary-only
- [ ] `nyquist_compliant: true` set in frontmatter (pending — flips once planner finalizes Per-Task map)

**Approval:** pending — planner finalizes Per-Task Verification Map with concrete task IDs.
