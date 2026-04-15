# Phase 21: OP-7 Hardening Bundle — Research

**Researched:** 2026-04-15
**Domain:** Rust swarm harness, Confluence pagination, SQLite WAL chaos, GitHub Actions macOS CI
**Confidence:** HIGH (all findings verified against actual codebase)

## Summary

Phase 21 is a hardening bundle of seven OP-7 sub-items. The most important
pre-work discovery is that **two items are already done**: the credential-hygiene
pre-push hook (CONTEXT.md commits `f357c92` + `5361fd5`) and all three SSRF
regression tests (commit `ea5e548`, covering `_links.base`, `webui_link`, and
arbitrary-string-field surfaces). The remaining five items need building from
scratch: contention swarm mode, 500-page WARN + `--no-truncate`, tenant-URL
redaction in 429 logs, audit-log chaos/WAL test, and macOS CI matrix.

The existing swarm harness (`crates/reposix-swarm/`) is well-structured. The
`SimDirectWorkload` uses wildcard etags (no If-Match sent) to avoid deliberate
409s; the `--contention` mode will be a new `ContentionWorkload` that sends
`If-Match: "<version>"` and specifically targets the same issue from all N
clients to provoke 409s deterministically. The sim's If-Match / 409 path is
already implemented and tested at the route level.

The 500-page cap in `crates/reposix-confluence/src/lib.rs` is a `tracing::warn!`
today; SG-05 compliance also requires a `--no-truncate` CLI flag on the backends
that list issues. The flag surface is at the `reposix-cli` / `reposix-swarm`
level; the backend itself needs only the behavior (error vs warn) toggleable via
a new method or an argument.

macOS + macFUSE CI is feasible on GitHub Actions using the `macos-latest` runner
with the community `gythialy/macfuse` action. The key behaviour difference is
that macOS FUSE uses `umount -f <mount>` instead of `fusermount3 -u <mount>` for
teardown, and `/dev/fuse` is absent before macFUSE installs.

**Primary recommendation:** Audit before coding. Start wave A with the audit of
the two pre-shipped items, then wave B (contention swarm), wave C (truncation +
tenant redaction + audit chaos), wave D (macOS CI matrix).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Contention swarm (If-Match 409 proof) | reposix-swarm | reposix-sim | Swarm drives load; sim enforces versioning |
| 500-page WARN + --no-truncate | reposix-confluence | reposix-cli/reposix-swarm (flag surface) | Backend holds cap logic; CLI exposes flag |
| Credential hygiene pre-push hook | scripts/hooks | CI (already green) | Hook runs at git push time |
| SSRF regression | reposix-confluence/tests | wiremock | Already done — contract.rs |
| Tenant-URL redaction in 429 logs | reposix-confluence | — | ingest_rate_limit emits the warn |
| Audit-log WAL chaos test | reposix-swarm | reposix-sim | New --chaos swarm mode; sim is the process |
| macOS + macFUSE parity CI | .github/workflows | reposix-fuse | Runner matrix + fusermount/umount conditional |

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| HARD-01 | `reposix-swarm --contention` proves If-Match 409 deterministic under N-client contention | ContentionWorkload pattern described; sim 409 path verified |
| HARD-02 | 500-page WARN + `--no-truncate` flag errors instead of silently capping (SG-05) | list_issues pagination loop identified; flag surface at CLI |
| HARD-03 | Chaos audit-log test: kill -9 sim mid-swarm, no dangling/torn rows in WAL DB | swarm --chaos mode design; WAL consistency semantics described |
| HARD-04 | macFUSE parity CI matrix for macOS with `umount -f` conditional | GitHub Actions macOS runner + gythialy/macfuse action approach |
</phase_requirements>

## Standard Stack

### Core (no new deps needed for most items)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rusqlite (bundled) | 0.32 | SQLite audit DB read in chaos test | Already in workspace |
| tokio | 1 | Async swarm + chaos kill-9 via nix/libc | Already in workspace |
| wiremock | 0.6 | SSRF regression tests (already used) | Already in workspace |

