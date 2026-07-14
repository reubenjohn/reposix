← [back to index](./index.md) · phase 83 research

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

