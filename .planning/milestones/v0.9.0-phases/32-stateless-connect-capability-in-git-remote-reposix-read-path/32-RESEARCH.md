# Phase 32 — Research: `stateless-connect` capability in `git-remote-reposix` (read path)

**Date:** 2026-04-24
**Requirements:** ARCH-04, ARCH-05
**Source artifacts consulted:**
- `.planning/phases/32-.../32-CONTEXT.md` (locked decisions)
- `.planning/research/v0.9-fuse-to-git-native/architecture-pivot-summary.md` §3
- `.planning/research/v0.9-fuse-to-git-native/partial-clone-remote-helper-findings.md`
- `.planning/research/v0.9-fuse-to-git-native/poc/git-remote-poc.py`
- `.planning/research/v0.9-fuse-to-git-native/poc/poc-helper-trace.log`
- `.planning/phases/31-.../31-VERIFICATION.md` (Phase 31 cache API)
- `crates/reposix-remote/src/main.rs`, `src/protocol.rs`

---

## 1. What we are actually building

Port the Python POC's `stateless-connect` handler to Rust in
`crates/reposix-remote/` so that:

```bash
git clone --filter=blob:none reposix::sim/proj-1 /tmp/clone
git cat-file -p <issue-oid>
git sparse-checkout set 'issues/PROJ-24*'
git checkout main
```

all work end-to-end, with blobs lazy-fetched on demand through the
helper. Push via `export` must keep working (hybrid helper).

**Implementation pattern:** Tunnel (NOT synthesis). The helper is a thin
pkt-line pipe between git's stdin/stdout and a
`git upload-pack --stateless-rpc` subprocess running against the Phase
31 `reposix-cache` bare repo.

The helper does NOT make REST calls in the read path. REST translation
happens at the next layer down (`reposix-cache::Cache::build_from` and
`Cache::read_blob`, delivered in Phase 31).

---

## 2. Wire protocol — the bytes on the wire

### 2.1 Helper-protocol handshake (text, line-based, binary-safe)

```
git  -> helper : capabilities\n
helper -> git  : import\nexport\nrefspec refs/heads/*:refs/reposix/*\nstateless-connect\nobject-format=sha1\n\n
git  -> helper : stateless-connect git-upload-pack\n
helper -> git  : \n                       (single empty line = "ready")
helper -> git  : <advertisement bytes from `upload-pack --advertise-refs --stateless-rpc`>
```

After the ready newline, git expects the service's bytes to start
**immediately**. The advertisement is the initial, unsolicited v2 stream.

### 2.2 Protocol-v2 RPC loop (binary, pkt-line framed)

Each RPC turn:

```
git  -> helper : <pkt-line stream, terminated by flush 0000>
helper -> git  : <response bytes from `upload-pack --stateless-rpc`> 0002
```

The helper re-spawns `upload-pack --stateless-rpc` **once per RPC turn**.
That is what "stateless" means in the capability name — there is no
long-lived upload-pack across turns.

### 2.3 pkt-line frame format

4-byte ASCII hex length header + payload:

| Header (hex) | Meaning |
|---|---|
| `0000` | flush |
| `0001` | delim (v2 uses this for request/response section boundaries) |
| `0002` | response-end (v2 only, terminates a helper `stateless-connect` response) |
| `0004` | empty-data |
| `000e..` | data packet; length includes the 4-byte header |

Our framer must pass through all four special shorts (`0000`, `0001`,
`0002`) without wrapping them in another header.

---

## 3. The three protocol gotchas — locked regression tests

From `partial-clone-remote-helper-findings.md` Q2 and
`architecture-pivot-summary.md` §3:

