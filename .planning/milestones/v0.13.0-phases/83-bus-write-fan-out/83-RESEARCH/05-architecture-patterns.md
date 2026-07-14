← [back to index](./index.md) · phase 83 research

## Architecture Patterns

### System Architecture Diagram

```
git push reposix main
         │
         ▼
git-remote-reposix <alias> reposix::<sot>?mirror=<mirror_url>
         │
         ▼  (P82 — already shipped)
bus_handler::handle_bus_export
  ├─ STEP 0  resolve mirror_remote_name from local git config
  ├─ PRECHECK A  git ls-remote -- <mirror_url> refs/heads/main
  └─ PRECHECK B  precheck_sot_drift_any (list_changed_since on SoT)
         │
         ▼  (P83 — NEW; replaces emit_deferred_shipped_error stub at line 173)
read fast-import stream from stdin into ParsedExport (reuse fast_import::parse_export_stream)
         │
         ▼
apply_writes(cache, backend, project, rt, parsed) -> WriteOutcome
  ├─ L1 precheck (precheck_export_against_changed_set)
  │     conflicts? → emit `error refs/heads/main fetch first` + reject hint, return Conflict
  ├─ plan(prior, parsed) -> Vec<PlannedAction>
  │     PlanError? → emit per-error protocol line, return PlanError
  ├─ for each action: execute_action (REST write to SoT)
  │     any_failure? → emit `error refs/heads/main some-actions-failed`, return SotPartialFail
  └─ on all-actions-succeeded:
        write helper_push_accepted cache-audit row (OP-3)
        write last_fetched_at cursor (best-effort)
        compute new SoT SHA via cache.refresh_for_mirror_head
        return WriteOutcome::SotOk { sot_sha, files_touched, summary }
         │  (only SotOk path proceeds to mirror)
         ▼
push_mirror(mirror_remote_name) -> MirrorResult
  └─ Command::new("git").args(["push", mirror_remote_name, "main"]).output()
         │
         ├─ Success → MirrorResult::Ok
         └─ Failure → MirrorResult::Failed { stderr_tail }
         │
         ▼
branch on (WriteOutcome, MirrorResult):
  (SotOk, MirrorResult::Ok) →
        write_mirror_head(sot_sha)              ──┐
        write_mirror_synced_at(now)             ──┼─ both refs current; lag = 0
        log_mirror_sync_written(sot_sha)        ──┘
        proto.send_line("ok refs/heads/main")
        return Ok

  (SotOk, MirrorResult::Failed) →
        write_mirror_head(sot_sha)              ──┐  head moves; synced_at FROZEN
        // synced_at NOT touched                ──┼─ lag observable: head ≠ synced_at
        log_helper_push_partial_fail_mirror_lag ──┘  (NEW op)
        diag("warning: SoT push succeeded; mirror push failed (Reason: …)")
        proto.send_line("ok refs/heads/main")    // SoT contract satisfied
        return Ok

  (Conflict | PlanError | SotPartialFail, _) →
        // mirror NEVER reached; refs UNCHANGED
        proto.send_line("error refs/heads/main <kind>")
        return Ok (state.push_failed = true)
```

### Recommended Project Structure