### New Dependencies (minimal)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| nix (optional) | 0.27 | `nix::sys::signal::kill(pid, Signal::SIGKILL)` for chaos test | Only if Tokio's `process::Child::kill()` is insufficient; prefer std or tokio-process first |

**Version verification:** No new workspace deps needed for waves B and C. The
chaos kill-9 can use `std::process::Child::kill()` or `tokio::process::Child::kill()`
— both already available. [VERIFIED: crates/reposix-swarm/Cargo.toml via grep]

**Installation:**
```bash
# No new deps for core items.
# If nix crate is chosen for SIGKILL path: cargo add nix -p reposix-swarm --features signal
```

## Architecture Patterns

### System Architecture Diagram

```
Contention Swarm (HARD-01):
  ┌──────────────────────────────────────────────┐
  │  reposix-swarm --mode contention             │
  │  50 clients, same issue ID, 30s              │
  │                                              │
  │  Each client loop:                           │
  │    GET issue → extract version               │
  │    PATCH with If-Match: "<version>"          │
  │      → 200 (win) or 409 (lose)              │
  └──────────────┬───────────────────────────────┘
                 │ HTTP
  ┌──────────────▼───────────────────────────────┐
  │  reposix-sim (in-process via spawn_sim)      │
  │  If-Match check in patch_issue (routes/      │
  │  issues.rs lines 370-381)                   │
  │  audit_events: one row per op               │
  └──────────────────────────────────────────────┘

  Assertions:
    - 409 count > 0 (deterministic conflict)
    - 200 count > 0 (at least one winner)
    - 200 count == unique versions seen in audit (no torn writes)
    - 0 Other-class errors

500-page Truncation (HARD-02):
  list_issues loop (lib.rs:763) reaches MAX_ISSUES_PER_LIST (500):
    tracing::warn! already present (line 766) ← DONE
    Need: --no-truncate flag → return Err instead of Ok(capped vec)
    Flag surface: reposix-cli (list subcommand) + reposix-swarm (confluence-direct)
    Backend change: ConfluenceBackend::list_issues takes a TruncatePolicy enum or
                    a separate method list_issues_strict(&self, ...) -> Result<...>

Chaos Audit (HARD-03):
  ┌─────────────────────────────────────────────────────┐
  │  chaos swarm loop (reposix-swarm --chaos):          │
  │    1. spawn sim as child process (std::process)     │
  │    2. run swarm for 10s                             │
  │    3. kill -9 sim process                           │
  │    4. restart sim on same DB path                   │
  │    5. repeat N cycles                               │
  └─────────┬───────────────────────────────────────────┘
            │ at end: open audit DB, count rows,
            │ verify no partial rows (all fields non-null)
            └─────────────────────────────────────────────►
               ASSERTION: no torn writes visible
               (SQLite WAL atomicity guarantee)

macOS CI Matrix (HARD-04):
  ci.yml matrix job: ubuntu-latest + macos-latest
  macos-latest: uses gythialy/macfuse action to install macFUSE
  Conditional unmount: ${{ runner.os == 'macOS' && 'umount -f' || 'fusermount3 -u' }}
```

### Recommended Project Structure

```
crates/reposix-swarm/src/
├── contention.rs    # NEW: ContentionWorkload (same-issue patching)
├── chaos.rs         # NEW: chaos test helper (kill-9 sim loop)
└── ... (existing files unchanged)

crates/reposix-swarm/tests/
└── contention_e2e.rs  # NEW: integration test for --contention mode

crates/reposix-confluence/src/lib.rs  # MODIFY: list_issues strict mode
crates/reposix-cli/src/main.rs        # MODIFY: --no-truncate flag
.github/workflows/ci.yml              # MODIFY: add macOS matrix job
```

### Pattern 1: ContentionWorkload — same-issue If-Match storm

**What:** N clients all target the same `IssueId`. Each step: GET to read
current version, then PATCH with `If-Match: "<version>"`. Exactly one client
wins (200); the rest get 409. The workload records wins vs conflicts separately.