| # | Gotcha | Symptom if wrong | Fix |
|---|---|---|---|
| 1 | Initial advertisement ends with flush `0000` only — NOT response-end `0002`. | `fatal: expected flush after ref listing` on first request. | Send `upload-pack --advertise-refs --stateless-rpc` output verbatim; do NOT append `0002`. |
| 2 | Subsequent RPC responses DO need trailing `0002`. | Helper hangs or git misframes the next request. | After each response, write `upload-pack --stateless-rpc` output followed by `b"0002"`. |
| 3 | Binary stdin throughout. | Helper hangs after handshake (`readline()` over-reads). | In Rust: pull pre-handshake text lines via a single `BufReader<Stdin>` with `read_until(b'\n')`. Do NOT mix `read_line()` with raw reads on separate buffers. |

Concrete test names (per CONTEXT.md):

- `initial_advertisement_ends_with_flush_only` (gotcha 1)
- `rpc_response_appends_response_end` (gotcha 2)
- `stdin_is_binary_throughout` (gotcha 3)
- `refspec_namespace_is_reposix`
- `capability_advertisement_lists_stateless_connect`

---

## 4. Rust port plan — module layout

Two new source files inside `crates/reposix-remote/src/`:

### 4.1 `src/pktline.rs` — pkt-line frame I/O

- `read_frame<R: Read>(r: &mut R) -> io::Result<Frame>` where
  `enum Frame { Flush, Delim, ResponseEnd, Data(Vec<u8>) }`.
- `encode_frame(frame: &Frame, out: &mut Vec<u8>)` for round-trip tests.
- Tests: round-trip flush/delim/response-end/data; rejects bad hex;
  short-read is `ErrorKind::UnexpectedEof`.

### 4.2 `src/stateless_connect.rs` — the tunnel handler

Entry point:

```rust
pub fn handle_stateless_connect<R, W>(
    proto: &mut Protocol<R, W>,
    state: &mut State,
    service: &str,
) -> anyhow::Result<()>
```

Flow:

1. Resolve the cache bare-repo path via `reposix_cache::Cache::open`
   using `state.backend`, `state.backend_name`, `state.project`. Audit
   row `op='helper_connect'`.
2. Call `cache.build_from().await` to ensure tree + refs are present
   before `upload-pack` reads the repo. (Idempotent — re-runs compute
   the same tree OID, though we may skip the rebuild if the caller
   sets `REPOSIX_CACHE_PREBUILT=1` in tests. **Decision:** always call
   `build_from` — determinism > speed for v0.9.0. Cache-skip is a
   Phase 33 optimisation.)
3. Reject any `service != "git-upload-pack"` with a stderr message and
   a non-zero exit (per POC bug for `git-upload-archive` — we just
   error out cleanly).
4. Write `\n` (empty line = "ready").
5. Call `send_advertisement(&cache_path, proto)`:
   - Runs `git upload-pack --advertise-refs --stateless-rpc <path>`
     with `GIT_PROTOCOL=version=2`.
   - Writes stdout verbatim to `proto` writer.
   - No trailing `0002` (gotcha 1).
6. Enter RPC loop `loop { proxy_rpc(...) }`:
   - Read pkt-line frames from git's stdin until flush.
   - Re-encode into a `Vec<u8>` buffer.
   - Count `want` lines on the fly (parse payload bytes starting with
     `b"want "`); stash count in `state.last_fetch_want_count`.
   - If buffer is empty (pure EOF), break.
   - Spawn `git upload-pack --stateless-rpc <path>` with the buffer
     on stdin; capture stdout.
   - Write response bytes to `proto`, then `b"0002"` (gotcha 2).
   - Flush.
   - Audit row: `op='helper_fetch'`, `meta` = JSON
     `{ "want_count": N, "request_bytes": M, "response_bytes": K,
       "command": "fetch"|"ls-refs"|... }`.

### 4.3 Integration into `main.rs` dispatch

In the existing match on `cmd`:

