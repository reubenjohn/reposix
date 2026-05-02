← [back to index](./index.md)

# Task 02-T04 — Wire helper to call `Cache::sync` on `stateless-connect`

<read_first>
- `crates/reposix-remote/src/stateless_connect.rs` (esp. lines 93-100)
- `crates/reposix-remote/src/main.rs:194-202` — `ensure_cache`
</read_first>

<action>
Edit `crates/reposix-remote/src/stateless_connect.rs` around line 98. Replace:

```rust
rt.block_on(cache.build_from())
    .context("cache.build_from before upload-pack tunnel")?;
```

with:

```rust
// Delta sync: on the first invocation in a fresh cache, sync() falls
// through to build_from internally (no last_fetched_at → seed). On
// subsequent invocations it queries the backend with the stored
// cursor and applies only the delta. Either way, tree + refs are
// up-to-date by the time we advertise to git.
let report = rt.block_on(cache.sync())
    .context("cache.sync before upload-pack tunnel")?;
tracing::debug!(
    changed = report.changed_ids.len(),
    since = ?report.since,
    "delta sync complete"
);
```

No other changes to the helper are needed — the audit row is written inside `cache.sync()`'s atomic transaction.
</action>

<acceptance_criteria>
- `cargo build -p reposix-remote` exits 0.
- `cargo clippy -p reposix-remote --all-targets -- -D warnings` exits 0.
- `grep -n 'cache.sync' crates/reposix-remote/src/stateless_connect.rs` matches once.
- `grep -n 'cache.build_from' crates/reposix-remote/src/stateless_connect.rs` returns no matches (the direct call is now inside `cache.sync`'s seed path).
</acceptance_criteria>

<threat_model>
The helper now has ONE tree-sync entry point (`cache.sync`) so the audit trail is uniform: every invocation that reaches the tunnel writes either a `tree_sync` (seed) or `delta_sync` (incremental) row. No code path bypasses the audit.
</threat_model>
