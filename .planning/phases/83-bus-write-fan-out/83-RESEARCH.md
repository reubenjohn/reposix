# Phase 83: Bus remote — write fan-out (SoT-first, mirror-best-effort, fault injection) — Research

**Researched:** 2026-05-01
**Domain:** git remote helper protocol; SoT-first fan-out write; mirror-best-effort with audit; fault injection via wiremock + file:// mirror fixtures
**Confidence:** HIGH (reuse path is fully shipped; fault-injection donors exist in `tests/push_conflict.rs` + `tests/bus_precheck_*.rs`)

## Summary

Phase 83 implements the **riskiest** part of the bus remote — steps 4-9 of the architecture-sketch's `§ 3` algorithm: read fast-import from stdin, write to SoT, fan out to mirror, audit-row both tables, update the two `refs/mirrors/<sot>-*` refs differently depending on which leg of the fan-out succeeded. Per `decisions.md` Q3.6 (RATIFIED 2026-04-30): **no helper-side retry** — surface failures, audit them, let the user retry the whole push.

The structural insight, validated against the live source: **the SoT-write half of the algorithm is `handle_export` lines 360-606 verbatim** — parse stdin, run the L1 precheck, plan, execute create/update/delete, write `helper_push_accepted` cache audit row, write `last_fetched_at` cursor, write the two mirror refs, write `mirror_sync_written` audit row, ack `ok refs/heads/main` to git. P83's job is NOT to rewrite this loop. It is to (a) lift the post-`bus_handler::handle_bus_export`-precheck portion of `handle_export` into a shared `apply_writes` function with a narrow signature (mirroring P81's `precheck_export_against_changed_set` narrow-deps refactor), (b) interpose a `git push <mirror_remote_name> main` shell-out between SoT write and the synced-at ref write, and (c) handle the new partial-failure end-state where SoT writes land but the mirror push fails. The mirror-push subprocess works in the bus_handler's `cwd` (the working tree where `mirror_remote_name` was resolved during P82's STEP 0) — that cwd is preserved across the function call.

