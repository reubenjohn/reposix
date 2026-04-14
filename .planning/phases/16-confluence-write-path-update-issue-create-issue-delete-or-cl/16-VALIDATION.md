---
phase: 16
slug: confluence-write-path-update-issue-create-issue-delete-or-cl
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-14
---

# Phase 16 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (tokio::test for async) |
| **Config file** | none — workspace default |
| **Quick run command** | `cargo test -p reposix-confluence` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p reposix-confluence`
- **After every plan wave:** Run `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 16-A-01 | A | A | WRITE-04 | — | N/A | unit (pure) | `cargo test -p reposix-confluence adf` | ❌ Wave A | ⬜ pending |
| 16-B-01 | B | B | WRITE-01 | SG-03 / Taint | Untainted<Issue> before POST | unit (wiremock) | `cargo test -p reposix-confluence create_issue` | ❌ Wave B | ⬜ pending |
| 16-B-02 | B | B | WRITE-02 | SG-03 / Taint | version.number = current+1 | unit (wiremock) | `cargo test -p reposix-confluence update_issue` | ❌ Wave B | ⬜ pending |
| 16-B-03 | B | B | WRITE-03 | SG-03 / Taint | DELETE returns Ok on 204 | unit (wiremock) | `cargo test -p reposix-confluence delete_or_close` | ❌ Wave B | ⬜ pending |
| 16-C-01 | C | C | LD-16-03 | Repudiation | audit row on every write | unit (wiremock + in-memory SQLite) | `cargo test -p reposix-confluence audit` | ❌ Wave C | ⬜ pending |
| 16-C-02 | C | C | LD-16-02 | SG-03 | sanitize() strips server fields | existing | `cargo test -p reposix-core sanitize` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/reposix-confluence/src/adf.rs` stub with failing tests — covers WRITE-04 (Wave A)
- [ ] Wiremock test stubs for write methods — covers WRITE-01/02/03 (Wave B)
- [ ] Audit log test stub with in-memory SQLite — covers LD-16-03 (Wave C)
- [ ] Add `pulldown-cmark = "0.13"` to `crates/reposix-confluence/Cargo.toml`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live Confluence create/update/delete round-trip | WRITE-01/02/03 | Requires real Atlassian credentials not in CI | Set ATLASSIAN_API_KEY + ATLASSIAN_EMAIL + REPOSIX_CONFLUENCE_TENANT; run `cargo test -p reposix-confluence -- --ignored live` |
| ADF fidelity for complex page types | WRITE-04 | Real-tenant pages may have schema variants | Mount a Confluence space with rich pages, cat each .md file, verify no `[unsupported-adf-node]` fallback markers |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
