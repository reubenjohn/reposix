← [back to index](./index.md)

## Subtle architectural points (read before T02 of either plan)

The two below are flagged because they are the most likely sources
of executor friction. The executor must internalize them before
writing the wiring code.

### S1 — `apply_writes` body lifts verbatim — preserve, don't rewrite

The lift in P83-01 T02 is **mechanical**, not creative. The body
of `apply_writes_impl` is `handle_export` lines 360-606 with three
specific replacements:

1. `state.cache.as_ref()` → `cache` (bound parameter).
2. `state.backend.as_ref()` → `backend` (bound parameter).
3. `state.backend_name`, `state.project`, `state.rt` → `backend_name`,
   `project`, `rt` (bound parameters).
4. `state.push_failed = true; return Ok(());` → `return
   Ok(WriteOutcome::<variant>);` (the function returns a
   `WriteOutcome` enum that the caller maps to the `push_failed`
   flag).
5. The synced-at write (`cache.write_mirror_synced_at(...)`) is
   REMOVED from the lifted body — D-01 defers it to the caller.
6. The `mirror_sync_written` audit row write
   (`cache.log_mirror_sync_written(...)`) is REMOVED from the
   lifted body — D-01 defers it to the caller (because in the bus
   path it lives behind the mirror-success branch).
7. The `log_token_cost` call at lines 593-599 is REMOVED from the
   lifted body — single-backend caller writes it (with
   `chars_in + ack-bytes`); bus caller writes it (with `chars_in
   + ack-bytes + mirror-push-stderr-tail-bytes`). RESEARCH.md
   § "Open Question 4".

The `head` ref write (`cache.write_mirror_head(...)`) STAYS in
the lifted body because it's unconditional on SoT-success (D-01).

**Why this matters for T02.** A reviewer or executor might be
tempted to "improve" the lifted code (deduplicate logic, add
helper functions, simplify error handling). DO NOT. The lift is
mechanical preservation; the only change is the parameter shape
and what gets returned vs written-and-mutated. Single-backend
behavior must be byte-for-byte equivalent post-refactor — that's
the regression contract D-05 names.

### S2 — `bus_handler::handle_bus_export` post-PRECHECK shape

Replace the body of `handle_bus_export` from line 172 onward (the
current `emit_deferred_shipped_error` stub) with the full
algorithm. The order matters:

```rust
// PRECHECK B passed (line 170 in current code). Now: read stdin,
// write SoT, push mirror.

let mut buffered = BufReader::new(ProtoReader::new(proto));
let parse_result = parse_export_stream(&mut buffered);
drop(buffered);
let parsed = match parse_result {
    Ok(v) => v,
    Err(e) => return bus_fail_push(proto, state, "parse-error",
        &format!("parse export stream: {e:#}")),
};

// log_helper_push_started — same OP-3 row as handle_export emits.
if let Some(cache) = state.cache.as_ref() {
    cache.log_helper_push_started("refs/heads/main");
}

// SoT write half — shared with handle_export, factored into apply_writes.
let write_outcome = write_loop::apply_writes(
    state.cache.as_ref(),
    state.backend.as_ref(),
    &state.backend_name,
    &state.project,
    &state.rt,
    proto,
    parsed,
)?;

let (sot_sha, _files_touched, _summary) = match write_outcome {
    write_loop::WriteOutcome::SotOk { sot_sha, files_touched, summary } =>
        (sot_sha, files_touched, summary),
    // All non-Ok outcomes already emitted reject lines + audit rows
    // inside apply_writes. Set push_failed and return cleanly.
    _ => {
        state.push_failed = true;
        return Ok(());
    }
};

// SoT side succeeded. Mirror push (no retry per Q3.6).
let mirror_result = push_mirror(&mirror_remote_name)?;

match mirror_result {
    MirrorResult::Ok => {
        // Both refs current; lag = 0.
        if let Some(cache) = state.cache.as_ref() {
            if let Err(e) = cache.write_mirror_synced_at(
                &state.backend_name, chrono::Utc::now()) {
                tracing::warn!("write_mirror_synced_at failed: {e:#}");
            }
            let oid_hex = sot_sha.map(|o| o.to_hex().to_string())
                .unwrap_or_default();
            cache.log_mirror_sync_written(&oid_hex, &state.backend_name);
        }
        proto.send_line("ok refs/heads/main")?;
        proto.send_blank()?;
        proto.flush()?;
    }
    MirrorResult::Failed { exit_code, stderr_tail } => {
        // SoT contract satisfied; mirror lags. NO RETRY (Q3.6).
        // synced-at INTENTIONALLY NOT WRITTEN — frozen at last successful sync.
        if let Some(cache) = state.cache.as_ref() {
            let oid_hex = sot_sha.map(|o| o.to_hex().to_string())
                .unwrap_or_default();
            cache.log_helper_push_partial_fail_mirror_lag(
                &oid_hex, exit_code, &stderr_tail);
        }
        crate::diag(&format!(
            "warning: SoT push succeeded; mirror push failed \
             (will retry on next push or via webhook sync). \
             Reason: exit={exit_code}; tail={stderr_tail}"
        ));
        proto.send_line("ok refs/heads/main")?;
        proto.send_blank()?;
        proto.flush()?;
    }
}
Ok(())
```

