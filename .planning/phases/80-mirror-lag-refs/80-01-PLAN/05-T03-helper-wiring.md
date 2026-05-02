← [back to index](./index.md) · phase 80 plan 01

## Task 80-01-T03 — Helper crate wiring: ref writes in `handle_export` success branch + reject-hint composition + advertisement widening

<read_first>
- `crates/reposix-remote/src/main.rs` lines 280-491 (`handle_export` —
  full function; the wiring is between lines 470-489 success branch
  + lines 384-407 conflict reject branch).
- `crates/reposix-remote/src/main.rs` lines 40-71 (`State` struct —
  confirm `state.cache: Option<Cache>` + `state.backend_name: String` +
  `state.rt: tokio::runtime::Runtime`).
- `crates/reposix-remote/src/stateless_connect.rs` (entire file — find
  where ref advertisement composes; investigate whether widening to
  `refs/mirrors/*` requires a filter change or is already automatic
  via tunneling to `git upload-pack`. RESEARCH.md A3: probably one-line
  edit; possibly zero-line if tunneling preserves all refs).
- `crates/reposix-cache/src/mirror_refs.rs` (the module from T02 —
  to know the function signatures the wiring calls).
- `crates/reposix-cache/src/cache.rs` line 444+ — `log_attach_walk`
  pattern (P79; how cache audit-write is invoked from the helper).
- `crates/reposix-remote/Cargo.toml` — confirm `tracing` is in deps
  (it is — used at line 310 by the existing wiring).
</read_first>

<action>
Two concerns in this task; keep ordering: stateless-connect investigation
→ handle_export success-branch wiring + reject-hint composition →
build + commit.

### 3a. Investigate stateless-connect advertisement

Read `crates/reposix-remote/src/stateless_connect.rs` to identify how
the helper's `list` advertisement is composed for protocol-v2 fetch.
RESEARCH.md A3 anticipated this is a one-line widening; verify.

The most likely shape: the helper tunnels `command=ls-refs` traffic
through to `git upload-pack --stateless-rpc` running against the
cache's bare repo. If so, `git upload-pack` advertises all refs the
cache has — including any new `refs/mirrors/*` refs — automatically.
NO helper-side widening required.

Less likely shape: the helper does its own ls-refs filtering (e.g.,
to suppress `refs/reposix/sync/*` per the `transfer.hideRefs` setup in
`crates/reposix-cache/src/cache.rs:466-540`). If there's an explicit
allowlist filter, widen it to include `refs/mirrors/*`.

Capture the finding in T3's commit message:

- **If automatic (no filter widening needed):** commit message reads
  "stateless-connect advertisement: no widening required — tunneling
  to git upload-pack preserves cache refs verbatim. Verified by reading
  stateless_connect.rs."
