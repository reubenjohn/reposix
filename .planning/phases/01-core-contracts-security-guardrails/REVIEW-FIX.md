---
phase: 01-core-contracts-security-guardrails
fixed_at: 2026-04-13T09:00:00Z
review_path: .planning/phases/01-core-contracts-security-guardrails/REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
deferred: 6
status: all_in_scope_fixed
---

# Phase 1: Code Review Fix Report

**Fixed at:** 2026-04-13
**Source review:** `.planning/phases/01-core-contracts-security-guardrails/REVIEW.md`
**Iteration:** 1
**Reviewer verdict at start:** `FIX-REQUIRED` (2 HIGH, 4 MEDIUM, 4 LOW)

## Summary

- Findings in scope (user-directed): **4** — H-01, H-02, M-04, L-02
- Fixed: **4**
- Deferred: **6** — M-01, M-02, M-03, L-01, L-03, L-04
- Status: `all_in_scope_fixed`

All phase-exit gates green post-fix:

- `cargo test --workspace` → 62 tests pass (+ 1 ignored timeout test).
- `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- `bash scripts/check_clippy_lint_loaded.sh` → exit 0, "OK: clippy.toml loaded, disallowed-methods enforced, workspace clean."
- Pushed to `origin/main` (commit `0500ff3`); CI kicked off.

## Fixed Issues

### H-01 — `http::client()` returned raw `reqwest::Client`; direct use bypassed the allowlist gate

**Files modified:**
- `crates/reposix-core/src/http.rs`
- `crates/reposix-core/tests/http_allowlist.rs`
- `crates/reposix-core/tests/compile_fail.rs`
- `crates/reposix-core/tests/compile-fail/http_client_inner_not_pub.rs` (new)
- `crates/reposix-core/tests/compile-fail/http_client_inner_not_pub.stderr` (new)
- `Cargo.toml`, `crates/reposix-core/Cargo.toml`, `Cargo.lock`

**Commit:** `6b0e0b6`

**Applied fix.** Introduced `pub struct HttpClient { inner: reqwest::Client }` with a private `inner` field. `http::client(opts)` now returns `Result<HttpClient>`. The freestanding `request(&Client, Method, &str)` function is gone; its logic moved onto `HttpClient::request<U: IntoUrl>(&self, method, url)`. Convenience wrappers `get`/`post`/`patch`/`delete` route through `request`. No public accessor, no `Deref`, no `AsRef` exposes the inner `reqwest::Client`. A new trybuild fixture at `tests/compile-fail/http_client_inner_not_pub.rs` fails to compile (`field 'inner' is private`), locking the invariant in CI.

**Downstream impact.** The only call site in the workspace was `tests/http_allowlist.rs`; updated to use `hc.request(Method::GET, url)`. Since `IntoUrl` accepts `&str`, `&String`, and `Url`, the change is source-level equivalent at the call sites.

---

### H-02 — Audit-log append-only invariant was bypassable via `PRAGMA writable_schema=ON`

**Files modified:**
- `crates/reposix-core/src/audit.rs`
- `crates/reposix-core/tests/audit_schema.rs`

**Commit:** `04bb1a2`

**Applied fix.** Added `audit::open_audit_db(path)` which opens the SQLite file, enables `SQLITE_DBCONFIG_DEFENSIVE` (via `rusqlite::Connection::set_db_config`), then loads the schema. The DEFENSIVE flag rejects `sqlite_master` edits under `PRAGMA writable_schema=ON`, so the trigger rows cannot be deleted by an attacker who shares the connection. Also exposed `audit::enable_defensive(&Connection)` as the primitive for callers with an already-open connection. Added three integration tests:

- `writable_schema_bypass_is_rejected` — `PRAGMA writable_schema=ON` + `DELETE FROM sqlite_master WHERE name IN (...)` is rejected, and the BEFORE UPDATE trigger still fires afterward.
- `drop_trigger_attack_has_documented_limit` — pins that the triggers fire on a DEFENSIVE handle and documents in code that raw `DROP TRIGGER` via the owning connection remains a privileged-caller concern (v0.1 threat model). Should that assumption weaken, the preceding test catches it.
- `rollback_does_not_break_invariant` — an UPDATE inside a transaction fires the trigger, the tx rolls back, and a subsequent UPDATE still fails.

Module doc updated with the schema-attack threat model and a pointer to `open_audit_db` as the Phase-2-mandatory entry point.

**Note on the review's "Better" recommendation.** The reviewer suggested `SQLITE_DBCONFIG_DEFENSIVE` as the "Better" option; it's exactly what shipped. The "Strongest" option (separate file + separate pool) is deferred to Phase 2 per scope.

---

### M-04 — `load_schema`'s `DROP TRIGGER IF EXISTS` / `CREATE TRIGGER` was a race window

**Files modified:**
- `crates/reposix-core/fixtures/audit.sql`

**Commit:** `0500ff3`

**Applied fix.** Wrapped the entire schema bootstrap in `BEGIN; ... COMMIT;`. SQLite serialises DDL inside a transaction, so no reader on the same DB can observe the table without its triggers during the DROP/CREATE sequence. Kept the `DROP TRIGGER IF EXISTS ... CREATE TRIGGER ...` pattern intact so the ROADMAP / `01-WAVES.md` one-line greps (`CREATE TRIGGER audit_no_update BEFORE UPDATE`, `CREATE TRIGGER audit_no_delete BEFORE DELETE`) still match — the review flagged this as the "whichever is smaller" criterion, and keeping the existing grep contract wins.

**Also verified.** `audit::load_schema_is_idempotent` still passes (SQLite accepts nested / repeat `BEGIN` + `COMMIT` in `execute_batch` on an already-open connection); `cargo run -p reposix-core --example show_audit_schema` still emits matching DDL.

---

### L-02 — `parse_one` used `rsplit_once(':')` which misparsed IPv6 allowlist entries

**Files modified:**
- `crates/reposix-core/src/http.rs` (bundled with H-01 commit)

**Commit:** `6b0e0b6` (merged with H-01 since both touch `http.rs`)

**Applied fix.** Replaced the hand-rolled `rsplit_once(':')` with `url::Url::parse`, added `url = "2"` to workspace deps, and added a small `:*` strip for the wildcard-port case (the `url` crate rejects `*` as a port). Added five new tests covering:

- `http://[::1]:7777` → parses; matches `http://[::1]:7777/`.
- `http://[::1]:*` → parses; matches any port on `[::1]`.
- `http://[::1]:7777` → does NOT match `http://[::1]:7778/`.
- `https://localhost:*` → parses; scheme=https, host=localhost, port=wildcard.

