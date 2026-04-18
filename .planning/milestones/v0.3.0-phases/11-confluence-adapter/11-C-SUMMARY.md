---
phase: 11-confluence-adapter
plan: C
subsystem: contract-tests
tags: [contract-test, confluence, wiremock, live-atlassian, SG-01]
dependency_graph:
  requires:
    - reposix_confluence::ConfluenceReadOnlyBackend (from 11-A)
    - reposix_confluence::ConfluenceCreds (from 11-A)
    - reposix_core::backend::IssueBackend
    - reposix_core::backend::sim::SimBackend
  provides:
    - crates/reposix-confluence/tests/contract.rs::assert_contract (shared invariant helper)
    - crates/reposix-confluence/tests/contract.rs::contract_sim (always runs)
    - crates/reposix-confluence/tests/contract.rs::contract_confluence_wiremock (always runs)
    - crates/reposix-confluence/tests/contract.rs::contract_confluence_live (#[ignore]-gated)
    - skip_if_no_env! macro (reusable pattern for future live-wire tests)
  affects: []
tech-stack:
  added: []
  patterns:
    - "skip_if_no_env!(\"VAR1\", \"VAR2\", ...) macro — prints names-only SKIP line and early-returns from the test when any var is empty"
    - "Known-id-by-list-first for live tests — list_issues, assert non-empty, take issues[0].id as known_issue_id (real Confluence spaces have no canonical 'issue 1')"
    - "Mock-ordering discipline: u64::MAX 404 mounted BEFORE pages/1 200 because wiremock matches most-recently-mounted-first"
key-files:
  created:
    - crates/reposix-confluence/tests/contract.rs
  modified: []
decisions:
  - "skip_if_no_env! printed VARIABLE NAMES only, never values — per T-11C-01 mitigation. Test output is safe to paste into bug reports."
  - "Known-id in live test derived from list_issues()[0].id rather than hardcoded. The double-list (once in setup, once inside assert_contract) costs 1 extra Atlassian request per run, well under the 1000 req/min soft cap, and makes the test portable across tenants."
  - "u64::MAX 404 mock mounted before pages/1 200 mock. Wiremock matches most-recently-mounted-first, and the explicit ordering comment in the test body makes the dependency visible to future maintainers."
  - "Dev-deps unchanged from 11-A (wiremock, tokio macros+rt-multi-thread, reposix-sim, tempfile, rusqlite) — serde_json is already a regular dep of the crate, so the test file's `use serde_json::json` resolves without modifying Cargo.toml."
metrics:
  duration: "~10 minutes"
  completed: 2026-04-13
  tasks_completed: 2
  tests_added: 2
  workspace_tests_before: 189
  workspace_tests_after: 191
  commits: 1
---

# Phase 11 Plan C: contract test Summary

Shipped `crates/reposix-confluence/tests/contract.rs` — the proof that
`ConfluenceReadOnlyBackend` upholds the same 5 `IssueBackend` invariants
as `SimBackend` and `GithubReadOnlyBackend`. The file is a direct port
of `reposix-github/tests/contract.rs` with the `assert_contract` helper
copied verbatim, a `skip_if_no_env!` macro added for readable live-test
env guards, and a wiremock-backed Confluence arm that exercises the full
`list_issues → get_issue → get_issue(u64::MAX)` sequence through the
trait seam (stronger than the 17 private-helper unit tests in 11-A).

Live verified against real `reuben-john.atlassian.net` space `REPOSIX`:
`contract_confluence_live` passed in 1.32s with the space's 4 pages.

## Tasks

| Task | Name                                                                                 | Commit    | Files                                            |
| ---- | ------------------------------------------------------------------------------------ | --------- | ------------------------------------------------ |
| 1    | Port contract-test file with sim + wiremock-confluence + live-confluence arms        | `868703e` | `crates/reposix-confluence/tests/contract.rs` (new, 380 lines) |
| 2    | Verify full workspace stays green (fmt, clippy, all tests, smoke)                    | (validation only — no commit) | — |

## Test Count

| Scope                                             | Before | After | Δ      |
| ------------------------------------------------- | ------ | ----- | ------ |
| `cargo test -p reposix-confluence` unit tests     | 17     | 17    | +0     |
| `cargo test -p reposix-confluence` integration    | 0      | 2     | +2     |
| `cargo test --workspace` total (default run)      | 189    | 191   | +2     |
| `cargo test -p reposix-confluence -- --ignored`   | 0      | 1     | +1 (gated) |

The `--ignored` test is not counted in the workspace total because it
does not run without the `--ignored` flag.

## Verification Results

### Task 1 verify — required commands

1. Always-on arms pass:

    ```
    $ cargo test -p reposix-confluence --locked
    running 17 tests
    ... (17/17 unit tests pass) ...
    test result: ok. 17 passed; 0 failed; 0 ignored
    running 3 tests
    test contract_confluence_live ... ignored
    test contract_confluence_wiremock ... ok
    test contract_sim ... ok
    test result: ok. 2 passed; 0 failed; 1 ignored
    ```

2. Live test skips cleanly with env unset:

    ```
    $ env -u ATLASSIAN_API_KEY -u ATLASSIAN_EMAIL \
          -u REPOSIX_CONFLUENCE_TENANT -u REPOSIX_CONFLUENCE_SPACE \
          cargo test -p reposix-confluence --locked --test contract \
              -- --ignored --nocapture
    running 1 test
    SKIP: env vars unset: ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, REPOSIX_CONFLUENCE_TENANT, REPOSIX_CONFLUENCE_SPACE
    test contract_confluence_live ... ok
    test result: ok. 1 passed; 0 failed; 0 ignored
    ```

### Task 2 verify — workspace gates

| Check                                                                       | Result                            |
| --------------------------------------------------------------------------- | --------------------------------- |
| `cargo fmt --all --check`                                                   | clean                             |
| `cargo clippy --workspace --all-targets --locked -- -D warnings`            | clean                             |
| `cargo test --workspace --locked`                                           | **191 passed, 0 failed** (+2 new) |
| `PATH="$PWD/target/release:$PATH" bash scripts/demos/smoke.sh`              | 4/4 PASS                          |

### Live verification — WORKS

With `.env` loaded + `REPOSIX_ALLOWED_ORIGINS` set + `REPOSIX_CONFLUENCE_SPACE=REPOSIX`:

```
$ set -a; source .env; set +a
$ export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
$ export REPOSIX_CONFLUENCE_SPACE=REPOSIX
$ cargo test -p reposix-confluence --locked --test contract -- --ignored --nocapture
running 1 test
test contract_confluence_live ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 1.32s
```

Exit 0. Real Atlassian tenant (`reuben-john.atlassian.net`) returned the
REPOSIX space's 4 pages (consistent with 11-B live verification:
`65916`, `131192`, `360556`, `425985`); the first one's id was used as
`known_issue_id` and round-tripped through `get_issue` successfully.
`IssueId(u64::MAX)` returned a 404 as expected — confirming the live
404-path assertion also works against real Confluence (not just
wiremock).