- **If filter widening required:** edit the relevant filter; the
  commit message reads "stateless-connect advertisement: widened ref
  allowlist to include refs/mirrors/* alongside refs/heads/*."

`transfer.hideRefs` setup (`crates/reposix-cache/src/cache.rs:466+`):
the cache hides `refs/reposix/sync/` from `git upload-pack`. Mirror
refs are NOT hidden (they're in a different namespace). No
`transfer.hideRefs` change needed.

### 3b. handle_export wiring

Edit `crates/reposix-remote/src/main.rs`. Two insertion points:

**Success branch insertion (between lines 471 and 484):**

The current success branch:

```rust
} else {
    if let Some(cache) = state.cache.as_ref() {
        cache.log_helper_push_accepted(files_touched, &summary);
        // §3c token-cost: ...existing comment...
        let chars_in: u64 = parsed.blobs.values().map(|b| u64::try_from(b.len()).unwrap_or(u64::MAX)).sum();
        let chars_out: u64 = "ok refs/heads/main\n".len() as u64;
        cache.log_token_cost(chars_in, chars_out, "push");
    }
    proto.send_line("ok refs/heads/main")?;
    proto.send_blank()?;
    proto.flush()?;
}
```

Becomes:

```rust
} else {
    if let Some(cache) = state.cache.as_ref() {
        cache.log_helper_push_accepted(files_touched, &summary);

        // Mirror-lag refs (DVCS-MIRROR-REFS-02). Best-effort: a ref-write
        // failure WARN-logs and does not poison the push ack. The audit
        // row is UNCONDITIONAL per OP-3 — written even on ref-write
        // failure. SoT SHA is the cache's post-write synthesis-commit
        // OID.
        //
        // P80 cost note: build_from() runs again here to capture the
        // post-write tree (one extra REST list_records call). P81 L1
        // migration replaces this with list_changed_since per
        // .planning/research/v0.13.0-dvcs/architecture-sketch.md
        // § Performance subtlety. L2/L3 cache-desync hardening defers
        // to v0.14.0 per the same doc.
        let sot_sha_opt = match state.rt.block_on(cache.refresh_for_mirror_head()) {
            Ok(oid) => Some(oid),
            Err(e) => {
                tracing::warn!("mirror-head SHA derivation failed: {e:#}");
                None
            }
        };
        if let Some(sha) = sot_sha_opt {
            if let Err(e) = cache.write_mirror_head(&state.backend_name, sha) {
                tracing::warn!("write_mirror_head failed: {e:#}");
            }
        }
        if let Err(e) = cache.write_mirror_synced_at(&state.backend_name, chrono::Utc::now()) {
            tracing::warn!("write_mirror_synced_at failed: {e:#}");
        }
        // OP-3 unconditional: audit-row write fires whether or not the
        // ref writes succeeded. Records the attempt's SHA (or empty
        // string if SHA derivation failed).
        let oid_hex = sot_sha_opt
            .map(|oid| oid.to_hex().to_string())
            .unwrap_or_default();
        cache.log_mirror_sync_written(&oid_hex, &state.backend_name);

        // §3c token-cost: ...existing comment...
        let chars_in: u64 = parsed.blobs.values().map(|b| u64::try_from(b.len()).unwrap_or(u64::MAX)).sum();
        let chars_out: u64 = "ok refs/heads/main\n".len() as u64;
        cache.log_token_cost(chars_in, chars_out, "push");
    }
    proto.send_line("ok refs/heads/main")?;
    proto.send_blank()?;
    proto.flush()?;
}
```

(The `gix::ObjectId::to_hex()` call returns a hex-encoded representation;
the trailing `.to_string()` is required because `to_hex` may return a
non-`String` type. Executor verifies the exact API at `cargo check`
time and adjusts; the observable behavior — `oid_hex` is the SHA-1
hex string — is invariant.)

**Conflict-reject hint composition (after line 397, before line 402):**

The current conflict-reject branch:

```rust
if !conflicts.is_empty() {
    conflicts.sort_by_key(|c| c.0 .0);
    let (first_id, local_v, backend_v, backend_ts) = &conflicts[0];
    diag(&format!(
        "issue {} modified on backend at {} since last fetch (local base version: {}, backend version: {}). Run: git pull --rebase",
        first_id.0, backend_ts, local_v, backend_v,
    ));
    if let Some(cache) = state.cache.as_ref() {
        cache.log_helper_push_rejected_conflict(&first_id.0.to_string(), *local_v, *backend_v);
    }
    proto.send_line("error refs/heads/main fetch first")?;
    proto.send_blank()?;
    proto.flush()?;
    state.push_failed = true;
    return Ok(());
}
```

Becomes (insert hint composition AFTER `cache.log_helper_push_rejected_conflict`
and BEFORE `proto.send_line("error refs/heads/main fetch first")`):

```rust
if !conflicts.is_empty() {
    conflicts.sort_by_key(|c| c.0 .0);
    let (first_id, local_v, backend_v, backend_ts) = &conflicts[0];
    diag(&format!(
        "issue {} modified on backend at {} since last fetch (local base version: {}, backend version: {}). Run: git pull --rebase",
        first_id.0, backend_ts, local_v, backend_v,
    ));
    if let Some(cache) = state.cache.as_ref() {
        cache.log_helper_push_rejected_conflict(&first_id.0.to_string(), *local_v, *backend_v);

        // Mirror-lag-ref reject hint (DVCS-MIRROR-REFS-03). When refs
        // are populated (post-first-push), name the staleness gap; when
        // absent (first-push case), omit the hint cleanly per
        // RESEARCH.md pitfall 7.
        if let Ok(Some(synced_at)) = cache.read_mirror_synced_at(&state.backend_name) {
            let ago = chrono::Utc::now().signed_duration_since(synced_at);
            let mins = ago.num_minutes().max(0);
            diag(&format!(
                "hint: your origin (GH mirror) was last synced from {sot} at {ts} ({mins} minutes ago); see refs/mirrors/{sot}-synced-at",
                sot = state.backend_name,
                ts = synced_at.to_rfc3339(),
                mins = mins,
            ));
            diag(&format!(
                "hint: run `reposix sync` to update local cache from {sot} directly, then `git rebase`",
                sot = state.backend_name,
            ));
        }
        // None case: existing diag(...) line above is the complete
        // diagnostic; no additional hint emitted.
    }
    proto.send_line("error refs/heads/main fetch first")?;
    proto.send_blank()?;
    proto.flush()?;
    state.push_failed = true;
    return Ok(());
}
```

The hint cites the literal `refs/mirrors/<sot>-synced-at` ref name in
the first hint line — satisfies the verifier shell's grep for
`refs/mirrors/sim-synced-at`.

**The plan-error reject branches (lines 411-432) do NOT cite mirror
refs.** Mirror-lag is not the diagnosis for a malformed-blob or
bulk-delete error. Hint composition is targeted at the conflict path
only. Confirmed at planning time + reflected in the
`reject_hint_first_push_omits_synced_at_line` integration test (which
exercises the conflict path with no prior refs).

### 3c. Build + commit

Build serially:

```bash
cargo check -p reposix-remote
cargo clippy -p reposix-remote -- -D warnings
```

If clippy fires (e.g., `clippy::cast_possible_truncation` on the
`num_minutes()` cast), fix inline. Likely lints + fixes:

- `gix::ObjectId::to_hex()` may need explicit `.to_string()` — already
  included.
- `signed_duration_since().num_minutes()` returns `i64`; the `.max(0)`
  + format produces a clean `i64` without further cast — no lint.
- `tracing::warn!` macro is already used elsewhere in the file (line
  310); no new dep.

Stage and commit (do NOT push — push is T04 terminal):

```bash
git add crates/reposix-remote/src/main.rs \
        crates/reposix-remote/src/stateless_connect.rs
git commit -m "$(cat <<'EOF'
feat(remote): wire mirror-lag refs into handle_export success + conflict-reject paths (DVCS-MIRROR-REFS-02 + DVCS-MIRROR-REFS-03)

- crates/reposix-remote/src/main.rs::handle_export
  - success branch: write refs/mirrors/<sot>-head + refs/mirrors/<sot>-synced-at + log_mirror_sync_written audit row (best-effort ref writes; OP-3 unconditional audit)
  - SoT SHA derivation: cache.refresh_for_mirror_head() (wraps Cache::build_from), called after execute_action succeeded for every action
  - Cost commentary cited inline (architecture-sketch § Performance subtlety; P81 L1 replacement target; v0.14.0 L2/L3 hardening)
  - conflict-reject branch: read_mirror_synced_at + emit hint lines citing refs/mirrors/<sot>-synced-at + RFC3339 + (N minutes ago); first-push None case omits hint cleanly per RESEARCH.md pitfall 7
- crates/reposix-remote/src/stateless_connect.rs: <NO CHANGE | one-line widening of ref allowlist> (executor records the actual finding here per T3a investigation)

NO new error variant; ref writes are best-effort tracing::warn! per RESEARCH.md "Wiring handle_export success path"; matches existing Cache::log_* family pattern.

Phase 80 / Plan 01 / Task 03 / DVCS-MIRROR-REFS-02 + DVCS-MIRROR-REFS-03.
EOF
)"
```

(The commit message has a placeholder `<NO CHANGE | one-line widening
of ref allowlist>` — executor replaces with the actual finding from
T3a.)
</action>

<verify>
  <automated>cargo check -p reposix-remote && cargo clippy -p reposix-remote -- -D warnings</automated>
</verify>

<done>
- `crates/reposix-remote/src/main.rs::handle_export` success branch
  writes both refs + the audit row before
  `proto.send_line("ok refs/heads/main")`.
- `crates/reposix-remote/src/main.rs::handle_export` conflict-reject
  branch composes the synced-at hint when refs are populated; omits
  the hint cleanly when refs are absent.
- `cargo check -p reposix-remote` exits 0.
- `cargo clippy -p reposix-remote -- -D warnings` exits 0.
- The plan-error reject branches (bulk-delete, invalid-blob) are
  unchanged — mirror-lag hint is targeted at the conflict path only.
- The token-cost block at lines 472-485 is preserved verbatim.
- T3a's investigation finding (whether stateless-connect needs widening
  or already advertises mirror refs automatically) is documented in
  the commit message.
- NO new error variant added; failures are best-effort
  `tracing::warn!`.
- Cargo serialized: T03 cargo invocations run only after T02's commit
  has landed.
</done>

---

