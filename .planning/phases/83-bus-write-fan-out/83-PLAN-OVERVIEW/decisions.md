← [back to index](./index.md)

## Decisions ratified at plan time

The six open questions surfaced by RESEARCH.md § "Open Questions
for the Planner" are RATIFIED here so the executing subagent and
the verifier subagent both grade against the same contract. Each
decision references the source artifact and the rationale. The user
directive (orchestrator instructions for P83) pre-resolved each
question; this section captures them as Plan-time decisions.

### D-01 — Q-A: `apply_writes` defers mirror-synced-at to caller (RATIFIED)

**Decision:** the shared `apply_writes` function writes
`refs/mirrors/<sot>-head` always (on SoT-success) but does NOT write
`refs/mirrors/<sot>-synced-at`. The single-backend caller
(`handle_export`) writes synced-at after `apply_writes` returns
`SotOk`. The bus caller (`bus_handler::handle_bus_export`) defers
synced-at until `push_mirror` returns Ok. This shape produces a
single `apply_writes` entry point — NO `apply_writes_bus` second
function, NO `update_synced_at: bool` flag.

**Why this shape (and not: dual entry points or a flag):** Open
Question 3 in RESEARCH.md flagged the boolean flag as cognitively
heavy. A single function with the synced-at write deferred to the
caller produces symmetric call sites — both `handle_export` and
`bus_handler` end with their own ref/audit write block. The bus
path's block branches on `MirrorResult`; the single-backend path's
block is unconditional (because for single-backend, "SoT success"
already means "mirror current" — single-backend has no separate
mirror leg). The lifted body in `write_loop.rs` is concerned only
with SoT-side correctness; mirror-side decisions live in the
respective caller.

**Implementation note (T02):** `handle_export`'s post-`apply_writes`
block writes `head` (already done by `apply_writes`), then
`synced-at`, then `mirror_sync_written` audit row, then
`log_token_cost`, then `ok refs/heads/main`. The bus path's
post-`apply_writes` block writes `head` (already done by
`apply_writes`), runs `push_mirror`, branches on outcome, writes
synced-at + `mirror_sync_written` ON success OR
`helper_push_partial_fail_mirror_lag` on failure, then `ok
refs/heads/main` either way.

**Source:** RESEARCH.md § "Open Questions for the Planner" Q-A;
Open Question 3; user directive *"`apply_writes` writes mirror refs
itself, or defers to caller? DEFER to caller"*.

### D-02 — Q-B: do NOT audit failed REST attempts in P83 (RATIFIED)

**Decision:** `apply_writes_bus`'s `execute_action` loop continues
the existing `handle_export` semantics — log only successes (the
backend adapter's per-record `audit_events` row writes inside the
adapter's success path). Failed REST attempts (e.g. 409 on PATCH
id=2) do NOT get a `helper_push_rest_failure` cache-audit row in
P83. Filed as v0.13.0 GOOD-TO-HAVES candidate for P88.

**Why defer (and not: ship the per-failure row in P83):** P83 is
already the riskiest phase. Adding a new audit op
(`helper_push_rest_failure`) plus integration coverage plus the
schema delta would expand scope unnecessarily. The existing audit
shape (per-success row in `audit_events` + the helper-RPC-turn row
in `audit_events_cache`) IS what `handle_export` ships today — bus
inherits the same trade-off. P85's troubleshooting docs may surface
user demand; if so, file a v0.14.0 row.

**Implementation note (T04 / P83-02 audit-completeness test):**
the test (c) post-precheck-409 scenario asserts `audit_events_cache`
has `helper_push_started` (always) but NOT `helper_push_accepted`
or `helper_push_partial_fail_mirror_lag` — and asserts the test
explicitly does NOT look for a per-failure row (i.e., no assertion
of the form `count(op = 'helper_push_rest_failure') == 1`).

**Source:** RESEARCH.md § "Open Question 1"; user directive *"Audit
failed REST attempts? NO — log only successes; v0.13.0 GOOD-TO-HAVE
filed for v0.14.0"*.

### D-03 — Q-C: schema delta is ATOMIC with the audit-helper-fn commit (RATIFIED)

