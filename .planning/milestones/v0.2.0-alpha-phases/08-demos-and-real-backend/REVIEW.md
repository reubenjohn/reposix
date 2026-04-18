---
phase: 08-demos-and-real-backend
reviewed: 2026-04-13T00:00:00Z
depth: standard
retry_of: 2026-04-13T10:45:00Z (scope narrowed after rate-limit retry)
files_reviewed: 7
files_reviewed_list:
  - crates/reposix-github/src/lib.rs
  - crates/reposix-github/tests/contract.rs
  - crates/reposix-core/src/backend.rs
  - crates/reposix-core/src/backend/sim.rs
  - crates/reposix-core/src/http.rs
  - docs/decisions/001-github-state-mapping.md
  - .github/workflows/ci.yml
findings:
  blocker: 0
  high: 1
  medium: 5
  low: 4
  total: 10
status: issues_found
---

# Phase 8 Retry Review — High-Risk Surface

**Reviewed:** 2026-04-13 (retry, narrowed scope)
**Depth:** standard (focused on user-specified audit questions)
**Status:** PASS (one HIGH carried over from prior review; mediums cluster around test strength and silent-failure modes)

## Scope

This is a **retry** after the first reviewer (whose full report remains above — see commits) hit a rate limit. The narrowed audit targets:

- `crates/reposix-github/src/lib.rs` — the ~738-line new adapter
- `crates/reposix-github/tests/contract.rs` — contract parity test
- `crates/reposix-core/src/backend.rs` + `backend/sim.rs` — the seam + reference impl
- `.github/workflows/ci.yml` — `integration-contract` and `demos-smoke` jobs only

Context read: `docs/decisions/001-github-state-mapping.md`, `crates/reposix-core/src/http.rs`.

## Audit Answers (user's 20 questions)

### `GithubReadOnlyBackend`

**1. `reqwest::Client` constructed outside `http::client()`?**
**PASS.** Workspace-wide grep returns zero matches inside `crates/reposix-github/` and zero matches inside `crates/reposix-core/src/backend/`. The only construction site is `crates/reposix-core/src/http.rs:360` (the clippy-sanctioned singleton, `#[allow(clippy::disallowed_methods)]`). `clippy.toml` at workspace root bans `reqwest::Client::new`, `reqwest::Client::builder`, and `reqwest::ClientBuilder::new` crate-wide. SG-01 seal holds.

**2. Link-header parser robustness.**
**MEDIUM** — See **MR-01** below. Parser at `crates/reposix-github/src/lib.rs:258-273` handles trailing whitespace (`entry.trim()`), comma-separated multi-rel, and absence-of-next. But: it does NOT handle (a) whitespace variations around `;` inside the rel clause, (b) `rel=next` without quotes, (c) `rel="NEXT"` uppercase, (d) relative URLs. GitHub always returns absolute URLs with the canonical `rel="next"` form so this is defensive, not a live bug.

**3. Pagination cap enforcement.**
**MEDIUM (carried from prior review).** `crates/reposix-github/src/lib.rs:83,304-341`: cap is 500 issues (5 × 100). At the cap the loop either `break`s (emitting a WARN log at line 307-310) or returns via `return Ok(out)` at line 335 — both silently truncate. Caller cannot distinguish "project has exactly 500 issues" from "we hit the cap". See **MR-02**.

**4. `body: None`.**
**PASS.** `gh.body.unwrap_or_default()` at `lib.rs:252`; `GhIssue.body` is `#[serde(default)] Option<String>`. Null, absent, and empty-string all collapse to `""`.

**5. `state_reason: null` with `state=closed` → `Done`.**
**PASS.** `lib.rs:217-223`: `match gh.state_reason.as_deref() { Some("not_planned") => WontFix, _ => Done }`. `None` falls into `_`, yielding `Done`, which matches ADR-001 rule 2 pessimistic-fallback. Covered by `closed_with_completed_reason_maps_to_done` at lib.rs:533.

**6. Multiple `status/*` labels precedence.**
**HIGH (carried from prior review — still the top finding).** `lib.rs:224-230` implements "`in-review` wins over `in-progress`" via `else if` order. ADR-001 rules 1a/1b do not document the both-labels case. See **HR-01** below.

**7. URL-encoding of `owner/repo`.**
**LOW.** `lib.rs:290-291,344`: `format!("{}/repos/{}/issues?...", base, project, ...)` concatenates `project` raw. Valid GitHub slugs (`[A-Za-z0-9._-]+/[A-Za-z0-9._-]+`) are always URL-safe so this is inert in practice. If `project` contained `/` beyond the single `owner/repo` split, `%2F`, `?`, `#`, or space, the URL would be malformed and either the allowlist gate (bad origin) or GitHub (404) would reject it. See **LR-01**.

