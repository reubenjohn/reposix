---
phase: 29
slug: jira-write-path-create-update-delete-or-close-via-transitions-api
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-16
---

# Phase 29 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust test + wiremock |
| **Config file** | `crates/reposix-jira/Cargo.toml` (test deps: wiremock, tokio) |
| **Quick run command** | `cargo test -p reposix-jira` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p reposix-jira`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|--------|
| 29-A-01 | 29-01 | A | JIRA-06 | — | ADF wrapper never includes raw user HTML | unit | `cargo test -p reposix-jira adf` | ⬜ pending |
| 29-A-02 | 29-01 | A | JIRA-06 | — | adf_to_markdown handles unknown nodes safely | unit | `cargo test -p reposix-jira adf` | ⬜ pending |
| 29-B-01 | 29-02 | B | JIRA-06 | T-28-01 | create_issue audit row written; no token in log | wiremock | `cargo test -p reposix-jira create_issue` | ⬜ pending |
| 29-B-02 | 29-02 | B | JIRA-06 | T-28-01 | update_issue audit row written; 204 → hydrate | wiremock | `cargo test -p reposix-jira update_issue` | ⬜ pending |
| 29-C-01 | 29-03 | C | JIRA-06 | — | delete_or_close picks WontFix transition | wiremock | `cargo test -p reposix-jira delete_or_close` | ⬜ pending |
| 29-C-02 | 29-03 | C | JIRA-06 | — | delete_or_close falls back to DELETE on empty transitions | wiremock | `cargo test -p reposix-jira delete_or_close` | ⬜ pending |
| 29-C-03 | 29-03 | C | JIRA-06 | — | supports() returns true for Delete and Transitions | unit | `cargo test -p reposix-jira supports` | ⬜ pending |
| 29-C-04 | 29-03 | C | JIRA-06 | — | contract write invariants pass (wiremock arm) | integration | `cargo test -p reposix-jira contract` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No new test files needed before execution — wiremock tests are added inline with implementation in each wave.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live JIRA tenant write round-trip | JIRA-06 | Requires real credentials + live tenant | `JIRA_EMAIL=... JIRA_API_TOKEN=... REPOSIX_JIRA_INSTANCE=... cargo test -p reposix-jira contract_jira_live -- --ignored` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
