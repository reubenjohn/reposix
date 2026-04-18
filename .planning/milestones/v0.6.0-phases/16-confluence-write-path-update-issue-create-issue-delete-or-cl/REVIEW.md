---
phase: 16-confluence-write-path
reviewed: 2026-04-14T00:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - crates/reposix-confluence/src/lib.rs
  - crates/reposix-confluence/src/adf.rs
  - crates/reposix-confluence/tests/roundtrip.rs
  - crates/reposix-core/src/http.rs
  - crates/reposix-cli/src/list.rs
  - crates/reposix-fuse/src/main.rs
findings:
  critical: 0
  warning: 4
  major: 4
  minor: 4
  total: 8
status: issues_found
---

# Phase 16: Code Review Report

**Reviewed:** 2026-04-14
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Phase 16 delivers the Confluence write path (`create_issue`, `update_issue`,
`delete_or_close`), the ADF↔Markdown converter (`adf.rs`), audit log wiring,
and a round-trip integration test. The threat-model controls are broadly
correct: `Untainted<Issue>` params are used for write methods, `audit_write`
stores only title (not body), and `request_summary` is title-truncated-to-256.
The ADF recursion cap (`MAX_ADF_DEPTH = 32`) is present and tested.

Four **Warning** findings require attention before Phase 17 ships. None are
security holes, but two affect correctness (HTTP timeout, audit-on-error order)
and two affect correctness of error classification. Four **Info** items are
quality notes that can be addressed opportunistically.

---

## Warnings

### WR-01: HTTP client timeout is 5 s — too short for write operations under load

**File:** `crates/reposix-core/src/http.rs:50` and `crates/reposix-confluence/src/lib.rs:474`

**Issue:** `ClientOpts::default()` sets a 5-second total timeout. For `POST`
and `PUT` to Confluence, the server-side page rendering pipeline can take 10–30
seconds on large pages or under load. With the current timeout, any write
against a real tenant will silently `Err` with a reqwest timeout — the call
looks like a transport failure, not a content or credential error. The simulator
and wiremock tests pass because they respond instantly; this gap will surface
only in production.

**Fix:** Add a `write_timeout` field to `ClientOpts` (default 30 s) and
construct a separate `HttpClient` for write calls in `ConfluenceBackend`, or
override the timeout per-request once reqwest supports per-request timeouts.
Short-term: bump the default to 30 s globally (the read path is unaffected
since reads are bounded by `MAX_ISSUES_PER_LIST`).

```rust
// crates/reposix-core/src/http.rs
impl Default for ClientOpts {
    fn default() -> Self {
        Self {
            total_timeout: Duration::from_secs(30), // was 5 s; write calls need headroom
            user_agent: Some(concat!("reposix/", env!("CARGO_PKG_VERSION")).to_owned()),
        }
    }
}
```

---

### WR-02: `audit_write` is called before error-path return in `update_issue`; 409 audit row records stale response bytes

**File:** `crates/reposix-confluence/src/lib.rs:983-998`

**Issue:** The current ordering is:

```rust
let bytes = resp.bytes().await?;        // consumes body
let status_u16 = status.as_u16();
let req_summary = …;
self.audit_write("PUT", &audit_path, status_u16, &req_summary, &bytes);  // line 989
if status == StatusCode::CONFLICT {
    return Err(Error::Other(format!("version mismatch: {}", …bytes)));    // line 994
}
```

This is correct: the audit row IS written before the error return, which is
the intended behaviour (T-16-C-01 — failed writes must be audited). The
existing test `audit_records_failed_writes` confirms it. However the test for
this in the integration `roundtrip.rs` only counts POST rows, not PUT rows.
There is no integration-level test that a failed PUT (409) is also audited in
`roundtrip.rs` — only in unit tests. That gap is acceptable for v0.6 but
should be noted.