```rust
"capabilities" => {
    proto.send_line("import")?;
    proto.send_line("export")?;
    proto.send_line("refspec refs/heads/*:refs/reposix/*")?;
    proto.send_line("stateless-connect")?;
    proto.send_line("object-format=sha1")?;
    proto.send_blank()?;
    proto.flush()?;
}
// existing "option", "list", "import", "export" ...
cmd if cmd == "stateless-connect" || cmd.starts_with("stateless-connect ") => {
    // Parse service name from line
    let service = trimmed.strip_prefix("stateless-connect ")
        .unwrap_or("").trim();
    handle_stateless_connect(&mut proto, &mut state, service)?;
    // After stateless-connect returns, we EXIT — git spawns a fresh
    // helper for every new transport operation per POC trace.
    return Ok(!state.push_failed);
}
```

The `state` struct gains:

```rust
struct State {
    rt: Runtime,
    backend: Arc<dyn BackendConnector>,
    backend_name: String,   // NEW — needed to build Cache
    project: String,
    push_failed: bool,
    last_fetch_want_count: u32, // instrument; Phase 34 enforces limit
}
```

---

## 5. Binary stdin discipline (gotcha 3)

Rust equivalent of the Python POC's "binary throughout" is already
correct in `main.rs` — `Protocol::new(stdin_handle.lock(), ...)` wraps
the stdin handle in a `BufReader<StdinLock>` and `read_line()` uses
`read_line` on the `BufReader` (which is binary-under-the-hood, with
UTF-8 validation via `String`).

The **risk** is that after the `stateless-connect` verb is read, we
hand the remaining bytes from the SAME `BufReader` to the pkt-line
parser. We must use `Protocol::read_raw_line` (or add
`Protocol::read_bytes_exact`) on the same `BufReader` — DO NOT create
a second `BufReader` on `stdin()` after the handshake, because the
first one may have buffered part of the pkt-line stream.

**Implementation note:** the cleanest shape is to give
`handle_stateless_connect` a mutable reference to the existing
`Protocol` and let it drain frames via a thin adapter that calls
`BufReader::read_exact` on the internal reader through a new
`Protocol::read_exact_bytes` method. That adapter is added to
`src/protocol.rs`.

### Test for gotcha 3

```rust
#[test]
fn stdin_is_binary_throughout() {
    // Craft input: handshake lines followed by raw pkt-line bytes that
    // contain a 0x00 byte. Parse back: every byte must round-trip.
    let mut input = Vec::new();
    input.extend_from_slice(b"capabilities\n");
    input.extend_from_slice(b"stateless-connect git-upload-pack\n");
    input.extend_from_slice(b"0009abc\x00\x00\x000000"); // data frame w/ NULs then flush
    // After handshake, we should read those bytes as one data frame
    // containing b"abc\x00\x00\x00" (5 payload bytes) then flush.
    // ...
}
```

---

## 6. Cache bridge — how the helper reaches `reposix-cache`

Phase 31 API reminder:

```rust
reposix_cache::Cache::open(
    backend: Arc<dyn BackendConnector>,
    backend_name: impl Into<String>,
    project: impl Into<String>,
) -> Result<Self>

Cache::build_from(&self) -> Result<gix::ObjectId>  // async
Cache::read_blob(&self, oid: gix::ObjectId) -> Result<Tainted<Vec<u8>>>  // async
Cache::repo_path(&self) -> &Path
```

Wiring in the helper:

- In `real_main()`: after parsing the remote URL, open the cache:
  ```rust
  let cache = reposix_cache::Cache::open(
      backend.clone(),
      spec.origin.as_ref(),   // backend name like "sim"
      spec.project.as_str(),
  )?;
  state.cache = Some(cache);
  ```
- In `handle_stateless_connect`:
  ```rust
  state.rt.block_on(state.cache.as_ref().unwrap().build_from())?;
  let repo_path = state.cache.as_ref().unwrap().repo_path();
  // spawn upload-pack against repo_path
  ```

**We do NOT call `Cache::read_blob` from the helper in this phase.** The
lazy-blob materialization path lives inside `reposix-cache` itself and
is triggered from within `upload-pack` via gix's ODB hooks... wait —
that is wrong. Re-read findings:

Actually, re-reading: in the Tunnel pattern, when `upload-pack` walks
the tree and finds a missing blob, it will fail unless the blob is
present in the bare repo. The POC avoids this because its bare repo
has all blobs already (normal git workflow). For reposix, we need the
**promisor-of-promisor** trick: the cache bare repo must itself be
configured as a partial clone (with its own promisor = nothing = blobs
just missing, served from REST on demand).

Checking phase 31 verification: `no_blob_objects_after_build_from` —
blobs are not materialized during `build_from`. So the tree references
blob OIDs that are absent from `.git/objects/`. When `upload-pack`
serves a fetch with `filter blob:none`, it should NOT need to read
blob content — tree + commit objects only go into the pack.

But when a client does a follow-up fetch for specific OIDs (lazy blob
fetch), `upload-pack` WILL try to stream those blobs → miss → error.

**Resolution for Phase 32:** For this phase's scope, we rely on
`Cache::read_blob` being called by the helper **before invoking
upload-pack** when we detect a `want <oid>` request that names a
specific blob OID. The RPC loop:

1. Read the full pkt-line request into a buffer.
2. Parse `want <oid>` lines and attempt `cache.read_blob(oid).await`
   for each that isn't already present in the bare repo's ODB.
   `read_blob` materializes the blob into the bare repo (writing to
   `.git/objects/`) as a side effect, then returns the bytes.
3. Spawn `upload-pack` with the request buffer on stdin. Now it finds
   every blob it needs.

This is the "blob pre-materialization" step — the simplest correct
approach and matches how Phase 31 engineered `read_blob`
(materialize-and-return).

Is this expensive? For a `--filter=blob:none` initial clone, no wants
refer to blob OIDs (wants refer to the HEAD commit). For
`command=ls-refs`, no wants at all. For lazy-fetch `cat-file -p <oid>`,
one want → one `read_blob` → one REST call. For sparse-checkout batch,
N wants → N `read_blob` calls serially. Good enough for v0.9.0; Phase
34 adds the blob-limit guardrail.

