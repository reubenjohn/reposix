---
phase: 28
status: complete
waves: 3
commits:
  - d44d312  # Wave 1: reposix-jira crate
  - dfb96da  # Wave 1 SUMMARY
  - b437415  # Wave 2: CLI integration + contract tests
  - bbf94e7  # Wave 2 SUMMARY
  - b9f397d  # Wave 3: ADR-005, docs, CHANGELOG, fmt, tag script
---

# Phase 28 Summary: JIRA Cloud Read-Only Adapter (reposix-jira v0.8.0)

## Objective

Add a `reposix-jira` crate implementing `BackendConnector` against JIRA Cloud REST v3,
wire it into the CLI (`list`, `mount`, `refresh`, `spaces`), and ship Phase 28 with
documentation, a contract test, and a green workspace.

## Requirements Addressed

| Req | Description | Status |
|-----|-------------|--------|
| JIRA-01 | `JiraBackend` implements `BackendConnector` | Done |
| JIRA-02 | `list_issues` with cursor pagination (POST JQL, max 500) | Done |
| JIRA-03 | `get_issue` by numeric ID | Done |
| JIRA-04 | Status+resolution mapping to `IssueStatus` | Done |
| JIRA-05 | `Issue.extensions` populated (`jira_key`, `issue_type`, `priority`, `status_name`, `hierarchy_level`) | Done |

## What Was Built

### Wave 1 — reposix-jira crate (commit d44d312)

- `crates/reposix-jira/Cargo.toml` — new crate with workspace deps
- `crates/reposix-jira/src/adf.rs` — ADF plain-text extractor (5 unit tests + 1 doc-test)
- `crates/reposix-jira/src/lib.rs` — `JiraBackend`, `JiraCreds`, serde structs, 12 wiremock tests
- Workspace `Cargo.toml` — `reposix-jira` added to members

### Wave 2 — CLI integration + contract tests (commit b437415)

- `crates/reposix-jira/tests/contract.rs` — 3-arm contract test (sim + wiremock + live `#[ignore]`)
- `crates/reposix-cli/src/list.rs` — `ListBackend::Jira`, `read_jira_env_from`, 3 unit tests
- `crates/reposix-cli/src/mount.rs` — `--backend jira` pre-spawn allowlist + env-var checks
- `crates/reposix-cli/src/refresh.rs` — Jira arm in `backend_label`, bucket mapping, `fetch_issues`
- `crates/reposix-cli/src/spaces.rs` — Jira arm returns clear not-supported error
- `crates/reposix-cli/Cargo.toml` — `reposix-jira` dependency added

### Wave 3 — Documentation and ship prep (commit b9f397d)

- `docs/decisions/005-jira-issue-mapping.md` — ADR-005 (5 decision areas)
- `docs/reference/jira.md` — user guide
- `CHANGELOG.md` — Phase 28 entries added to `[v0.8.0]`
- `scripts/tag-v0.8.0.sh` — 7-guard annotated tag script
- `cargo fmt --all` — formatting fixes across workspace
- `.planning/STATE.md` — Phase 28 complete

## Security Invariants

| Invariant | Status |
|-----------|--------|
| `JiraCreds` manual `Debug` redacts `api_token` | Confirmed |
| `JiraBackend` manual `Debug` uses `finish_non_exhaustive()` | Confirmed |
| `validate_tenant` blocks SSRF via DNS-label rules | Confirmed + tested |
| `read_jira_env_from` names vars but never echoes values | Confirmed |
| `contract_jira_live` gated by `#[ignore]` + `skip_if_no_env!` | Confirmed |
| All network bytes wrapped in `Tainted::new()` at ingress | Confirmed |

## Test Counts

| Suite | Count | Notes |
|-------|-------|-------|
| `reposix-jira` unit tests (adf.rs + lib.rs) | 18 | always-run |
| `reposix-jira` wiremock tests | 12 | always-run |
| `reposix-jira` doc-test | 1 | always-run |
| `reposix-jira` contract: sim + wiremock | 2 | always-run |
| `reposix-jira` contract: live | 1 | `#[ignore]`-gated |
| `reposix-cli` unit tests (read_jira_env_from) | 3 | always-run |

Full workspace `cargo test --workspace`: 0 failures.

## Green Gauntlet

All three CI checks pass at commit `b9f397d`:
- `cargo test --workspace` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS
- `cargo fmt --all --check` — PASS

Workspace version: 0.8.0