The `head` ref write happens INSIDE `apply_writes` (D-01). The
`synced-at` ref write happens HERE (D-01). The
`mirror_sync_written` audit row also moves HERE (under the
`MirrorResult::Ok` arm); the `helper_push_partial_fail_mirror_lag`
audit row is the new addition for the failure arm.

**Why this matters for T04.** A reviewer might wonder why
`apply_writes` doesn't just write everything. D-01's deferral is
intentional — the bus path's mirror-failure leg needs synced-at
NOT written, and a single function with a flag is more confusing
than a clear caller-side block.

## Threat model crosswalk

Per CLAUDE.md § "Threat model" — this phase introduces TWO new
trifecta surfaces (the `git push` shell-out's argument boundary +
the per-record SoT write's expanded fault surface) and reuses three
existing surfaces unchanged:

| Existing surface              | What P83 changes                                                                                                                                                                                                                                                                                                                                                          |
|-------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Helper outbound HTTP          | UNCHANGED — `apply_writes`'s SoT REST writes are the same `BackendConnector` trait + `client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist used since v0.9.0. NO new HTTP construction site.                                                                                                                                                                            |
| Cache prior-blob parse (`Tainted` bytes) | UNCHANGED — `apply_writes` runs `precheck_export_against_changed_set` (P81) which parses prior blobs. The parser path is preserved verbatim by the lift (S1).                                                                                                                                                                                                              |
| `Tainted<T>` propagation      | UNCHANGED — `parse_export_stream` produces `Tainted<Record>`; `execute_action`'s `sanitize(Tainted::new(issue), meta)` boundary is the same. NO new tainted-bytes seam.                                                                                                                                                                                                    |
| **`git push` shell-out (NEW)** | NEW: `Command::new("git").args(["push", mirror_remote_name, "main"]).output()`. The `mirror_remote_name` is helper-resolved from `git config` (NOT user-controlled at this point — P82's STEP 0 already validated). STRIDE category: Tampering — mitigated by `mirror_remote_name.starts_with('-')` defensive reject + bounded-by-`git`-remote-name-validation provenance. |
| **Mirror push subprocess stderr_tail (NEW operator-readable seam)** | NEW: 3-line stderr tail captured for the audit row + WARNING log. The stderr is git-controlled (not record-content controlled), but operator-readable; could leak repo-internal info (commit SHAs, ref names). Trimming to 3 lines bounds the leak surface. STRIDE category: Information Disclosure — mitigated by trim. |
| **Per-record SoT-fail seam** (no new shell-out, but expanded fault surface in tests) | UNCHANGED in code path, EXPANDED in test coverage. Tests inject 5xx + 409 via wiremock — the helper code is unchanged; the fault-injection surface validates that the helper's existing failure handling is correct, not that new failure handling exists. |

`<threat_model>` STRIDE register addendum (carried into the plan
bodies):

- **T-83-01 (Tampering — argument injection via `mirror_remote_name`
  in `git push` shell-out):** reject `-`-prefix on
  `mirror_remote_name` BEFORE shell-out, mirroring P82's T-82-01.
  `mirror_remote_name` is helper-resolved (not user-controlled), so
  the defense is defensive-in-depth.
- **T-83-02 (Information Disclosure — stderr_tail leakage in audit
  row):** trim to 3 lines (RESEARCH.md Pattern 2). 3-line bound
  documented in `audit.rs::log_helper_push_partial_fail_mirror_lag`
  doc comment.
- **T-83-03 (Repudiation — partial-fail with mirror lag undetected):**
  the `helper_push_partial_fail_mirror_lag` audit row records the
  SoT SHA + exit code + stderr tail. Plus the head≠synced-at
  invariant on the refs side gives a vanilla-`git`-only operator a
  way to detect lag without database access.
- **T-83-04 (Denial of Service — `git push` against private mirrors
  hangs on SSH-agent prompt):** documented in CLAUDE.md update.
  Tests use `file://` fixture exclusively. Same disposition as
  T-82-03 (accept).
- **T-83-05 (Tampering — Confluence non-atomicity across actions):**
  ACCEPT. RESEARCH.md Pitfall 3 + D-09. The recovery story is
  next-push reads new SoT via PRECHECK B; documented inline in
  `bus_handler.rs` module-doc + CLAUDE.md update.
