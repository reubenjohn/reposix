---
phase: 11-confluence-adapter
reviewed: 2026-04-13T00:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - crates/reposix-confluence/src/lib.rs
  - crates/reposix-confluence/tests/contract.rs
  - crates/reposix-confluence/Cargo.toml
  - crates/reposix-cli/src/list.rs
  - crates/reposix-cli/src/mount.rs
  - crates/reposix-fuse/src/main.rs
  - crates/reposix-core/src/http.rs (grounding reference)
findings:
  critical: 0
  warning: 3
  high: 0
  medium: 2
  low: 3
  total: 5
status: issues_found
---

# Phase 11: Code Review Report

**Reviewed:** 2026-04-13
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

Phase 11 ships the `reposix-confluence` read-only adapter cleanly overall. The
security-critical items from the scope checklist all pass:

- **SG-01 enforced:** every HTTP call goes through `reposix_core::http::HttpClient`;
  no direct `reqwest::Client` construction found.
- **T-11-01 (creds Debug redaction):** `ConfluenceCreds` has a manual `Debug`
  impl that replaces `api_token` with `"<redacted>"`. The backend struct's own
  `Debug` delegates to the creds' redacted impl. Both are unit-tested.
- **T-11-02 (SSRF via tenant injection):** `validate_tenant` checks length (1..=63),
  no leading/trailing hyphen, and character set `[a-z0-9-]`. Ten adversarial
  inputs are tested including path traversal, scheme-bending, dots, and uppercase.
- **SG-05 (tainted ingress):** Both `list_issues` and `get_issue` wrap decoded
  `ConfPage` structs in `Tainted::new` before calling `translate`.
- **Basic-auth format:** `basic_auth_header` uses `base64::engine::general_purpose::STANDARD`
  (not URL-safe), formats exactly `Basic {base64(email:token)}`, and is
  byte-exact unit-tested with a custom wiremock `Match` impl.
- **Write methods short-circuit:** `create_issue`, `update_issue`, and
  `delete_or_close` all return `Err(Error::Other("not supported: ..."))` before
  any HTTP call. Unit-tested against an unreachable base URL.
- **Error messages:** No token values appear in any error path. `grep token`
  in lib.rs only hits doc comments, the field name, and the redacted Debug impl.

No HIGH severity findings. Two MEDIUM findings around query-parameter injection
and a server-controlled path component in URL construction; three LOW findings
around code quality. All findings have concrete fixes.

---

## Warnings (Medium Severity)

### WR-01: `space_key` interpolated into URL without percent-encoding

**File:** `crates/reposix-confluence/src/lib.rs:471`
**Severity:** Medium
**Issue:** The `space_key` argument (passed directly from the CLI `--project`
flag) is interpolated into a URL query string via `format!` without
percent-encoding:

```rust
let url = format!("{}/wiki/api/v2/spaces?keys={}", self.base(), space_key);
```

A project value like `"PROJ&limit=1"` yields the URL
`.../spaces?keys=PROJ&limit=1`, silently injecting an extra query parameter.
A value like `"A=B"` could corrupt the `keys=` parameter. While the
`REPOSIX_ALLOWED_ORIGINS` gate prevents cross-origin SSRF, the query-string
injection allows a CLI user (or any caller controlling the `project` argument)
to manipulate the Confluence API request in unexpected ways.

**Fix:** Percent-encode the space key using `url::form_urlencoded::byte_serialize` or
the `url` crate's query-pair builder. The cleanest approach uses `Url` directly
to add a typed query parameter:

```rust
use url::Url;

async fn resolve_space_id(&self, space_key: &str) -> Result<String> {
    let mut url = Url::parse(&format!("{}/wiki/api/v2/spaces", self.base()))
        .map_err(|e| Error::Other(format!("bad base url: {e}")))?;
    url.query_pairs_mut().append_pair("keys", space_key);
    // rest unchanged
    let resp = self
        .http
        .request_with_headers(Method::GET, url.as_str(), &header_refs)
        .await?;
```

The `url` crate is already a transitive dependency (via `reposix-core`), so no
new dependency is needed.

---

### WR-02: Server-controlled `space_id` used in URL path without validation

