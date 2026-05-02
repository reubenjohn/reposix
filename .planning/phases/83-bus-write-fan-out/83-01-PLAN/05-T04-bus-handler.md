← [back to index](./index.md) · phase 83 plan 01

## Task 83-01-T04 — `bus_handler` write fan-out: replace deferred-shipped stub with full algorithm

<read_first>
- `crates/reposix-remote/src/bus_handler.rs` lines 1-329 (full file
  post-P82 — locate the `emit_deferred_shipped_error` invocation
  at line 173 + the `emit_deferred_shipped_error` function at line
  300; both are replaced).
- `crates/reposix-remote/src/main.rs` (the `mirror_url: Option<String>`
  field on `State` from P82; confirm name).
- `crates/reposix-remote/src/write_loop.rs` (post-T02; the
  `apply_writes` + `WriteOutcome` shapes consumed).
- `crates/reposix-remote/src/protocol.rs` — `Protocol`,
  `ProtoReader`, `BufReader` shapes.
- `crates/reposix-remote/src/fast_import.rs::parse_export_stream` —
  signature.
- `crates/reposix-cache/src/cache.rs::Cache::log_helper_push_started` +
  `Cache::log_mirror_sync_written` (mirror_refs.rs:274) +
  `Cache::write_mirror_synced_at` (mirror_refs.rs) +
  `Cache::log_token_cost` (cache.rs:298) +
  `Cache::log_helper_push_partial_fail_mirror_lag` (post-T03) —
  the wrappers the caller invokes after `apply_writes` returns
  `SotOk`.
</read_first>

<action>
Two concerns: replace the stub → write the new logic → cargo check.
The cargo work is held sequentially with T03's `reposix-cache`
work — T04 holds the `reposix-remote` cargo lock.

### 4a. Replace `bus_handler::handle_bus_export`'s post-PRECHECK branch

Current code at lines 172-174:

```rust
    // STEPS 4-9 — write fan-out (DEFERRED to P83 per Q-B / D-02).
    emit_deferred_shipped_error(proto, state)
}
```