**When to use:** HARD-01 only.

**Sketch:**
```rust
// crates/reposix-swarm/src/contention.rs
// Source: derived from sim_direct.rs pattern + routes/issues.rs If-Match semantics

pub struct ContentionWorkload {
    backend: SimBackend,
    project: String,
    target_id: IssueId,  // shared: all clients hammer the same issue
    rng: Mutex<StdRng>,
}

// In step():
//   1. get_issue → extract issue.version
//   2. update_issue(..., Some(issue.version))
//      → Ok(_)  → record OpKind::Patch (win)
//      → Err(VersionMismatch / Conflict) → record ErrorKind::Conflict (expected)
```

**Metrics extension needed:** Add `OpKind::ContentionWin` and
`ErrorKind::Conflict` already exists. The summary must show
`wins + conflicts = total_patch_attempts` to prove no torn writes.

**Test assertion (HARD-01):**
```rust
// contention_e2e.rs
assert!(conflict_count > 0, "no 409s means If-Match not being sent");
assert!(win_count > 0, "at least one client must win each round");
// Total wins == version increments in audit log:
let final_version = sim.get_issue("demo", target_id).version;
assert_eq!(win_count, final_version, "each win must produce exactly one version bump");
```

### Pattern 2: 500-page WARN + --no-truncate

**What:** `list_issues` in `ConfluenceBackend` already emits `tracing::warn!`
when it hits the 500-page cap. The missing piece is an opt-in strict mode that
returns `Err` instead.

**Implementation approach:** Add a `pub fn list_issues_strict` method (or a
`TruncatePolicy` enum parameter) to `ConfluenceBackend`. The CLI flag
`--no-truncate` in the `reposix list` subcommand calls the strict variant.

The `IssueBackend` trait's `list_issues` signature is `async fn list_issues(&self, project: &str) -> Result<Vec<Issue>>` — it should not change. The strict
variant lives as a concrete method on `ConfluenceBackend`, not on the trait, to
avoid breaking all other backends.

**Alternative:** Pass a `ListOptions { no_truncate: bool }` via an extended
trait method. This is more invasive and not needed for this phase.

**Test (HARD-02):**
```rust
// lib.rs unit test: mount 6 pages-pages, set MAX_ISSUES_PER_LIST=5 via
// test override, call list_issues_strict → assert Err(truncation)
// call list_issues → assert Ok(5 pages) + tracing warn emitted
```

### Pattern 3: Tenant-URL redaction in 429 logs

**What:** `ingest_rate_limit` (lib.rs:565) emits:
```
tracing::warn!(wait_secs, "Confluence rate limit — backing off until retry-after")
```
This log does NOT include the URL today. However, other `tracing::warn!` calls
in the error paths DO include the URL (e.g. lines 782-784 in list_issues loop:
`format!("confluence returned {status} for GET {url}")`). The URL contains the
full tenant name.

**Fix:** Introduce a helper `fn redact_url(url: &str) -> String` that replaces
the tenant segment with `<tenant>`. Pattern: `https://<tenant>.atlassian.net/...`
→ `https://<tenant-redacted>.atlassian.net/...`. Or simply omit the URL from
warn/error messages and log only the path portion. The simpler approach is to
log `url.path()` (via `reqwest::Url::path_and_query()`) rather than the full URL.

**Implementation note:** `CLAUDE.md` security requirement: "Tenant-name leakage"
is listed as an OP-7 item. The fix is small — a one-line helper plus updating
the three log sites in lib.rs that include full URLs.

### Pattern 4: Chaos audit-log WAL test (HARD-03)

**What:** Spawn sim as a **child process** (not in-process), run swarm for 10s,
kill -9 the child, restart it on the same DB, assert no torn rows.

**Why child process, not in-process:** `tokio::spawn` tasks cannot be
`kill -9`'d independently; in-process abort unwinds cleanly. A real chaos test
requires an OS-level SIGKILL to test WAL durability.