## Success Criteria — Results

| # | Criterion                                                                                                                                                             | Result |
|---|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------|
| 1 | `test -f crates/reposix-confluence/tests/contract.rs`                                                                                                                 | PASS   |
| 2 | `grep -q 'async fn assert_contract' crates/reposix-confluence/tests/contract.rs`                                                                                      | PASS   |
| 3 | `grep -q 'async fn contract_sim' crates/reposix-confluence/tests/contract.rs`                                                                                         | PASS   |
| 4 | `grep -q 'async fn contract_confluence_wiremock' crates/reposix-confluence/tests/contract.rs`                                                                         | PASS   |
| 5 | `grep -q 'async fn contract_confluence_live' crates/reposix-confluence/tests/contract.rs`                                                                             | PASS   |
| 6 | `grep -q 'macro_rules! skip_if_no_env' crates/reposix-confluence/tests/contract.rs`                                                                                   | PASS   |
| 7 | `grep -q '#\[ignore\]' crates/reposix-confluence/tests/contract.rs`                                                                                                   | PASS   |
| 8 | `cargo test -p reposix-confluence --locked \| grep -E 'contract_(sim\|confluence_wiremock) \.\.\. ok' \| wc -l` returns 2                                             | PASS   |
| 9 | env-unset `cargo test -p reposix-confluence -- --ignored` prints `SKIP: env vars unset`                                                                               | PASS   |
| 10 | `cargo test --workspace --locked` exits 0                                                                                                                            | PASS (191) |
| 11 | `bash scripts/demos/smoke.sh` exits 0                                                                                                                                | PASS (4/4) |
| 12 | `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0                                                                                             | PASS   |

## Must-Have Truths

- [x] `cargo test -p reposix-confluence` runs the contract test against SimBackend AND a wiremock-backed ConfluenceReadOnlyBackend, both pass.
- [x] `cargo test -p reposix-confluence -- --ignored` runs the live-Atlassian half; skips cleanly (returns early, test passes) if any of `ATLASSIAN_API_KEY` / `ATLASSIAN_EMAIL` / `REPOSIX_CONFLUENCE_TENANT` / `REPOSIX_CONFLUENCE_SPACE` are unset.
- [x] Live-Atlassian half also passes when env IS fully set (verified against `reuben-john.atlassian.net`).
- [x] The same 5 IssueBackend invariants are exercised against all three backends via a shared `assert_contract` helper (copied verbatim from reposix-github/tests/contract.rs).
- [x] Compiles even when env vars are unset — no compile-time conditional test gating (verified by `cargo test -p reposix-confluence --test contract --no-run` succeeding in a fresh shell).

## Claude's-Discretion Choices

1. **`skip_if_no_env!` macro instead of a helper fn.** A plain fn returning `Option<Error>` would have forced the test body to write `if missing.is_some() { return; }` at every call site. A macro that does `return` from the enclosing fn is the idiomatic Rust pattern for this (same shape as `assert!`, `matches!`, etc.) and gives one-line usage at every live-test entry point. The trade-off — macro expansion is harder to step through than fn calls — is worth it for a 9-line macro exercised once.

2. **Mock-ordering: u64::MAX 404 BEFORE pages/1 200.** Wiremock matches most-recently-mounted-first. If the pages/1 mock were mounted after, it would shadow the u64::MAX 404 for any path that happened to include the substring (not a risk with the current `path()` matcher, but ordering discipline matters if a future maintainer switches to `path_regex`). The code comment on the 404 mount calls this out explicitly so it won't get silently reordered during a future refactor.

3. **Known-id via list-first in the live test, not a hardcoded value.** The plan's description allowed either approach; hardcoding `IssueId(131192)` would have couple-bound the test to the current seed state of the reuben-john tenant. List-first costs one extra HTTP request per run (~50ms) but makes the test self-configuring across any tenant with any seeded page. That matches the spirit of `skip_if_no_env!` — the test should work wherever the invoker points it.

4. **Did not modify `Cargo.toml` dev-deps.** Plan Task 1 said "add serde_json to dev-deps if needed". `serde_json` is a regular dep of reposix-confluence (11-A), and regular deps are automatically available to integration tests, so no edit was needed. Verified by `cargo test -p reposix-confluence --test contract --no-run` finishing in 1.44s on first try.

## Threat-Model Status

| Threat   | Status                | Evidence                                                                                                                                             |
| -------- | --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| T-11C-01 | mitigated             | `skip_if_no_env!` uses `$var:literal` fragment and pushes `$var` (the name) into `missing`, never the env-var *value*. The stderr line is names-only. |
| T-11C-02 | mitigated             | `assert_contract` panics with `{error:?}`, which is `Error`'s Debug — reposix-core's `Error::Other(String)` Debug is `Other("...")`, no header content. |
| T-11C-03 | mitigated             | `HttpClient`'s 5-s timeout bounds each request; live test does ≤3 round-trips per run; CI job `timeout-minutes: 5` from 11-B bounds the full job.     |
| T-11C-04 | accepted              | Read-path audit coverage deferred to v0.4; consistent with 11-A T-11-05.                                                                             |

## Threat Flags

None. The contract test file introduces no new security-relevant
surface — it only consumes `ConfluenceReadOnlyBackend` (already covered
by 11-A's threat model) and `SimBackend` (long-standing fixture).

## Deviations from Plan

None. Plan executed exactly as written. One minor implementation note:

- The plan's verify command for Task 1 was a compound bash pipe over
  300 chars. The repo's `deny-ad-hoc-bash.js` hook correctly rejected
  it, which pushed the verification into multiple separate Bash tool
  calls. Same results, just not a single one-liner. This is the
  self-improving-infrastructure pattern (CLAUDE.md OP #4) working as
  intended, not a plan deviation.

## Auth Gates

**None.** `.env` was already populated with valid Atlassian credentials
from the Phase 11 bootstrap (commit `1e58dd0`). The live test passed
on first invocation against `reuben-john.atlassian.net`.

## Known Stubs

**None.** `contract_sim`, `contract_confluence_wiremock`, and
`contract_confluence_live` all execute real assertions against real
(or mock) backends; there are no placeholder returns or TODO arms
anywhere in the test file.

## Self-Check: PASSED

- `crates/reposix-confluence/tests/contract.rs` FOUND (380 lines)
- Commit `868703e` (test(11-C-1)) FOUND in `git log`
- `cargo test -p reposix-confluence --locked` → 17 unit + 2 contract pass, 1 ignored — VERIFIED
- `cargo test --workspace --locked` → 191 passed — VERIFIED
- `cargo clippy --workspace --all-targets --locked -- -D warnings` → clean — VERIFIED
- `cargo fmt --all --check` → clean — VERIFIED
- `bash scripts/demos/smoke.sh` → 4/4 passed — VERIFIED
- Live Atlassian contract test passed against real tenant — VERIFIED
