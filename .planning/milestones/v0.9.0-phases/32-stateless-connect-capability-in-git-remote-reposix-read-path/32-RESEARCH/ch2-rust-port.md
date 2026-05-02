# Phase 32 Research — Rust Port Plan

← [back to index](./index.md)

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
