---
phase: 28
slug: jira-cloud-read-only-adapter-reposix-jira-v0-8-0
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-16
---

# Phase 28 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) + `wiremock` crate |
| **Config file** | `crates/reposix-jira/Cargo.toml` |
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

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 28-01-01 | 01 | 1 | JIRA-01 | T-28-01 | HTTP client via `reposix_core::http::client()` only | unit | `cargo test -p reposix-jira` | ❌ W0 | ⬜ pending |
| 28-01-02 | 01 | 1 | JIRA-01 | T-28-02 | Tenant validation rejects SSRF patterns | unit | `cargo test -p reposix-jira tenant_validation_rejects_ssrf` | ❌ W0 | ⬜ pending |
| 28-01-03 | 01 | 1 | JIRA-02 | — | `list_single_page` wiremock | integration | `cargo test -p reposix-jira list_single_page` | ❌ W0 | ⬜ pending |
| 28-01-04 | 01 | 1 | JIRA-02 | — | `list_pagination_cursor` wiremock | integration | `cargo test -p reposix-jira list_pagination_cursor` | ❌ W0 | ⬜ pending |
| 28-01-05 | 01 | 1 | JIRA-02 | — | `get_by_numeric_id` wiremock | integration | `cargo test -p reposix-jira get_by_numeric_id` | ❌ W0 | ⬜ pending |
| 28-01-06 | 01 | 1 | JIRA-02 | — | `get_404_maps_to_not_found` wiremock | integration | `cargo test -p reposix-jira get_404_maps_to_not_found` | ❌ W0 | ⬜ pending |
| 28-01-07 | 01 | 1 | JIRA-03 | — | `status_mapping_matrix` wiremock | integration | `cargo test -p reposix-jira status_mapping_matrix` | ❌ W0 | ⬜ pending |
| 28-01-08 | 01 | 1 | JIRA-03 | — | `adf_description_strips_to_markdown` wiremock | integration | `cargo test -p reposix-jira adf_description_strips_to_markdown` | ❌ W0 | ⬜ pending |
| 28-01-09 | 01 | 1 | JIRA-03 | — | `parent_hierarchy` wiremock | integration | `cargo test -p reposix-jira parent_hierarchy` | ❌ W0 | ⬜ pending |
| 28-01-10 | 01 | 1 | JIRA-04 | T-28-03 | `rate_limit_429_honors_retry_after` wiremock | integration | `cargo test -p reposix-jira rate_limit_429_honors_retry_after` | ❌ W0 | ⬜ pending |
| 28-01-11 | 01 | 1 | JIRA-05 | — | `supports_reports_hierarchy_only` wiremock | integration | `cargo test -p reposix-jira supports_reports_hierarchy_only` | ❌ W0 | ⬜ pending |
| 28-01-12 | 01 | 1 | JIRA-05 | — | `extensions_omitted_when_empty` wiremock | integration | `cargo test -p reposix-jira extensions_omitted_when_empty` | ❌ W0 | ⬜ pending |
| 28-01-13 | 01 | 1 | JIRA-05 | — | `write_ops_return_not_supported` wiremock | integration | `cargo test -p reposix-jira write_ops_return_not_supported` | ❌ W0 | ⬜ pending |
| 28-02-01 | 02 | 2 | JIRA-01 | — | Contract test: `contract_sim` | integration | `cargo test -p reposix-jira contract_sim` | ❌ W0 | ⬜ pending |
| 28-02-02 | 02 | 2 | JIRA-01 | — | Contract test: `contract_jira_wiremock` | integration | `cargo test -p reposix-jira contract_jira_wiremock` | ❌ W0 | ⬜ pending |
| 28-03-01 | 03 | 3 | JIRA-01 | — | CI green: `cargo clippy --workspace -D warnings` | lint | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ | ⬜ pending |
| 28-03-02 | 03 | 3 | JIRA-01 | — | CHANGELOG + docs committed | manual | `cargo test --workspace` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/reposix-jira/tests/contract.rs` — stubs for JIRA-01..05
- [ ] `crates/reposix-jira/src/lib.rs` — crate skeleton
- [ ] `crates/reposix-jira/src/adf.rs` — ADF plain-text walker stub

*Wave 0 creates the crate structure; Wave 1 fills the implementation.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live JIRA tenant round-trip | JIRA-01 | Requires real credentials | Run `contract_jira_live` with `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE` set |
| `--no-truncate` CLI flag | JIRA-02 | Requires 500+ issue project | Use a large Jira project with opt-in flag |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
