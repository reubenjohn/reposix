---
phase: 21
slug: op-7-hardening-bundle-contention-swarm-500-page-truncation-p
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-15
---

# Phase 21 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) |
| **Config file** | Cargo.toml workspace |
| **Quick run command** | `cargo test --workspace` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all --check` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --workspace`
- **After every plan wave:** Run full suite command above
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 21-audit-01 | A | 1 | HARD-00 | — | Credential hook 6/6 tests pass | unit | `bash scripts/hooks/test-pre-push.sh` | ✅ | ⬜ pending |
| 21-audit-02 | A | 1 | HARD-00 | SG-06 | SSRF tests pass in contract.rs | unit | `cargo test --test contract ssrf` | ✅ | ⬜ pending |
| 21-contention-01 | B | 2 | HARD-01 | — | 409 deterministic from If-Match | integration | `cargo test --test contention` | ❌ W0 | ⬜ pending |
| 21-truncation-01 | C | 3 | HARD-02 | SG-05 | WARN emitted at 500-page cap | unit | `cargo test confluence truncation_warn` | ✅ | ⬜ pending |
| 21-truncation-02 | C | 3 | HARD-02 | SG-05 | --no-truncate errors on cap | unit | `cargo test confluence no_truncate_errors` | ❌ W0 | ⬜ pending |
| 21-chaos-01 | D | 4 | HARD-03 | — | WAL consistent after kill-9 | integration | `cargo test --test chaos_audit` | ❌ W0 | ⬜ pending |
| 21-tenant-01 | E | 5 | HARD-05 | — | Log URLs contain path only | unit | `cargo test confluence tenant_url_redact` | ❌ W0 | ⬜ pending |
| 21-macos-01 | F | 6 | HARD-04 | — | macOS CI job defined in .github/workflows | manual | Check .github/workflows/ for macos-14 runner | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/reposix-swarm/tests/contention.rs` — stubs for HARD-01 contention test
- [ ] `crates/reposix-sim/tests/chaos_audit.rs` — stubs for HARD-03 chaos test
- [ ] URL redaction unit test stub in `crates/reposix-confluence/src/lib.rs`

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| macOS CI green | HARD-04 | Requires paid macOS runner; cannot be faked locally | Inspect .github/workflows/ci.yml for macos-14 runner job |
| Live macFUSE mount | HARD-04 | macOS hardware required | Check workflow runs after PR |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