**For want OIDs that aren't blobs (commits, trees, annotated tags),
`Cache::read_blob` would fail to find them as blobs.** Handle this
gracefully: treat `read_blob` errors as non-fatal ("probably a
commit/tree, let upload-pack resolve it"). Log at debug level.

### Classifying want OIDs

Approach:

```rust
for oid in want_oids {
    // Try to peek the object type in the ODB:
    match repo_find_object_type(&cache, oid) {
        Ok(gix::object::Kind::Blob) | Err(NotFound) => {
            // Blob we may need to materialize, OR any missing object -
            // try read_blob and silently swallow "wrong kind" errors.
            let _ = cache.read_blob(oid).await;
        }
        Ok(_) => { /* commit/tree/tag already present */ }
    }
}
```

A simpler v0.9.0 cut: always call `read_blob`, ignore any error, let
`upload-pack` do the real work. The audit trail captures failures via
the normal error-logging path.

---

## 7. Audit logging surface (per OP-3 "Audit log is non-optional")

Phase 31 already provides `audit_events_cache` table with append-only
triggers. Rows this phase writes (via the existing `audit::log_*`
helpers or a new `log_helper_*` helper in `reposix-cache/src/audit.rs`):

| op | When | Meta (JSON) |
|---|---|---|
| `helper_connect` | `handle_stateless_connect` entered | `{ "service": "git-upload-pack", "caller": "remote" }` |
| `helper_advertise` | advertisement sent | `{ "bytes": N }` |
| `helper_fetch` | each RPC turn completes | `{ "command": "fetch"\|"ls-refs"\|..., "want_count": N, "request_bytes": M, "response_bytes": K }` |
| `helper_fetch_error` | upload-pack non-zero exit | `{ "exit_code": N, "stderr_tail": "..." }` |

The Phase 31 `audit.rs` exposes generic `log_event(db, op, meta_json)`
primitives we can call directly; extending it with helper-specific
functions is a two-line add.

---

## 8. Binary test harness for end-to-end (integration tests)

Integration tests live in `crates/reposix-remote/tests/stateless_connect.rs`:

### 8.1 Prerequisites

- A running simulator (or a `SimBackend` direct). We reuse the in-process
  sim pattern used by `crates/reposix-cache/tests/common/`.
- A temporary cache directory (set `REPOSIX_CACHE_DIR` to tempdir).
- Real system `git` binary. On dev host: git 2.25.1 does NOT support
  `filter` over protocol v2. The integration test that exercises
  `--filter=blob:none` must be `#[ignore]` by default and gated behind
  a feature or an env check for `git --version >= 2.27`.

### 8.2 Minimum viable smoke test (works on all git versions)

```rust
#[test]
fn helper_advertises_stateless_connect_capability() {
    // Run the helper binary directly via assert_cmd with "capabilities"
    // on stdin. Assert stdout includes "stateless-connect" and
    // "refspec refs/heads/*:refs/reposix/*" and "object-format=sha1".
}
```

### 8.3 Full clone test (gated)

```rust
#[test]
#[cfg_attr(not(feature = "integration-git"), ignore)]
fn partial_clone_against_sim_is_lazy() {
    // 1. Start in-process sim; seed 3 issues.
    // 2. Point REPOSIX_CACHE_DIR at tempdir.
    // 3. Prepend target/debug to PATH so git finds git-remote-reposix.
    // 4. Run: git clone --filter=blob:none --no-checkout
    //            reposix::sim/proj-1 /tmp/clone
    // 5. Assert exit 0.
    // 6. Run: git -C /tmp/clone rev-list --objects --missing=print --all
    //    Assert every issue blob line has a leading "?".
    // 7. Run: git -C /tmp/clone cat-file -p <issue-1-blob-oid>
    //    Assert exit 0 and content matches fixture.
    // 8. Run same cat-file again; assert audit row count for
    //    op='materialize' equals 1 (idempotent).
}
```

Dev-host git 2.25.1: gate with a runtime check, skip with a `println!`
if unsupported. The CI alpine job (git 2.52) will actually run it.

### 8.4 Sparse batching test (gated)

```rust
#[test]
#[cfg_attr(not(feature = "integration-git"), ignore)]
fn sparse_checkout_batches_wants() {
    // Clone; sparse-checkout set 'issues/*' ; checkout main.
    // Assert exactly ONE audit row with op='helper_fetch'
    // and want_count = N (matching the set of issues).
}
```

### 8.5 Push regression

`tests/protocol.rs` already covers export. No changes needed — we must
not regress.

---

## 9. Port-specific decisions — idiomatic Rust over POC semantics

| POC (Python) | Rust port |
|---|---|
| `subprocess.run(["git", "upload-pack", ...], capture_output=True, env=env)` | `std::process::Command::new("git").args(...).env("GIT_PROTOCOL","version=2").output()?` — synchronous is fine; this is inside the single-threaded runtime and blocks anyway. Avoid `tokio::process::Command` unless async buys something. |
| `sys.stdin.buffer.read(1)` byte-by-byte | `BufReader::read_until(b'\n', ...)` and `read_exact` — no per-byte reads. |
| `proc.communicate()` footgun | `Command::output()` in Rust does the right thing. |
| `STDOUT.write(b"0002")` | `proto.send_raw(b"0002")` via existing method. |
| `log()` to stderr + optional file | `tracing::debug!` (existing subscriber writes to stderr via `with_writer(std::io::stderr)`). No env-var log file — audit log is the persistence layer. |

---

## 10. POC bugs to NOT port

From `push-path-stateless-connect-findings.md`:

1. **Empty-delta refspec bug** (fixed in POC and existing helper):
   `refs/heads/*:refs/heads/*` → must be `refs/heads/*:refs/reposix/*`.
   The existing main.rs already uses `refs/reposix/*`. Regression test
   asserts this.

2. **`line.startswith("commit ")`** — not relevant to stateless-connect
   (fast-export parsing bug; Phase 32 doesn't touch `fast_import.rs`).

3. **Python `proc.communicate()` after `stdin.close()`** — not
   applicable to Rust's `Command::output()`.

---

## 11. File manifest (what gets touched)

- **New:** `crates/reposix-remote/src/pktline.rs`
- **New:** `crates/reposix-remote/src/stateless_connect.rs`
- **Edit:** `crates/reposix-remote/src/main.rs` — add capability lines,
  dispatch arm, `State.backend_name`, `State.cache`,
  `State.last_fetch_want_count`.
- **Edit:** `crates/reposix-remote/src/protocol.rs` — add
  `read_exact_bytes` method (or expose inner BufReader).
- **Edit:** `crates/reposix-remote/Cargo.toml` — add
  `reposix-cache = { path = "../reposix-cache" }` dependency.
- **Edit:** `crates/reposix-cache/src/audit.rs` — add
  `log_helper_connect`, `log_helper_advertise`, `log_helper_fetch`
  helpers (one-liner wrappers around existing `log_event`).
- **New:** `crates/reposix-remote/tests/stateless_connect.rs` — unit
  tests for the three gotchas + capability advertisement.
- **New (gated):** integration test at the same file or sibling,
  `#[cfg_attr(not(feature = "integration-git"), ignore)]`.
- **New:** `.planning/research/v0.9-fuse-to-git-native/rust-port-trace.log`
  — captured from an actual run of the Rust helper (OP-1 feedback-loop
  artifact).

---

## 12. Sizing estimate

- `pktline.rs`: ~120 LOC + ~60 LOC tests.
- `stateless_connect.rs`: ~200 LOC + ~80 LOC tests.
- `main.rs` edits: ~30 LOC added.
- `protocol.rs` edits: ~15 LOC added.
- `audit.rs` edits: ~30 LOC added.
- Integration test: ~120 LOC.

Total new code: ~ 650 LOC. Fits CONTEXT.md "~200 lines" estimate for
the core tunnel, plus pktline lib + tests.

---

## 13. Threat-model touch points (per CLAUDE.md "Threat model")

- **Outbound HTTP allowlist.** Not newly triggered in this phase — the
  helper doesn't open HTTP in the read path; the cache backend does,
  and it already enforces `REPOSIX_ALLOWED_ORIGINS`.
- **No shell escape from FUSE writes.** Not applicable (we're deleting
  FUSE); `upload-pack` input bytes flow through a `Command` with no
  shell.
- **Tainted-by-default.** All response bytes from `upload-pack` are
  tainted (they're derived from REST responses). The helper writes
  them to stdout which flows to git, which stores them in `.git/objects`
  — the working tree ends up materializing tainted content. This is
  the intended design: the mount point IS a git checkout of tainted
  data.
- **Audit log append-only.** Phase 31 triggers already enforce. No
  change needed.

---

## 14. Open questions (deferred, not blocking)

1. Should we pre-warm `Cache::build_from` lazily or eagerly on helper
   startup? **Decision:** eagerly in `handle_stateless_connect` only
   (not on every invocation of the helper — `capabilities`,
   `list`, and `export` don't need the bare repo's refs to be current).
2. Multi-repo concurrency — two `git` processes invoking the helper
   simultaneously. Phase 31's cache uses a SQLite WAL + `gix` which is
   process-safe. Not revisiting here.
3. `upload-pack` binary discovery — assume `git` is on `PATH`. The
   `reposix init` command (Phase 35) will document the requirement.

---

## RESEARCH COMPLETE

Deliverables: three gotchas locked with named tests, cache-bridge flow
documented, file manifest sized, port-specific idiomatic-Rust decisions
recorded, POC bugs identified for NO-PORT. Ready for planning.