Replace with the full algorithm (read `mirror_remote_name` from
the variable already in scope from P82's STEP 0):

```rust
    // STEPS 4-9 — write fan-out (P83 / D-01 / D-08 / Q3.6 / D-09).
    //
    // PRECHECK B passed. Now: read stdin, write SoT, push mirror,
    // branch on outcomes, ack git.
    use std::io::BufReader;
    use crate::fast_import::parse_export_stream;
    use crate::protocol::ProtoReader;

    let parsed = {
        let mut buffered = BufReader::new(ProtoReader::new(proto));
        let parse_result = parse_export_stream(&mut buffered);
        drop(buffered);
        match parse_result {
            Ok(v) => v,
            Err(e) => {
                return bus_fail_push(
                    proto, state, "parse-error",
                    &format!("parse export stream: {e:#}"),
                );
            }
        }
    };

    if let Some(cache) = state.cache.as_ref() {
        cache.log_helper_push_started("refs/heads/main");
    }

    let outcome = crate::write_loop::apply_writes(
        state.cache.as_ref(),
        state.backend.as_ref(),
        &state.backend_name,
        &state.project,
        &state.rt,
        proto,
        &parsed,  // borrow per B1 — apply_writes takes &ParsedExport (matches precheck/plan shape)
    )?;

    let (sot_sha, _files_touched, _summary) = match outcome {
        crate::write_loop::WriteOutcome::SotOk { sot_sha, files_touched, summary } => {
            (sot_sha, files_touched, summary)
        }
        _ => {
            state.push_failed = true;
            return Ok(());
        }
    };

    let mirror_result = push_mirror(&mirror_remote_name)?;

    match mirror_result {
        MirrorResult::Ok => {
            if let Some(cache) = state.cache.as_ref() {
                if let Err(e) = cache.write_mirror_synced_at(
                    &state.backend_name, chrono::Utc::now(),
                ) {
                    tracing::warn!("write_mirror_synced_at failed: {e:#}");
                }
                let oid_hex = sot_sha
                    .map(|o| o.to_hex().to_string())
                    .unwrap_or_default();
                cache.log_mirror_sync_written(&oid_hex, &state.backend_name);

                // M4: chars_out is the count of ALL stdout bytes ack'd to git
                // (including the `ok refs/heads/main\n` line). Stderr is NOT
                // counted in chars_out — keeps semantics consistent across success
                // AND failure arms. Same definition is used in the failure arm below.
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
        }
        MirrorResult::Failed { exit_code, stderr_tail } => {
            if let Some(cache) = state.cache.as_ref() {
                let oid_hex = sot_sha
                    .map(|o| o.to_hex().to_string())
                    .unwrap_or_default();
                cache.log_helper_push_partial_fail_mirror_lag(
                    &oid_hex, exit_code, &stderr_tail,
                );

                // M4: same chars_out definition as the success arm — count ALL
                // stdout bytes ack'd to git (the `ok refs/heads/main\n` line is
                // emitted regardless of mirror-fail per Q3.6 contract). Stderr is
                // NOT counted; the WARNING printed via `crate::diag` below goes
                // to stderr but lives outside the token-cost ledger.
                let chars_in: u64 = parsed
                    .blobs.values()
                    .map(|b| u64::try_from(b.len()).unwrap_or(u64::MAX))
                    .sum();
                let chars_out: u64 = "ok refs/heads/main\n".len() as u64;
                cache.log_token_cost(chars_in, chars_out, "push");
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
}
```

### 4b. Add `MirrorResult` enum + `push_mirror` helper

Append to `bus_handler.rs` (place AFTER `precheck_mirror_drift`,
BEFORE the now-deleted `emit_deferred_shipped_error`):

```rust
/// Outcome of the mirror push subprocess (Pattern 2 of RESEARCH.md).
#[derive(Debug)]
enum MirrorResult {
    /// `git push <mirror_remote_name> main` exited zero.
    Ok,
    /// Non-zero exit; `stderr_tail` is the trimmed (<= 3 lines)
    /// stderr from the subprocess (T-83-02). `exit_code` is the
    /// process exit code (`-1` if signaled).
    Failed { exit_code: i32, stderr_tail: String },
}

/// Run `git push <mirror_remote_name> main` from the bus handler's
/// cwd (the working tree git invoked the helper from — Pitfall 6).
/// NO RETRY (Q3.6 RATIFIED). NO `--force-with-lease` (D-08
/// RATIFIED — P84 owns force-with-lease).
///
/// `mirror_remote_name` is helper-resolved via P82's STEP 0
/// (`resolve_mirror_remote_name`); it's bounded by git's own
/// remote-name validation. Defensive-in-depth (T-83-01): reject
/// `-`-prefixed names BEFORE shell-out.
///
/// # Errors
/// Returns `Err` on `Command::output()` spawn failure (subprocess
/// could not be created — e.g., git not on PATH). A non-zero
/// `git push` exit is `Ok(MirrorResult::Failed { ... })`, NOT a
/// propagated error.
fn push_mirror(mirror_remote_name: &str) -> Result<MirrorResult> {
    if mirror_remote_name.starts_with('-') {
        return Err(anyhow!(
            "mirror_remote_name cannot start with `-`: {mirror_remote_name}"
        ));
    }
    let out = Command::new("git")
        .args(["push", mirror_remote_name, "main"])
        .output()
        .with_context(|| {
            format!("spawn `git push {mirror_remote_name} main`")
        })?;
    if out.status.success() {
        Ok(MirrorResult::Ok)
    } else {
        let stderr_tail = String::from_utf8_lossy(&out.stderr)
            .lines()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" / ");
        Ok(MirrorResult::Failed {
            exit_code: out.status.code().unwrap_or(-1),
            stderr_tail,
        })
    }
}
```

### 4c. Remove `emit_deferred_shipped_error` function

Delete the function entirely (current lines 300-310). It's no
longer called anywhere — `handle_bus_export` ends in either the
`MirrorResult::Ok` or `MirrorResult::Failed` branch.

### 4d. Update module-doc

Update `bus_handler.rs`'s module-doc (lines 1-42). Replace the
"Steps 4-9 — the WRITE fan-out — are DEFERRED to P83" paragraph
with the now-shipped algorithm + Pitfall 6 cwd note + D-09
non-atomicity note (verbatim text in OVERVIEW S2).

### 4e. Cargo check + clippy

```bash
cargo check -p reposix-remote 2>&1 | tail -10
cargo clippy -p reposix-remote -- -D warnings 2>&1 | tail -10
```

If `clippy::pedantic` complains about something the original
`handle_export` body did NOT trigger, the lifted code or new
bus-handler code introduced something pedantic-unfriendly.
Diagnose + fix.

### 4f. Atomic write-fan-out commit

```bash
git add crates/reposix-remote/src/bus_handler.rs
git commit -m "feat(reposix-remote): bus_handler write fan-out replacing deferred-shipped stub (DVCS-BUS-WRITE-01..05)

- crates/reposix-remote/src/bus_handler.rs::handle_bus_export — post-PRECHECK-B branch reads stdin via parse_export_stream, calls shared write_loop::apply_writes (T02 refactor), runs git push via new push_mirror helper (no retry per Q3.6; no --force-with-lease per D-08), branches on (WriteOutcome, MirrorResult) for ref/audit writes
- new fn push_mirror — Command::new(git).args([push, name, main]).output() with T-83-01 defensive '-' reject + T-83-02 3-line stderr tail trim
- new enum MirrorResult { Ok | Failed { exit_code, stderr_tail } }
- removed: emit_deferred_shipped_error (Q-B / D-02 stub from P82 — no longer called)
- module-doc updated: write fan-out steps 4-9 documented; D-09 confluence non-atomicity + Pitfall 6 cwd assumption named
- D-01 RATIFIED: synced-at + mirror_sync_written audit row + log_token_cost are caller-side
- Q3.6 RATIFIED: NO retry on transient mirror-write failure
- D-08 RATIFIED: plain git push, NO --force-with-lease

Phase 83 / Plan 01 / Task 04 / DVCS-BUS-WRITE-01 + 02 + 03 + 04 + 05."
```

NO push — T06 is terminal.
</action>

<verify>
  <automated>cargo check -p reposix-remote 2>&1 | tail -5 && grep -q 'fn push_mirror' crates/reposix-remote/src/bus_handler.rs && grep -q 'enum MirrorResult' crates/reposix-remote/src/bus_handler.rs && ! grep -q 'emit_deferred_shipped_error' crates/reposix-remote/src/bus_handler.rs && grep -q 'write_loop::apply_writes' crates/reposix-remote/src/bus_handler.rs && grep -q 'log_helper_push_partial_fail_mirror_lag' crates/reposix-remote/src/bus_handler.rs && bash quality/gates/agent-ux/bus-write-no-helper-retry.sh</automated>
</verify>

<done>
- `crates/reposix-remote/src/bus_handler.rs::handle_bus_export`
  post-PRECHECK-B branch calls `write_loop::apply_writes`, then
  `push_mirror`, then branches on `(WriteOutcome, MirrorResult)`
  for ref/audit writes.
- `bus_handler.rs` exports `fn push_mirror(mirror_remote_name: &str)
  -> Result<MirrorResult>` (private to module) with T-83-01
  defensive reject + T-83-02 3-line stderr tail trim.
- `bus_handler.rs` exports `enum MirrorResult { Ok | Failed { exit_code,
  stderr_tail } }` (private to module).
- `emit_deferred_shipped_error` function REMOVED.
- Module-doc updated to reflect P83's write fan-out shipped.
- `cargo check -p reposix-remote` exits 0; clippy pedantic clean.
- `bash quality/gates/agent-ux/bus-write-no-helper-retry.sh` exits 0
  (verifier confirms no retry constructs in bus_handler.rs; row 3
  ready to flip at T06).
- Single atomic commit per the architecture-sketch's bus algorithm
  steps 4-9 contract; commit message names D-01 + D-08 + Q3.6 +
  T-83-01 + T-83-02.
- NO push — T06 is terminal.
</done>

---

## Task 83-01-T05 — 2 integration tests + `tests/common.rs` helpers
