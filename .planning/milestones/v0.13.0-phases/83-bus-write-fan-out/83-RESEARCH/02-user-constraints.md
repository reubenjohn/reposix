← [back to index](./index.md) · phase 83 research

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