More importantly: on a 409, `String::from_utf8_lossy(&bytes)` is called at
line 995 after the `bytes` variable was already moved into `audit_write` at
line 989. **This will fail to compile unless `bytes` is `Clone` or `&bytes`
was passed.** Checking: `audit_write` takes `&[u8]`, so `&bytes` is a borrow
— `bytes` itself is not moved. The code is correct, but the ordering risks
a future refactor that passes `bytes` by value. Add a comment to lock the
borrow contract.

**Fix:** Add a comment at the `audit_write` call site:

```rust
// `bytes` is borrowed (&[u8]) — not moved — so it remains available for the
// error messages below. Do not change to a by-value move without updating
// all uses below this line.
self.audit_write("PUT", &audit_path, status_u16, &req_summary, &bytes);
```

---

### WR-03: `render_node` signature has dead parameters `ordered` and `list_depth` that are never used at the top level — causes clippy::pedantic lint (`unused_variables`)

**File:** `crates/reposix-confluence/src/adf.rs:130`

**Issue:** `render_node(node, out, list_depth, ordered, depth)` — the
`list_depth` and `ordered` parameters exist in the signature but for the
top-level dispatch match (paragraph, heading, codeBlock, hardBreak, text)
they are forwarded only to sub-helpers. The `"text"` arm calls
`render_text_node(node)` which ignores both. The `"listItem"` arm passes
`ordered` through but the `_ordered` parameter name in `render_list_item`
shows it is still unused. The Rust compiler will emit an `unused_variables`
warning (or under `clippy::pedantic`, a lint) for `ordered` in
`render_list_item`/`render_node`. The CLAUDE.md requires `#![warn(clippy::pedantic)]`
with allow-list per lint, not blanket suppression. The `_ordered` underscore
prefix suppresses the compiler warning but is evidence of a design smell: the
parameter participates in no logic.

**Fix:** Remove the `ordered` parameter from `render_node` and
`render_list_item` entirely (it was carried forward from an earlier design
where the list type was tracked at every level, but `render_nested_lists` now
re-reads the node type directly):

```rust
// Before:
fn render_node(node: &Value, out: &mut String, list_depth: usize, ordered: bool, depth: usize)
// After:
fn render_node(node: &Value, out: &mut String, list_depth: usize, depth: usize)
```

And update all call sites to drop the `ordered` argument.

---

### WR-04: No request-body verification in `update_issue` wiremock test (B6.1) — version increment is not confirmed wire-level

**File:** `crates/reposix-confluence/src/lib.rs:1867-1891`

**Issue:** Test `update_issue_sends_put_with_version` verifies the *response*
shape but does not verify that the PUT request body contained
`"version": {"number": 43}`. The test merely checks `result.version == 43`,
which is the value from the *mocked response*, not proof that the adapter sent
`version.number = current + 1 = 43`. If `update_issue` accidentally sent
`version.number = 0` but the mock always responds with `version: 43`, this
test would still pass.

A `VersionNumberMatches` wiremock custom matcher (analogous to `ParentIdMatches`
at line 2011) is needed to confirm the wire-level PUT body. This is the most
critical test-quality gap because version mismatch detection is the primary
safety property of `StrongVersioning`.

**Fix:**

```rust
struct VersionNumberMatches(u64);
impl wiremock::Match for VersionNumberMatches {
    fn matches(&self, request: &Request) -> bool {
        let Ok(body) = serde_json::from_slice::<serde_json::Value>(&request.body) else {
            return false;
        };
        body.get("version")
            .and_then(|v| v.get("number"))
            .and_then(|n| n.as_u64())
            .is_some_and(|n| n == self.0)
    }
}

// In update_issue_sends_put_with_version:
Mock::given(method("PUT"))
    .and(path("/wiki/api/v2/pages/99"))
    .and(VersionNumberMatches(43))   // ← enforce wire-level body
    .respond_with(…)
    .mount(&server)
    .await;
```

---

## Info

### IN-01: `render_inline_content` silently falls through when node is not a `text` but has no `content` children

**File:** `crates/reposix-confluence/src/adf.rs:313-320`