```
crates/reposix-remote/
├── src/
│   ├── main.rs              # dispatch loop unchanged from P82
│   ├── bus_handler.rs       # EXTENDED: replace emit_deferred_shipped_error stub with write fan-out
│   ├── bus_url.rs           # unchanged P82
│   ├── precheck.rs          # unchanged P81
│   ├── fast_import.rs       # unchanged
│   ├── diff.rs              # unchanged
│   ├── write_loop.rs        # NEW: shared apply_writes (refactored from handle_export)
│   └── ...
└── tests/
    ├── bus_write_happy.rs           # NEW (P83a): SoT-success + mirror-success → ok + both refs + 2 audit tables
    ├── bus_write_no_mirror_remote.rs # NEW (P83a): regression for SC4 (Q3.5 P82 hint preserved)
    ├── bus_write_mirror_fail.rs     # NEW (P83b): fault (a) — mirror push fails
    ├── bus_write_sot_fail.rs        # NEW (P83b): fault (b) — SoT 500 mid-stream
    ├── bus_write_post_precheck_409.rs # NEW (P83b): fault (c) — confluence 409 after precheck passed
    ├── bus_write_audit_completeness.rs # NEW (P83b): both tables read for every end-state
    └── common.rs                    # extend with helpers: make_failing_mirror_fixture, count_audit_rows

crates/reposix-cache/
├── src/
│   ├── audit.rs             # EXTENDED: log_helper_push_partial_fail_mirror_lag (sibling of log_helper_push_accepted)
│   └── cache.rs             # unchanged (the new audit fn lives in audit.rs)
└── fixtures/
    └── cache_schema.sql     # EXTENDED: add 'helper_push_partial_fail_mirror_lag' to op CHECK list
```

### Pattern 1: `apply_writes` shared SoT-write loop (the refactor)

**What:** Lift `handle_export` lines 360-606 (parse → L1 precheck → plan → execute → audit → ref-write) into a free function with the P81-style narrow-deps signature.

**When to use:** Always — both `handle_export` (single-backend) and `bus_handler::handle_bus_export` call this same function.

**Why this shape works:** P81's `precheck_export_against_changed_set` already pioneered the narrow-deps refactor pattern: take `(cache, backend, project, rt, parsed)` instead of `&mut State`. P83's `apply_writes` follows the same shape, plus a `&mut Protocol` for sending error lines and a `&mut bool` for the `push_failed` flag (or returns an outcome enum that the caller maps).

```rust
// Source: NEW crates/reposix-remote/src/write_loop.rs
//
// Signature mirrors P81's precheck::precheck_export_against_changed_set
// (M1 narrow-deps fix). Returns an outcome enum so callers can branch
// on what kind of end-state we reached without the function knowing
// whether it lives inside single-backend or bus context.

pub(crate) enum WriteOutcome {
    /// All SoT writes succeeded; ref/audit writes done; cache cursor advanced.
    /// `sot_sha` is the cache's post-write synthesis-commit OID (input to
    /// `refs/mirrors/<sot>-head`). `files_touched` and `summary` are the
    /// existing audit-row payload shape.
    SotOk {
        sot_sha: Option<gix::ObjectId>,
        files_touched: u32,
        summary: String,
    },
    /// L1 precheck found a conflict; reject lines already emitted on stdout +
    /// stderr; cache audit row already written. Caller sets push_failed = true.
    Conflict,
    /// `diff::plan` rejected (bulk-delete cap or invalid blob); reject lines
    /// already emitted. Caller sets push_failed = true.
    PlanRejected,
    /// At least one execute_action returned Err; protocol error already
    /// emitted. Caller sets push_failed = true.
    SotPartialFail,
    /// L1 precheck itself errored (REST unreachable); fail_push already
    /// emitted "backend-unreachable: ..." line. Caller sets push_failed = true.
    PrecheckBackendUnreachable,
}

pub(crate) fn apply_writes<R, W>(
    cache: Option<&Cache>,
    backend: &dyn BackendConnector,
    backend_name: &str,
    project: &str,
    rt: &Runtime,
    proto: &mut Protocol<R, W>,
    parsed: ParsedExport,
) -> Result<WriteOutcome>
where R: std::io::Read, W: std::io::Write,
{
    // Body lifted verbatim from handle_export L377-606 with three
    // mechanical replacements:
    //   - state.cache.as_ref()  →  cache
    //   - state.backend.as_ref() →  backend
    //   - state.backend_name     →  backend_name
    //   - state.project          →  project
    //   - state.rt               →  rt
    //   - state.push_failed = true → return Ok(WriteOutcome::<variant>)
    //
    // The function does NOT execute the per-token-cost log_token_cost
    // line that ends handle_export — that's left in the caller because
    // the bus path may have additional bytes-out from the mirror push
    // and that should be counted in the SAME audit row, not a separate
    // log_token_cost call.
    //
    // The function DOES include refresh_for_mirror_head + write_mirror_head
    // + write_mirror_synced_at because those are conditional on
    // SoT-success, and the caller (bus or single-backend) is the layer
    // that decides whether to ALSO update synced-at (single-backend always
    // does; bus only on mirror-success).
    //
    // BUT: for the bus path, write_mirror_synced_at MUST NOT fire here —
    // it has to wait for mirror-success. So the function takes a flag:

    apply_writes_impl(cache, backend, backend_name, project, rt, proto, parsed,
                     /* update_synced_at = */ true)
}

// Bus-mode entry point passes update_synced_at = false; mirror-success
// branch in the caller writes synced_at AFTER push_mirror succeeds.
pub(crate) fn apply_writes_bus<R, W>(...) -> Result<WriteOutcome> {
    apply_writes_impl(..., /* update_synced_at = */ false)
}
```

