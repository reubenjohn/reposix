---
phase: 08-demos-and-real-backend
reviewed: 2026-04-13T10:45:00Z
depth: deep
files_reviewed: 20
files_reviewed_list:
  - .github/workflows/ci.yml
  - Cargo.toml
  - crates/reposix-core/Cargo.toml
  - crates/reposix-core/src/backend.rs
  - crates/reposix-core/src/backend/sim.rs
  - crates/reposix-core/src/lib.rs
  - crates/reposix-cli/src/list.rs
  - crates/reposix-cli/src/main.rs
  - crates/reposix-cli/tests/cli.rs
  - crates/reposix-github/Cargo.toml
  - crates/reposix-github/src/lib.rs
  - crates/reposix-github/tests/contract.rs
  - docs/decisions/001-github-state-mapping.md
  - docs/demos/index.md
  - scripts/demo.sh
  - scripts/demos/_lib.sh
  - scripts/demos/_record.sh
  - scripts/demos/01-edit-and-push.sh
  - scripts/demos/02-guardrails.sh
  - scripts/demos/03-conflict-resolution.sh
  - scripts/demos/04-token-economy.sh
  - scripts/demos/assert.sh
  - scripts/demos/full.sh
  - scripts/demos/parity.sh
  - scripts/demos/smoke.sh
findings:
  blocker: 0
  high: 1
  medium: 7
  low: 6
  total: 14
status: issues_found
---

# Phase 8: Code Review — Demos + Real-Backend Integration

**Reviewed:** 2026-04-13T10:45:00Z
**Depth:** deep
**Commits in scope:** `dfc3a61` → `7585e5f` (29 commits)
**Status:** PASS (no blockers; one HIGH worth a follow-up quick)

## Summary

The trait seam is clean. `IssueBackend` is dyn-compatible (there is
even a compile-check at `crates/reposix-core/src/backend.rs:202`), all
async through `#[async_trait]`, `Send + Sync`. `SimBackend::new` and
`GithubReadOnlyBackend::new` both route construction through
`reposix_core::http::client()` — I grepped the full workspace and
found zero direct `reqwest::Client` / `reqwest::ClientBuilder` sites
outside `crates/reposix-core/src/http.rs:360` (the clippy-sanctioned
singleton). SG-01 holds. The `Authorization: Bearer …` header is only
added when `token.is_some()` (`crates/reposix-github/src/lib.rs:187`),
so anonymous callers do NOT send an empty bearer.

`update_issue` serializes the etag in RFC-7232 form (`"<v>"`, literal
quotes) at `crates/reposix-core/src/backend/sim.rs:242`, tested in
`update_with_expected_version_attaches_if_match`. `None`
`expected_version` correctly omits the header; the sim treats absent
`If-Match` as wildcard (`crates/reposix-sim/src/routes/issues.rs:335`).

The contract test covers the 5 spec invariants; `contract_sim` always
runs, `contract_github` is `#[ignore]`-gated as promised; the sim
fixture teardown is handle-abort + NamedTempFile drop, which is
correct-enough.

The demo suite is self-asserting (ASSERTS grep markers), self-cleaning
(idempotent EXIT traps), and time-bounded (`timeout 90` self-wrap).
`demos-smoke` is `continue-on-error: false` as specified;
`integration-contract` is `continue-on-error: true` for rate-limit
flake tolerance. Both depend on `test` green, good ordering.

Issues found below do not block ship. The one HIGH is a label
precedence / read-path documentation tightening that the FUSE layer
should not surprise on.

---

## High

### HR-01: `status/in-progress` + `status/in-review` precedence is undocumented in ADR-001

**Files:** `crates/reposix-github/src/lib.rs:224-230` · `docs/decisions/001-github-state-mapping.md:46-59`
**Issue:** When a GitHub issue carries BOTH `status/in-review` AND
`status/in-progress`, the translator at line 224 checks `in-review`
first and returns `InReview`. This is a reasonable choice
("further-along state wins") but ADR-001 rule (1) does not explicitly
cover the both-labels case. It reads as mutually-exclusive rules
without saying what happens when a team drifts and applies both.

Without an explicit tiebreak, two problems:

1. FUSE layer consumers who round-trip an issue (read `InReview`,
   write `InReview`) will silently strip the `status/in-progress`
   label per the v0.2 write rules — a behavior not documented as a
   read-path consequence.
2. A future adapter could reasonably pick the opposite precedence and
   break parity.

**Fix:** Add a "Tiebreak" section to ADR-001 near rule (1):