**Decision:** P83-01 T03 lands the schema-delta + helper function +
`Cache::` wrapper in a SINGLE atomic commit. `cache_schema.sql:28-48`
gains `'helper_push_partial_fail_mirror_lag'` in the op CHECK list,
the comment narrative on lines 22-27 is extended to name P83's new
op, `audit.rs` gains `log_helper_push_partial_fail_mirror_lag`,
`cache.rs` gains the `Cache::log_helper_push_partial_fail_mirror_lag`
wrapper, and the `audit.rs::mod tests` block gains one INSERT
roundtrip test. Single commit. Matches P79 (`attach_walk` added in
one commit) + P80 (`mirror_sync_written` added in one commit)
precedent.

**Why atomic (and not: schema delta as separate prelude commit):**
the schema delta is meaningless without the helper that writes the
row, AND vice versa. Bisecting through a "schema delta only" commit
would land in a state where fresh caches accept rows that no helper
writes (semantically benign but confusing). Atomic commit is the
established pattern from P79/P80.

**Stale cache.db semantics:** existing cache.db files keep the
legacy CHECK list (per the existing comment at `cache_schema.sql:11-21`
*"On stale cache.db files the new ops will fail the CHECK and fall
through the audit best-effort path (warn-logged)"*). The audit
helper is best-effort (returns `()`, WARN-logs on INSERT failure),
so stale caches WARN-log + the push still succeeds. Fresh caches
accept the row immediately. No migration script. RESEARCH.md
Pitfall 7.

**Source:** RESEARCH.md § "Open Question for the Planner" Q-C;
user directive *"Schema delta atomic with helper, or separate
commit? ATOMIC with the audit-helper-fn commit"*; P79/P80 atomic-
schema-delta precedent.

### D-04 — Q-D: failing-update-hook fixture gated `#[cfg(unix)]` (RATIFIED)

**Decision:** the `make_failing_mirror_fixture` helper added to
`crates/reposix-remote/tests/common.rs` (P83-01 T05) writes a
POSIX shell script to `<bare>/hooks/update` and chmods it 0o755 via
`std::os::unix::fs::PermissionsExt`. The helper itself is gated
`#[cfg(unix)]` and the consuming test files
(`bus_write_mirror_fail.rs` in P83-02 T02) gate their `#[test]`
functions `#[cfg(unix)]` likewise. Windows CI (not currently
supported per CLAUDE.md "Tech stack" — Linux only) skips the test
cleanly without a build error.

**Why gate (and not: write a portable fixture):** the `update` hook
mechanism is POSIX-shell-script-based; emulating it on Windows
requires either WSL or a `pwsh` rewrite. Reposix's CI is Linux-only
at this phase (no Windows runners in `.github/workflows/`). Gating
preserves cross-platform compilation while skipping execution
where the fixture cannot run.

**Implementation note (P83-01 T05):**

```rust
#[cfg(unix)]
pub fn make_failing_mirror_fixture() -> (tempfile::TempDir, String) {
    use std::os::unix::fs::PermissionsExt;
    // ... `git init --bare`, write hooks/update with `exit 1`, chmod 0o755 ...
}
```

`#[cfg(unix)]` on the public helper means consuming tests must
gate likewise; trying to call it from a non-unix `#[cfg]` block
fails at compile time, surfacing the gating intent immediately.

**Source:** RESEARCH.md § "Open Question for the Planner" Q-D +
Assumption A2; user directive *"Failing-update-hook fixture
portable to Windows? `#[cfg(unix)]` gate if needed; reposix CI is
Linux-only at this phase"*.

### D-05 — Q-E: refactor task does NOT carry a regression catalog row (RATIFIED)

**Decision:** P83-01 T02's refactor (lift `handle_export` write
loop into `write_loop::apply_writes`) does NOT mint a catalog row
asserting "existing single-backend tests still GREEN." The
existing per-test `[[test]]` targets in
`crates/reposix-remote/Cargo.toml` (`mirror_refs`, `push_conflict`,
`bulk_delete_cap`, `perf_l1`, `stateless_connect`,
`stateless_connect_e2e`) are the regression check. Pre-push gate
runs the workspace-wide test suite; if the refactor regresses
single-backend behavior, the gate fires before the per-phase push
lands.