**Refactor commit shape:** ONE atomic commit. `git diff` should show:
1. New `crates/reposix-remote/src/write_loop.rs` containing `apply_writes_impl`.
2. `mod write_loop;` added to `main.rs`.
3. `handle_export` body shrinks to: `ensure_cache(state); log_helper_push_started; let parsed = parse_export_stream(...); let outcome = apply_writes(... &parsed); map outcome → state.push_failed flag; on SotOk: log_token_cost; ack ok refs/heads/main`.
4. All existing `handle_export` integration tests still green (regression invariant).

This refactor is a **prelude task** for P83a — must land before any bus-handler write code.

### Pattern 2: `push_mirror` subprocess + branching

**What:** A small helper in `bus_handler.rs` that runs `git push <mirror_remote_name> main` from the bus_handler's cwd (the working tree).

**When to use:** Once, after `apply_writes_bus` returns `WriteOutcome::SotOk`.

```rust
// Source: NEW addition to crates/reposix-remote/src/bus_handler.rs

/// Outcome of the mirror push subprocess.
enum MirrorResult {
    Ok,
    Failed { exit_code: i32, stderr_tail: String },
}

/// Run `git push <mirror_remote_name> main` from the cwd. Returns Ok
/// on zero exit; Failed with stderr tail on non-zero. NO RETRY (Q3.6).
fn push_mirror(mirror_remote_name: &str) -> Result<MirrorResult> {
    // No -- separator needed because mirror_remote_name is helper-resolved
    // via git config --get-regexp (NOT user-controlled at this point —
    // it was already validated by resolve_mirror_remote_name in P82's
    // STEP 0). But we still reject `-`-prefixed names defensively in case
    // the helper-resolved name picked up a malicious remote section.
    if mirror_remote_name.starts_with('-') {
        return Err(anyhow!(
            "mirror_remote_name cannot start with `-`: {mirror_remote_name}"
        ));
    }
    let out = Command::new("git")
        .args(["push", mirror_remote_name, "main"])
        .output()
        .with_context(|| format!("spawn `git push {mirror_remote_name} main`"))?;
    if out.status.success() {
        Ok(MirrorResult::Ok)
    } else {
        let stderr_tail = String::from_utf8_lossy(&out.stderr)
            .lines()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .join(" / ");
        Ok(MirrorResult::Failed {
            exit_code: out.status.code().unwrap_or(-1),
            stderr_tail,
        })
    }
}
```

**cwd confirmation:** `git-remote-reposix` is invoked by `git push` with the working-tree's `cwd` already set (git itself spawns the helper from the working directory). `bus_handler::resolve_mirror_remote_name` (P82) used this same fact when it ran `git config --get-regexp` and `git rev-parse refs/remotes/<name>/main` — both depend on the same cwd. The mirror-push subprocess inherits this cwd via `Command::output()` defaulting to the parent's cwd. **No `current_dir(...)` call needed — preserves P82's idiom.** [VERIFIED: `bus_handler.rs:181` and `bus_handler.rs:269` use bare `Command::new("git")` without `current_dir`; integration tests in `bus_precheck_a.rs` confirm this works against the test working tree by setting `current_dir` on the helper invocation, not on individual git subprocess calls inside the helper.]

