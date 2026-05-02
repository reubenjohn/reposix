← [back to index](./index.md) · phase 83 research

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