**Why no row (and not: mint a regression-asserting row):** the
refactor's invariant is "`handle_export` external behavior
unchanged" — best expressed as "all existing tests pass," not as a
new catalog row. A regression-asserting row would duplicate what
the test runner already proves; minting it adds noise to the
catalog without adding signal. The blast-radius of a refactor
regression is "single-backend push breaks" which is already
covered by the existing tests + their own catalog rows
(`agent-ux/sync-reconcile-subcommand`,
`agent-ux/mirror-refs-write-on-success`, etc., all of which
exercise the lifted code path).

**Implementation note (T02):** at the end of T02, run `cargo
nextest run -p reposix-remote --tests` (per-crate, sequential, per
CLAUDE.md "Build memory budget"). All existing integration tests
must pass. The verifier subagent confirms via the verdict file.

**Source:** RESEARCH.md § "Open Question for the Planner" Q-E;
user directive *"Refactor task carries its own regression catalog
row? NO — existing tests + pre-push runner are sufficient"*.

### D-06 — Q-F: NEW op `helper_push_partial_fail_mirror_lag` (RATIFIED)

**Decision:** P83-01 T03 mints a NEW audit op
`helper_push_partial_fail_mirror_lag` in `audit_events_cache` for
the SoT-succeed-mirror-fail end-state. Does NOT reuse
`mirror_sync_written` with a status-field overload.

**Why new op (and not: reuse `mirror_sync_written`):**

1. **CHECK constraint clarity.** The op CHECK list at
   `cache_schema.sql:28-48` enumerates the legitimate ops; querying
   *"all partial-fails in last 24h"* becomes one `WHERE op =
   'helper_push_partial_fail_mirror_lag'` clause. Reusing
   `mirror_sync_written` with a `status` field embedded in `reason`
   would require fragile substring-grepping (`WHERE reason LIKE
   '%fail%'`).
2. **Symmetric with `helper_push_accepted` vs
   `helper_push_rejected_conflict`.** Each push end-state has its
   own op. Adding `helper_push_partial_fail_mirror_lag` extends
   the existing pattern: each distinct end-state → distinct op.
3. **Forensic queries on the dual-table audit shape (OP-3) become
   trivially expressible.** A reader asking *"which bus pushes
   landed SoT writes but failed mirror?"* writes one SQL query
   against `audit_events_cache` with `op =
   'helper_push_partial_fail_mirror_lag'`, joined to
   `audit_events` by `(backend, project, ts)` to fetch the
   per-record mutations.

**Schema row layout** (RESEARCH.md § "Mirror-Lag Audit Row Shape"):

```
op:          'helper_push_partial_fail_mirror_lag'
backend:     <backend_name>           e.g. 'sim' / 'confluence'
project:     <project>                e.g. 'demo' / 'TokenWorld'
issue_id:    NULL                     (helper-RPC turn, not per-record)
oid:         <NEW_SHA hex>            the SoT SHA head moved to
bytes:       NULL
reason:      "exit=<N>;tail=<stderr_tail>"
ts:          <RFC3339>
```

**Helper signature** (mints in `crates/reposix-cache/src/audit.rs`,
sibling of `log_helper_push_accepted` at line 230):

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

**`Cache::` wrapper** (sibling of `log_mirror_sync_written` at
`mirror_refs.rs:274`):

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

**Source:** RESEARCH.md § "Mirror-Lag Audit Row Shape" + § "Open
Question for the Planner" Q-F; user directive *"New op vs reuse
`mirror_sync_written`? NEW op `helper_push_partial_fail_mirror_lag`
(clear semantics; clean queries)"*; P79 (`attach_walk`) + P80
(`mirror_sync_written`) precedent for distinct-op-per-end-state.

### D-07 — `agent-ux.json` is the catalog home (NOT a new `bus-write.json`)

**Decision:** P83-01 T01 + P83-02 T01 add 8 new rows to the
existing `quality/catalogs/agent-ux.json` (joining the 12 P82 rows
+ the prior P79/P80/P81 rows). NOT a new `bus-write.json`.

**Why:** dimension catalogs route to `quality/gates/<dim>/` runner
discovery — `agent-ux` is the existing dimension. Splitting it into
two catalog files would force the runner to discover both via tag,
adding indirection for no benefit. P82's D-04 set this precedent;
P83 inherits.

**Source:** RESEARCH.md § "Catalog Row Design" (recommends
`agent-ux.json`); P82 D-04.

