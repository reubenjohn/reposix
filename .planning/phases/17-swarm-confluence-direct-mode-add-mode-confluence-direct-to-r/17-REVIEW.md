---
phase: 17
status: findings
severity: low
finding_count: 6
---

# Phase 17 Code Review

## Summary

Phase 17 adds `--mode confluence-direct` to `reposix-swarm`, wiring `ConfluenceDirectWorkload` through the existing swarm driver. The implementation is structurally sound: it correctly mirrors `SimDirectWorkload` (list + 3×get, no writes), uses per-client `ConfluenceBackend` instances (each with its own `rate_limit_gate`), and the wiremock CI test exercises the full round-trip against stubbed Confluence endpoints. Credential redaction is enforced by the existing `ConfluenceCreds` manual `Debug` impl; the `--api-token` → `ATLASSIAN_API_KEY` env fallback is correctly wired via clap's `env` attribute.

No critical security vulnerabilities or correctness bugs were found. There is one low-severity security finding (email exposed on CLI), two test coverage gaps, two code quality issues (duplicated helper, missing assertion), and one doc-convention gap.

---

## Findings

### [LOW] `--email` has no env-var fallback — exposes address in process listing (main.rs:66-67)

**File:** `crates/reposix-swarm/src/main.rs:66`

`--api-token` is backed by `env = "ATLASSIAN_API_KEY"`, meaning users can supply it via environment variable and keep it off the command line. `--email` has no corresponding `env = "ATLASSIAN_EMAIL"` attribute, so users must pass their Atlassian email as a CLI argument. This makes the email address visible in `/proc/<pid>/cmdline`, `ps auxww`, and shell history on every invocation. The project convention (`.env.example` line 38, `confluence_real_tenant.rs` line 34) already documents `ATLASSIAN_EMAIL` as the env var for this value.

**Impact:** Email address leakage in shared-host environments; inconsistent with the documented env-var-first credential pattern; frustrating UX (users expect parity with `--api-token`).

**Fix:** Add `env = "ATLASSIAN_EMAIL"` to the `--email` argument:

```rust
/// Atlassian account email (required for `confluence-direct`). Falls back
/// to the `ATLASSIAN_EMAIL` env var.
#[arg(long, env = "ATLASSIAN_EMAIL")]
email: Option<String>,
```

---

### [LOW] Wiremock test never asserts `get` row in the summary (mini_e2e.rs:280-283)

**File:** `crates/reposix-swarm/tests/mini_e2e.rs:280`

`confluence_direct_3_clients_5s` asserts `markdown.contains("| list ")` but never asserts `markdown.contains("| get ")`. The workload issues 3 `get_issue` calls per cycle. If the `get_issue` path silently stopped recording metrics (or if the page-get stub was broken), the test would still pass as long as at least one `list` op completed. The `total_ops >= 3` floor is too permissive — 3 list-only ops would satisfy it even with zero gets.

**Fix:**

```rust
assert!(
    markdown.contains("| get "),
    "summary missing get row — get_issue calls not being recorded:\n{markdown}"
);
```

Add this assertion after the `| list ` check.

---

### [LOW] Wiremock page-get stub ignores the requested id — masks id-routing bugs (mini_e2e.rs:243-249)

**File:** `crates/reposix-swarm/tests/mini_e2e.rs:243`

The page-get stub uses `path_regex(r"^/wiki/api/v2/pages/\d+$")` but always responds with `sample_page("10001", "Page 1")` regardless of which page id was requested. This is fine for load testing, but it means a bug in `ConfluenceBackend::get_issue` that passes the wrong id in the URL (e.g., always using id `0`) would go undetected — the stub would still respond 200 and the test would pass.

**Impact:** Reduced confidence in id-routing correctness. Does not cause a false positive, but creates a false negative blind spot.

**Fix (optional — acceptable to defer):** Add a second stub for a specific id (e.g., `path("/wiki/api/v2/pages/10002")`) that returns `sample_page("10002", "Page 2")`, and assert that `get_issue` for id `10002` returns a page with title `"Page 2"`. Alternatively, document the intentional limitation with a `// NOTE:` comment at the stub declaration.

---

### [INFO] `elapsed_us` helper duplicated across three modules (confluence_direct.rs:113, sim_direct.rs:157, fuse_mode.rs:146)

**Files:** `crates/reposix-swarm/src/confluence_direct.rs:113`, `crates/reposix-swarm/src/sim_direct.rs:157`, `crates/reposix-swarm/src/fuse_mode.rs:146`

The `elapsed_us(start: Instant) -> u64` helper is copy-pasted verbatim into all three workload modules. Any future change (e.g., switching to milliseconds, adding overflow logging) must be applied in three places.

**Fix:** Move to `crates/reposix-swarm/src/metrics.rs` or a new `crates/reposix-swarm/src/util.rs` as a `pub(crate)` function, then replace the three copies with a use import. This is a one-line change per callsite.

---

### [INFO] `ConfluenceDirectWorkload::ids` uses a comment, not a doc comment (confluence_direct.rs:29)

**File:** `crates/reposix-swarm/src/confluence_direct.rs:29`

The `ids` field has a `// Cached ids…` comment rather than a `///` doc comment. With `#![warn(missing_docs)]` enabled in `lib.rs`, private fields are exempt, but the comment style is inconsistent with the rest of the codebase where all struct fields use `///` docs. The struct is `pub`, and the comment provides useful information that should be visible in `rustdoc`.

**Fix:** Change `// Cached ids from the most recent…` to `/// Cached ids from the most recent…`.

---

### [INFO] 5-second wiremock test may inflate CI time on slow runners (mini_e2e.rs:261)

**File:** `crates/reposix-swarm/tests/mini_e2e.rs:261`

`confluence_direct_3_clients_5s` runs the swarm for a full 5 seconds against wiremock stubs with zero network latency. The `sim_mini_e2e` test uses 1.5 seconds. Since the stubs respond immediately, 500–750 ms would accumulate far more than the `total_ops >= 3` threshold while keeping CI fast. On a 2-core runner, this test alone adds ~5s to the test suite wall time.

**Fix (low priority):** Reduce to `Duration::from_millis(750)` and rename the test to `confluence_direct_3_clients_750ms`. The `total_ops >= 3` floor remains valid; the 5s name in the constant is misleading once changed.

---

## Verdict

**FLAG** — one low-severity security finding (`--email` missing env fallback) and two test correctness gaps should be addressed before this phase is considered fully closed. The core logic, credential redaction, and allowlist compliance are correct. No blocker.