```md
### Tiebreak: simultaneous status labels

When an open issue carries BOTH `status/in-progress` AND
`status/in-review`, the read path returns `InReview` (more-advanced
state wins). This is a deliberate lossy collapse: the `status/in-progress`
label is preserved on disk (so the `labels` field still reflects
reality), but the normalized `IssueStatus` is `InReview`. The v0.2
write path will similarly drop the losing label when writing back.
```

No code change needed; the behavior at line 224-230 is already the
precedence the ADR should document.

**Severity rationale:** HIGH because this is the kind of invariant a
parity demo should codify — but the fix is pure docs, no runtime risk.

---

## Medium

### MR-01: `reposix-github` Cargo.toml carries three unused deps

**File:** `crates/reposix-github/Cargo.toml:14-28`
**Issue:** `thiserror`, `tokio` (runtime), and `rusqlite` (dev-dep)
are declared but never used by the crate:
- `thiserror`: no `#[derive(Error)]` or `thiserror::` usage anywhere
  in `src/` (all errors flow through `reposix_core::Error::Other`).
- `tokio` (regular dep): runtime is pulled in via
  `reposix-core.workspace` transitively; the crate itself does not
  spawn tasks or hold runtime handles. Only `#[tokio::test]` in inline
  tests needs it — and that should be a dev-dep.
- `rusqlite` (dev-dep): grepped — zero references in `tests/` or
  `src/`.

Compile time and dependency surface bloat. If anyone audits
"does reposix-github link sqlite?" the answer is technically yes
(workspace inherits the feature), and that answer is misleading.

**Fix:** Remove lines 19, 20, and 28. If inline tests need
`#[tokio::test]`, move `tokio` to `[dev-dependencies]`:

```toml
[dependencies]
reposix-core.workspace = true
reqwest = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
tracing = { workspace = true }
chrono = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
wiremock = "0.6"
reposix-sim = { path = "../reposix-sim" }
tempfile = "3"
```

### MR-02: `list_issues` silently truncates at `MAX_ISSUES_PER_LIST` with only a WARN log

**File:** `crates/reposix-github/src/lib.rs:83,304-341`
**Issue:** The pagination loop breaks at 500 issues (5 pages × 100)
and returns `Ok(out)` — caller cannot distinguish "this project has
exactly 500 issues" from "we hit the cap". Downstream consumers
(parity demo, FUSE listing) will silently show an incomplete view.
The WARN log at line 308 is not a machine-readable signal.

For `octocat/Hello-World` this cap never bites (~13 issues today), so
the contract test won't catch it. But any reasonably active repo has
more than 500 issues.

**Fix options (v0.2 work, flag for follow-up):**
- Add a `truncated: bool` field to the return shape, OR
- Return `Err(Error::Other("truncated: …"))` when the cap is hit
  (simple and aligns with the rest of the error surface), OR
- Surface the next-page URL so the caller can resume.

For v0.1 ship: at minimum bump the cap comment to explicitly flag the
silent truncation as a known v0.2 gap, and consider widening
`MAX_ISSUES_PER_LIST` to a more realistic number (e.g. 5000) while the
semantics are being designed.

### MR-03: `assert.sh` uses `grep -Fi` (case-insensitive) — markers are too lenient

**File:** `scripts/demos/assert.sh:71`
**Issue:** `grep -Fiq -- "$m"` matches case-insensitively. Markers
like `"Permission denied"` will match any log line containing those
words in any case, including noise. Marker `"DEMO COMPLETE"` is
case-sensitive in every script, so `-i` adds no value there but weakens
the other markers.

Specific risk: `01-edit-and-push.sh`'s marker `"in_review"` could
match an unrelated log line containing "IN_review" or "InReview"
(the demo doesn't produce such text today, but it's a brittle
contract). `"reduction"` in `04-token-economy.sh` could match log
noise.

**Fix:** Change line 71 from `grep -Fiq` to `grep -Fq` (drop `-i`).
The markers are all literal strings with fixed case in the source
scripts; case-sensitivity costs nothing and tightens the invariant.
If a specific demo needs case-insensitive matching, introduce a
per-script opt-in (e.g. `# ASSERTS_INSENSITIVE:` variant).

### MR-04: `parity.sh` calls `gh api` without verifying `gh` is authenticated

**File:** `scripts/demos/parity.sh:41,84`
**Issue:** `require gh` checks the binary is on PATH. `gh api …`
requires `gh auth login` (or `GITHUB_TOKEN` env) to actually work.
An unauthenticated user on a fresh host gets a cryptic failure partway
through the demo.

Not load-bearing in CI (`parity.sh` is Tier 3, not in `smoke.sh`), but
the demo is linked from `docs/demos/index.md` as runnable, so first-
time human users will hit this. `docs/demos/index.md:58-62` does not
mention the requirement.

