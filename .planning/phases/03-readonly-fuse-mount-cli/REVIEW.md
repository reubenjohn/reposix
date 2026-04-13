---
phase: 02-simulator-audit-log + 03-readonly-fuse-mount-cli
reviewed: 2026-04-13
reviewer: gsd-code-reviewer (Claude Opus 4.6 1M)
depth: deep
files_reviewed: 17
scope:
  sim:
    - crates/reposix-sim/src/lib.rs
    - crates/reposix-sim/src/main.rs
    - crates/reposix-sim/src/state.rs
    - crates/reposix-sim/src/db.rs
    - crates/reposix-sim/src/seed.rs
    - crates/reposix-sim/src/error.rs
    - crates/reposix-sim/src/middleware/audit.rs
    - crates/reposix-sim/src/middleware/rate_limit.rs
    - crates/reposix-sim/src/routes/issues.rs
    - crates/reposix-sim/src/routes/transitions.rs
    - crates/reposix-sim/tests/api.rs
    - crates/reposix-sim/fixtures/seed.json
    - crates/reposix-core/src/http.rs (request_with_headers*)
  fuse:
    - crates/reposix-fuse/src/lib.rs
    - crates/reposix-fuse/src/main.rs
    - crates/reposix-fuse/src/fs.rs
    - crates/reposix-fuse/src/fetch.rs
    - crates/reposix-fuse/src/inode.rs
    - crates/reposix-fuse/tests/readdir.rs
    - crates/reposix-fuse/tests/sim_death_no_hang.rs
  cli:
    - crates/reposix-cli/src/main.rs
    - crates/reposix-cli/src/demo.rs
    - crates/reposix-cli/src/mount.rs
    - crates/reposix-cli/src/sim.rs
    - crates/reposix-cli/src/binpath.rs
    - crates/reposix-cli/tests/cli.rs
findings:
  blocker: 0
  high: 3
  medium: 4
  low: 6
  total: 13
verdict: FIX-REQUIRED
---

# Phase 2 + 3 Code Review — Combined

## Summary verdict: FIX-REQUIRED

No BLOCKERs (no security regressions, no broken security invariants). The
HTTP allowlist gate (SG-01), frontmatter-field stripping (SG-02), filename
validation (SG-04), audit append-only triggers (SG-06), and the 5-second
FUSE kernel cap (SG-07) are all intact and covered by tests.

