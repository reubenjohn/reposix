---
phase: 114
slug: t4-confluence-oid-drift-fix-first-reconcile-audit
status: planned
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-14
---

# Phase 114 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Detailed validation architecture lives in `114-RESEARCH.md` § Validation Architecture;
> the planner populates the per-task map below from it.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo nextest` (unit/integration) + bash P0 real-backend gate |
| **Config file** | workspace `Cargo.toml` / `.config/nextest.toml` (existing) |
| **Quick run command** | `cargo nextest run -p reposix-confluence` (per-crate — build-memory budget, ONE cargo invocation) |
| **Full suite command** | `cargo nextest run -p reposix-confluence -p reposix-cache` |
| **Estimated runtime** | ~30–90s (per-crate) |

Real-backend acceptance gate (Success Criteria 1–2) is NOT a cargo test:
`bash quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` — requires
`.env` sourced in the SAME invocation + `scripts/refresh-tokenworld-mirror.sh` pre-step
(mirror-lag false-negative guard). Owner-sanctioned real target: Confluence TokenWorld.

---

## Sampling Rate

- **After every task commit:** Run `cargo nextest run -p reposix-confluence` (Plan 01) / `cargo nextest run -p reposix-cache --test oid_drift_reconcile` (Plan 02)
- **After every plan wave:** Run the full suite command above
- **Before `/gsd-verify-work`:** Full suite green + the P0 real-backend gate GREEN
- **Max feedback latency:** ~90 seconds (unit); real-backend gate is a phase-close acceptance run

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| P01-T1 | 114-01 | 1 (W0) | FIX-01 | T-114-01 taint (fix preserves `Tainted<T>`/`sanitize`) | list-render body == get-render body for an ADF-native page (render-parity) | unit (wiremock) | `cargo nextest run -p reposix-confluence list_and_get_render_parity` | ❌ W0 (RED before fix) | ⬜ pending |
| P01-T2 | 114-01 | 1 | FIX-01 | T-114-03 (do NOT weaken `read_blob` drift check) | `list_issues_impl` sends `body-format=atlas_doc_format`; `written_oid != oid` count in builder.rs unchanged | unit (wiremock) | `cargo nextest run -p reposix-confluence` | ✅ (created P01-T1) | ⬜ pending |
| P01-SC1/SC2 | 114-01 | 1 | FIX-01 | T-114-02 SSRF cursor (allowlist re-check) | live TokenWorld `git checkout -B main` (incl. 7766017) zero oid-drift abort; P0 gate GREEN | shell-subprocess, real-backend (env-gated) | `set -a; . ./.env; set +a; bash scripts/refresh-tokenworld-mirror.sh; bash quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` | ✅ gate exists | ⬜ pending (phase-close) |
| P02-T1 | 114-02 | 2 (W0) | FIX-02 | T-114-04 (drift check stays intact) | divergent list/get bodies → `Error::OidDrift`; repeated `build_from` leaves stale list-oid unchanged; aligned resolves | unit (cache mock `BackendConnector`) | `cargo nextest run -p reposix-cache --test oid_drift_reconcile` | ❌ W0 (new file) | ⬜ pending |
| P02-T2 | 114-02 | 2 | FIX-02 | T-114-05 (doc overclaim) | error.rs/sync.rs/main.rs docs name 3 drift classes; cache.rs cursor-drift doc left intact | manual / docs (prose) + `cargo doc` builds | `cargo doc -p reposix-cache -p reposix-cli --no-deps` | N/A (doc prose) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/reposix-confluence/tests/contract.rs::list_and_get_render_parity` — asserts the LIST wiremock stub receives `body-format=atlas_doc_format` AND `list_records` body == `get_record` body for an ADF-native fixture page (render-parity regression guard for FIX-01). RED before the Plan 01 Task 2 fix.
- [ ] `crates/reposix-cache/tests/oid_drift_reconcile.rs` (NEW) — `DriftingMock` `BackendConnector` proving: (a) divergent list/get bodies → `Error::OidDrift` (repro), (b) a second `build_from()` (= `sync --reconcile`) leaves the stale list-oid unchanged while bodies diverge (FIX-02 non-recovery), (c) aligned bodies resolve cleanly (FIX-01 resolution). Reproduction-backed, not assumed.

*Existing `reposix-confluence` / `reposix-cache` test infrastructure covers the framework
(`wiremock`, `tokio::test`, `cargo nextest`, `CappingMock`/`CacheDirGuard`/`sample_issues`
analogs); only the render-parity + reconcile-scope guards are net-new.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live TokenWorld `git checkout -B main` (incl. page 7766017) completes with zero oid-drift abort | FIX-01 (SC1) | Needs live Confluence creds + `.env` | `set -a; . ./.env; set +a; reposix init confluence::TokenWorld /tmp/p114-repro && cd /tmp/p114-repro && git checkout -B main` (SAME invocation, leaf isolation); expect no `OidDrift` |
| P0 gate GREEN against live TokenWorld | FIX-01 (SC2) | Env-gated real-backend cadence | `set -a; . ./.env; set +a; bash scripts/refresh-tokenworld-mirror.sh; bash quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` (exit 0 = GREEN; exit 75 = NOT-VERIFIED env-missing) |
| error.rs/sync.rs/main.rs reconcile docs are accurate (3 drift classes) | FIX-02 (SC4) | Prose accuracy has no automated assertion | Verifier subagent reads the corrected comments against 114-RESEARCH.md § FIX-02 table; the `oid_drift_reconcile.rs` reconcile-non-recovery test backs the "systematic class not reconcile-recoverable" claim empirically |

*Automated P0 gate `t4-conflict-rebase-ancestry-real-backend.sh` covers SC1/SC2 in the
CI-gated real-backend cadence; the manual recipe is the developer repro.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (contract.rs render-parity + oid_drift_reconcile.rs)
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** planner-populated 2026-07-14 (per-task map + Wave 0 + nyquist_compliant set from 114-01/114-02 PLANs). SC4 doc-accuracy remains a verifier-subagent prose check (no automated prose assertion possible).
