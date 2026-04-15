---
phase: 21
status: findings
severity: medium
finding_count: 5
reviewed_files: 12
files_reviewed_list:
  - .github/workflows/ci.yml
  - crates/reposix-cli/src/list.rs
  - crates/reposix-cli/src/main.rs
  - crates/reposix-cli/tests/no_truncate.rs
  - crates/reposix-confluence/src/lib.rs
  - crates/reposix-fuse/tests/nested_layout.rs
  - crates/reposix-fuse/tests/sim_death_no_hang.rs
  - crates/reposix-swarm/src/contention.rs
  - crates/reposix-swarm/src/lib.rs
  - crates/reposix-swarm/src/main.rs
  - crates/reposix-swarm/tests/chaos_audit.rs
  - crates/reposix-swarm/tests/contention_e2e.rs
---

# Phase 21: Code Review Report

**Reviewed:** 2026-04-15T00:00:00Z
**Depth:** standard
**Files Reviewed:** 12
**Status:** issues_found

## Summary

Phase 21 delivers four well-scoped hardening units: the `ContentionWorkload` (If-Match / 409
determinism proof), `list_issues_strict` / `--no-truncate` (SG-05 taint-escape cut), tenant
URL redaction via `redact_url` (OP-7 HARD-05), and the `chaos_kill9_no_torn_rows` WAL-durability
test. The CI YAML is structurally correct. The contention logic, pagination cap, rate-limit gate,
space-id numeric validation, and FUSE teardown helpers are all sound.

Two medium-severity findings require attention before this phase is declared done:

1. **409 response body echoed untruncated into error message** — the one place `redact_url` is
   conspicuously absent, and the one place tainted server bytes flow into an error string with no
   size bound.
2. **`no_truncate` integration test does not reach the mock** — the flag-plumbing test cannot
   assert the actual `"strict mode"` message because the CLI builds the real Atlassian URL from the
   tenant env var and never redirects to the wiremock server. The test asserts non-zero exit for the
   wrong reason (connection refused, not cap exceeded), giving false confidence.

Three info-level observations round out the review.

---

## Critical Issues

None.

---

## Warnings

### WR-01: 409 response body echoed unredacted and unbounded into error string

**File:** `crates/reposix-confluence/src/lib.rs:1063-1067`

**Issue:** The `CONFLICT` arm of `update_issue` formats the raw server response bytes directly into
the error message without truncation or redaction:

```rust
if status == StatusCode::CONFLICT {
    return Err(Error::Other(format!(
        "version mismatch: {}",
        String::from_utf8_lossy(&bytes)
    )));
}
```

Every other error path in the same function uses `redact_url(&url)` to strip the tenant hostname
and omits raw body bytes (or truncates them). The CONFLICT path is the exception. A 409 body from
Confluence includes the page title, space name, and a human-readable message — all
tenant/user-controlled strings. This means:

- The tenant URL (hostname including the customer's subdomain) can appear in the error string if
  Confluence includes it in the 409 body (it sometimes does for redirect hints).
- An adversarially crafted page title (attacker-influenced data, per the lethal-trifecta model) can
  inject arbitrary-length content into the error string that propagates to tracing spans, CLI
  stderr, and the swarm markdown report.
- The lack of a length bound is a DoS amplifier: a 429 KB 409 response body becomes a 429 KB
  tracing span.

The non-2xx fallthrough immediately below does include `redact_url` and the raw body, which is also
inconsistent and also worth tightening, but the CONFLICT path is the primary concern here because it
is the new hot path added in Phase 21.

**Fix:** Mirror the pattern used by every other error path — strip URL hostname and truncate body:

```rust
if status == StatusCode::CONFLICT {
    let body_preview: String = String::from_utf8_lossy(&bytes).chars().take(256).collect();
    return Err(Error::Other(format!(
        "confluence version conflict for PUT {}: {body_preview}",
        redact_url(&url),
    )));
}
```

The same truncation should be applied to the generic `!status.is_success()` fallthrough at lines
1069-1074 for consistency, though that path was pre-existing and is lower priority.

---

### WR-02: `no_truncate` integration test never reaches the mock; asserts the wrong failure mode

**File:** `crates/reposix-cli/tests/no_truncate.rs:96-148`

**Issue:** The test `no_truncate_errors_when_space_exceeds_cap` correctly mounts a wiremock server
and sets `REPOSIX_CONFLUENCE_TENANT=test-tenant`, but `ConfluenceBackend::new` constructs the URL
as `https://test-tenant.atlassian.net` — it ignores `REPOSIX_ALLOWED_ORIGINS` for URL selection
and uses the tenant env var directly. The wiremock server is never reached. The assertion

```rust
assert!(
    !output.status.success(),
    "reposix list --no-truncate must exit non-zero when confluence is unreachable or cap exceeded"
);
```

passes because the connection to `test-tenant.atlassian.net` is refused (or blocked by the
allowlist), not because the 500-page cap fired. The test comment acknowledges this limitation but
promotes it as a complete test of the `--no-truncate` flag. In practice it only tests that the
`--no-truncate` flag is accepted by the CLI argument parser and that any failure exits non-zero —
it does not exercise `list_issues_strict` at all from the CLI boundary.

This creates a coverage gap: if `list::run` were changed to call `list_issues` instead of
`list_issues_strict` when `no_truncate` is true (i.e., the wire-up were inverted), this test would
still pass.

The unit tests in `reposix-confluence` do exercise `list_issues_strict` correctly; the issue is
that the CLI-level integration test does not close the wire-up gap.

**Fix (two options, pick one):**

Option A — Add a `REPOSIX_CONFLUENCE_BASE_URL_OVERRIDE` env var (or extend `ConfluenceBackend::new`
to check for it) that lets the test redirect the backend to the wiremock URL. This is the minimal
invasive change.

Option B — Restructure the test to call `list::run(...)` directly via the library API rather than
spawning the binary, so the `ConfluenceBackend` can be built with `new_with_base_url`. The
`no_truncate_flag_appears_in_list_help` test already validates CLI flag presence; the
`no_truncate_errors_when_space_exceeds_cap` test should validate logic, not argparse.

Until one of these is implemented, the test should be renamed or its doc-comment updated to clearly
state it is an argparse smoke test, not an integration test for the cap logic:

```rust
/// Smoke test: `--no-truncate` is accepted by the CLI argument parser and
/// causes non-zero exit (for any reason). For cap-logic coverage see the
/// unit tests in `reposix-confluence::tests` (list_strict_errors_at_cap).
```

---

## Info

### IN-01: Chaos test uses fixed port 7979 — concurrent `--ignored` runs will collide

**File:** `crates/reposix-swarm/tests/chaos_audit.rs:41`

**Issue:** `SIM_BIND` is hardcoded to `127.0.0.1:7979`. The `pkill -f reposix-sim` best-effort
teardown at line 164 can kill a legitimate simulator process from a parallel test run or from a
developer's local session. If two test suites run the `--ignored` suite concurrently (e.g., two
CI runners on the same host), both `SIGKILL` each other's sims and the test becomes flaky.

This is a known pattern risk called out in RESEARCH.md §Pitfall 2 for in-process tests. The
subprocess case compounds it because `pkill -f` matches on the full command line, not PID.

**Fix:** Bind to port 0 (ephemeral), capture the assigned port, and use it for both health-check
and load URLs. This requires the simulator to print or expose its bound address — check if
`reposix-sim` already supports a `--print-addr` flag or similar. Alternatively, pick a port from
a per-test `NamedTempFile`-seeded value to reduce the collision window.

---

### IN-02: `ConfLinks` struct is dead code — `#[allow(dead_code)]` masks a structural oddity

**File:** `crates/reposix-confluence/src/lib.rs:214-223`

**Issue:** `ConfLinks.next` is annotated `#[allow(dead_code)]` because cursor extraction goes
through the `parse_next_cursor` JSON-Value helper rather than the typed struct. The struct is
deserialized (so it isn't truly unreachable) but its fields are never read after deserialization,
and `list.links` is explicitly discarded at line 615:

```rust
let _ = list.links;
```

The field exists purely for documentation. The allow-lint comment documents intent, but the result
is a type that is deserialized and immediately thrown away while the `#[allow(dead_code)]`
suppresses the compiler's attempt to tell you the same thing. This is not a bug, but it is a
maintenance hazard: a future refactor that removes `parse_next_cursor` and reads `list.links.next`
directly would need to remove the `allow` — and it's easy to forget.

**Fix:** Remove `ConfLinks` and the typed `links` field from `ConfPageList`, since the cursor is
fully handled by `parse_next_cursor`. Alternatively, use the struct directly in
`list_issues_impl` and remove `parse_next_cursor`. Either path removes the allow-lint and the dead
`let _ = list.links`.

---

### IN-03: `contention_e2e` invariant 1 asserts `markdown.contains("| patch ")` — fragile string match

**File:** `crates/reposix-swarm/tests/contention_e2e.rs:119-122`

**Issue:** The assertion that at least one PATCH op was recorded relies on the exact Markdown table
row format:

```rust
assert!(
    markdown.contains("| patch "),
    "expected patch op row in summary:\n{markdown}"
);
```

If the summary renderer changes column formatting (e.g., adds padding, renames the op to
`"Patch"`, or changes the column separator), this assertion silently becomes vacuously true or
false. Invariant 2 (`"| Conflict"`) has the same fragility.

This is a test quality issue, not a correctness bug in the production code. The swarm driver and
contention workload are correct.

**Fix:** Return a structured summary type from `run_swarm` alongside the markdown string, so tests
can assert on counts directly rather than parsing rendered text. This is a larger refactor; as a
minimal fix, add a comment noting that these string assertions are format-coupled and must be
updated if `metrics.rs` render format changes.

---

_Reviewed: 2026-04-15T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