### D-08 — Plain `git push` for the mirror push (NO `--force-with-lease`)

**Decision:** `bus_handler::push_mirror` runs
`git push <mirror_remote_name> main` — plain. NO `--force-with-lease`.
NO `--force`. NO any flag-based override. The verifier shell
`quality/gates/agent-ux/bus-write-no-helper-retry.sh` greps the
helper source for the absence of these tokens and fails RED if any
are present.

**Why plain push (and not: cargo-cult `--force-with-lease` from
P84):** P84's webhook workflow uses `--force-with-lease` because
it races with bus pushes. Bus push doesn't race with itself within
a single `git push` invocation: by the time we reach `push_mirror`,
PRECHECK A already trapped any concurrent mirror drift, AND our
SoT write IS the new authoritative state. If a concurrent
webhook-sync raced in between PRECHECK A and our `push_mirror`,
our push fails with non-fast-forward → that's the partial-fail
path → audit + ok + recover on next push. NO force.

**Implementation note (T04):**

```rust
let out = Command::new("git")
    .args(["push", mirror_remote_name, "main"])
    .output()
    .with_context(|| format!("spawn `git push {mirror_remote_name} main`"))?;
```

NO third arg in the `args(...)` list beyond `["push", name, "main"]`.

**Source:** RESEARCH.md § "Anti-Patterns to Avoid" + Pitfall 2;
user directive *"NO `--force-with-lease`. Plain `git push <mirror>
main`. P84 owns force-with-lease"*.

### D-09 — Confluence non-atomicity across actions documented as inherited semantic

**Decision:** the existing `handle_export::execute_action` per-action
loop (lines 502-512) is best-effort-stop-on-first-error. Bus path
inherits this verbatim — no transaction boundary spanning multiple
PATCH/PUT calls; on partial-fail, SoT state is `id_1: new_v, id_2:
old_v` (only id_1 succeeded before the loop bailed). RESEARCH.md
Pitfall 3 documents this. P83 plan body reproduces the documentation
in `bus_handler.rs`'s module-doc.

**Why document (and not: add atomicity):** atomic two-phase commit
across SoT actions is OUT OF SCOPE per `REQUIREMENTS.md § "Out of
Scope"` (*"Bus remote is 'SoT-first, mirror-best-effort with lag
tracking,' not 2PC. Document the asymmetry; don't try to hide
it."*). The recovery story is the user's next push reads new SoT
state via PRECHECK B's `list_changed_since` and either accepts
the change (if version still matches) or rejects with conflict.

**Test contract (P83-02 T03 fault-injection (b)):**
`bus_write_sot_fail.rs` asserts the EXACT partial state — id=1 has
new version (PATCH 200), id=2 unchanged (PATCH 500), no audit row
for ids subsequent to id=2, mirror unchanged. RESEARCH.md § "Test
(b)" is the verbatim contract.

**Source:** RESEARCH.md § "Pitfall 3"; `REQUIREMENTS.md § "Out of
Scope"`; existing `handle_export` per-action loop semantic.

### D-10 — `last_fetched_at` cursor advance is L1-trade-off documented inline

**Decision:** `apply_writes` advances `last_fetched_at` to `now`
on SoT-success — BEFORE `push_mirror`. If mirror push fails, the
cursor has advanced past the SoT state, but PRECHECK B's
`list_changed_since(cursor)` will return EMPTY on the next push
attempt (no new SoT changes between mirror-fail and retry). The
next push proceeds normally.

**Why this trade-off (and not: defer cursor advance until mirror
success):** the L1 trade-off documented in
`architecture-sketch.md § "Performance subtlety"` is the canonical
basis for this — `reposix sync --reconcile` (DVCS-PERF-L1-02
shipped P81) is the on-demand escape hatch for cache desync. Bus
inherits the L1 trust model.

**Race window:** if a confluence-side edit lands between
SoT-write-success and `push_mirror`, the cursor advance to `now`
would mask that edit on the next push. P83 documents this as a
known L1 trade-off, NOT a P83 bug. RESEARCH.md § "Open Question 2".

**Source:** RESEARCH.md § "Open Question 2"; `architecture-sketch.md
§ "Performance subtlety"`; `decisions.md § Q3.1` RATIFIED L1.