**Implementation:**
```rust
// crates/reposix-swarm/tests/chaos_audit.rs (or chaos subcommand)
// Steps:
//   1. write DB path to tempfile
//   2. spawn sim binary: std::process::Command::new("reposix-sim") with --db path
//   3. poll /healthz
//   4. run swarm for 10s
//   5. child.kill() → SIGKILL on Linux
//   6. sleep 100ms (kernel flushes WAL checkpoint)
//   7. reopen DB, query audit_events → assert 0 rows with NULL fields
//   8. restart sim on same DB, poll /healthz
//   9. repeat 2-3 cycles
```

**SQLite WAL guarantee:** A committed transaction in WAL mode is durable if:
- The WAL write completed before SIGKILL
- The DB file is not corrupt

SIGKILL on the sim means in-flight writes (between `BEGIN` and WAL-write) are
lost, but completed transactions are recoverable. SQLite recovers by replaying
WAL on next open. The test asserts no *corrupt* rows (partial field writes)
exist, not that all rows survived. [VERIFIED: SQLite WAL docs; rusqlite bundled
so no system lib dependency]

**CI note:** This test requires building the `reposix-sim` binary. Tag it
`#[ignore]` and gate it behind `REPOSIX_CHAOS_TEST=1` env var for normal CI;
add it to a dedicated `chaos` CI job that runs weekly or on-demand.

### Pattern 5: macOS + macFUSE CI (HARD-04)

**What:** Add `macos-latest` as a matrix entry in ci.yml for the `integration`
job. macFUSE is the macOS equivalent of Linux `fuse3`.

**Actions recipe (ASSUMED — needs verification against gythialy/macfuse latest):**
```yaml
# In ci.yml integration job, add matrix:
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest]
runs-on: ${{ matrix.os }}

steps:
  - name: Install FUSE (Linux)
    if: runner.os == 'Linux'
    run: sudo apt-get install -y fuse3

  - name: Install macFUSE
    if: runner.os == 'macOS'
    uses: gythialy/macfuse@v1   # verify version

  - name: Run integration tests
    run: |
      UNMOUNT="${{ runner.os == 'macOS' && 'umount -f' || 'fusermount3 -u' }}"
      cargo test -p reposix-fuse --release --locked -- --ignored --test-threads=1
    env:
      REPOSIX_UNMOUNT_CMD: ${{ runner.os == 'macOS' && 'umount -f' || 'fusermount3 -u' }}
```

**`reposix-fuse` unmount conditional:** The FUSE tests call `fusermount3 -u`
directly as a shell command in test teardown. This needs to be `$REPOSIX_UNMOUNT_CMD`
or a compile-time cfg flag. Check `crates/reposix-fuse/tests/` for teardown
patterns. [VERIFIED: CI uses `fusermount3 -u` — found in fuse tests grep]

**macFUSE SIP / kernel extension:** macOS Sequoia (15.x) requires approving
kernel extensions in System Preferences. GitHub's `macos-latest` runner
(macOS 14 Sonoma as of 2026-04) typically has macFUSE pre-installable via the
`gythialy/macfuse` action without SIP approval. [ASSUMED — runner image may
change; verify against GitHub runner changelog]

### Anti-Patterns to Avoid

- **In-process chaos:** Don't use `tokio::spawn` + `JoinHandle::abort()` for
  the kill-9 chaos test — abort unwinds too cleanly to test WAL durability.
  Use `std::process::Command` to spawn sim as a real child process.
- **Mutating IssueBackend trait for --no-truncate:** The trait serves all
  backends; add the strict method only on `ConfluenceBackend` concrete type.
- **Wildcard If-Match in contention test:** The existing `SimDirectWorkload`
  uses `None` (wildcard) for `expected_version`, which never triggers 409s.
  `ContentionWorkload` must explicitly pass `Some(version)`.
