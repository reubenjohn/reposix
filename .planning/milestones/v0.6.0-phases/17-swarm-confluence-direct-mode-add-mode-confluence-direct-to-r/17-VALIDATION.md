---
phase: 17
slug: swarm-confluence-direct-mode-add-mode-confluence-direct-to-r
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-14
---

# Phase 17 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (tokio::test for async) |
| **Config file** | none — workspace default |
| **Quick run command** | `cargo test -p reposix-swarm` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p reposix-swarm`
- **After every plan wave:** Run `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 17-A-01 | A | 1 | SW-01 | — | N/A | unit | `cargo check -p reposix-swarm` | ❌ Wave A | ⬜ pending |
| 17-A-02 | A | 1 | SW-01 | — | N/A | integration (wiremock) | `cargo test -p reposix-swarm confluence_direct` | ❌ Wave A | ⬜ pending |
| 17-B-01 | B | 2 | SW-02 | — | N/A | integration (wiremock) | `cargo test -p reposix-swarm --test mini_e2e` | ❌ Wave B | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/reposix-swarm/src/confluence_direct.rs` — new file for ConfluenceDirectWorkload
- [ ] `crates/reposix-swarm/tests/mini_e2e.rs` — extend with confluence_direct test

*Existing infrastructure covers all other requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Real Confluence rate-limit respect under 50 clients | SW-01 | Requires real Atlassian credentials + expensive | Set ATLASSIAN_API_KEY + ATLASSIAN_EMAIL + REPOSIX_CONFLUENCE_TENANT; run `cargo test -p reposix-swarm -- --ignored live_confluence_direct` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