**Fix (two options):**
1. Add an auth precheck near line 42:
   ```bash
   if ! gh auth status -h github.com >/dev/null 2>&1; then
       echo "ERROR: 'gh' is not authenticated to github.com" >&2
       echo "       run 'gh auth login' or export GITHUB_TOKEN" >&2
       exit 2
   fi
   ```
2. Mention the requirement in `docs/demos/index.md` Tier 3 row.

### MR-05: `render_patch_body` sends all fields on every PATCH, contradicting the "patch" semantic

**File:** `crates/reposix-core/src/backend/sim.rs:119-149`
**Issue:** The trait docstring at `backend.rs:162-166` says "`patch`
carries the fields to overwrite; untouched fields retain their current
server value". But `render_patch_body` always emits
`title`, `body`, `status`, `assignee`, `labels` — the full mutable
set. Because the caller passes `Untainted<Issue>` (a full issue), the
impl has no way to know which fields the caller actually wanted to
update; it just overwrites everything.

This is not a bug against the sim (the sim's `PatchIssueBody` with
`deny_unknown_fields` accepts this shape), but it misleads readers of
the trait doc and will cause real divergence when a write-path GitHub
backend lands — GitHub's PATCH genuinely IS field-presence semantic,
and sending `null` for an unset field does the wrong thing.

**Fix (v0.2 scope — flag for follow-up):** Introduce an
`IssuePatch` struct with `Option<Field>` per mutable field, or adopt
a field-presence pattern via `serde_json::Value`. For v0.1.5 ship:
update the trait docstring at `backend.rs:162-166` to explicitly say:

```rust
/// In v0.1.5 the simulator-backed implementation sends ALL mutable
/// fields on every PATCH; "untouched" fields are whatever the caller
/// left in the `Untainted<Issue>` they passed. A field-presence
/// `IssuePatch` type is tracked for v0.2.
```

### MR-06: `contract_github` allowlist sanity-check uses substring match

**File:** `crates/reposix-github/tests/contract.rs:180`
**Issue:** `origins.contains("api.github.com")` would match
`REPOSIX_ALLOWED_ORIGINS=https://api.github.com.attacker.com` as a
pass. The actual allowlist gate (`http::client` →
`load_allowlist_from_env` → `OriginGlob::matches`) does exact host
matching, so the attacker origin would NOT actually work — but the
test's error message would be misleading ("REPOSIX_ALLOWED_ORIGINS is
set" when the gate will still reject).

This is a sanity-check-error-message quality issue, not a security
hole.

**Fix:** Tighten to scheme+host:
```rust
assert!(
    origins.contains("https://api.github.com"),
    "contract_github requires REPOSIX_ALLOWED_ORIGINS to include \
     https://api.github.com (exact scheme://host); got {origins:?}"
);
```

### MR-07: `update_without_expected_version_is_wildcard` test does not actually prove `If-Match` absence

**File:** `crates/reposix-core/src/backend/sim.rs:441-466`
**Issue:** The test comment at line 444-451 is honest: wiremock
doesn't have a `header_absent` matcher, and the test falls back to
"request landed on the right URL". It thus does NOT prove that
`update_issue(…, None)` omits the `If-Match` header. If a future
refactor accidentally started sending `If-Match: ""` or
`If-Match: "null"`, this test would still pass.

**Fix:** Mount a stricter matcher using a custom matcher function, or
assert via `.expect_request()` after-the-fact. Rough sketch:

```rust
use wiremock::matchers::header_exists;
// Register a mock that accepts only if If-Match is MISSING:
Mock::given(method("PATCH"))
    .and(path("/projects/demo/issues/42"))
    .and(move |req: &wiremock::Request| {
        !req.headers.contains_key("If-Match")
    })
    .respond_with(...)
    .expect(1)
    .mount(&server).await;
```

Low cost, and it closes the "regression proves backward" loop.

---

## Low

### LR-01: `_lib.sh`'s `pkill -f "reposix-fuse "` pattern could nuke unrelated processes on a shared host

**File:** `scripts/demos/_lib.sh:160,166`
**Issue:** `pkill -f "reposix-fuse "` matches any process whose
command line starts with that string — including another user's
FUSE daemon on a shared CI runner. GitHub Actions runners are clean
per-job VMs so this is a non-issue in CI; it's only a footgun for
developers running demos on long-lived shared hosts.

**Fix:** Track PIDs in `_REPOSIX_SIM_PIDS` / explicit `MOUNT_PID`
arrays (most of the code already does this) and prefer targeted
`kill "$pid"` over pattern-based `pkill`. Keep the `pkill` as a
last-resort belt-and-braces, but gate it behind a
`_REPOSIX_PKILL_FALLBACK=${REPOSIX_PKILL_FALLBACK:-1}` flag the user
can disable.

