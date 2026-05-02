# Open Questions (Resolved)

> **Resolution note:** These questions were raised during research and resolved in
> `83-PLAN-OVERVIEW.md § D-01..D-10` (see also `83-PLAN-OVERVIEW.md` for the
> authoritative decision record). The question content is preserved here verbatim
> for lossless archival — the research file is the canonical record of what was
> uncertain before planning began.

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

## Open Questions for the Planner (RESOLVED — see 83-PLAN-OVERVIEW.md § D-01..D-10)

1. **Q-A** Does `apply_writes` write the `synced-at` ref itself for the single-backend path, or always defer to the caller? (Pattern 1 + Open Question 3.) — recommend: defer to caller, which means `handle_export` gets one extra `write_mirror_synced_at` line at its end. Symmetric with bus path.
2. **Q-B** Do we audit failed REST attempts (Open Question 1)? — recommend: NO for P83; defer to v0.13.0 GOOD-TO-HAVES.
3. **Q-C** Schema delta — does the new `op` get added to the inline `cache_schema.sql` comment narrative AND the CHECK list in the same task as the helper, or is the schema delta a separate prelude commit? — recommend: same atomic commit (T03). Matches P79/P80 pattern.
4. **Q-D** Is the failing-update-hook fixture portable to Windows CI? CLAUDE.md doesn't list Windows as a supported dev OS but cross-platform tests exist. — recommend: gate the test with `#[cfg(unix)]` if needed; document in fixture helper.
5. **Q-E** Should P83a's T02 refactor (`apply_writes` factor-out) carry its own catalog row asserting the regression invariant (existing `handle_export` tests still GREEN)? — recommend: NO; the existing `mirror_refs` / `push_conflict` / `bulk_delete_cap` integration tests ARE the regression check; pre-push gate is sufficient.
6. **Q-F** Reuse `mirror_sync_written` op vs new op? (Mirror-Lag Audit Row Shape section.) — recommend: NEW op `helper_push_partial_fail_mirror_lag`. Decided in research; confirm in plan.
