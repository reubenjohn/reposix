---
phase: 11-confluence-adapter
plan: A
subsystem: backend-adapter
tags: [confluence, rest-v2, read-only, wiremock, SG-01, SG-05]
dependency_graph:
  requires:
    - reposix-core::backend::IssueBackend
    - reposix-core::http::HttpClient
    - reposix-core::taint::Tainted
  provides:
    - ConfluenceReadOnlyBackend
    - ConfluenceCreds (redacting Debug)
    - basic_auth_header (pure fn)
    - parse_next_cursor (pure fn)
    - status_from_confluence (pure fn)
  affects:
    - Cargo.toml (workspace member + base64 dep)
tech_stack:
  added:
    - base64 0.22 (workspace dep)
    - reposix-confluence 0.1.0 (new crate)
  patterns:
    - Cursor-in-body pagination (prepend tenant base to relative _links.next)
    - Basic-auth header via STANDARD base64 alphabet + padding
    - Retry-After-driven rate gate (mirrors GitHub's reset-epoch gate)
    - Space-key → space-id resolver (extra round-trip before first list)
key_files:
  created:
    - crates/reposix-confluence/Cargo.toml
    - crates/reposix-confluence/src/lib.rs
  modified:
    - Cargo.toml
decisions:
  - "Option A flattening: page.id → Issue.id as u64 parse; hierarchy/space/comments lost (documented for ADR-002)"
  - "parse_next_cursor returns the raw relative path; list_issues prepends self.base() to defeat SSRF by construction"
  - "ingest_rate_limit treats any 429 (not just remaining==0) as a throttle signal — stricter than GitHub, matches Atlassian contract"
  - "ConfLinks.next field kept for documentation + allow(dead_code); actual cursor extraction goes through serde_json::Value via parse_next_cursor to keep a narrow test surface"
metrics:
  duration: "~20 minutes"
  completed: 2026-04-13
  tasks_completed: 3
  tests_added: 17
  workspace_tests_total: 186
  lines_lib_rs: 1106
---

# Phase 11 Plan A: reposix-confluence crate core Summary

Shipped the `reposix-confluence` crate — a read-only `IssueBackend` for
Atlassian Confluence Cloud REST v2, structurally isomorphic to
`reposix-github` with the four wire-shape deltas from 11-RESEARCH.md
(cursor-in-body pagination, Basic auth, space-key resolver, `Retry-After`
rate gate). Write methods all return `NotSupported` without emitting HTTP.
Covered by 17 wiremock + pure-fn + threat-model tests (plan required ≥10).

## Tasks

| Task | Name                                                    | Commit   | Files                                                                   |
| ---- | ------------------------------------------------------- | -------- | ----------------------------------------------------------------------- |
| 1    | Scaffold crate + wire into workspace                    | `705225c` | `crates/reposix-confluence/{Cargo.toml,src/lib.rs}`, `Cargo.toml`       |
| 2    | Implement backend + helpers + wiremock unit tests       | `509acc8` | `crates/reposix-confluence/src/lib.rs` (+1103 lines)                    |
| 3    | Workspace-wide green check                              | (no commit — fmt+clippy+tests already clean) | validation only |

## Test Floor

17 unit tests in `crates/reposix-confluence/src/lib.rs` `#[cfg(test)] mod tests`:

**Required by plan (13):**

| # | Name | Covers |
|---|------|--------|
| 1 | `list_resolves_space_key_and_fetches_pages` | Space-key resolver + list of pages |
| 2 | `list_paginates_via_links_next` | Cursor-in-body pagination (relative path prepending) |
| 3 | `get_issue_returns_body_storage_value` | body.storage.value extraction with ?body-format=storage |
| 4 | `get_404_maps_to_not_found` | 404 → Err(Error::Other("not found: …")) |
| 5 | `status_current_maps_to_open` | status="current" → IssueStatus::Open |
| 6 | `status_trashed_maps_to_done` | status="trashed" → IssueStatus::Done |
| 7 | `auth_header_is_basic_with_correct_base64` | Custom `BasicAuthMatches` matcher verifies byte-exact header |
| 8 | `rate_limit_429_retry_after_arms_gate` | 429 + Retry-After → shared gate armed ≤ MAX_RATE_LIMIT_SLEEP |
| 9 | `write_methods_return_not_supported` | create/update/delete short-circuit, no HTTP |
| 10 | `supports_reports_no_features` | Capability matrix + name() == "confluence-readonly" |
| 11 | `parse_next_cursor_extracts_relative_path` | Pure-fn test |
| 12 | `parse_next_cursor_absent_returns_none` | Pure-fn test (null branch) |
| 13 | `basic_auth_header_format` | Pure-fn byte-exact format check |

**Threat-model required (4):**

| # | Name | Covers |
|---|------|--------|
| 14 | `creds_debug_redacts_api_token` | T-11-01: `format!("{:?}", creds)` contains "<redacted>" and NOT the token |
| 15 | `backend_debug_redacts_creds` | T-11-01: Debug on the backend redacts nested creds |
| 16 | `new_rejects_invalid_tenant` | T-11-02: rejects `""`, `tenant.with.dots`, `tenant/slash`, `tenant@at`, leading/trailing hyphen, uppercase, underscore, path-traversal, 64+ chars |
| 17 | `new_accepts_valid_tenants` | T-11-02 negative: `"a"`, `"reuben-john"`, `"tenant1"`, `"1tenant"`, `"a0-b1-c2"` all pass |

## Must-Have Truths

- [x] `cargo build -p reposix-confluence` succeeds on a clean checkout
- [x] `cargo test -p reposix-confluence` runs 17 unit tests, all passing
- [x] `cargo clippy -p reposix-confluence --all-targets -- -D warnings` exits 0
- [x] Every outbound HTTP call goes through `reposix_core::http::HttpClient` (SG-01) — verified by `grep -c "request_with_headers" crates/reposix-confluence/src/lib.rs` = 3 call sites (resolve_space_id, list_issues, get_issue), all using `self.http`
- [x] Every `Issue` returned from public methods was produced by translating a wiremock JSON payload
- [x] Writing methods return `Err(Error::Other("not supported: …"))` without emitting any HTTP (tested with unreachable 127.0.0.1:1)
- [x] Basic-auth header is exactly `Basic base64(email:token)` with STANDARD base64 alphabet + padding (byte-exact test via custom `BasicAuthMatches` wiremock Match)
- [x] Cursor pagination follows `_links.next` relative path, prepending the tenant base URL (wiremock test uses `/wiki/api/v2/spaces/12345/pages?cursor=ABC` relative path and the mock only matches on `cursor=ABC`)

## Workspace Verification (Task 3)

- `cargo fmt --all --check`: clean
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean
- `cargo test --workspace --locked`: **186 passed, 0 failed, ≥180 target met** (was ~169 before this plan per ROADMAP, +17 new = 186)

## Claude's-Discretion Choices

1. **Flat `lib.rs`, not submodules.** Same as `reposix-github`. The crate is a single concern (one adapter) and submodules would fight the copy-shape-verbatim goal. 1106 lines is manageable.
2. **`ConfLinks.next` field kept + `#[allow(dead_code)]`.** The field is documentation — real cursor extraction goes through `serde_json::Value` → `parse_next_cursor`. This keeps the pure helper independently testable without loading the full typed list. The alternative (remove the field) would have silently eroded the schema-documentation surface.
3. **429 triggers gate even if `x-ratelimit-remaining` is absent.** The plan's pseudocode treated the two signals symmetrically ("429 OR remaining==0"). Implemented as strict OR: any 429 arms the gate, using `Retry-After` if present else defaulting to 60s (clamped to `MAX_RATE_LIMIT_SLEEP`). This is slightly stricter than GitHub's adapter but matches 11-RESEARCH.md §Rate-limit Contract.
4. **No separate `#[cfg(test)] use` for `base64::Engine`.** Inlined `use base64::Engine;` where needed (inside helper fn + test BasicAuthMatches) rather than polluting the module-level use list.
5. **`finish_non_exhaustive()` on the backend's Debug.** Clippy flagged the manual Debug as "missing fields" (we deliberately omit `http: Arc<HttpClient>`). Used `finish_non_exhaustive()` to signal intent without adding the noisy `http` field.

## Threat-Model Status

| Threat ID | Status | Evidence |
|-----------|--------|----------|
| T-11-01 (creds leak via Debug) | **mitigated + tested** | Tests `creds_debug_redacts_api_token` and `backend_debug_redacts_creds`. Manual Debug impls on both `ConfluenceCreds` and `ConfluenceReadOnlyBackend`. |
| T-11-02 (SSRF via tenant injection) | **mitigated + tested** | Tests `new_rejects_invalid_tenant` (10 bad inputs) and `new_accepts_valid_tenants` (5 good inputs). DNS-label rules enforced in `validate_tenant`. |
| T-11-03 (Tainted HTML in Issue.body) | accepted for v0.3 | SG-05 wrapping in place (`Tainted::new(page)` at both ingress sites); v0.3 text-surface-only. Documented for ADR-002 (future web renderer must sanitize). |
| T-11-04 (attacker-controlled `_links.next`) | **mitigated by construction** | Relative-path prepending to `self.base()`; even if an attacker supplies a full URL, `HttpClient`'s SG-01 allowlist rejects it. |
| T-11-05 (read-path audit gap) | accepted for v0.3 | Consistent with `reposix-github`; cross-cutting v0.4 concern. |

## Open Questions for ADR-002 (for 11-E author)

None surfaced during execution that weren't already in 11-RESEARCH.md §Open Questions. All 6 OQs (base64 version, body.storage absence, id parse failure, space-not-found-vs-empty, 5s timeout, CI secret gating) are addressed by the shipped code and documented in module comments.

## Deviations from Plan

None. Plan executed exactly as written. Three minor clippy nits surfaced during implementation and were auto-fixed in the same Task 2 commit:

- `manual Debug missing fields` → `finish_non_exhaustive()` on backend struct (Rule 1, within the same task)
- `match_same_arms` on `status_from_confluence` → `#[allow(clippy::match_same_arms)]` with comment explaining why explicit allowlist beats wildcard collapse
- `map_unwrap_or` on `BasicAuthMatches::matches` → `is_some_and` (direct rewrite)

## Self-Check: PASSED

- `crates/reposix-confluence/Cargo.toml` FOUND
- `crates/reposix-confluence/src/lib.rs` FOUND (1106 lines)
- Commit `705225c` (feat(11-A-1)) FOUND in git log
- Commit `509acc8` (feat(11-A-2)) FOUND in git log
- Workspace membership FOUND in Cargo.toml
- `base64 = "0.22"` FOUND in `[workspace.dependencies]`
- All 15 plan success-criteria bash assertions PASS
- 17/17 crate tests PASS
- 186/186 workspace tests PASS (≥180 target)
- `cargo clippy --workspace --all-targets -- -D warnings` PASS
- `cargo fmt --all --check` PASS