**Primary recommendation:** **Split into P83a + P83b.** P83a delivers the write-fan-out core: refactor `handle_export` write-loop into `apply_writes(...)` with narrow deps, wire `bus_handler::handle_bus_export` to call it, add the `git push <mirror>` subprocess + ref/audit branching, ship 4 happy-path/no-mirror-remote integration tests. P83b delivers the 3 fault-injection scenarios + audit-completeness verification + P82↔P83 integration smoke. This split aligns with the ROADMAP's explicit *"may want to split"* carve-out (P83 §147 last sentence) and the build-memory-budget constraint (CLAUDE.md §"Build memory budget" — fault-injection tests are heavy linkage; serializing them into a second phase keeps each phase's cargo budget tight). A single P83 is doable but compounds risk: any architectural ambiguity caught during fault-injection forces a re-plan of the core, doubling cost.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| Read fast-import stream from stdin | reposix-remote (`bus_handler` calls existing `parse_export_stream`) | reposix-remote (`fast_import.rs` unchanged) | The `fast_import::parse_export_stream` parser is already used by `handle_export` line 362; bus handler reuses verbatim. |
| L1 precheck (intersect changed-set with push-set) | reposix-remote (`precheck::precheck_export_against_changed_set`) | reposix-cache, reposix-core | Already shipped P81 with the narrow-deps signature `(cache, backend, project, rt, parsed)` — bus handler can call it without `&mut State` coupling per the P81 M1 fix. |
| Plan create/update/delete actions | reposix-remote (`diff::plan`) | — | Pure function (`prior, parsed) -> Vec<PlannedAction>`; no I/O. Reused verbatim. |
| Execute SoT REST writes (create/update/delete) | reposix-remote (`execute_action`) | reposix-core (`BackendConnector::create_record / update_record / delete_or_close`) | Per-action loop that sanitizes server-controlled fields and calls the backend. Reused verbatim per architecture-sketch § "Tie-back to existing helper code". |
| Write cache audit row (`helper_push_accepted` / partial-fail) | reposix-cache (`audit::log_helper_push_*`) | — | OP-3 unconditional. Existing helper for accepted; P83 mints `helper_push_partial_fail_mirror_lag` for the new partial state. |
| Write backend audit row (`update_record` etc.) | reposix-core (`audit_events`) | sim/confluence/jira adapters | Already written by adapter-side `record_audit` calls; bus handler does NOT touch this — the per-action `execute_action` invocation makes the adapter write the row. |
| Update `last_fetched_at` cursor | reposix-cache (`Cache::write_last_fetched_at`) | — | Best-effort; WARN-on-fail. Unchanged from P81. |
| Mirror push (subprocess `git push <mirror_remote_name> main`) | reposix-remote (new `bus_handler::push_mirror`) | OS git binary | Works in bus_handler's cwd (the working tree). Subprocess invocation idiom mirrors `bus_handler::precheck_mirror_drift`'s `git ls-remote` shell-out (P82). |
| Update `refs/mirrors/<sot>-head` (always on SoT-success) | reposix-cache (`Cache::write_mirror_head`) | — | Records the new SoT SHA whether or not mirror push succeeded. |
| Update `refs/mirrors/<sot>-synced-at` (only on mirror-success) | reposix-cache (`Cache::write_mirror_synced_at`) | — | Skipped on mirror-fail per architecture-sketch § 3 step 7. This is the load-bearing observability signal — synced-at lagging head means "mirror push failed, recoverable on next push." |
| Mirror-lag audit row | reposix-cache (extend `audit_events_cache` CHECK list with new `op`) | — | New `op = 'helper_push_partial_fail_mirror_lag'` per OP-3 dual-table contract; record on the SoT-succeed-mirror-fail end-state. |
| Fault-injection test fixtures | reposix-remote tests (wiremock SoT + file:// bare-repo mirror with failing `update` hook) | tempfile, wiremock | Donor pattern: `tests/push_conflict.rs` (wiremock SoT errors) + `tests/bus_precheck_b.rs::make_synced_mirror_fixture` (file:// mirror). |

## User Constraints (from upstream)

No CONTEXT.md exists for P83 (no /gsd-discuss-phase invocation). Constraints flow from:

### Locked Decisions (RATIFIED in `decisions.md` 2026-04-30)
- **Q3.6 — No helper-side retry** on transient mirror-write failure. Surface, audit, let user retry. **Verbatim:** *"User retries the whole push. Helper-side retry would hide signal and complicate the audit trail."*
- **Q3.5 — No auto-mutation of git config** when mirror remote isn't configured. Already enforced at P82 level (`bus_handler::resolve_mirror_remote_name` returns `None` → emit verbatim hint). P83 inherits and regression-tests.
- **Q3.3 — `?mirror=<url>` URL form.** Already shipped P82.
- **Q2.3 — Both refs updated on success.** P83 updates both `head` AND `synced-at` on the SoT-succeed-mirror-succeed path. On SoT-succeed-mirror-fail: head updated, synced-at left at last-successful-sync timestamp.
- **OP-3 dual-table audit non-optional.** Every push end-state writes rows to BOTH `audit_events_cache` (helper RPC) AND `audit_events` (per-record backend mutation). Mirror-push outcome is a `audit_events_cache` row.
- **OP-1 simulator-first.** Fault-injection tests use wiremock + file:// fixtures, not real backends.
- **OP-2 Tainted-by-default.** Bytes from stdin (the fast-import stream) are tainted; `execute_action`'s existing `sanitize(Tainted::new(issue), meta)` boundary is preserved verbatim — no new tainted-bytes seam introduced.

### Claude's Discretion
- Whether to split P83 into P83a + P83b (RECOMMEND: yes — see § "Plan splitting recommendation").
- Whether the `apply_writes` refactor lands as part of P83a or as a separate prelude task within P83a (RECOMMEND: prelude task — single atomic refactor commit before any bus-handler changes; donor of the M1-style narrow-deps shape from P81).
- Whether the new mirror-lag audit op gets its own `op` value or reuses `mirror_sync_written` with a status field (RECOMMEND: separate op `helper_push_partial_fail_mirror_lag`; see § "Mirror-lag audit row shape" for rationale).
- Whether the mirror-push subprocess is `Command::new("git")` shell-out or gix-native (RECOMMEND: shell-out — matches P82's `precheck_mirror_drift` idiom and the helper already runs in a `git`-on-PATH context).

### Deferred Ideas (OUT OF SCOPE)
- Helper-side retry on transient mirror failure (Q3.6 RATIFIED no-retry).
- L2/L3 cache-desync hardening (deferred to v0.14.0 per architecture-sketch § "Performance subtlety").
- Atomic two-phase commit across SoT + mirror (REQUIREMENTS § "Out of Scope" — bus is "SoT-first, mirror-best-effort," not 2PC).
- Bidirectional bus (REQUIREMENTS § "Out of Scope").
- `--force-with-lease` for the mirror push (this is P84 webhook-sync territory; bus-push uses plain `git push` because SoT-first means we *are* the authoritative writer at this turn — see § "Pitfalls" item 2).

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DVCS-BUS-WRITE-01 | SoT-first write — buffer fast-import; apply REST; on success, audit BOTH tables + update `last_fetched_at`; on failure, bail (mirror unchanged). | § Pattern 1 (`apply_writes` refactor) lifts existing `handle_export` loop verbatim; only the surface is reshaped. |
| DVCS-BUS-WRITE-02 | Mirror write — `git push` to mirror after SoT success; on mirror-fail, write mirror-lag audit row, update `head` ref but NOT `synced-at`, stderr warn, return ok to git. | § Pattern 2 (`push_mirror` subprocess) + § "Mirror-write algorithm" exact state transitions. |
| DVCS-BUS-WRITE-03 | On mirror-write success: update `synced-at` ref to now; send `ok refs/heads/main` to git. | § Pattern 2; ref-write helpers `Cache::write_mirror_synced_at` already shipped P80. |
| DVCS-BUS-WRITE-04 | No helper-side retry on transient mirror failure (Q3.6). | § Pitfall 4; `push_mirror` returns `Err` on first non-zero exit. |
| DVCS-BUS-WRITE-05 | Bus URL with no local `git remote` for the mirror fails with P82's verbatim hint (no auto-mutation). | Already shipped P82 (`bus_handler::resolve_mirror_remote_name`); P83 adds a regression integration test. |
| DVCS-BUS-WRITE-06 | Fault-injection tests cover (a) mirror-push fail, (b) SoT-write mid-stream fail, (c) post-precheck SoT 409. Each → correct audit + recoverable state. | § "Fault-injection test infrastructure" section 4. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---|---|---|---|
| `std::process::Command` | std | spawn `git push <mirror_remote_name> main` | Idiom established by `bus_handler::precheck_mirror_drift` (P82) — `Command::new("git").args(["ls-remote", "--", mirror_url, "refs/heads/main"]).output()`. Same approach for push. [VERIFIED: `bus_handler.rs:247`] |
| `anyhow::{Context, Result}` | 1.x | error propagation | Helper crate's existing idiom. [VERIFIED: `main.rs:18`, `bus_handler.rs:45`] |
| `wiremock` | 0.6.5 | SoT mock for fault-injection tests | Existing dep; per-route `Mock::given(...).respond_with(ResponseTemplate::new(409))`. [VERIFIED: `tests/push_conflict.rs:31`, `tests/bus_precheck_b.rs:21`] |
| `assert_cmd` | (existing dev-dep) | spawn helper subprocess in tests | Existing idiom — `Command::cargo_bin("git-remote-reposix")` shape across all `tests/bus_*` files. [VERIFIED: `tests/bus_precheck_b.rs:165`] |
| `tempfile` | (existing dev-dep) | working-tree + bare-mirror fixtures | Existing idiom — `tempfile::tempdir()` for both wtree and bare mirror in `make_synced_mirror_fixture`. [VERIFIED: `tests/bus_precheck_b.rs:60`] |
| `reposix-cache` (existing) | path | `write_mirror_head`, `write_mirror_synced_at`, `log_mirror_sync_written`, `write_last_fetched_at`, `log_helper_push_accepted`, new `log_helper_push_partial_fail_mirror_lag` | All shipped P80/P81 except the new partial-fail helper which P83 mints. |

**No new third-party dependencies required.** Phase 83 is pure helper-crate + cache-crate work using crates already in the workspace [VERIFIED: `cargo metadata` need not run; `crates/reposix-remote/Cargo.toml` and `crates/reposix-cache/Cargo.toml` carry everything needed].

**Version verification:** No new versions to verify — every dependency already pinned.

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

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---|---|---|---|
| Subprocess-based git push | A custom protocol-v2 push client | `Command::new("git").args(["push", ...]).output()` | Helper already runs in a `git`-on-PATH context; matches P82's `precheck_mirror_drift` idiom; gix-native push surface is large and not where the value is. |
| Stderr tail extraction | `grep`/`tail` shell pipeline inside Command | `String::from_utf8_lossy(&out.stderr).lines().rev().take(3)` | std-lib idiom; no shell needed. |
| Mirror-bare-repo "fail on push" fixture | Rolling our own packfile-rejecting bare repo | `git init --bare <dir>` + `<dir>/hooks/update` script that exits 1 | git's own update-hook mechanism is the canonical "make this push fail." Fixture: 5-line shell script `#!/bin/sh\nexit 1`. Fast, deterministic, runs in tempdir. |
| 409 conflict simulation | Custom HTTP server | `wiremock::Mock::given(method("PATCH")).and(path_regex(...)).respond_with(ResponseTemplate::new(409))` | Existing wiremock idiom from `tests/push_conflict.rs`; per-route status injection. |
| Partial-stream failure simulation | Custom `tokio::io::AsyncRead` middleware | `wiremock` returning 500 on the second matching PATCH; or use `respond_with` with `set_delay` + `up_to_n_times(1)` for "first call ok, second fails" | wiremock has `up_to_n_times` — chain a 200 then a 500 per matching route. |
| Audit-row count assertions | Hand-rolled SQL parsing | `rusqlite::Connection::query_row("SELECT COUNT(*) FROM audit_events_cache WHERE op = ?1", params![...])` | Existing test idiom in `crates/reposix-cache/src/audit.rs` `mod tests` (e.g. line 543 `log_delta_sync_tx_inserts_row`). |

**Key insight:** Every fault-injection scenario can be expressed as a (wiremock-mock-config, mirror-fixture-config) pair. The helper code is exercised end-to-end via `assert_cmd::Command::cargo_bin("git-remote-reposix")`. No new test infrastructure is needed — the existing harness from `tests/bus_precheck_b.rs` covers it.

## Common Pitfalls

### Pitfall 1: `synced_at` write order vs `head` write order

**What goes wrong:** If `head` is written AFTER `synced_at`, an observer reading mid-write sees `synced_at > head` for a moment, which violates the invariant "synced_at is the timestamp the head ref was last brought current."

**Why it happens:** Lazy programming. Both writes are independent ref edits; ordering looks irrelevant.

**How to avoid:** Always write `head` FIRST, then `synced_at`. On the partial-fail path, `head` is written but `synced_at` is intentionally skipped — preserving the invariant `synced_at <= head_ts_implicit`.

**Warning signs:** A fault-injection test where `synced_at` parses to a time later than the cache's most recent commit timestamp.

### Pitfall 2: Mirror push uses `--force-with-lease`

**What goes wrong:** P84's webhook workflow uses `--force-with-lease` because it races with bus pushes. If P83's bus-push helper *also* uses `--force-with-lease`, two concurrent bus pushes can each pass the lease check (different leases) and one will silently overwrite the other's commit.

**Why it happens:** Cargo-cult from P84.

**How to avoid:** Plain `git push <mirror> main` — no `--force-with-lease`, no `--force`. SoT-first means by the time we get here, our SoT write IS the new authoritative state. Whatever's on the mirror is either (a) what we just left there last push, or (b) drift that PRECHECK A already trapped at the top of `handle_bus_export`. If a concurrent webhook-sync raced in between PRECHECK A and our mirror push, our push will fail with a non-fast-forward error → that's the partial-fail path, audit it, return ok, recover on next push.

**Warning signs:** Any code change that introduces `--force` or `--force-with-lease` flags into `push_mirror`. Reject in code review.

### Pitfall 3: Confluence partial-write semantics (PATCH 1 succeeds, PATCH 2 fails)

**What goes wrong:** If `apply_writes_bus` is partway through a 5-action plan (`Update id=1, Update id=2, Create id=99, Delete id=3`) and the PATCH for id=2 returns 500, what's the SoT state? Answer: id=1 IS updated; id=2 is NOT; id=99 was never attempted; id=3 was never attempted. The cache has NO record of id=1's new version (because the per-action loop bails on first error per `handle_export:504-512`).

**Why it happens:** The current `handle_export` per-action loop is best-effort-stop-on-first-error. There's no transaction boundary spanning the actions; each REST call is a discrete write to the SoT.

**How to avoid:**
1. **Document explicitly** in P83's plan that the bus path inherits this semantic (no atomicity across actions). The architecture-sketch is silent; this research closes that ambiguity: **non-atomic, fail-stop, partial state is observable on SoT after partial failure.**
2. **The recovery story:** the next push from any pusher reads the new SoT state via PRECHECK B's `list_changed_since`, sees id=1 has a newer version, and either accepts the local change (if the prior version still matches) or rejects with conflict (if not). The audit row records which actions executed (the `summary` field is `1,2,99,3`-ordered; on partial fail, `summary` reflects only the actions that completed — modify the existing summary-build to occur AFTER each successful action, not from the planned set).
3. **Test:** the fault-injection (b) scenario (kill confluence-write mid-stream) asserts the exact partial state — id=1 has new version, id=2 unchanged, no audit row for ids 99 / 3. Mirror unchanged.

**Warning signs:** A test that asserts "all-or-nothing" behavior. Reject — that's not what the helper does.

### Pitfall 4: Audit-row ordering on the partial-fail path

**What goes wrong:** Defensive form is "write audit row BEFORE the operation that might fail." On the SoT-succeed-mirror-fail path, this means writing `helper_push_partial_fail_mirror_lag` BEFORE attempting `git push <mirror>`. But if mirror push *succeeds*, we'd have a lying audit row (says "partial fail" when in fact everything worked).

**Why it happens:** Reasonable defensiveness applied at the wrong layer.

**How to avoid:** Write the audit row AFTER the mirror-push outcome is known. The atomicity property we want is "the audit row reflects the actual end-state," not "the audit row predicts the end-state." On a crash between mirror-push-result and audit-row-write, we lose ONE audit row but the ref state is internally consistent (head reflects SoT, synced_at reflects last-successful-mirror-sync). Acceptable trade — same shape as `handle_export`'s mirror-ref writes, where ref-write failures WARN-log without poisoning the push ack (per `mirror_refs.rs` module doc: "Ref writes are best-effort").

**Warning signs:** An audit row written speculatively before the operation completes. Reject in code review.

### Pitfall 5: First-push case (no `last_fetched_at` cursor, no prior mirror refs)

**What goes wrong:** PRECHECK B's no-cursor path returns Stable (per `precheck_sot_drift_any:359` — first-push policy). The bus handler proceeds to write fan-out. Mirror is empty. `git push <mirror> main` succeeds — but `refs/mirrors/<sot>-head` write needs a SoT SHA, and `refresh_for_mirror_head` fires `cache.build_from()` which itself does a full `list_records` walk on first run... wait, we're on the L1 path now (P81). `precheck_export_against_changed_set` returned Stable, so `prior` was synthesized from the cache's existing tree state. The cache's tree state is empty (first push). `plan(empty_prior, parsed) → Vec<Create>` for every record in the push.

**Why it's tricky:** The existing `handle_export` first-push path already works (P80 integration test `mirror_refs::write_on_success_updates_both_refs` exercises it). Bus path inherits.

**How to avoid:** Add a first-push integration test in P83a (happy-path) that asserts: empty cache + empty mirror + N records in push → SoT receives N create_record calls; mirror receives a fresh `main` ref; both `refs/mirrors/<sot>-head` and `refs/mirrors/<sot>-synced-at` are populated; `audit_events_cache` has `helper_push_started + helper_push_accepted + mirror_sync_written` rows.

**Warning signs:** A test that hard-codes a `last_fetched_at` value in a fixture without exercising the no-cursor path.

### Pitfall 6: `bus_handler` cwd assumption

**What goes wrong:** Section 2 above asserts the bus_handler runs in the working tree's cwd (because git invokes the helper from the working tree). If a future refactor moves bus_handler invocation to a separate thread or async task with `tokio::spawn`, the cwd may be lost or the env may be inherited differently.

**How to avoid:** **Pin the cwd assumption with a test.** Add a test that asserts `std::env::current_dir()?` inside `handle_bus_export` resolves to the working tree (not the cache dir, not `/tmp`). And/or: capture cwd in `state` at helper-startup time and pass it explicitly to `push_mirror` if the architecture ever needs to.

**Warning signs:** Any future refactor that moves git subprocess calls into async closures. Add a doc comment to `bus_handler.rs` warning about this.

### Pitfall 7: `cache_schema.sql` `op` CHECK list on stale cache.db files

**What goes wrong:** Per the existing comment in `cache_schema.sql:11-27`: *"On stale cache.db files the new ops will fail the CHECK and fall through the audit best-effort path (warn-logged); fresh caches see the full list."*

**How to avoid:** P83's new `helper_push_partial_fail_mirror_lag` op gets added to the CHECK list. Existing caches will reject the row at INSERT time, but the audit helper is best-effort (returns `()`, WARN-logs on error) — so the push still succeeds, the warning is the diagnostic. Fresh caches accept the row immediately. **This is the established pattern (P79 added `attach_walk`, P80 added `mirror_sync_written` — both via CHECK list extension).** No migration needed.

**Warning signs:** A migration script trying to ALTER TABLE the CHECK constraint. Don't — the IF NOT EXISTS pattern is the contract.

## Mirror-Write Algorithm (exact state transitions)

```
Pre-state (after PRECHECK A + PRECHECK B pass; stdin still buffered):
  refs/mirrors/<sot>-head:        OLD_SHA       (or absent if first push)
  refs/mirrors/<sot>-synced-at:   OLD_TS_TAG    (or absent if first push)
  cache.last_fetched_at:          CURSOR_T0     (or absent)
  audit_events_cache rows:        … (existing) …
  audit_events rows:              … (existing) …
  SoT records:                    {id_1: v1, id_2: v2, …}
  mirror main:                    OLD_SHA

Action 1: parse stdin → ParsedExport (BufReader on ProtoReader)
Action 2: apply_writes_bus(...) executes full SoT write loop
   - L1 precheck: returns Proceed { prior } (no conflicts) or Conflicts
   - plan(prior, parsed): returns Vec<Create|Update|Delete>
   - for each PlannedAction: execute_action() does ONE REST call
       - on success: backend adapter writes ONE audit_events row (OP-3)
       - on failure: returns Err; loop continues but any_failure = true
   - on all-success:
       cache.log_helper_push_accepted(files_touched, summary)   ← audit_events_cache
       cache.write_last_fetched_at(now)                         ← cursor advances to T1
       sot_sha = cache.refresh_for_mirror_head().await         ← NEW_SHA
       returns WriteOutcome::SotOk { sot_sha = Some(NEW_SHA), … }
   - on any failure: returns SotPartialFail / Conflict / etc.

Branch on WriteOutcome:
  - SotOk → continue to action 3
  - else → emit reject line; mirror UNCHANGED; return

Action 3: cache.write_mirror_head(<sot>, NEW_SHA)              ← head ref MOVES to NEW_SHA
Action 4: push_mirror(mirror_remote_name)
   subprocess: git push <mirror_remote_name> main
   - on success: return MirrorResult::Ok
   - on failure: return MirrorResult::Failed { exit, stderr_tail }

Branch on MirrorResult:

  MirrorResult::Ok →
     post-state:
       refs/mirrors/<sot>-head:        NEW_SHA               ✓ updated
       refs/mirrors/<sot>-synced-at:   NOW_TS_TAG            ✓ updated (action 5a)
       cache.last_fetched_at:          T1                    ✓ updated
       audit_events_cache rows:        + helper_push_started
                                       + helper_push_accepted
                                       + mirror_sync_written  (action 5c)
       audit_events rows:              + per-record mutations (action 2 inner)
       mirror main:                    NEW_SHA               ✓ updated
     emit: ok refs/heads/main

  MirrorResult::Failed →
     post-state:
       refs/mirrors/<sot>-head:        NEW_SHA               ✓ updated
       refs/mirrors/<sot>-synced-at:   OLD_TS_TAG            ✗ FROZEN (action 5b)
       cache.last_fetched_at:          T1                    ✓ updated
       audit_events_cache rows:        + helper_push_started
                                       + helper_push_accepted
                                       + helper_push_partial_fail_mirror_lag  (action 5d, NEW op)
       audit_events rows:              + per-record mutations (action 2 inner)
       mirror main:                    OLD_SHA               ✗ unchanged (push failed)
     emit warning to stderr: "SoT push succeeded; mirror push failed (Reason: …)"
     emit: ok refs/heads/main         ← Q3.6 contract: SoT promise satisfied
```

**Lag observability:** the difference `head ≠ synced-at-target` is precisely the lag. A vanilla-git `git fetch origin` brings both refs into a Dev-B clone; `git log refs/mirrors/<sot>-synced-at -1` shows the timestamp; `git rev-parse refs/mirrors/<sot>-head` shows the SHA the SoT advanced to. If they disagree, lag is real.

## Mirror-Lag Audit Row Shape

**Recommendation: NEW op `helper_push_partial_fail_mirror_lag`.** Add to `cache_schema.sql:28-48` CHECK list (sibling of existing `helper_push_accepted`).

**Schema row:**
```
op:          'helper_push_partial_fail_mirror_lag'
backend:     <backend_name>           e.g. 'sim' / 'confluence'
project:     <project>                e.g. 'demo' / 'TokenWorld'
issue_id:    NULL                     (this is a helper-RPC turn, not per-record)
oid:         <NEW_SHA hex>            the SoT SHA that head moved to
bytes:       NULL                     (no natural byte payload)
reason:      "exit=<N>;tail=<stderr_tail>"
ts:          <RFC3339>
```

**Why a new op vs. `mirror_sync_written` with status:**
- The existing `mirror_sync_written` row is written on the success path AND on the SoT-succeed-but-SHA-derivation-failed path; it conflates "ref writes attempted" semantics. Reusing it for partial-fail would muddy the success-vs-fail distinction.
- The CHECK constraint enumerates the op set; querying *"all partial-fails in last 24h"* is one `WHERE op = 'helper_push_partial_fail_mirror_lag'` clause. A status field would require `WHERE reason LIKE '%fail%'` — fragile string-matching.
- Consistent with the existing `helper_push_accepted` vs `helper_push_rejected_conflict` distinction: each push end-state has its own op.

**Helper signature** (mints in `crates/reposix-cache/src/audit.rs`, sibling of `log_helper_push_accepted` at line 230):

```rust
pub fn log_helper_push_partial_fail_mirror_lag(
    conn: &Connection,
    backend: &str,
    project: &str,
    sot_sha_hex: &str,
    exit_code: i32,
    stderr_tail: &str,
) {
    let reason = format!("exit={exit_code};tail={stderr_tail}");
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, oid, reason) \
         VALUES (?1, 'helper_push_partial_fail_mirror_lag', ?2, ?3, ?4, ?5)",
        params![Utc::now().to_rfc3339(), backend, project, sot_sha_hex, reason],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, exit_code,
              "log_helper_push_partial_fail_mirror_lag failed: {e}");
    }
}
```

**Wrapped accessor on Cache** (sibling of `log_mirror_sync_written` at `mirror_refs.rs:274`):

```rust
impl Cache {
    pub fn log_helper_push_partial_fail_mirror_lag(
        &self, sot_sha_hex: &str, exit_code: i32, stderr_tail: &str,
    ) {
        let conn = self.db.lock().expect("cache.db mutex poisoned");
        audit::log_helper_push_partial_fail_mirror_lag(
            &conn, &self.backend_name, &self.project,
            sot_sha_hex, exit_code, stderr_tail,
        );
    }
}
```

**Schema delta:** add `'helper_push_partial_fail_mirror_lag'` to the CHECK list in `crates/reposix-cache/fixtures/cache_schema.sql:28-48`. Update the comment on line 26-27 to cite "P83 sibling events extend this further" → "P83 ships `helper_push_partial_fail_mirror_lag`".

## Audit Completeness Contract (per OP-3)

For each end-state, both tables must have the expected rows. **This is the load-bearing contract** — incomplete audit = incomplete feature.

| End-state | `audit_events_cache` rows (cache) | `audit_events` rows (backend) |
|---|---|---|
| **Bus push, SoT ok, mirror ok** (DVCS-BUS-WRITE-01..03) | `helper_backend_instantiated` + `helper_push_started` + `helper_push_accepted` + `mirror_sync_written` + (per-record `helper_push_sanitized_field` for any Update with sanitize) | One row per executed `create_record` / `update_record` / `delete_or_close` |
| **Bus push, SoT ok, mirror fail** (DVCS-BUS-WRITE-02 partial path) | `helper_backend_instantiated` + `helper_push_started` + `helper_push_accepted` + `helper_push_partial_fail_mirror_lag` (NEW; **NO `mirror_sync_written` row**) + sanitize rows | One row per executed REST mutation (same as success — SoT writes already landed) |
| **Bus push, SoT precheck conflict** (DVCS-BUS-WRITE-01 bail path) | `helper_backend_instantiated` + `helper_push_started` + `helper_push_rejected_conflict` (existing op from P81 path) | None (no REST mutations attempted) |
| **Bus push, SoT 409 post-precheck** (fault inj c) | `helper_backend_instantiated` + `helper_push_started` (no accepted/conflict op — see § Pitfall 3 + Open Question 1) | One row for any record whose PATCH succeeded BEFORE the 409; none for the 409'd record or subsequent records |
| **Bus push, mirror remote not configured** (DVCS-BUS-WRITE-05) | `helper_backend_instantiated` only (P82 bails before stdin read) | None |

**Verification approach for the audit-completeness catalog row:**
- `bus_write_audit_completeness.rs` integration test runs each end-state once.
- After each run, the test opens the cache.db (via `rusqlite::Connection::open(<cache_path>/cache.db)`) and queries both tables for the expected op set + count.
- Asserts row counts match the table above.

## Fault-Injection Test Infrastructure

### Donor patterns

- **wiremock SoT** — `tests/push_conflict.rs::stale_base_push_emits_fetch_first_and_writes_no_rest` is the canonical donor for SoT-side faults. Uses `Mock::given(method("PATCH")).respond_with(ResponseTemplate::new(409))` to inject error status.
- **file:// mirror with passing push** — `tests/bus_precheck_b.rs::make_synced_mirror_fixture` is the canonical donor for a bare mirror that accepts pushes. **Reuse verbatim** for fault scenarios where mirror succeeds.
- **file:// mirror with failing push** — NEW. Pattern: `git init --bare <dir>` + write `<dir>/hooks/update` containing `#!/bin/sh\necho "update hook intentionally failed for fault test" >&2\nexit 1\n` + `chmod +x`. The update hook fires on every ref update and exits non-zero, causing `git push` to report `! [remote rejected] main -> main (hook declined)`.

### Test (a) — Mirror push fails between confluence-write and ack

**Setup:** wiremock SoT (full happy path: list_records returns prior, list_changed_since returns empty for PRECHECK B since cursor is fresh, PATCH for the changed record returns 200) + file:// mirror with FAILING update hook.

**Test name:** `bus_write_mirror_fail_returns_ok_with_lag_audit_row` (in `tests/bus_write_mirror_fail.rs`).

**Assertions:**
1. Helper exits zero (Q3.6 — SoT contract satisfied → ok).
2. Helper stdout contains `ok refs/heads/main`.
3. Helper stderr contains `warning: SoT push succeeded; mirror push failed`.
4. `refs/mirrors/<sot>-head` resolves to a NEW SHA (head moved).
5. `refs/mirrors/<sot>-synced-at` either absent (first push) or unchanged from baseline (frozen).
6. `audit_events_cache` count where op = `helper_push_partial_fail_mirror_lag`: 1.
7. `audit_events_cache` count where op = `mirror_sync_written`: 0 (the success-only row).
8. `audit_events_cache` count where op = `helper_push_accepted`: 1 (SoT side did succeed).
9. wiremock saw exactly 1 PATCH (assert via `Mock::expect(1)`).

### Test (b) — Confluence write fails mid-stream (5xx on second PATCH)

**Setup:** wiremock SoT with TWO records to update; first PATCH (id=1) returns 200, second PATCH (id=2) returns 500. file:// mirror with PASSING hook.

**Test name:** `bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit` (in `tests/bus_write_sot_fail.rs`).

**Assertions:**
1. Helper exits non-zero.
2. Helper stdout contains `error refs/heads/main some-actions-failed` (existing handle_export-shape protocol error).
3. wiremock saw 2 PATCH requests (id=1 + id=2; the loop bailed at id=2's 500).
4. `audit_events_cache` count where op = `helper_push_accepted`: 0 (didn't reach the success branch).
5. `audit_events_cache` count where op = `helper_push_partial_fail_mirror_lag`: 0 (mirror never attempted).
6. `audit_events_cache` count where op = `helper_push_started`: 1 (always written).
7. **Mirror baseline ref unchanged** — file:// bare repo's main still points at the seed SHA (assert via `git -C <mirror_dir> rev-parse main`).
8. `refs/mirrors/<sot>-head` and `synced-at` UNCHANGED from baseline.
9. NOTE: per § Pitfall 3, the SoT *partially* committed — id=1 is updated server-side; id=2 is not. The test asserts this exact state by querying wiremock's request log for which PATCHes returned 200.

### Test (c) — Confluence 409 after PRECHECK B passed

**Setup:** wiremock SoT — PRECHECK B's `?since=` route returns `[]` (Stable); list_records returns prior; PATCH for id=1 returns 409 with version-mismatch body. file:// mirror with PASSING hook.

**Test name:** `bus_write_post_precheck_conflict_409_no_mirror_push` (in `tests/bus_write_post_precheck_409.rs`).

**Assertions:**
1. Helper exits non-zero.
2. Helper stdout contains `error refs/heads/main some-actions-failed` (the existing fail-on-execute path).
3. Helper stderr names the failing record id.
4. wiremock saw exactly 1 PATCH (the one that 409'd) AND exactly 1 list_changed_since (PRECHECK B Stable).
5. **Mirror NOT pushed** — `git -C <mirror_dir> rev-parse main` returns the seed SHA.
6. `audit_events_cache` count where op = `helper_push_accepted`: 0.
7. `audit_events_cache` count where op = `helper_push_partial_fail_mirror_lag`: 0.
8. `refs/mirrors/<sot>-head` and `synced-at` UNCHANGED.
9. **OPEN: do we audit the failed REST attempt?** See Open Question 1.

### Test (d) — Audit completeness happy-path

**Setup:** Standard happy-path (wiremock SoT + file:// mirror with passing hook + 2 records to update + 1 to create).

**Test name:** `bus_write_audit_completeness_happy_path_writes_both_tables` (in `tests/bus_write_audit_completeness.rs`).

**Assertions:**
1. Helper exits zero, stdout `ok refs/heads/main`.
2. Open both `audit_events_cache` (in cache.db) and `audit_events` (in sim's DB if exposed; OR via dedicated test sim that exposes its audit table).
3. Cache audit has rows for: `helper_backend_instantiated`, `helper_push_started`, `helper_push_accepted`, `mirror_sync_written`. Optionally: `helper_push_sanitized_field` × 2 (one per Update).
4. Backend audit has 3 rows: 2× `update_record` + 1× `create_record`.
5. Both tables have row counts matching the table in § "Audit Completeness Contract".

### `tests/common.rs` extension

Append two helpers:

```rust
/// Build a file:// bare mirror whose `update` hook always fails.
/// Used by mirror-fail fault tests.
pub fn make_failing_mirror_fixture() -> (tempfile::TempDir, String) {
    let mirror = tempfile::tempdir().expect("mirror tempdir");
    run_git_in(mirror.path(), &["init", "--bare", "."]);
    let hook = mirror.path().join("hooks").join("update");
    std::fs::write(&hook,
        "#!/bin/sh\necho \"intentional fail for fault test\" >&2\nexit 1\n"
    ).expect("write update hook");
    let mut perms = std::fs::metadata(&hook).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    std::fs::set_permissions(&hook, perms).expect("chmod update hook");
    let url = format!("file://{}", mirror.path().display());
    (mirror, url)
}

/// Open the cache.db at the deterministic path for (backend, project)
/// and count rows matching `op`. Used by audit-completeness assertions.
pub fn count_audit_cache_rows(cache_dir: &std::path::Path,
                              backend: &str, project: &str, op: &str) -> i64 {
    let db_path = cache_dir.join("reposix")
        .join(format!("{backend}-{project}.git"))
        .join("cache.db");
    let conn = rusqlite::Connection::open(&db_path).expect("open cache.db");
    conn.query_row(
        "SELECT COUNT(*) FROM audit_events_cache WHERE op = ?1",
        rusqlite::params![op],
        |r| r.get(0),
    ).expect("count audit rows")
}
```

## Catalog Row Design (per QG-06)

Land in `quality/catalogs/agent-ux.json` (per P82 D-04: agent-ux is the home, NOT a new bus-remote.json). All hand-edited per the existing `_provenance_note` pattern (Principle A binding for non-docs-alignment dimensions defers to GOOD-TO-HAVES-01).

**P83a rows (4 rows + 4 verifiers):**

1. `agent-ux/bus-write-sot-first-success` — happy path; SoT writes + mirror writes + both refs updated; ok returned. Verifier: `quality/gates/agent-ux/bus-write-sot-first-success.sh` runs `cargo test -p reposix-remote --test bus_write_happy -- happy_path_writes_both_refs_and_acks_ok`.
2. `agent-ux/bus-write-mirror-fail-returns-ok` — SoT succeeds + mirror fails → `ok` returned + lag audit row + warn. Verifier: runs `bus_write_mirror_fail_returns_ok_with_lag_audit_row` test.
3. `agent-ux/bus-write-no-helper-retry` — single mirror push only (no retry). Verifier: greps `crates/reposix-remote/src/bus_handler.rs` for absence of retry constructs (`for _ in 0..` adjacent to push_mirror calls); EXIT 0 if no retry present, EXIT 1 if any retry shape detected.
4. `agent-ux/bus-write-no-mirror-remote-still-fails` — regression for SC4 / Q3.5. Verifier: runs `bus_write_no_mirror_remote_emits_q35_hint` test (P82 hint preserved end-to-end after P83 lands).

**P83b rows (4 rows + 4 verifiers):**

5. `agent-ux/bus-write-fault-injection-mirror-fail` — fault (a). Verifier: runs `bus_write_mirror_fail_*` tests asserting (lag audit row, head ref moved, synced-at frozen, ok returned).
6. `agent-ux/bus-write-fault-injection-sot-mid-stream` — fault (b). Verifier: runs `bus_write_sot_mid_stream_fail_*` tests asserting (no mirror push, no lag audit, mirror baseline preserved).
7. `agent-ux/bus-write-fault-injection-post-precheck-409` — fault (c). Verifier: runs `bus_write_post_precheck_conflict_409_*` tests asserting (no mirror push, version-mismatch error cites record id).
8. `agent-ux/bus-write-audit-completeness` — both tables have expected row sets on every end-state. Verifier: runs `bus_write_audit_completeness_happy_path_writes_both_tables` test which queries both audit tables.

**Total: 8 rows across P83a + P83b.** Catalog-first invariant: each phase's first commit mints its rows status:FAIL with TINY shell verifier shells BEFORE any Rust changes.

## Plan Splitting Recommendation

**RECOMMEND: split into P83a + P83b.**

**P83a — Write fan-out core (~6 tasks).**
- T01 (catalog-first): mint 4 P83a catalog rows status:FAIL with TINY verifiers.
- T02 (refactor prelude): factor `handle_export` write loop into `crates/reposix-remote/src/write_loop.rs::apply_writes` with narrow-deps signature. Single atomic commit. Verifier: existing `handle_export` integration tests (`mirror_refs.rs`, `push_conflict.rs`, `bulk_delete_cap.rs`) still GREEN.
- T03 (cache audit op): add `helper_push_partial_fail_mirror_lag` to schema CHECK list; mint helper in `audit.rs` + `Cache::` wrapper in `cache.rs`. Unit test asserting INSERT roundtrips.
- T04 (bus_handler write fan-out): replace `emit_deferred_shipped_error` stub with the full algorithm: read stdin → `apply_writes_bus` → `push_mirror` → branch → ref/audit writes → emit `ok`. Single atomic commit per § Pattern 3.
- T05 (happy-path integration tests): `bus_write_happy.rs` + `bus_write_no_mirror_remote.rs` + extend `tests/common.rs` with mirror fixture helper + audit-row count helper.
- T06 (catalog flip + CLAUDE.md + close): flip 4 P83a rows FAIL→PASS; update CLAUDE.md § Architecture "Bus URL form" paragraph to reference P83 write fan-out; `git push origin main`; verifier subagent dispatch.

**P83b — Fault injection + audit completeness (~4 tasks).**
- T01 (catalog-first): mint 4 P83b catalog rows status:FAIL with TINY verifiers.
- T02 (mirror-fail test): `bus_write_mirror_fail.rs` with failing-update-hook fixture.
- T03 (sot-fail tests): `bus_write_sot_fail.rs` (mid-stream 5xx) + `bus_write_post_precheck_409.rs` (post-precheck 409).
- T04 (audit-completeness test + catalog flip + close): `bus_write_audit_completeness.rs` queries BOTH audit tables for the happy path; flip 4 P83b rows FAIL→PASS; `git push origin main`; verifier subagent dispatch.

**Rationale for the split:**

1. **Cargo memory budget** (CLAUDE.md "Build memory budget"): each phase keeps cargo invocations sequential per-crate. P83a's 6 tasks each touch reposix-remote OR reposix-cache (rarely both); P83b's 4 tasks all touch reposix-remote tests/. Splitting halves the per-phase cargo budget.

2. **Risk isolation**: T04 (bus_handler write fan-out) is the riskiest single task. If during implementation it surfaces architectural ambiguity (e.g., the cwd assumption in § Pitfall 6 turns out wrong), P83a's plan can absorb the surprise without dragging fault-injection tests through a re-plan.

3. **Verifier readability**: 8 catalog rows in one phase = one verdict file with 8 sub-sections. 4+4 = two cleaner verdict files. The unbiased verifier subagent's grading is more honest when each phase's contract is tighter.

4. **Per-phase push cadence** (CLAUDE.md, codified 2026-04-30): each phase closes with `git push origin main`. P83a shipping the core gives an early "the bus actually works on the happy path" signal to origin BEFORE the fault-injection complexity lands.

5. **The ROADMAP explicitly invites the split**: P83 §147 final sentence: *"If during planning this phase looks > 1 PR's worth of work, split into P83a (write fan-out core) + P83b (fault injection + audit completeness)."* Six tasks vs four tasks is exactly that boundary.

**Cost of NOT splitting:** ~10-task phase with cargo invocations spanning reposix-remote + reposix-cache (schema delta) + multiple test files (each its own cargo build) + 8 catalog rows + 1 verdict file. Approaches the ceiling where memory pressure or pre-push-gate failures compound.

## Open Questions (RESOLVED — see 83-PLAN-OVERVIEW.md § D-01..D-10)

### Open Question 1 — Audit failed REST attempts?

**What we know:** `audit_events_cache` records helper-RPC turns. `audit_events` records SUCCESSFUL backend mutations (the adapters write the row inside the success path of `create_record` etc.). When a REST call fails (e.g., 409), neither table records it. The agent reading the audit log later sees: "I started a push, I emitted some sanitize rows, then nothing — push failed somehow." The detail of which record the 409 was on is lost.

**What's unclear:** Should the bus-handler's `apply_writes_bus` write a `helper_push_rest_failure` cache audit row when `execute_action` returns Err?

**Recommendation:** **DEFER to surprises-intake.** P83 ships without per-failure audit (consistent with current `handle_export` behavior). If P85's troubleshooting docs reveal users need this signal, file a v0.13.0 GOOD-TO-HAVES item (op `helper_push_rest_failure`, fields `issue_id` + `reason="status=409;message=…"`).

### Open Question 2 — Order of cache cursor advance vs mirror push

**What we know:** `apply_writes_bus` advances `last_fetched_at` to `now` on SoT-success. This happens BEFORE `push_mirror`. If mirror push fails, the cursor has already advanced past the SoT state — but PRECHECK B's `list_changed_since(cursor)` will return EMPTY on the next push attempt (because the cursor is "now" and no new changes hit confluence yet), so the next push proceeds normally.

**What's unclear:** Is there a race where confluence-side edit lands between SoT-write-success and mirror-push? The cursor advance to `now` would mask that edit — next push's PRECHECK B returns empty even though the mirror needs a sync.

**Recommendation:** This is the L1 trade-off documented in `architecture-sketch.md` § "Performance subtlety". `reposix sync --reconcile` (P81 DVCS-PERF-L1-02) is the on-demand escape hatch. NOT a blocker for P83. **Document explicitly in P83 plan as a known L1 trade-off**, NOT a P83 bug.

### Open Question 3 — `apply_writes` `update_synced_at` flag — is the dual-entry-point ergonomic?

**What we know:** § Pattern 1 proposes two entry points: `apply_writes` (single-backend; updates synced_at after SoT-success) and `apply_writes_bus` (bus; defers synced_at to caller).

**What's unclear:** Is the boolean flag awkward? An alternative is to make `apply_writes` always defer synced_at to the caller and have `handle_export` do the synced_at write itself.

**Recommendation:** **PLANNER decides during T02**. Both shapes work; the flag is slightly less duplication but slightly more cognitive load. Either is acceptable.

### Open Question 4 — Should `apply_writes` log_token_cost or leave to caller?

**What we know:** `handle_export:593-599` emits `log_token_cost` with the parsed bytes-in + ack bytes-out. The bus path has additional bytes-out from the mirror push subprocess (stdout/stderr). Should `log_token_cost` reflect both?

**Recommendation:** **Leave to caller.** Single-backend caller writes `log_token_cost` with bytes-in + ack-out. Bus caller writes `log_token_cost` with bytes-in + ack-out + mirror-push-subprocess bytes (estimated as the stderr-tail length on failure, near-zero on success). Keeps `apply_writes` agnostic to ack-bytes shape.

## Pitfalls and Risks (summary, calling out the new risks beyond the per-pitfall section above)

- **Subprocess cwd assumption — pin with a test (Pitfall 6).** Document explicitly in `bus_handler.rs` module doc.
- **`--force-with-lease` cargo-cult from P84 (Pitfall 2).** Reject in code review.
- **Confluence non-atomicity across actions (Pitfall 3).** Document in P83 plan.
- **`synced-at` write order matters (Pitfall 1).** Always write head before synced-at.
- **Audit-row ordering on partial-fail (Pitfall 4).** Write audit row AFTER outcome known.
- **First-push case (Pitfall 5).** Add explicit happy-path test for empty cache + empty mirror.
- **Stale cache.db CHECK list (Pitfall 7).** Established pattern from P79/P80; no migration.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` + `cargo nextest run` (existing) |
| Config file | `Cargo.toml` workspace; per-crate `[dev-dependencies]` already carry `wiremock`, `assert_cmd`, `tempfile` |
| Quick run command | `cargo test -p reposix-remote --test bus_write_<name> -- --nocapture` |
| Full suite command | `cargo nextest run -p reposix-remote -p reposix-cache` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|---|---|---|---|---|
| DVCS-BUS-WRITE-01 | SoT-first write + dual audit | integration | `cargo test -p reposix-remote --test bus_write_happy happy_path_writes_both_refs_and_acks_ok` | ❌ T05 / P83a |
| DVCS-BUS-WRITE-02 | Mirror-fail returns ok with lag | integration | `cargo test -p reposix-remote --test bus_write_mirror_fail bus_write_mirror_fail_returns_ok_with_lag_audit_row` | ❌ P83b T02 |
| DVCS-BUS-WRITE-03 | Mirror-success updates synced-at + ok | integration | `cargo test -p reposix-remote --test bus_write_happy happy_path_writes_both_refs_and_acks_ok` (combined) | ❌ T05 / P83a |
| DVCS-BUS-WRITE-04 | No helper-side retry | mechanical | `bash quality/gates/agent-ux/bus-write-no-helper-retry.sh` (greps source) | ❌ T01 / P83a |
| DVCS-BUS-WRITE-05 | No-mirror-remote regression | integration | `cargo test -p reposix-remote --test bus_write_no_mirror_remote bus_write_no_mirror_remote_emits_q35_hint` | ❌ T05 / P83a |
| DVCS-BUS-WRITE-06 | Three fault scenarios + audit completeness | integration | `cargo test -p reposix-remote --test 'bus_write_*'` | ❌ P83b |

### Sampling Rate
- **Per task commit:** `cargo test -p reposix-remote --test bus_write_<active_test>` for the test under change.
- **Per wave merge:** `cargo nextest run -p reposix-remote -p reposix-cache` (per-crate due to CLAUDE.md memory budget).
- **Phase gate:** Full workspace nextest GREEN before `git push origin main` and verifier subagent dispatch.

### Wave 0 Gaps

- [ ] `crates/reposix-remote/src/write_loop.rs` — new file (P83a T02 prelude refactor).
- [ ] `crates/reposix-cache/src/audit.rs` — extend with `log_helper_push_partial_fail_mirror_lag` (P83a T03).
- [ ] `crates/reposix-cache/src/mirror_refs.rs` — extend `impl Cache` with `log_helper_push_partial_fail_mirror_lag` wrapper (P83a T03).
- [ ] `crates/reposix-cache/fixtures/cache_schema.sql` — extend op CHECK list (P83a T03).
- [ ] `crates/reposix-remote/tests/common.rs` — append `make_failing_mirror_fixture` + `count_audit_cache_rows` (P83a T05 + P83b T02-T04).
- [ ] `crates/reposix-remote/tests/bus_write_happy.rs` — new file (P83a T05).
- [ ] `crates/reposix-remote/tests/bus_write_no_mirror_remote.rs` — new file (P83a T05).
- [ ] `crates/reposix-remote/tests/bus_write_mirror_fail.rs` — new file (P83b T02).
- [ ] `crates/reposix-remote/tests/bus_write_sot_fail.rs` — new file (P83b T03).
- [ ] `crates/reposix-remote/tests/bus_write_post_precheck_409.rs` — new file (P83b T03).
- [ ] `crates/reposix-remote/tests/bus_write_audit_completeness.rs` — new file (P83b T04).
- [ ] `quality/catalogs/agent-ux.json` — 8 new rows (4 in P83a T01, 4 in P83b T01).
- [ ] `quality/gates/agent-ux/bus-write-*.sh` — 8 new TINY verifier shells.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V2 Authentication | no | Helper inherits `BackendConnector` auth from existing connectors (already V2-clean per v0.11.x audits). |
| V3 Session Management | no | No sessions; per-invocation. |
| V4 Access Control | yes | Egress allowlist (`REPOSIX_ALLOWED_ORIGINS`) gates SoT REST calls AND `git push <mirror>` shells out to a URL the user already configured locally (so the URL is not user-controllable at this point). |
| V5 Input Validation | yes | Stdin (fast-import stream) is `Tainted<>`; sanitize boundary preserved verbatim from `handle_export`. mirror_remote_name is helper-resolved (not user-controlled) but defensively reject `-`-prefix per § Pattern 2. |
| V6 Cryptography | no | No crypto introduced. |

### Known Threat Patterns for git remote helper + subprocess

| Pattern | STRIDE | Standard Mitigation |
|---|---|---|
| Argument injection via mirror_remote_name | Tampering | Reject `-`-prefixed names before `Command::new("git").args(["push", ...])`. mirror_remote_name comes from helper-resolved git config, not user argv, but defensive check matches T-82-01 idiom. |
| Working-tree cwd hijack via env | Elevation of Privilege | The `bus_handler` cwd is inherited from git's invocation; any test or future code path that spawns the helper from an unexpected cwd would land git pushes in the wrong tree. Mitigation: Pitfall 6 doc-warning + cwd assertion test. |
| Tainted stdin → mirror push amplification | Information Disclosure | Stdin bytes flow through `parse_export_stream` → `Tainted<Record>` → `sanitize` (existing seam) → REST PATCH. Mirror push doesn't see record bodies — it pushes git objects synthesized by the cache's bare repo, which themselves embed sanitized frontmatter. NEW exposure: mirror push subprocess's stderr_tail captured for the audit row. The stderr_tail comes from `git push` and is git-controlled (not record-content controlled), but the audit row is operator-readable and could leak repo-internal info. Trim to 3 lines (already in § Pattern 2). |
| Helper-side retry leaks transient state | Repudiation / Tampering | Q3.6 explicit no-retry + audit-the-attempt-once removes the leak vector. |
| Audit table CHECK constraint bypass | Tampering | Existing append-only triggers + DEFENSIVE flag (per `cache.rs:db.rs::open_cache_db`) cover this; no new attack surface. |

## Sources

### Primary (HIGH confidence)
- **`.planning/research/v0.13.0-dvcs/architecture-sketch.md`** § "3. Bus remote with cheap-precheck + SoT-first-write" — algorithm steps 1-9, the canonical design.
- **`.planning/research/v0.13.0-dvcs/decisions.md`** § Q3.6 — RATIFIED no-retry; § Q2.3 — both refs on success; § Q3.5 — no auto-mutation.
- **`.planning/REQUIREMENTS.md`** L74-79 — DVCS-BUS-WRITE-01..06 verbatim.
- **`.planning/ROADMAP.md`** § "Phase 83" — 8 success criteria + the explicit "may want to split" carve-out.
- **`crates/reposix-remote/src/main.rs::handle_export`** lines 343-606 — the verbatim write loop the bus wraps.
- **`crates/reposix-remote/src/bus_handler.rs`** P82 state — current `emit_deferred_shipped_error` stub at line 173 (lines 173-174); P83 replaces this.
- **`crates/reposix-remote/src/precheck.rs`** lines 90 (`precheck_export_against_changed_set`) + 352 (`precheck_sot_drift_any`) — narrow-deps signature donor.
- **`crates/reposix-cache/src/audit.rs`** lines 230-279 — `helper_push_accepted` / `helper_push_rejected_conflict` shape donor for new partial-fail op.
- **`crates/reposix-cache/src/mirror_refs.rs`** lines 110-300 — ref-write helpers + `log_mirror_sync_written` wrapper donor.
- **`crates/reposix-cache/fixtures/cache_schema.sql`** lines 11-48 — op CHECK list extension pattern (P79 + P80 precedent for adding a new op).
- **`crates/reposix-cache/src/cache.rs`** lines 443-483 (`read/write_last_fetched_at`) + lines 547-567 (`log_attach_walk` — JSON-payload-via-reason donor).
- **`crates/reposix-remote/tests/bus_precheck_b.rs`** — wiremock + file:// mirror fixture donor for fault-injection tests.
- **`crates/reposix-remote/tests/push_conflict.rs`** — 409-injection donor (`Mock::given(method("PATCH")).respond_with(ResponseTemplate::new(409))`).

### Secondary (MEDIUM confidence)
- `quality/catalogs/agent-ux.json` rows P82 7-12 — provenance-note pattern + verifier-shell shape donor.
- `.planning/phases/82-bus-remote-url-parser/82-RESEARCH.md` + `82-PLAN-OVERVIEW.md` + `82-01-SUMMARY.md` — direct precedent for plan/research shape and 6-task organization.
- `.planning/phases/82-bus-remote-url-parser/PLAN-CHECK.md` — verifier checklist that P83 plans should anticipate.

### Tertiary (LOW confidence)
None — no LOW-confidence claims in this research; everything traces to a verified file or ratified decision.

## Assumptions Log

> All assumed claims tagged for confirmation.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The cwd of a `git push` invocation is preserved into the helper subprocess and into `Command::output()` calls inside the helper | Pattern 2; Pitfall 6 | Mirror-push subprocess targets wrong tree → catastrophic; mitigated by pin-with-test in P83a T05. |
| A2 | `git update`-hook returning non-zero is sufficient to make `git push <bare>` fail in tests | Don't Hand-Roll; Test (a) | Test (a) wouldn't actually fail. Mitigation: any other deterministic mirror-fail vector (e.g., `pre-receive` hook, `chmod 0` the bare repo dir mid-test). |
| A3 | wiremock 0.6.5's `Mock::expect(0)` panics on Drop if the route was hit | Test (c) | Tests would silently pass when they should fail; mitigation: use `Mock::expect(1)` + `Mock::expect(0)` consistently and verify panic-on-Drop semantics in a smoke test. |
| A4 | The dual `apply_writes` / `apply_writes_bus` shape (with `update_synced_at` flag) is preferred over a single function with the flag explicit on the public surface | Pattern 1; Open Question 3 | Cognitive-load-heavy API; planner can choose alternative. Either works. |

## Open Questions for the Planner (RESOLVED — see 83-PLAN-OVERVIEW.md § D-01..D-10)

1. **Q-A** Does `apply_writes` write the `synced-at` ref itself for the single-backend path, or always defer to the caller? (Pattern 1 + Open Question 3.) — recommend: defer to caller, which means `handle_export` gets one extra `write_mirror_synced_at` line at its end. Symmetric with bus path.
2. **Q-B** Do we audit failed REST attempts (Open Question 1)? — recommend: NO for P83; defer to v0.13.0 GOOD-TO-HAVES.
3. **Q-C** Schema delta — does the new `op` get added to the inline `cache_schema.sql` comment narrative AND the CHECK list in the same task as the helper, or is the schema delta a separate prelude commit? — recommend: same atomic commit (T03). Matches P79/P80 pattern.
4. **Q-D** Is the failing-update-hook fixture portable to Windows CI? CLAUDE.md doesn't list Windows as a supported dev OS but cross-platform tests exist. — recommend: gate the test with `#[cfg(unix)]` if needed; document in fixture helper.
5. **Q-E** Should P83a's T02 refactor (`apply_writes` factor-out) carry its own catalog row asserting the regression invariant (existing `handle_export` tests still GREEN)? — recommend: NO; the existing `mirror_refs` / `push_conflict` / `bulk_delete_cap` integration tests ARE the regression check; pre-push gate is sufficient.
6. **Q-F** Reuse `mirror_sync_written` op vs new op? (Mirror-Lag Audit Row Shape section.) — recommend: NEW op `helper_push_partial_fail_mirror_lag`. Decided in research; confirm in plan.

## Metadata

**Confidence breakdown:**
- Algorithm shape: HIGH — directly traced to architecture-sketch § 3 + decisions.md Q3.6.
- `apply_writes` refactor: HIGH — donor pattern is shipped P81 narrow-deps fix.
- Mirror-push subprocess: HIGH — donor pattern is shipped P82 `precheck_mirror_drift`.
- Audit-op design: HIGH — donor pattern is shipped P79 (`attach_walk`) + P80 (`mirror_sync_written`).
- Fault-injection scenarios: HIGH — donors are shipped `tests/push_conflict.rs` + `tests/bus_precheck_b.rs`.
- Plan splitting recommendation: MEDIUM — based on cargo memory budget heuristic + ROADMAP carve-out; planner may choose unified phase.

**Research date:** 2026-05-01
**Valid until:** 2026-05-08 (DVCS milestone is fast-moving; re-validate if P83 hasn't started by then)
