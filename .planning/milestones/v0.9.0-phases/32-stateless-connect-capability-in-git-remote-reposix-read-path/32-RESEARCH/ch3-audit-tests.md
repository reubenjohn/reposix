# Phase 32 Research — Audit Logging & Tests

← [back to index](./index.md)

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