- **Polling /healthz without timeout in chaos test:** The sim might not come
  back up if the DB is corrupt. Cap the healthz poll at 5s and fail the test
  rather than hanging.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Process kill-9 | Custom ptrace / signal FFI | `std::process::Child::kill()` | Sends SIGKILL on Unix by default |
| SQLite WAL row integrity check | Custom binary parser | `rusqlite::Connection::open` + standard query | Bundled SQLite handles WAL replay automatically |
| macFUSE installation | Homebrew script in CI | `gythialy/macfuse` GitHub Action | Handles SIP, kext signing, reboot quirks |
| 429 URL redaction | Regex replacement | `reqwest::Url::path_and_query()` | Parse → extract path; no regex needed |

## Audit of Session-4 Drive-By Items

### Credential hygiene pre-push hook — COMPLETE

**Status: DONE.** [VERIFIED: codebase grep]

- `scripts/hooks/pre-push` exists, is executable, scans for `ATATT3*`, `ghp_*`,
  `github_pat_*`, `Bearer ATATT3*` patterns with 20+ char entropy requirement.
- `scripts/install-hooks.sh` links the hook into `.git/hooks/pre-push`.
- `scripts/hooks/test-pre-push.sh` has 6 test cases, all documented as passing.
- Pattern matching is robust: distinguishes real tokens (20+ entropy chars after
  prefix) from documentation strings. EXCLUDE_DIRS includes `scripts/hooks` so
  the hook itself doesn't self-reject.