However, there is one **HIGH-severity demo-fidelity bug** that breaks a
user-facing PROJECT.md requirement ("Demo recording must show guardrails
firing") and two **HIGH-severity correctness bugs** in the CLI surface:

1. `reposix demo` step 5 ("tail audit rows") is effectively dead code —
   it ALWAYS early-returns with a misleading "audit DB not yet flushed"
   message because the sim runs `--ephemeral` (in-memory) and the
   hardcoded DB path never materialises on disk. The column name in the
   query is ALSO wrong (`agent` vs `agent_id`) so even if the DB existed
   the query would hard-fail.

2. `reposix sim --no-seed` and `reposix sim --rate-limit N` are silently
   ignored — clap parses them but the match arm in `main.rs` destructures
   both with `: _` without forwarding them to `sim::run`.

3. The audit middleware does not survive a handler panic — if any
   downstream handler panics (even via a library-level precondition in
   axum/rusqlite), the audit INSERT is skipped, leaving an "invisible"
    5xx in the audit trail. SG-06 says the log is the ground truth; a
   panic-shaped hole violates that spirit.

These are all fixable in <60 LOC each. Phase 4 demo recording should not
proceed until #1 is resolved, because the canonical walkthrough will
otherwise lie about what it is showing.

Everything else is MEDIUM or LOW (DoS surfaces under untrusted-agent
spoofing, tmp-file leaks, RFC 7232 wildcard non-compliance, misleading
comments). No BLOCKER class findings.

---

## Findings

### H-01 — `reposix demo` audit-tail step never actually runs (HIGH)

**File:** `crates/reposix-cli/src/demo.rs:180-211`

**Issue.** `reposix demo` orchestrates sim + mount + scripted ls/cat/grep
and claims to "tail last 5 audit rows" in step 5. In practice:

1. The demo spawns the sim via `SimProcess::spawn_ephemeral(...)`, which
   always passes `--ephemeral` to `reposix-sim`. That flag forces
   `Connection::open_in_memory()` (see `crates/reposix-sim/src/db.rs:50`
   and `lib.rs:103-105`). The `--db <path>` argument is retained but
   ignored.

2. Therefore the file `runtime/demo-sim-<pid>.db` **never exists**.

3. `print_audit_tail` checks `if !db.exists() { return Ok(()); }` first
   and always early-returns with `"(audit DB not yet flushed to disk)"`.
   The message is false; the DB is not "not yet flushed" — it is
   permanently in-process and will never persist.

4. Even if the DB did exist on disk, the query
   ```sql
   SELECT ts, agent, method, path, status FROM audit_events ...
   ```
   references a column `agent` that does not exist in the schema
   (`crates/reposix-core/fixtures/audit.sql:18` defines `agent_id`, and
   the audit middleware writes `agent_id` at
   `crates/reposix-sim/src/middleware/audit.rs:119`). `prepare` would
   fail with `no such column: agent`.

**Impact.** This is a direct violation of the PROJECT.md "Demo-ready by
2026-04-13 morning" requirement that includes "Demo recording must show
guardrails firing. [...] A demo that only shows happy-path is dishonest
about what reposix is." The asciinema recording will show step 5's info
banner followed by a misleading line, never showing any audit row.

**Fix recommendation.** Pick one:

- **Preferred**: switch `spawn_ephemeral` → `spawn` in `demo.rs:57` for
  the demo path, so the sim persists to `runtime/demo-sim-<pid>.db`. WAL
  sibling files (`*-wal`, `*-shm`) need cleanup — add a best-effort
  `std::fs::remove_file` for each in `Guard::drop`. Also fix the column
  name to `agent_id` and add a regression test that opens the fixture DB
  and asserts the query prepares OK against a fresh `db::open_db` output.
- **Alternative**: keep `--ephemeral` but add a new
  `GET /audit?limit=N` sim endpoint and have the demo HTTP-GET it. More
  work but avoids the cross-process file-DB round trip.

Either way: delete or fix the current `print_audit_tail` — a function
that never actually executes its body is a lie the reader of the code
has to untangle.

---

### H-02 — `reposix sim --no-seed` and `--rate-limit` are silently ignored (HIGH)

**File:** `crates/reposix-cli/src/main.rs:100-107`

**Issue.**
```rust
Cmd::Sim {
    bind, db, seed_file,
    no_seed: _,        // <-- dropped
    ephemeral,
    rate_limit: _,     // <-- dropped
} => sim::run(&bind, db, seed_file, ephemeral),
```

`sim::run` takes `(bind, db, seed, ephemeral)` — no rate_limit, no
no_seed parameter. A user running `reposix sim --no-seed` gets the seed
loaded anyway; `--rate-limit 10` gets the default 100 rps. Both flags
appear in `--help` so a user has no indication they're no-ops.

**Impact.** Broken CLI contract. On a demo where the user types
`reposix sim --rate-limit 1` to demonstrate 429s, they'll see no 429s
and wonder why. Also a regression risk: if a user runs
`reposix sim --no-seed` expecting a clean DB, they get the adversarial
`<script>alert(1)</script>` seed body instead.

**Fix recommendation.** Plumb both through:
```rust
Cmd::Sim { bind, db, seed_file, no_seed, ephemeral, rate_limit }
    => sim::run(&bind, db, seed_file, no_seed, ephemeral, rate_limit),
```
Update `sim::spawn` / `spawn_ephemeral` to accept `rate_limit: u32` and
a `seed: Option<&Path>` that is set to `None` when `no_seed=true` (it's
already `Option<&Path>`, just wire `no_seed` to clobber it). Add a CLI
integration test that spawns with `--rate-limit 1` and asserts the 2nd
GET returns 429.

---

### H-03 — Audit middleware skipped on handler panic (HIGH)

**File:** `crates/reposix-sim/src/middleware/audit.rs:101-131`

**Issue.** The middleware calls `next.run(rebuilt).await` and then, below
that `.await`, executes the `INSERT INTO audit_events`. If the
downstream handler panics (or any tower/axum layer below panics), the
panic unwinds through `audit_middleware` and the INSERT is never
executed. Hyper catches the panic at the task boundary and returns 500
to the client; the audit log has no record of the event.

This is also the answer to the stated scrutiny item #1 ("what happens
if the handler panics? Is the audit row still written?") — **no, it is
not**.

SG-06 is "append-only" and the Phase-1 triggers enforce that the table
cannot be rewritten. But the complementary property — "every request
produces exactly one row" — is NOT enforced at all. A panic silently
drops an event.

**Impact.** In practice, the sim's handlers are panic-free under the
inputs accepted by `deny_unknown_fields` + `ApiError`. But the sim is
also the simulation substrate for an adversarial swarm harness (FC-07,
active requirement). A fuzzing agent that learns to panic a handler
(e.g. by a hypothetical rusqlite internal bug, a corrupt WAL, or a
future `ApiError::Internal` path that calls `.unwrap()`) would evade
audit attribution. A panic is also the most likely shape of a
hot-patched exploit.

**Fix recommendation.** Wrap the downstream call in
`AssertUnwindSafe(next.run(rebuilt)).catch_unwind().await`, and on `Err`
synthesize a 500 response AND still fall through to the audit INSERT.
Alternatively, add `tower_http::catch_panic::CatchPanicLayer` INSIDE the
audit layer (i.e. attach catch-panic before audit, so audit wraps
catch-panic's converted-to-500 response). The latter is simpler:
```rust
pub fn build_router(state: AppState, rate_limit_rps: u32) -> Router {
    let handlers = /* ... */;
    let with_rate_limit = middleware::rate_limit::attach(handlers, rate_limit_rps);
    // NEW: catch-panic layer INSIDE audit.
    let with_catch = with_rate_limit.layer(tower_http::catch_panic::CatchPanicLayer::new());
    middleware::audit::attach(with_catch, state)
}
```
Add a test: a `/panic` route that calls `panic!()`, assert the response
is 500 AND an audit row with status=500 was written.

---

### M-01 — Rate-limit DashMap is unbounded (MEDIUM)

**File:** `crates/reposix-sim/src/middleware/rate_limit.rs:47-67`

**Issue.** `Arc<DashMap<String, Arc<Limiter>>>` grows one entry per
distinct `X-Reposix-Agent` header value, ever. No eviction, no cap, no
TTL. An attacker sending `X-Reposix-Agent: <random>` on each request
inflates the map until the process OOMs. At ~200 bytes/entry,
~5M unique agents ~= 1 GB. Trivially exploitable.

Defense: loopback-only allowlist + local-only threat model means the
attacker is already on the machine. But a fuzzing agent in the swarm
harness (FC-07) running the sim unsupervised for hours IS that
attacker.

**Fix recommendation.** Cap the map at N=10_000 entries. On insert,
when `map.len() >= N`, evict the oldest (track `VecDeque<String>` of
insertion order alongside the map). Or use a keyed governor directly
(`RateLimiter<String, DashMapStateStore<String>, DefaultClock>` is a
governor-native option) which handles eviction via `retain` in the
governor internals. The simplest drop-in: add a periodic task that
every 60s calls `map.retain(|_, v| v.check().is_ok())` to prune idle
buckets (since a long-idle bucket is by definition available to re-take
the token from scratch).

---

### M-02 — `X-Reposix-Agent` is unauthenticated; victim-quota attack possible (MEDIUM)

**File:** `crates/reposix-sim/src/middleware/rate_limit.rs:52-57`,
`crates/reposix-sim/src/middleware/audit.rs:67-72`

**Issue.** The agent ID is read straight from a client-controlled
header. Any caller can set `X-Reposix-Agent: reposix-fuse-12345` and:
- Consume the legitimate fuse daemon's rate-limit bucket (denial).
- Attribute malicious audit rows to the legitimate fuse daemon (frame
  job).

No auth, no HMAC, no rotating token.

**Impact.** Within the stated local-only threat model this is
acceptable, but the code pretends otherwise — the audit table schema
carries `agent_id` as if it were an identity, and other code may over-
trust it downstream. In Phase-S (write path) this becomes dangerous: if
a write handler ever reads `agent_id` from the audit table to authorize
something, the spoofing becomes a privilege escalation.

**Fix recommendation.** Short term: document the trust level in the
audit.rs module docstring and in `SG` constraints ("agent_id is a
self-claimed hint, not an authentication token"). Medium term: when
Phase-S lands, derive agent_id from a locally-authenticated source
(e.g. a per-process token emitted by `reposix` CLI at spawn) rather
than a header. Add an assertion test that writes with a mismatched
`X-Reposix-Agent` do not affect any authz decision.

---

### M-03 — `NamedTempFile` + SQLite WAL leaks `*.db-wal` / `*.db-shm` siblings (MEDIUM)

**File:** `crates/reposix-sim/tests/api.rs:32` (and elsewhere using
`NamedTempFile` for WAL-mode DBs)

**Issue.** `tempfile::NamedTempFile::new()` creates one file and removes
that one file on drop. SQLite in WAL mode creates two siblings:
`<path>-wal` and `<path>-shm`. Those are NOT cleaned up on
`NamedTempFile::drop`. Repeated integration test runs under `cargo test`
accumulate `/tmp/.tmpXXXXXX-wal` / `/tmp/.tmpXXXXXX-shm` files.

**Impact.** Low in practice (OS cleans `/tmp` on boot, CI runners are
ephemeral). But an integration-test loop or a long-running CI without
tmp rotation fills tmpfs. This is the answer to scrutiny item #5.

**Fix recommendation.** Replace the `NamedTempFile` with a `TempDir`
(`tempfile::Builder::new().prefix("reposix-sim-").tempdir()`) and
derive `db_path = tempdir.path().join("sim.db")`. `TempDir::drop` is
recursive and sweeps the wal/shm siblings. Alternative: switch the test
DB to `PRAGMA journal_mode=DELETE` (rollback journal, no siblings), at
the cost of slightly slower writes — but the test suite is tiny and
this is the one-line fix.

---

### M-04 — SIGTERM to `reposix-fuse` does NOT trigger Drop; cleanup relies solely on `fusermount3 -u` (MEDIUM)

**File:** `crates/reposix-cli/src/mount.rs:102-142`,
`crates/reposix-fuse/src/main.rs:57-59`

**Issue.** `MountProcess::watchdog_unmount` comment says
"SIGTERM the fuse child so it drops its BackgroundSession (which
triggers fuser's UmountOnDrop)". This is wrong. Rust's default SIGTERM
handler is SIG_DFL → immediate process termination. No Drop impls run.
`fuser::BackgroundSession::drop` never executes. The kernel mount would
leak if the subsequent `fusermount3 -u` call also failed (e.g. binary
missing from `PATH`).

`reposix-fuse/src/main.rs:57-59` only handles SIGINT via
`tokio::signal::ctrl_c()`. SIGTERM is not installed as a tokio signal
handler, so the process dies without unmounting cooperatively.

**Impact.** Current host has `fusermount3`, CI installs it via
`apt install fuse3`. On a fresh host without fuse3, SIGTERM from the
CLI leaks a dangling kernel mount. Also if fusermount3 PATH resolution
changes (e.g. inside a restricted `PATH` env), the 3s watchdog runs
`spawn()` → Err → "let _ = self.child.wait()" → return, leaking the
mount.

**Fix recommendation.** Two-pronged:
1. Install a SIGTERM handler in `reposix-fuse/src/main.rs`:
   ```rust
   let mut sigterm = tokio::signal::unix::signal(
       tokio::signal::unix::SignalKind::terminate())?;
   rt.block_on(async {
       tokio::select! {
           _ = tokio::signal::ctrl_c() => {},
           _ = sigterm.recv() => {},
       }
   });
   ```
   This ensures `_mount` drops on SIGTERM, running `UmountOnDrop` ->
   `fusermount3` path even if the CLI's own `fusermount3` spawn fails.
2. Fix the comment in `mount.rs:103-104` to accurately reflect that
   SIGTERM is a "best-effort kill" and the graceful path relies on
   either the child handling SIGTERM (after fix #1) or the CLI's
   subsequent `fusermount3 -u` call.

---

### L-01 — `If-Match` wildcard (`*`) not honored per RFC 7232 (LOW)

**File:** `crates/reposix-sim/src/routes/issues.rs:124-133, 368-379`

**Issue.** RFC 7232 §3.1 defines `If-Match = "*" / 1#entity-tag`. A bare
`*` means "any existing resource". Current code:
```rust
if let Some(ref raw_etag) = if_match {
    let sent_ok = raw_etag.parse::<u64>().map(|n| n == current).unwrap_or(false);
    if !sent_ok {
        return Err(ApiError::VersionMismatch { ... });
    }
}
```
`"*"` fails `u64::parse`, yields `sent_ok=false`, returns 409. Should
be an unconditional allow (the resource exists, else the earlier
`QueryReturnedNoRows → NotFound` fired).

Also: a comma-separated list (`If-Match: "1", "2"`) would all fail.
Weak-etag prefix (`W/"1"`) is rejected. Not security-relevant but
spec-non-compliant.

**Fix recommendation.** Before `parse::<u64>`, check:
```rust
if raw_etag == "*" { /* allow, fall through */ }
else if raw_etag.parse::<u64>().map(|n| n == current).unwrap_or(false) { /* allow */ }
else { return Err(VersionMismatch { ... }); }
```
Add tests: `patch_with_star_if_match_allows`, and
`patch_with_weak_etag_w_prefix_...` for documentation.

---

### L-02 — `HttpClient::request_with_headers_and_body` body attach happens after allowlist check (correctly), but the `Content-Length` header is client-controlled (LOW)

**File:** `crates/reposix-core/src/http.rs:266-293`

**Issue.** This is NOT a bug — reviewed per scrutiny item 7. The
allowlist gate (`into_url` + `load_allowlist_from_env` + `g.matches`)
fires BEFORE `builder.body(body)`. No body bytes reach
`reqwest::ClientBuilder` sockets if the origin is rejected.

BUT: a caller passing an explicit `Content-Length` in `headers` could
desynchronize the framing (reqwest would attach ANOTHER Content-Length
derived from body). reqwest should handle duplicate header dedup, but
behavior under duplicate Content-Length is undefined-ish. Worth
documenting.

**Fix recommendation.** INFO-only: add a module doc comment saying
"callers must not pass Content-Length in `headers` — reqwest computes
it from the body". Or proactively filter such headers from the slice in
the loop. No runtime symptom observed.

---

### L-03 — `sim` binary creates `runtime/sim.db` parent silently on non-ephemeral (LOW)

**File:** `crates/reposix-sim/src/main.rs:47-53`

**Issue.** `std::fs::create_dir_all(parent).ok()` silently swallows the
result. If `runtime/` is e.g. on a read-only volume, the error is
swallowed and the subsequent `Connection::open` fails with a less
descriptive message.

**Fix recommendation.** Propagate the error:
`std::fs::create_dir_all(parent).with_context(|| format!("create {:?}", parent))?;`
This is a UX tweak. Move from `.ok()` to `?` or explicit log.

---

### L-04 — Inode exhaustion is theoretically possible; server-returned `id=0` collides with reserved-range documentation, not with the root inode (LOW)

**File:** `crates/reposix-fuse/src/inode.rs:29-71`

**Issue.** Scrutiny items 3 ("id=0?", "u64 exhaustion?"):
- `IssueId(0)` → `intern` returns 0x1_0000 (first allocation). No
  collision with root (1). **Safe.**
- `AtomicU64::fetch_add(1)` wraps on u64 exhaustion. 1.8e19 allocations
  at 1M allocations/sec = 570,000 years. **Not practical.**
- The doc at line 9 says "inodes 2..=0xFFFF are reserved for future
  synthetic files". If a future phase wires an inode from that range,
  `lookup_ino` returns None (by design) and the Filesystem impl hard-
  coded the root == 1 check only. Safe.

No bug. One doc nit: the test `reserved_range_is_unmapped` enforces the
invariant; good.

**Fix recommendation.** None. Resolved as "no findings" for scrutiny
item 3.

---

### L-05 — `mount.rs` comment about SIGTERM is misleading (LOW, folded into M-04)

**File:** `crates/reposix-cli/src/mount.rs:103-104`

Already addressed in M-04.

---

### L-06 — `/healthz` counts against per-agent rate limit, with default agent = "anonymous" shared bucket (LOW)

**File:** `crates/reposix-sim/src/lib.rs:85-92`

**Issue.** `/healthz` is composed INSIDE the rate-limit layer, so every
health poll counts against the "anonymous" bucket (since no agent
header is set by the typical curl or wait_for_healthz caller). At
rps=100 default this is fine; at rps=1 (integration-test configs),
rapid poll loops can starve the real request path.

**Impact.** The existing `rate_limit_returns_429_on_overflow` test
actually leverages this (hitting /healthz to saturate). But it's a
footgun for operators: setting `rate_limit=10` with a noisy k8s
liveness probe would 429 actual requests.

**Fix recommendation.** Exclude `/healthz` from the rate-limit layer:
```rust
let (health_router, api_router) = (health_only, api_only);
let api_with_rate_limit = rate_limit::attach(api_router, rate_limit_rps);
let merged = api_with_rate_limit.merge(health_router);
audit::attach(merged, state)
```
Or set up rate_limit to short-circuit for `/healthz` based on `uri.path`.
Low severity; design choice.

---

## Scrutiny checklist — direct answers

**Sim (Phase 2):**

1. **Audit on handler panic?** → **H-03**: row is NOT written on panic.
   Fix required.
2. **2MB body → 413 audited?** → **verified**: `to_bytes(body, 1MiB)`
   returns Err → `oversize=true` → 413 response built inside
   middleware, THEN audit INSERT runs. Covered by
   `oversized_body_returns_413_and_audits` at
   `src/middleware/audit.rs:308`. PASS.
3. **If-Match parser?** → **L-01**: missing/malformed handled as
   wildcard-pass / 409-fail correctly. Literal `*` wildcard is NOT
   honored per RFC 7232. Weak etags not supported. LOW severity.
4. **transitions endpoint leak?** → **no findings**. Static
   `ALL_STATUSES` filter; no per-user state; no RBAC (because v0.1 has
   no users to gate on).
5. **tempfile WAL cleanup?** → **M-03**: `NamedTempFile` leaks
   `*-wal` / `*-shm`.
6. **Rate-limit agent spoofing?** → **M-02**: header is
   unauthenticated; victim-bucket-consumption + audit-frame-job both
   possible. Accepted within v0.1 threat model, should be documented.
7. **request_with_headers* allowlist check?** → **verified, no
   findings**. Every delegation path re-parses URL + re-checks
   allowlist before any socket I/O or body bytes. Body is attached
   AFTER the gate.

**FUSE (Phase 3):**

1. **Tokio runtime drop ordering?** → **no findings**. `Arc<Runtime>`
   inside `ReposixFs`; dropped when `Mount` drops; `BackgroundSession`
   ensures in-flight callbacks complete before the runtime joins. The
   outer CLI signal runtime is separate and unrelated.
2. **lookup calls `validate_issue_filename`?** → **verified, no
   findings**. `fs.rs:273-276` rejects invalid names with EINVAL before
   any HTTP call. `resolve_name` calls it again defensively.
3. **Inode exhaustion / id=0?** → **L-04**: no practical exhaustion;
   id=0 is safe. No findings.
4. **5s timeout covers body reads?** → **verified, no findings**. The
   reqwest-layer `total_timeout=5s` (ClientOpts default) caps the whole
   request. The defensive `tokio::time::timeout(5s, resp.bytes())` is
   belt-and-suspenders. Test `fetch_issue_times_out_within_budget`
   asserts <5.8s wall clock. No header-headers-only gap.
5. **AllowOther off?** → **verified, no findings**. `lib.rs:77-82`
   mounts with `FSName`, `Subtype`, `DefaultPermissions`, `RO` —
   neither `AllowOther` nor `AllowRoot`. `phase3_exit_check.sh` asserts
   "AllowOther not present".
6. **AutoUnmount dropped — cleanup story?** → **M-04**: `fuser::
   BackgroundSession::UmountOnDrop` runs on graceful exit. SIGTERM from
   CLI does NOT trigger Drop (Rust default SIG_DFL), so cleanup relies
   on the CLI's `fusermount3 -u` watchdog. If `fusermount3` is missing
   from PATH the mount leaks. Medium severity.

**CLI (Phase 3):**

1. **Guard field order?** → **verified, no findings**.
   `{ mount, sim, tempdir }` drops in that order. `Drop::drop` also
   explicitly takes-and-drops in the same order. Mount-before-sim is
   correct: fusermount3 -u must run before the sim dies, else the
   daemon's in-flight GETs time out.
2. **Ctrl-C before Guard drop?** → **verified, no findings**.
   `tokio::select!` awaits both body and ctrl_c; guard is declared
   outside the select; `drop(guard)` is the line after select returns.
   Drop runs on all exit paths (Ok, Err, select-via-Ctrl-C).
   Minor double-listener subtlety when `--keep-running` is set (outer
   select + inner `signal::ctrl_c().await.ok()` both listen); either
   one winning triggers the same teardown. LOW.
3. **fusermount3 not on PATH?** → **M-04** covers it. Graceful
   fallback is `self.child.wait()` which no-ops if the child is
   already dead, leaking the kernel mount.
4. **demo prints structured progress for asciinema?** → **verified**.
   Six `[step N/6] ...` lines via `tracing::info!`, captured by the
   default env-filter=info subscriber in `main.rs:88-93`. Legible on
   asciinema. **Caveat**: step 5 ("audit tail") prints its banner then
   outputs nothing substantive because of H-01. The recording would
   show the banner and then silence — awkward for a demo.

---

## Final verdict

**FIX-REQUIRED.**

Three HIGH-severity issues blocking a clean Phase 4 demo recording:
- H-01: demo step 5 is effectively dead code; the asciinema will lie.
- H-02: `reposix sim` CLI flags are broken.
- H-03: audit middleware has a panic-sized hole (SG-06 fidelity gap).

Four MEDIUM-severity issues that should ship with fixes in a follow-up
phase but do not block Phase 4 recording as long as H-01/H-02/H-03 are
resolved:
- M-01: rate-limit DashMap unbounded (DoS surface).
- M-02: `X-Reposix-Agent` is unauthenticated (threat-model doc gap).
- M-03: tempfile WAL sibling leak.
- M-04: SIGTERM → no Drop; mount cleanup fragile.

Six LOW items are either cosmetic (docs, comments) or documented-
acceptable design choices.

**No BLOCKERs** — no security regressions or broken security invariants.
SG-01 (allowlist), SG-02 (frontmatter stripping in place of server-
managed fields via `deny_unknown_fields`), SG-04 (filename validation),
SG-06 (append-only triggers), and SG-07 (5s FUSE ceiling) are all
intact and covered by tests.

Recommend: fix H-01, H-02, H-03 before Phase 4 asciinema recording.
File M-01..M-04 as tracked follow-ups in a Phase 4a hardening task.

---

_Reviewed: 2026-04-13_
_Reviewer: gsd-code-reviewer (Claude Opus 4.6 1M)_
_Depth: deep — cross-file analysis of sim ↔ core ↔ fuse ↔ cli boundaries + integration and unit tests_