### Pattern 3: End-state branching after `apply_writes_bus + push_mirror`

```rust
// Source: NEW body of crates/reposix-remote/src/bus_handler.rs::handle_bus_export
//         after PRECHECK B passes (replaces L173 emit_deferred_shipped_error)

// PRECHECK B passed. Now: read stdin, write SoT, push mirror.

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

// SoT write half — shared logic with handle_export, factored into apply_writes_bus.
let write_outcome = write_loop::apply_writes_bus(
    state.cache.as_ref(),
    state.backend.as_ref(),
    &state.backend_name,
    &state.project,
    &state.rt,
    proto,
    parsed,
)?;

let (sot_sha, files_touched, summary) = match write_outcome {
    write_loop::WriteOutcome::SotOk { sot_sha, files_touched, summary } =>
        (sot_sha, files_touched, summary),
    // All non-Ok outcomes already emitted reject lines + audit rows
    // inside apply_writes_bus. Set push_failed and return Ok cleanly.
    _ => {
        state.push_failed = true;
        return Ok(());
    }
};

// SoT side succeeded. Write the head ref unconditionally — the SoT moved.
if let (Some(cache), Some(sha)) = (state.cache.as_ref(), sot_sha) {
    if let Err(e) = cache.write_mirror_head(&state.backend_name, sha) {
        tracing::warn!("write_mirror_head failed: {e:#}");
    }
}

// Mirror push (no retry per Q3.6).
let mirror_result = push_mirror(&mirror_remote_name)?;

match mirror_result {
    MirrorResult::Ok => {
        // Both refs current; lag = 0.
        if let Some(cache) = state.cache.as_ref() {
            if let Err(e) = cache.write_mirror_synced_at(&state.backend_name, Utc::now()) {
                tracing::warn!("write_mirror_synced_at failed: {e:#}");
            }
            let oid_hex = sot_sha.map(|o| o.to_hex().to_string()).unwrap_or_default();
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
            let oid_hex = sot_sha.map(|o| o.to_hex().to_string()).unwrap_or_default();
            cache.log_helper_push_partial_fail_mirror_lag(
                &oid_hex,
                exit_code,
                &stderr_tail,
            );
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

### Anti-Patterns to Avoid

- **Helper-side retry on transient mirror failure.** Q3.6 RATIFIED no-retry. Don't loop. Don't sleep-and-retry. Don't add a `retries=N` field to `push_mirror`. The recovery path is "user runs `git push` again, next push catches up the mirror."
- **Updating `synced-at` on mirror-fail.** That ref is the load-bearing observability signal. Updating it would mask the lag from `git log refs/mirrors/<sot>-synced-at` readers and from the bus-remote's own reject hint composition (`bus_handler::handle_bus_export` cites it on PRECHECK B drift).
- **Force-push to the mirror.** Plain `git push` is correct. SoT-first means we are the authoritative writer for THIS turn — there's nothing on the mirror that should make us back off. (Webhook sync uses `--force-with-lease` per P84 because it might race with a bus push; bus push doesn't race with itself in this turn.)
- **Re-implementing `parse_export_stream` / `diff::plan` / `execute_action` in the bus path.** All three are pure functions reused verbatim. The bus path differs only in (a) precheck shape (already P82) and (b) post-write mirror handling.
- **Writing a single `op = 'mirror_sync_written'` row with a `status` field.** OP-3 favors distinct ops because the SQLite CHECK constraint already enumerates the op set; querying "all partial-fails in last 24h" is a single `WHERE op = 'helper_push_partial_fail_mirror_lag'` instead of a substring grep on `reason`. Consistent with the existing `helper_push_accepted` vs `helper_push_rejected_conflict` distinction.