**8. Auth header absent when `token: None`.**
**PASS.** `lib.rs:187-189`: `if let Some(ref tok) = self.token { h.push(("Authorization", format!("Bearer {tok}"))); }`. No `Authorization` header is constructed when token is None — not even an empty bearer. Confirmed by `list_builds_the_right_url` test which uses `None` and passes.

**9. Rate-limit WARN vs backoff — documented?**
**PASS (but weakly).** `lib.rs:193-205` `log_rate_limit` logs a WARN when `x-ratelimit-remaining < 10` and does not back off. The doc comment at line 193-194 explicitly says "that's the caller's policy". The phase CONTEXT.md at line 94-95 documents this as spec. There is no user-facing docs link noting this as a known limitation in v0.1; the behavior is only discoverable by reading the impl. See **LR-02**.

### `SimBackend`

**1. `Client::new` check.**
**PASS.** Zero matches in `crates/reposix-core/src/backend/`. Construction at `backend/sim.rs:53` goes through `http::client()`.

**2. `expected_version: None` semantics.**
**PASS on wire.** `backend/sim.rs:242-246`: `if let Some(ref v) = if_match_val { headers.push(("If-Match", v.as_str())); }` — the header is omitted entirely when `expected_version` is `None`. Sim's handler at `reposix-sim/src/routes/issues.rs` treats absent `If-Match` as wildcard. Test coverage is **WEAK** — see **MR-03** — the "wildcard" test at `backend/sim.rs:441-466` does not actually prove header absence (wiremock lacks `header_absent`).

**3. 404 mapped to discriminable error.**
**PARTIAL.** `backend/sim.rs:88-90` returns `Error::Other(format!("not found: {context}"))`. Callers must string-match `msg.starts_with("not found:")`. The trait docstring at `backend.rs:114,144-145,187-188` acknowledges this as a v0.2 cleanup (typed `NotFound` variant). `contract.rs:78-83` only checks `is_err()` — loose but consistent with the documented v0.1.5 contract. See **LR-03**.

### `contract.rs`

**1. `contract_github` missing-token behavior.**
**FAIL CLOSED — but loudly.** `contract.rs:178-183` asserts `REPOSIX_ALLOWED_ORIGINS` contains `api.github.com` and panics if missing. `GITHUB_TOKEN` at line 185 is `.ok()` — absence means anonymous access (60 req/hr). If rate-limited, the list/get calls will return Err and the test panics with the full error. This is LOUD, not graceful — if CI is behind a shared IP that has burned the 60/hr budget, the job reports red. In practice the `integration-contract` job injects `${{ secrets.GITHUB_TOKEN }}` at 1000 req/hr so this won't bite; but there is no graceful-skip path for local devs who want `cargo test -- --ignored` without a token and got unlucky. See **MR-04**.

**2. Sim task leak on assertion panic.**
**PARTIAL.** `contract.rs:155-163`: happy path calls `handle.abort()`; panic path skips it. Tokio DOES abort tasks on `JoinHandle` drop — confirmed by reading the tokio docs behavior since 1.0. The `NamedTempFile _db` drops and unlinks the DB file; on Linux the inode stays alive until the sim's sqlite handle closes. No leak in tokio, but WAL writes hit a zombie inode briefly. See **LR-04**.

**3. `unwrap()` cryptic-failure audit.**
**PASS.** `contract.rs:46-83` uses `unwrap_or_else(|e| panic!(...))` with inline context (backend name, operation, args) for every fallible call. The spawn helper at lines 113-147 has `.expect("tempfile")`, `.expect("bind")`, `.expect("local_addr")`, `.expect("http client")` — these all represent environmental prerequisites (tempdir writable, loopback bindable) and would produce readable panics.

### CI

**1. `integration-contract` flake risk.**
**PASS.** `.github/workflows/ci.yml:93-110`. `GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}` is auto-injected by Actions at 1000 req/hr per repo. The test makes 3 HTTP calls (list, get, get-missing) × 1 per push. Well under budget. `continue-on-error` is NOT set, so this is load-bearing. Comment at line 94-98 has a truncated sentence — see **LR-05**.

**2. `demos-smoke` timeout.**
**MEDIUM.** `.github/workflows/ci.yml:76-91`. No explicit `timeout-minutes:` on the job. GitHub Actions defaults job timeout to 360 minutes (6h), meaning a runaway smoke.sh could burn quota. The spec says "~90s budget". Individual demos carry `timeout 90` wrappers (per prior review), but the outer job has no belt-and-braces cap. See **MR-05**.