### LR-02: `contract_sim` tempfile can be unlinked while sim holds an open SQLite handle

**File:** `crates/reposix-github/tests/contract.rs:113-147,154-163`
**Issue:** On panic in `assert_contract`, `_db` drops and
`NamedTempFile`'s Drop unlinks the file. The sim's tokio task still
has an open SQLite connection (via WAL) pointing at the now-unlinked
path. On Linux the OS keeps the inode alive until the last handle
closes, so WAL writes go to a zombie inode — no crash, but the sim
may log a warning about missing shm/wal on the final sync.

**Fix:** In `spawn_sim`, `persist()` the `NamedTempFile` or return a
`tempfile::TempDir` so cleanup happens after `handle.abort()` has
fully unwound. Optional; current behavior is inert on Linux.

### LR-03: `integration-contract` CI job does not install `fuse3` apt package

**File:** `.github/workflows/ci.yml:92-112`
**Issue:** The `test` job (which `integration-contract` depends on)
does install `fuse3`. The `integration-contract` job itself does not,
but it also doesn't need it — the contract test doesn't mount
anything. OK as-is.

But: `cargo test -p reposix-github -- --ignored` will compile the
`reposix-fuse` crate only if its deps are in the resolution graph;
since `reposix-github`'s dev-deps include `reposix-sim` but NOT
`reposix-fuse`, this should not compile `fuser`. Confirmed by
inspection. No change needed; flagging for future reference when the
dep graph changes.

### LR-04: `full.sh` counts test pass/fail via `awk '{s+=$6}'` which is brittle to format changes

**File:** `scripts/demos/full.sh:98-103`
**Issue:** The "test result:" line has format `test result: ok. N passed; M failed; P ignored; ...`. The awk extracts `$4`/`$6`/`$8` by position. A future cargo version that prefixes "ok." with "FAILED." or changes the separator would silently count zero. Not a regression from Phase 8 (this code pre-dated the phase) but worth flagging.

**Fix (low priority):** Use
`cargo test --workspace --no-fail-fast --format json` + `jq` to count
pass/fail cleanly. Or drop the summary entirely since `cargo test`
already propagates exit code.

### LR-05: `seed` field of `SimConfig` defaults to implicit `true` via CLI but must be set explicitly via `run_with_listener`

**File:** `crates/reposix-github/tests/contract.rs:121-128`
**Issue:** The contract test builds `SimConfig` directly with
`seed: true` (line 124). OK. But `SimConfig` has no `Default`; any
future change that adds a field and forgets to update this test will
fail to compile (which is actually correct behavior, so this is a
note, not a bug). No fix required; documenting for future readers.

### LR-06: `gh api` invocation in `parity.sh` hardcodes `per_page=30` while `reposix-github` uses 100

**File:** `scripts/demos/parity.sh:84`
**Issue:** `gh api '/repos/octocat/Hello-World/issues?state=all&per_page=30'` fetches 30 per page. The Rust library fetches 100. The diff is therefore PAGE-size-asymmetric: if octocat/Hello-World has >30 open+closed issues, the gh-side list may be truncated (at 30) while the rust side gets a full 100. Today octocat/Hello-World has <30 active issues so this won't bite.

**Fix:** Match the per-page for the demo to avoid future drift:
```bash
gh api '/repos/octocat/Hello-World/issues?state=all&per_page=100'
```

Or, since octocat/Hello-World has <30 issues, pin explicitly to 100
and add a comment that the demo depends on the fixture repo having
<100 issues.

---

## Verdict

**PASS.** No security-guardrail break. No invariant violation. The
trait seam is correctly dyn-compatible, both backends construct via
the sealed `http::client()` factory, the allowlist gate fires on
every `GithubReadOnlyBackend` request (I re-read
`HttpClient::request_with_headers_and_body` — the allowlist recheck
is per-request, so `parse_next_link` pagination URLs are also gated),
and the contract test proves normalized-shape parity for the two
concrete backends. Demos self-assert, self-clean, and self-timeout.
CI is wired correctly with the right `continue-on-error` posture.

The HIGH is a docs tightening the project can absorb in a 5-minute
quick. The MEDIUMs cluster around two themes:

- **Dead/weak tests** (MR-07, MR-06) — low-cost tightenings that buy
  a stronger regression shield.
- **Silent-truncation / silent-coercion** (MR-02, MR-05) — real
  semantic gaps but v0.2-scoped and flagged in the code comments
  already.

No finding is a ship-blocker for v0.1.5.

---

_Reviewed: 2026-04-13T10:45:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