**Issue:** The fallback at lines 313–319 checks `if out.is_empty()` and then
tries `node.get("text")`. This runs when a non-text node (e.g. a paragraph
with no children) is passed to `render_inline_content`. The guard `let _ =
depth` at line 315 is a tell: `depth` is accepted but unused in this branch.
The current logic is safe but slightly confusing. A future change that adds a
`link` ADF node type would need to add a new arm to the match at line 301
rather than relying on this fallback.

**Fix:** No behaviour change needed, but add a doc comment to `render_inline_content` clarifying the fallback is an emergency clause for malformed inputs, not the normal path for new node types.

---

### IN-02: `markdown_to_storage` does not sanitize raw HTML embedded in Markdown — documented but missing a test for the known-risky case

**File:** `crates/reposix-confluence/src/adf.rs:34-36`

**Issue:** The module doc explicitly acknowledges that raw HTML embedded in
Markdown is passed through to Confluence's server-side parser (T-16-A-04,
Phase 21+ deferred). This is a documented, intentional decision. However,
there is no test that demonstrates the Confluence server-side gate is the
actual last line of defence — the current test suite only covers well-formed
Markdown. A `TODO` comment or a failing/ignored test labeled `#[ignore = "T-16-A-04: relies on Confluence server-side HTML strip"]` would make the
deferred obligation explicit and prevent someone from closing Phase 21 without
revisiting this.

---

### IN-03: `audit_write` stores response body SHA-256 prefix (16 hex chars) — prefix length constraint is not enforced at the call site

**File:** `crates/reposix-confluence/src/lib.rs:697`

**Issue:** `response_summary` is formatted as `"{status}:{sha_hex[..16]}"`.
The 16-char slice panics if `sha_hex.len() < 16`, but SHA-256 always produces
64 hex chars so this is impossible in practice. However the format is
consistent with a 20-char total (`"200:" + 16`) and the test at line 2463
hard-codes `response_summary.len() == 4 + 16`. If the status code is ever 3
digits + colon this is still `4 + 16 = 20` — fine. For a 3xx code it would be
`"302:" + 16` = 20, also fine. No action needed, but a debug-assert would
make the assumption explicit:

```rust
debug_assert!(sha_hex.len() >= 16, "SHA-256 output must be >= 16 hex chars");
```

---

### IN-04: `fetch_current_version` passes empty string as `project` to `get_issue`

**File:** `crates/reposix-confluence/src/lib.rs:667`

**Issue:** `self.get_issue("", id).await` — `get_issue` ignores `_project`
(`_project: &str`), so this works. But passing `""` is surprising to future
readers and inconsistent with every other `get_issue` call in the test suite
that passes `"REPOSIX"`. If the `_project` parameter is ever promoted to be
meaningful in a future phase, this silent empty string will cause a hard-to-
diagnose issue.

**Fix:** Pass the actual `_project` value through to `fetch_current_version`:

```rust
async fn fetch_current_version(&self, project: &str, id: IssueId) -> Result<u64> {
    let issue = self.get_issue(project, id).await?;
    Ok(issue.version)
}
```

And update `update_issue` to call `self.fetch_current_version(_project, id)`.

---

## Verdict

**No Critical findings.** The security properties are correct: writes use
`Untainted<Issue>`, audit rows never store body content, and the ADF recursion
cap prevents stack overflow. The `render_node` unused-parameter issue (WR-03)
is the most likely to fail CI under `clippy::pedantic -D warnings` and should
be fixed before Phase 17 begins. WR-04 (missing wire-level version body check)
is the most important test-quality gap and should be addressed in the same PR.

**Recommended before Phase 17 starts:**
- WR-03: remove dead `ordered` parameter from `render_node`/`render_list_item`
- WR-04: add `VersionNumberMatches` body-check to `update_issue_sends_put_with_version`

**Can be deferred to Phase 17 or later:**
- WR-01: HTTP timeout bump (important for production, invisible in CI)
- WR-02: add borrow-contract comment to `audit_write` call site
- IN-01 through IN-04: informational quality notes

---

_Reviewed: 2026-04-14_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