**Side effect.** The `url` crate normalises default ports (`:80` on `http`, `:443` on `https`) to `None`. To keep the existing `http://127.0.0.1:80` test behaviour (`:80` allowlist must NOT match `:81`), `parse_one` now uses `parsed.port_or_known_default()`, so the glob stores `Some(80)` instead of `None`, and matches against `url.port_or_known_default()` on both sides consistently. No existing tests regressed.

## Deferred Issues

Per the user's explicit instruction — "Do NOT apply the LOW findings (L-01, L-03, L-04) or M-01/M-02/M-03 — note them in REVIEW-FIX.md as 'deferred' with rationale."

### M-01 — `validate_path_component` accepts trailing whitespace / CRLF / RTLO unicode

**Status:** deferred (post-demo hardening; not on the critical path).

**Rationale.** Phase 1's only user is `validate_issue_filename`, which rejects all of the above. The footgun lands in Phase 3 (FUSE boundary). The docstring rename + strict-variant work is a Phase 3 polish commit. Tracked as a known gap for the Phase 3 planner.

### M-02 — `validate_issue_filename("0000000000.md")` returns `IssueId(0)` silently

**Status:** deferred (Phase 2 will decide whether `IssueId(0)` is reserved).

**Rationale.** The simulator's id allocation contract is Phase 2 work. Deciding "is 0 a sentinel or a legal id" there (and either banning it in the validator or documenting it at the sim level) is cheaper than pre-choosing here.

### M-03 — `check_clippy_lint_loaded.sh`'s grep misses multi-line / renamed `reqwest::Client` patterns

**Status:** deferred (not on the critical path; the clippy rule itself is the real defence).

**Rationale.** The script's `cargo clippy --workspace --all-targets -- -D warnings` invocation is the actual proof-of-enforcement; the grep is belt-and-braces. The review's recommended "drop the grep entirely and add a `#[cfg(clippy_proof)]` decoy" is good future work; not shipped here because it removes a (weak) check rather than adds one and is low ROI for demo time.

### L-01 — `Error::Other(String)` for env-var parse errors

**Status:** deferred (typed errors are a Phase 2 CLI concern).

**Rationale.** The review's own severity note: "If Phase 2 surfaces these to the user, consider `Error::InvalidOrigin` variants..." — that's exactly when this work belongs.

### L-03 — `std::env::set_var` in multi-threaded tokio tests

**Status:** deferred (migrate when edition 2024 lands).

**Rationale.** No observed flake; the SAFETY-note wording tweak is not worth a commit-audit ping. Track as a sweep when the workspace upgrades to edition 2024.

### L-04 — `ClientOpts` fields are `pub`; future additions would be breaking changes

**Status:** deferred (no downstream consumers yet).

**Rationale.** All in-workspace call sites use `ClientOpts::default()`. The `#[non_exhaustive]` + builder migration is five lines and zero-risk, but out of scope for a review-response commit. File under "pre-v0.1.0 publish cleanup".

---

## Test + gate summary

| Gate | Pre-fix | Post-fix |
|---|---|---|
| `cargo test --workspace` | 5 unit + 2 compile-fail + 7 http_allowlist + 5 audit_schema = 19 active | 44 unit + 3 compile-fail + 7 http_allowlist + 8 audit_schema = 62 active |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean | clean |
| `bash scripts/check_clippy_lint_loaded.sh` | exit 0 | exit 0 |
| `cargo run -q -p reposix-core --example show_audit_schema` | emits both triggers | emits both triggers (+ BEGIN/COMMIT) |
| Compile-fail fixtures | 2 (tainted, untainted_new) | 3 (+ http_client_inner) |

## Commit SHAs

| Finding | Commit | Files |
|---|---|---|
| H-01 (+ L-02) | `6b0e0b6` | http.rs, http_allowlist.rs, compile_fail.rs, http_client_inner_not_pub.{rs,stderr}, Cargo.{toml,lock} x3 |
| H-02 | `04bb1a2` | audit.rs, audit_schema.rs |
| M-04 | `0500ff3` | fixtures/audit.sql |

All commits pushed to `origin/main` at `0500ff3`.

---

_Fixed: 2026-04-13_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