**File:** `crates/reposix-confluence/src/lib.rs:515-519`
**Severity:** Medium
**Issue:** The `space_id` string is returned by the Confluence API, deserialized
as a bare `String` with no character-set validation, and then interpolated
directly into a URL path:

```rust
let space_id = self.resolve_space_id(project).await?;
let first = format!(
    "{}/wiki/api/v2/spaces/{}/pages?limit={}",
    self.base(),
    space_id,   // <-- server-controlled, untrusted
    PAGE_SIZE
);
```

Under the project's threat model, "every byte from the network is tainted."
A malicious Confluence tenant (or a man-in-the-middle on an improperly
configured allowlist) could return `space_id = "12345/../../some-other-endpoint"`,
which `reqwest`'s URL parser would normalize into an unintended Confluence
endpoint path on the same origin. The SG-01 allowlist only checks the origin
(scheme + host + port), not the path, so this would not be blocked.

**Fix:** Validate that the `space_id` contains only numeric digits (which is
what Confluence actually returns for all documented space IDs) before using
it in the URL path:

```rust
// Inside resolve_space_id, after extracting the id:
let id = list.results.into_iter().next().unwrap().id;
if !id.chars().all(|c| c.is_ascii_digit()) {
    return Err(Error::Other(format!(
        "confluence returned non-numeric space id: {:?}", id
    )));
}
Ok(id)
```

---

## Info (Low Severity)

### IN-01: Misleading comment in contract test (wrong status label)

**File:** `crates/reposix-confluence/tests/contract.rs:244`
**Issue:** The comment above the list-pages mock says:

```
// Two pages with different statuses to exercise invariant 5 on both a
// `current`→Open and an `archived`→InProgress mapping.
```

But `"archived"` maps to `IssueStatus::Done`, not `InProgress` (per the
spec in `lib.rs:17` and the implementation in `status_from_confluence`).
The actual test logic is correct (invariant 5 only checks that the status
is a valid enum variant), so no behavior is affected — but the comment
will mislead future readers.

**Fix:** Change the comment to:
```
// `current`→Open and an `archived`→Done mapping.
```

---

### IN-02: Unused `rusqlite` dev-dependency in `Cargo.toml`

**File:** `crates/reposix-confluence/Cargo.toml:30`
**Issue:** `rusqlite = { workspace = true }` is listed as a dev-dependency
but is not imported or used in either `src/lib.rs` or `tests/contract.rs`.

**Fix:** Remove the line:
```toml
rusqlite = { workspace = true }
```

---

### IN-03: Unused `thiserror` production dependency in `Cargo.toml`

**File:** `crates/reposix-confluence/Cargo.toml:21`
**Issue:** `thiserror = { workspace = true }` is listed as a production
dependency but is never referenced in `src/lib.rs`. The crate delegates all
error construction to `reposix_core::Error` via `Error::Other(...)` and never
defines a `#[derive(thiserror::Error)]` type of its own.

**Fix:** Remove the line:
```toml
thiserror = { workspace = true }
```

---

## Security Checklist (for sign-off record)

| Item | Status | Evidence |
|------|--------|---------|
| SG-01: all HTTP via `HttpClient` | PASS | No `reqwest::Client` direct use; `disallowed-methods` clippy lint in workspace |
| SG-05: tainted ingress wrapping | PASS | `Tainted::new(page)` at both `list_issues:559` and `get_issue:613` |
| T-11-01: creds Debug redacts token | PASS | Manual impl at `lib.rs:118-125`; unit-tested at `lib.rs:1035` |
| T-11-02: tenant validation | PASS | `validate_tenant` at `lib.rs:341`; 10 adversarial inputs tested |
| Basic-auth format (email:token, STANDARD b64) | PASS | `basic_auth_header` at `lib.rs:242`; custom wiremock `Match` test |
| Error messages do not leak token | PASS | No token value interpolated in any `Err(...)` path |
| Write methods short-circuit before HTTP | PASS | All three return `Err` immediately; unit-tested against port 1 |
| `space_key` URL-encoded | FAIL | See WR-01 |
| `space_id` path-validated | FAIL | See WR-02 |

---

_Reviewed: 2026-04-13_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