**Gap check:** The test script runs against the current branch — it creates
throwaway commits on a detached HEAD and cleans up. CI does not run the hook
test automatically (it's not in `ci.yml`). Consider adding
`bash scripts/hooks/test-pre-push.sh` as a CI step. This is a small quality
gap, not a blocker for HARD items.

**Conclusion for wave A:** Read the hook file and test — confirm the 6/6 claim.
No new code needed for this item.

### SSRF regression tests — COMPLETE

**Status: DONE.** [VERIFIED: crates/reposix-confluence/tests/contract.rs]

Three tests exist in `contract.rs`:
1. `adversarial_links_base_does_not_trigger_outbound_call` — covers `_links.base`
   at top-level space-list and per-page list response shapes.
2. `adversarial_webui_link_does_not_trigger_outbound_call` — covers `webui_link`,
   `_links.webui`, `_links.tinyui`, `_links.self`, `_links.edit`. Also exercises
   `get_issue` with an adversarial single-page response.
3. `adversarial_host_in_arbitrary_string_field_is_ignored` — covers `title`,
   `ownerId`, `parentId`, body text containing adversarial URLs.

All three use the `legit_server` + `decoy_server` pattern with `.expect(0)` on
the decoy, so any accidental URL-following fails the test on drop.

Test 3 (`adversarial_host_in_arbitrary_string_field_is_ignored`) currently runs
as a non-ignored test (`cargo test --workspace` output shows it passing).

**Gap check:** The `get_issue` ADF response path in tests 1 and 2 properly mounts
adversarial `webui_link` / `_links.webui` on the single-page response shape.
No remaining uncovered surfaces identified in the current `ConfPage` deserialization
struct.

**Conclusion for wave A:** Grep contract.rs, verify test names and count, confirm
the 3 tests pass in `cargo test --workspace`. No new SSRF tests needed.

## Common Pitfalls

### Pitfall 1: Contention test without a version read step
**What goes wrong:** The workload patches with a hardcoded or stale version,
causing every request after the first to 409 without any wins.
**Why it happens:** If-Match requires reading the current version first. The
existing `SimDirectWorkload` uses `None` (wildcard), skipping this.
**How to avoid:** `ContentionWorkload::step` must call `get_issue` before each
`update_issue` to get the live version. Race condition is intentional: reads and
patches are unsynchronized across clients, so some reads will return a stale
version and their patches will 409.
**Warning signs:** Win count stays at 1 (only the first patch ever wins).

### Pitfall 2: Chaos test using in-process sim
**What goes wrong:** `tokio::spawn` task abort unwinds cleanly; the WAL is never
mid-write at abort time. The test becomes a vacuous pass.
**Why it happens:** In-process abort calls async cancellation, not SIGKILL.
**How to avoid:** Use `std::process::Command` to spawn the `reposix-sim` binary.
**Warning signs:** The test passes in 0ms (no real kill happened).

### Pitfall 3: macOS GitHub runner kernel extension approval
**What goes wrong:** The macFUSE kext requires SIP approval on newer macOS,
causing the test binary to hang or produce "Operation not permitted" from fuser.
**Why it happens:** macOS 15 Sequoia changed kext approval flows; `macos-latest`
runner image varies.
**How to avoid:** Pin to `macos-14` (Sonoma) initially; verify `gythialy/macfuse`
action supports it. Add a `--if-not-fuse-skip` guard in the integration test
invocation (or use the existing `#[ignore]` guard with a healthz-style check).
**Warning signs:** `cargo test ... -- --ignored` hangs indefinitely on mount.

### Pitfall 4: --no-truncate on the IssueBackend trait
**What goes wrong:** Adding a `no_truncate: bool` parameter to `list_issues` on
the trait forces changes to SimBackend, GitHubBackend, and every other implementor.
**Why it happens:** Premature generalization.
**How to avoid:** Add `list_issues_strict` as a concrete method on `ConfluenceBackend`
only. The CLI flag calls it only when `--backend confluence` is active.
**Warning signs:** Compiler errors in reposix-github, reposix-sim when changing
the trait.

### Pitfall 5: WAL chaos test expecting all rows to survive SIGKILL
**What goes wrong:** Test fails because some in-flight transactions were lost.
**Why it happens:** SIGKILL aborts in-flight WAL writes. Rows that weren't
committed before the kill are legitimately lost — that's correct SQLite WAL
behavior.
**How to avoid:** Assert row *integrity* (no partial writes / NULL fields in
committed rows), not row *count*. The count will be unpredictable across runs;
the structure of committed rows must be valid.
**Warning signs:** Test flakes between runs because the row count threshold is too
tight.

## Code Examples

### Current If-Match path in sim (verified)

```rust
// Source: crates/reposix-sim/src/routes/issues.rs lines 370-381
if let Some(ref raw_etag) = if_match {
    let sent_ok = raw_etag
        .parse::<u64>()
        .map(|n| n == current_version)
        .unwrap_or(false);
    if !sent_ok {
        return Err(ApiError::VersionMismatch {
            current: current_version,
            sent: raw_etag.clone(),
        });
    }
}
```

### Current 500-page cap in ConfluenceBackend (verified)

```rust
// Source: crates/reposix-confluence/src/lib.rs lines 763-769
while let Some(url) = next_url.take() {
    pages += 1;
    if pages > (MAX_ISSUES_PER_LIST / PAGE_SIZE) {
        tracing::warn!(pages, "reached MAX_ISSUES_PER_LIST cap; stopping pagination");
        break;  // ← silent truncation; must become Err in strict mode
    }
    // ...
    if out.len() >= MAX_ISSUES_PER_LIST {
        return Ok(out);  // ← also silent; strict mode should Err here
    }
}
```

### Current 429 log — no URL leak (verified)

```rust
// Source: crates/reposix-confluence/src/lib.rs lines 563-569
tracing::warn!(
    wait_secs = wait,
    "Confluence rate limit — backing off until retry-after"
);
```

The 429 warn itself does NOT leak the tenant URL. The leak risk is in the
list_issues error path (line 782-784) which formats the full URL into the
error string returned from list_issues. That error propagates up to CLI output
and could appear in logs. The `--no-truncate` work should also sanitize this
error message.

### Swarm driver factory pattern (for ContentionWorkload)

```rust
// Source: crates/reposix-swarm/src/main.rs lines 104-113
// Pattern to follow for adding --contention mode:
Mode::SimDirect => {
    let origin = args.target.clone();
    let project = args.project.clone();
    run_swarm(cfg, |i| {
        ContentionWorkload::new(
            origin.clone(),
            project.clone(),
            target_issue_id,   // new: fixed shared issue id
            u64::try_from(i).unwrap_or(0),
        )
    }).await?
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Linux-only FUSE in CI | Linux-only (macOS deferred in release.yml) | Phase 14 | macOS is the gap HARD-04 closes |
| Wildcard If-Match (no 409s) | Intentional in SimDirectWorkload | Phase 9 | ContentionWorkload must opt in to explicit etags |
| Silent pagination cap | tracing::warn only | Phase 11 | HARD-02 adds strict mode |
| No credential hook | pre-push hook shipped | Session 4 | Already done — audit only |
| No SSRF tests | 3 wiremock tests shipped | Session 4 | Already done — audit only |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo / rustc | All | ✓ | stable (1.82+) | — |
| fusermount3 | Chaos test binary spawn | ✓ (Ubuntu dev host) | fuse3 package | Skip chaos test on hosts without fuse3 |
| reposix-sim binary | Chaos test | Build artifact | n/a | Must `cargo build -p reposix-sim` in Wave 0 |
| gythialy/macfuse action | HARD-04 CI | ✓ (GitHub Actions) | v1 | None — macOS CI requires this action |

**Missing dependencies with fallback:**
- `reposix-sim` binary for chaos test — not pre-built; wave 0 must `cargo build --release -p reposix-sim` before chaos test can run. In CI, use the existing `cargo build --release --workspace --bins` step.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in) + tokio test macros |
| Config file | none (workspace Cargo.toml) |
| Quick run command | `cargo test --workspace --locked` |
| Full suite command | `cargo test --workspace --release --locked -- --test-threads=4` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HARD-01 | --contention 50 clients yields deterministic 409s | integration | `cargo test -p reposix-swarm -- contention` | ❌ Wave 0 |
| HARD-01 | win_count == final version in audit | integration | same | ❌ Wave 0 |
| HARD-02 | list_issues_strict returns Err when cap hit | unit | `cargo test -p reposix-confluence -- truncat` | ❌ Wave 0 |
| HARD-02 | --no-truncate CLI flag wires to strict mode | integration | `cargo test -p reposix-cli -- no_truncate` | ❌ Wave 0 |
| HARD-03 | kill-9 sim mid-swarm: no torn rows in WAL DB | chaos integration | `cargo test -p reposix-swarm -- chaos --ignored` | ❌ Wave 0 |
| HARD-04 | macOS FUSE integration tests pass on macos-latest | CI matrix | CI only (macos-latest runner) | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --workspace --locked`
- **Per wave merge:** `cargo test --workspace --release --locked`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `crates/reposix-swarm/src/contention.rs` — ContentionWorkload for HARD-01
- [ ] `crates/reposix-swarm/tests/contention_e2e.rs` — integration test for HARD-01
- [ ] Unit test in `crates/reposix-confluence/src/lib.rs` — strict truncation for HARD-02
- [ ] `crates/reposix-swarm/tests/chaos_audit.rs` — chaos kill-9 test for HARD-03 (ignored by default)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | n/a |
| V3 Session Management | no | n/a |
| V4 Access Control | no | n/a |
| V5 Input Validation | yes | Tainted<T> wrapper already in use; contention workload must not echo issue body to sim without sanitization |
| V6 Cryptography | no | n/a |

### Known Threat Patterns for this Phase

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Tenant name in log URL | Information Disclosure | `reqwest::Url::path_and_query()` — log path only |
| Chaos test spawning arbitrary binary | Elevation of Privilege | Hard-code binary path to `target/release/reposix-sim`; do not accept path from env/config |
| Credential token in pre-push test fixtures | Info Disclosure | Existing hook already uses fake-token patterns; test fixtures must use clearly-fake prefixes |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `gythialy/macfuse` v1 action installs macFUSE on `macos-14` Sonoma runners without SIP approval | macOS CI Matrix | macOS CI job fails; pin to specific runner image + verify action docs |
| A2 | `macos-latest` on GitHub Actions maps to macOS 14 as of 2026-04 | macOS CI Matrix | If it maps to macOS 15, kext approval flow may differ |
| A3 | `std::process::Child::kill()` sends SIGKILL on Linux (not SIGTERM) | Chaos test | If SIGTERM, sim has time to clean up and WAL chaos is not tested |

Note on A3: Rust docs state `Child::kill()` sends SIGKILL on Unix. [ASSUMED from
Rust stdlib docs knowledge; can verify with `man 2 kill` or source code if needed]

## Open Questions

1. **Chaos test binary path**
   - What we know: chaos test must spawn `reposix-sim` as a child process
   - What's unclear: whether to hard-code `target/release/reposix-sim` or use
     `env!("CARGO_BIN_EXE_reposix-sim")` (the Cargo test helper for binary paths)
   - Recommendation: use `env!("CARGO_BIN_EXE_reposix-sim")` — it's the idiomatic
     Rust pattern for integration tests that exercise binaries and avoids path
     hard-coding

2. **--no-truncate flag placement in the CLI**
   - What we know: the `reposix list` subcommand is the user-facing surface; the
     `--backend confluence` case is where truncation can happen
   - What's unclear: whether `--no-truncate` belongs at the top-level `reposix`
     command or only on `reposix list`
   - Recommendation: add to `reposix list` subcommand only; document it as
     Confluence-specific behavior

3. **Pre-push hook test in CI**
   - What we know: `scripts/hooks/test-pre-push.sh` is not in `ci.yml`
   - What's unclear: whether Phase 21 should add it to CI (small scope addition)
     or leave it as a manual script
   - Recommendation: add as a 2-line CI step in a new `hooks` job to close the
     gap cheaply; the test takes < 5s and has no external dependencies

## Sources

### Primary (HIGH confidence)

- [VERIFIED: crates/reposix-swarm/src/sim_direct.rs] — SimDirectWorkload wildcard etag pattern
- [VERIFIED: crates/reposix-swarm/src/driver.rs] — run_swarm factory pattern
- [VERIFIED: crates/reposix-swarm/src/main.rs] — Mode enum + CLI structure
- [VERIFIED: crates/reposix-swarm/tests/mini_e2e.rs] — spawn_sim + audit_row_count patterns
- [VERIFIED: crates/reposix-sim/src/routes/issues.rs lines 370-381] — If-Match 409 implementation
- [VERIFIED: crates/reposix-confluence/src/lib.rs lines 763-769] — 500-page cap implementation
- [VERIFIED: crates/reposix-confluence/src/lib.rs lines 547-578] — ingest_rate_limit + 429 log (no tenant URL in the warn itself)
- [VERIFIED: crates/reposix-confluence/tests/contract.rs] — 3 SSRF tests present and passing
- [VERIFIED: scripts/hooks/pre-push] — credential hygiene hook implementation
- [VERIFIED: scripts/hooks/test-pre-push.sh] — 6-case hook test suite
- [VERIFIED: .github/workflows/ci.yml] — current CI matrix (Linux only, no macOS)
- [VERIFIED: cargo test --workspace output] — baseline 318 tests; SSRF test 3 passes as non-ignored

### Tertiary (LOW confidence / assumed)

- [ASSUMED] `gythialy/macfuse` GitHub Action for macOS FUSE installation
- [ASSUMED] `std::process::Child::kill()` = SIGKILL on Unix (standard Rust stdlib behavior)
- [ASSUMED] `macos-latest` runner = macOS 14 Sonoma on GitHub Actions as of 2026-04

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new deps needed; all existing patterns verified
- Architecture: HIGH — swarm/sim codebase fully read; If-Match path confirmed
- Pitfalls: HIGH — derived directly from code inspection of existing patterns
- macOS CI: MEDIUM — action name and approach are well-known but version/runner details assumed

**Research date:** 2026-04-15
**Valid until:** 2026-05-15 (stable codebase; macOS runner image may drift faster)