---

## Findings

### HIGH

#### HR-01: `status/in-progress` + `status/in-review` precedence undocumented in ADR-001

**Files:** `crates/reposix-github/src/lib.rs:224-230` · `docs/decisions/001-github-state-mapping.md:46-59`
**Issue:** When an open issue carries BOTH `status/in-review` AND `status/in-progress`, `translate()` returns `InReview` (the `else if` order makes `in-review` win). ADR-001 rules 1a/1b read as mutually-exclusive without covering the both-labels case.
**Fix:** Docs-only. Add a "Tiebreak" paragraph to ADR-001 stating `in-review` beats `in-progress` (more-advanced state wins), and note that the v0.2 write path must drop the losing label. No code change required.
**Severity rationale:** HIGH because parity/round-trip behavior depends on an undocumented invariant; fix is pure docs.

### MEDIUM

#### MR-01: Link-header parser is a hand-rolled substring scanner

**File:** `crates/reposix-github/src/lib.rs:258-273`
**Issue:** `parse_next_link` checks `rest.contains("rel=\"next\"")` — case-sensitive, requires exact quotes, does not handle whitespace between `;` and `rel`, does not handle `rel=next` without quotes (RFC 8288 allows unquoted tokens). GitHub happens to emit the exact canonical form this parses, so it is defensive not live; but a single quirky proxy / GHE tenant injecting whitespace could break pagination silently (single-page result, same failure mode as MR-02 cap).
**Fix:** Use an existing parser (`hyperx::header::Link`, `parse_link_header` crate) OR at minimum normalize whitespace and case-fold the rel tag before comparison:
```rust
let rel = rest.split(';')
    .map(str::trim)
    .find_map(|s| s.strip_prefix("rel="));
let rel_unquoted = rel.map(|r| r.trim_matches('"'));
if rel_unquoted == Some("next") { return Some(url.to_owned()); }
```

#### MR-02: `list_issues` silently truncates at 500 issues

**File:** `crates/reposix-github/src/lib.rs:83,304-341`
**Issue:** At the cap the loop breaks (line 311) or returns (line 335) without signaling truncation. Caller cannot distinguish "has 500 issues" from "hit cap". `octocat/Hello-World` has ~13 issues so contract test does not catch it.
**Fix (v0.2 follow-up):** Return `Err(Error::Other("truncated: project has more than MAX_ISSUES_PER_LIST"))` at the cap, OR widen the cap and emit a machine-readable marker. For v0.1 ship: add a code comment at line 83 flagging silent truncation as known.

#### MR-03: `update_without_expected_version_is_wildcard` does not prove header absence

**File:** `crates/reposix-core/src/backend/sim.rs:441-466`
**Issue:** The test comment (444-451) admits wiremock has no `header_absent` matcher and falls back to URL matching. If a future refactor started emitting `If-Match: ""` or `If-Match: "null"` when `expected_version` is None, this test would still pass.
**Fix:** Use a custom closure matcher:
```rust
.and(|req: &wiremock::Request| !req.headers.contains_key("If-Match"))
```
Low cost; closes the regression-proves-backward loop.

#### MR-04: `contract_github` hard-panics on rate limit

**File:** `crates/reposix-github/tests/contract.rs:174-191`
**Issue:** With `GITHUB_TOKEN` unset and IP already rate-limited, the test panics on the first list/get. No graceful skip path for local devs. CI is fine (1000/hr via `secrets.GITHUB_TOKEN`). Consider a pre-check that `get /rate_limit` → warn-and-skip when `remaining < 3`, keeping the hard-fail path for "token set but still failing".
**Fix (optional):**
```rust
// If we can't even fetch the rate-limit endpoint, bail out LOUDLY (that's a real failure).
// If we CAN, and remaining < 3, print a skip-notice and return.
```
Low priority — the documented CI path works. Flagging because `#[ignore]`-gated tests are typically expected to be best-effort.

#### MR-05: `demos-smoke` job has no explicit `timeout-minutes`

**File:** `.github/workflows/ci.yml:76-91`
**Issue:** Job inherits the 360-minute default. A runaway demo (hung FUSE mount, sim deadlock) would burn CI minutes before auto-kill. Individual demos have `timeout 90` wrappers, but the outer job does not.
**Fix:** Add `timeout-minutes: 10` at the job level (line 78):
```yaml
demos-smoke:
  name: demos smoke
  runs-on: ubuntu-latest
  needs: [test]
  timeout-minutes: 10
  steps:
    ...
```
Same recommendation for `integration-contract` (the contract test is 3 HTTP calls; `timeout-minutes: 5` is plenty).

