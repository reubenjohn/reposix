---
phase: 114
slug: t4-confluence-oid-drift-fix-first-reconcile-audit
status: draft
nyquist_compliant: false
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

- **After every task commit:** Run `cargo nextest run -p reposix-confluence`
- **After every plan wave:** Run the full suite command above
- **Before `/gsd-verify-work`:** Full suite green + the P0 real-backend gate GREEN
- **Max feedback latency:** ~90 seconds (unit); real-backend gate is a phase-close acceptance run

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| _planner-populated_ | — | — | FIX-01 / FIX-02 | OP-2 taint (verify fix preserves `Tainted<T>`/`sanitize`) | list-render body matches get-render body (render-parity) | unit + real-backend gate | _see planner_ | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Unit test asserting `list_issues_impl` and `get_record` request the SAME body representation (render-parity regression guard for FIX-01)
- [ ] Test/assertion proving `sync --reconcile` doc scope matches actual recovery behavior (FIX-02) — reproduction-backed, not assumed

*Existing `reposix-confluence` / `reposix-cache` test infrastructure covers the framework; only the render-parity + reconcile-scope guards are net-new.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live TokenWorld `git checkout -B main` (incl. page 7766017) completes with zero oid-drift abort | FIX-01 | Needs live Confluence creds + `.env` | `reposix init confluence::TokenWorld /tmp/x && cd /tmp/x && git checkout -B main` (SAME invocation, leaf isolation); expect no `OidDrift` |

*Automated P0 gate `t4-conflict-rebase-ancestry-real-backend.sh` covers this in CI-gated real-backend cadence; the manual recipe is the developer repro.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
