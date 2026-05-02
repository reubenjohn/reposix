← [back to index](./index.md)

## Task 83-01-T02 (continued) — `### 2b` through `### 2f`

*This is part 2 of 2. Part 1 (preamble, HARD-BLOCK analysis, `### 2a` — the `write_loop.rs` module) is in [T02-step-1.md](./T02-step-1.md).*

### 2b. Refactor `execute_action` to narrow-deps signature

In `crates/reposix-remote/src/main.rs`, replace the
`execute_action` definition (line 619) and the single call site
(line 505 in `handle_export`'s loop):

```rust
// New signature (line 619):
pub(crate) fn execute_action(
    backend: &dyn BackendConnector,
    project: &str,
    rt: &Runtime,
    cache: Option<&Cache>,
    action: PlannedAction,
) -> Result<()> {
    // body lifted verbatim with state.X → bound parameter
}
```

Mark `pub(crate)` so `write_loop::apply_writes` can call it via
`crate::execute_action`.

### 2c. Replace `handle_export` body with the wrapper shape

Lines 343-606 of `main.rs` shrink to:

```rust
fn handle_export<R: std::io::Read, W: std::io::Write>(
    state: &mut State,
    proto: &mut Protocol<R, W>,
) -> Result<()> {
    if let Err(e) = ensure_cache(state) {
        tracing::warn!("cache unavailable for push audit: {e:#}");
    }
    if let Some(cache) = state.cache.as_ref() {
        cache.log_helper_push_started("refs/heads/main");
    }

    // Parse stdin.
    let mut buffered = BufReader::new(ProtoReader::new(proto));
    let parse_result = parse_export_stream(&mut buffered);
    drop(buffered);
    let parsed = match parse_result {
        Ok(v) => v,
        Err(e) => {
            return fail_push(
                proto, state, "parse-error",
                &format!("parse export stream: {e:#}"),
            ).map_err(Into::into);
        }
    };

    // Apply writes.
    let outcome = write_loop::apply_writes(
        state.cache.as_ref(),
        state.backend.as_ref(),
        &state.backend_name,
        &state.project,
        &state.rt,
        proto,
        &parsed,  // borrow per B1 — apply_writes takes &ParsedExport (matches precheck/plan shape)
    )?;

    let (sot_sha, files_touched, summary) = match outcome {
        write_loop::WriteOutcome::SotOk { sot_sha, files_touched, summary } =>
            (sot_sha, files_touched, summary),
        _ => {
            state.push_failed = true;
            return Ok(());
        }
    };

    // Single-backend caller writes synced-at + mirror_sync_written
    // unconditionally (D-01).
    if let Some(cache) = state.cache.as_ref() {
        if let Err(e) = cache.write_mirror_synced_at(&state.backend_name, Utc::now()) {
            tracing::warn!("write_mirror_synced_at failed: {e:#}");
        }
        let oid_hex = sot_sha.map(|o| o.to_hex().to_string()).unwrap_or_default();
        cache.log_mirror_sync_written(&oid_hex, &state.backend_name);

        // log_token_cost — lifted verbatim from old lines 593-599.
        let chars_in: u64 = parsed
            .blobs.values()
            .map(|b| u64::try_from(b.len()).unwrap_or(u64::MAX))
            .sum();
        let chars_out: u64 = "ok refs/heads/main\n".len() as u64;
        cache.log_token_cost(chars_in, chars_out, "push");
    }
    proto.send_line("ok refs/heads/main")?;
    proto.send_blank()?;
    proto.flush()?;
    let _ = (files_touched, summary);  // captured for future use
    Ok(())
}
```

**`parsed` ownership shape — RESOLVED (borrow per B1).** The
shared `apply_writes` takes `parsed: &ParsedExport` — borrow, NOT
consume-by-value. This matches the existing
`precheck_export_against_changed_set(... parsed: &ParsedExport)`
and `plan(... parsed: &ParsedExport)` shape (`precheck.rs:95`,
`diff.rs:101`). Both callers (`handle_export` here AND
`bus_handler::handle_bus_export` in T04) pass `&parsed` and retain
ownership so `parsed.blobs` remains available for the caller-side
`log_token_cost` write. NO clone needed; no `Clone` derive added to
`ParsedExport` at `fast_import.rs:71-79`.

### 2d. Add `mod write_loop;` declaration

In `crates/reposix-remote/src/main.rs` mod declarations (alphabetical):

```rust
mod backend_dispatch;
mod bus_handler;
mod bus_url;
mod diff;
mod fast_import;
mod pktline;
mod precheck;
mod protocol;
mod stateless_connect;
mod write_loop;     // NEW
```

### 2e. Cargo check + run existing single-backend tests

```bash
cargo check -p reposix-remote 2>&1 | tail -20
# expect: 0 errors, 0 warnings (clippy::pedantic clean)
cargo nextest run -p reposix-remote --tests 2>&1 | tail -40
# expect: all existing tests pass — mirror_refs, push_conflict,
# bulk_delete_cap, perf_l1, stateless_connect, stateless_connect_e2e,
# bus_url, bus_capabilities, bus_precheck_a, bus_precheck_b
```

If ANY existing test fails: the lift dropped a behavior. Diagnose,
fix, re-run. Do NOT proceed to T03 until all existing tests pass.

### 2f. Atomic refactor commit

```bash
git add crates/reposix-remote/src/write_loop.rs \
        crates/reposix-remote/src/main.rs
git commit -m "$(cat <<'EOF'
refactor(reposix-remote): lift handle_export write loop into write_loop::apply_writes (P83 prelude)

- crates/reposix-remote/src/write_loop.rs (new) — apply_writes function with narrow-deps signature `(cache, backend, backend_name, project, rt, proto, parsed)` returning WriteOutcome { SotOk { sot_sha, files_touched, summary } | Conflict | PlanRejected | SotPartialFail | PrecheckBackendUnreachable }
- crates/reposix-remote/src/main.rs — execute_action widened to pub(crate) with narrow-deps signature `(backend, project, rt, cache, action)` so write_loop::apply_writes can call it; handle_export body shrunk to wrapper shape (parse → apply_writes → match SotOk → write synced-at + mirror_sync_written + log_token_cost + ok); mod write_loop declaration added alphabetical
- D-01 RATIFIED: synced-at + mirror_sync_written audit row + log_token_cost + ok-line emit are CALLER's job; apply_writes writes only head ref + helper_push_accepted + last_fetched_at on SotOk

Single-backend behavior preserved verbatim — D-05 RATIFIED: existing integration tests (mirror_refs.rs, push_conflict.rs, bulk_delete_cap.rs, perf_l1.rs, stateless_connect.rs, stateless_connect_e2e.rs) ALL GREEN post-refactor as the regression invariant.

Phase 83 / Plan 01 / Task 02 / DVCS-BUS-WRITE-01 prelude (apply_writes refactor; bus_handler T04 will consume).
EOF
)"
```

NO push — terminal task is T06.
</action>

<verify>
  <automated>cargo check -p reposix-remote --tests 2>&1 | tail -5 && cargo nextest run -p reposix-remote --test mirror_refs --test push_conflict --test bulk_delete_cap --test perf_l1 --test stateless_connect --test bus_url --test bus_capabilities --test bus_precheck_a --test bus_precheck_b 2>&1 | tail -10 && grep -q 'pub(crate) fn apply_writes' crates/reposix-remote/src/write_loop.rs && grep -q 'pub(crate) fn execute_action' crates/reposix-remote/src/main.rs && grep -q 'mod write_loop' crates/reposix-remote/src/main.rs</automated>
</verify>

<done>
- `crates/reposix-remote/src/write_loop.rs` exists with
  `pub(crate) fn apply_writes(...)` and `pub(crate) enum WriteOutcome`.
- `crates/reposix-remote/src/main.rs::execute_action` widened to
  `pub(crate)` with narrow-deps signature
  `(backend, project, rt, cache, action)`.
- `crates/reposix-remote/src/main.rs::handle_export` body shrunk
  to wrapper shape: parse → `apply_writes` → match `SotOk` →
  caller-side synced-at + `mirror_sync_written` + `log_token_cost`
  + `ok` ack.
- `mod write_loop;` declaration in `main.rs` mod block,
  alphabetical placement.
- `cargo check -p reposix-remote --tests` exits 0; clippy
  pedantic clean.
- `cargo nextest run -p reposix-remote` passes ALL existing
  integration tests (regression invariant per D-05): mirror_refs,
  push_conflict, bulk_delete_cap, perf_l1, stateless_connect,
  stateless_connect_e2e, bus_url, bus_capabilities,
  bus_precheck_a, bus_precheck_b.
- Single atomic refactor commit; commit message names D-01 +
  D-05 + the lift contract.
- NO push — T06 is terminal.
</done>

---