### LOW

#### LR-01: URL-path components (`project`, issue id) are not percent-encoded

**File:** `crates/reposix-github/src/lib.rs:290-291,344`
**Issue:** `format!("{}/repos/{}/issues", base, project)` concatenates raw. Valid GitHub owner/repo slugs are URL-safe so inert today. Defensive fix: use `percent_encoding::utf8_percent_encode`. Not a security issue (`HttpClient` re-parses and rejects malformed URLs via the allowlist gate before I/O) and not a bug for well-formed GitHub inputs. Flagging for consistency with any future input that might flow in from a less-trusted source.

#### LR-02: Rate-limit "WARN only, no backoff" is not in the crate-level docs

**File:** `crates/reposix-github/src/lib.rs:1-51`
**Issue:** Crate module doc covers scope, state mapping, contract test, security; does not mention the WARN-only rate-limit policy. A downstream consumer reading the docs does not learn "if you hit 5000/hr the backend will keep trying and get 403s".
**Fix:** Add one sentence to the `# Scope` section:
```rust
//! Rate-limit handling is WARN-log only — the caller is responsible for
//! backoff. See `log_rate_limit` for the threshold (< 10 remaining).
```

#### LR-03: "not found" error is positional string match

**File:** `crates/reposix-github/src/lib.rs:356` · `crates/reposix-core/src/backend/sim.rs:89` · `crates/reposix-core/src/backend.rs:114,144-145`
**Issue:** Callers discriminate not-found via `msg.starts_with("not found:")`. Typed `NotFound` variant is tracked as v0.2 in trait docstring. Contract test at `contract.rs:78-83` only checks `is_err()` — so a 500 response would also pass. Acceptable for v0.1.5; locked into the contract.
**Fix:** v0.2 cleanup as already documented; no v0.1.5 action.

#### LR-04: `contract_sim` tempfile unlinked with live sim handle on panic

**File:** `crates/reposix-github/tests/contract.rs:113-147,154-163`
**Issue:** Panic in `assert_contract` drops `_db` (unlink) while the sim tokio task still holds an open sqlite handle. Linux keeps the inode alive until the handle closes, so WAL writes go to a zombie inode. No crash, but noisy.
**Fix:** Return `tempfile::TempDir` from `spawn_sim` and defer cleanup until after `handle.abort()` has unwound. Optional.

#### LR-05: Truncated comment in `integration-contract` CI job

**File:** `.github/workflows/ci.yml:94-98`
**Issue:** The comment reads:
```
# Flaky by design: unauthenticated GITHUB_TOKEN-less CI hits the 60
# GITHUB_TOKEN is auto-injected by Actions and lifts the 60/hr anonymous
```
Line 95 is truncated mid-sentence ("hits the 60" then next line starts a new topic). Probably meant "hits the 60/hr ceiling. With Actions, GITHUB_TOKEN is auto-injected ...". Cosmetic but confusing for future readers.
**Fix:** Rewrite the comment block:
```yaml
# Hits real GitHub (octocat/Hello-World) via the contract test helper.
# Unauthenticated requests get 60/hr per IP which CI can exhaust. Actions
# auto-injects GITHUB_TOKEN which raises the limit to 1000/hr — ample for
# per-push contract testing. Load-bearing, no continue-on-error.
```

---

## Verdict

**PASS.** The narrowed audit surface passes every security-critical gate:

- **SG-01 seal intact.** Zero direct `reqwest::Client` construction outside the single sanctioned site; both backends go through `http::client()`; the allowlist gate re-fires on every request (including paginated next-URLs).
- **Auth header correctness.** No empty bearer when `token: None`.
- **State mapping matches ADR-001.** `null state_reason` + closed → Done (pessimistic); `not_planned` → WontFix; `completed` / other → Done.
- **Contract test structurally sound.** 5 invariants, both backends, `#[ignore]`-gated GitHub half, allowlist pre-check.
- **CI load-bearing.** `demos-smoke` and `integration-contract` both have `continue-on-error` unset; `GITHUB_TOKEN` is injected.

Findings cluster in two areas: (a) docs tightening (HR-01, LR-02, LR-05) and (b) silent-failure weaknesses in tests/truncation (MR-02, MR-03, MR-04, MR-05). None block v0.1.5 ship.

---

_Reviewed: 2026-04-13 (retry, narrowed scope)_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
