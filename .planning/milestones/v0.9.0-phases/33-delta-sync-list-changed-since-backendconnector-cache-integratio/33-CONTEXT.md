# Phase 33: Delta sync — `list_changed_since` on `BackendConnector` + cache integration - Context

**Gathered:** 2026-04-24
**Status:** Ready for planning
**Mode:** Auto-generated (discuss skipped via workflow.skip_discuss=true)

<domain>
## Phase Boundary

Add incremental backend queries so `git fetch` after a backend mutation transfers only the changed issue's tree+blob, not the whole project. This phase wires four backends (`SimBackend`, `GithubBackend`, `ConfluenceBackend`, `JiraBackend`) into a unified `list_changed_since(timestamp)` trait method, persists `last_fetched_at` in the Phase 31 cache DB, and integrates the delta-sync flow into the helper's `command=fetch` path so the agent's pure-git workflow (`git fetch && git diff --name-only origin/main`) just works.

This phase is the bridge between Phase 31's "build cache from full REST listing" and Phase 32's "tunnel protocol-v2 from cache to git." Without it, every `git fetch` re-syncs the entire backend — which is correct but expensive. After this phase, fetch is incremental and idempotent.

The "since" parameter values per backend are locked from architecture-pivot-summary §4: GitHub `?since=<ISO8601>`, Jira `JQL: updated >= "<datetime>"`, Confluence `CQL: lastModified > "<datetime>"`. The simulator implements its own `?since=` query parameter to be backwards-compatible with v0.8.0 callers (absent param → return all).

</domain>

<decisions>
## Implementation Decisions

### Operating-principle hooks (non-negotiable — per project CLAUDE.md)

- **Simulator-first (OP-1).** All delta-sync tests run against `SimBackend`. Real-backend exercise is Phase 35's job under explicit credentials and a non-default `REPOSIX_ALLOWED_ORIGINS`. The simulator's REST surface gains a `?since=<ISO8601>` query parameter for this phase.
- **Audit log non-optional (OP-3).** One audit row per delta-sync invocation, schema: `(ts, backend, project, since_ts, items_returned, op="delta_sync")`. The audit row is written in the same SQLite transaction as the `last_fetched_at` update (atomicity — see below).
- **Ground truth obsession (OP-6).** Test assertion: after `agent_a` mutates exactly issue `proj-1/42` on the simulator and `agent_b` runs `git fetch origin`, `git diff --name-only origin/main` returns exactly `issues/42.md`. Other blob OIDs MUST be unchanged. Synthetic mocks of "delta returned 1 item" do not satisfy this — the assertion is on git's view of the working tree.
- **No hidden state (OP-4).** `last_fetched_at` lives in the cache DB at the deterministic path from Phase 31 (`$XDG_CACHE_HOME/reposix/<backend>-<project>.git/cache.db`, overridable via `REPOSIX_CACHE_DIR`). No env-var fallbacks, no per-process timestamp caches.

### Trait method signature (locked)

```rust
#[async_trait]
pub trait BackendConnector {
    // ... existing methods ...

    /// Returns issue IDs whose `updated_at` is strictly greater than `since`.
    /// Backend-native query: GitHub `?since=`, Jira `JQL updated >=`,
    /// Confluence `CQL lastModified >`. Sim filters its in-memory set.
    ///
    /// # Errors
    /// Returns `Error::Backend` on transport / parse failure.
    /// Returns `Error::EgressDenied` if the backend origin is not in
    /// `REPOSIX_ALLOWED_ORIGINS`.
    async fn list_changed_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<IssueId>, Error>;
}
```

`IssueId` is the existing `reposix_core` newtype. Returning IDs (not full issues) lets the cache decide whether to materialize blobs eagerly or lazily — symmetry with Phase 31's lazy-blob design.

### Atomic ordering (locked)

For delta sync, the sequence is:

1. Read `last_fetched_at` from cache DB (one SQLite read).
2. Call `BackendConnector::list_changed_since(last_fetched_at)`.
3. For each returned ID, fetch full issue via `get_issue(id)`, materialize blob into bare repo, update tree.
4. **Inside one SQLite transaction:** write the new `last_fetched_at` row AND the audit row. Commit.

Rationale: if step 3 partially completes and then crashes, `last_fetched_at` is unchanged and the next sync retries the still-pending IDs (idempotent). If steps 1–3 succeed but the transaction in step 4 fails, we double-fetch on the next sync but never lose data. Tree sync is unconditional — full tree advertised every fetch — so partial blob materialization is recoverable.

The kill-9 chaos test in success criterion #5 borrows the Phase 21 HARD-03 pattern: spawn a child mid-sync, SIGKILL it, then run sync again and assert convergence with the post-sync expected state.

### Tree sync vs. blob materialization (locked)

Tree sync is **unconditional and not gated by `REPOSIX_BLOB_LIMIT`**. Tree metadata is small (architecture-pivot-summary §4: "a project with 10,000 issues produces maybe 500KB of tree objects"). The blob limit only applies in Phase 34's helper guard on `command=fetch` `want` lines.

This means after Phase 33 lands, `git fetch` always advertises the full updated tree even if the agent has narrowed sparse-checkout to a single issue. That is correct — sparse-checkout controls *blob materialization*, not tree visibility.

### Test surface

- `trait_method_implemented_for_all_backends` — compile-time assert: `impl BackendConnector for SimBackend, GithubBackend, ConfluenceBackend, JiraBackend` all carry `list_changed_since`.
- `sim_respects_since_param` — seed sim with 5 issues; mutate 1; `list_changed_since(t_before_mutation)` returns exactly 1.
- `sim_absent_since_returns_all` — backwards-compatibility check: a request without the `since` param returns the full set.
- `delta_sync_one_issue_one_blob_diff` — the headline test. Two simulator clients, mutation by agent_a, `git fetch` by agent_b, exactly one path in `git diff --name-only origin/main`.
- `audit_row_per_delta_sync` — one `op=delta_sync` row per invocation; `since_ts` and `items_returned` populated.
- `transaction_atomicity_chaos` — kill-9 between step 3 and step 4 leaves `last_fetched_at` unchanged; rerun converges.
- `tree_sync_unbounded` — set `REPOSIX_BLOB_LIMIT=1`; tree sync of a 100-issue project still succeeds (the limit applies only at fetch-RPC time, not at tree sync time).

### Claude's Discretion

Whether `list_changed_since` is a default method on the trait (with a fallback that calls `list_issues` and filters in memory — useful for backends that lack a native incremental query) is at Claude's discretion. The four target backends all have native queries, so the fallback is purely belt-and-suspenders.

Whether to add a synthesis-commit-per-delta-sync (one new commit per fetch round) or to amend the single synthesis commit from Phase 31 is at Claude's discretion. Multi-commit is closer to git's model and helps `git log` tell a story; single-commit is simpler. Phase 31 deferred this; Phase 33 may continue to defer or pick. Recommend: continue to defer until v0.10.0 observability work — pick the simpler single-commit-per-sync approach now.

</decisions>

<code_context>
## Existing Code Insights

### Reusable assets

- `reposix_core::BackendConnector` trait (`crates/reposix-core/src/backend.rs`) — Phase 27's rename landed this name; `list_changed_since` is the new method to add. Keep the existing `list_issues` as the v0.8.0 fallback.
- Existing `list_issues` impls in each backend crate (`crates/reposix-sim/src/`, `crates/reposix-github/src/`, `crates/reposix-confluence/src/`, `crates/reposix-jira/src/`) — these are the templates the new method follows. Each backend already has its REST/GraphQL surface area mapped; the new method is one more call shape.
- `crates/reposix-cli/src/cache_db.rs` — currently holds the `refresh_meta` single-row table with `last_fetched_at`. Lift / reuse this in `reposix-cache` (the Phase 31 crate). Phase 33 may move this code if Phase 31 hasn't already.
- Phase 21 `HARD-03` chaos pattern — kill-9 mid-operation + assert recovery. Reuse the harness shape.
- `chrono::DateTime<Utc>` is the established serialized form (per CLAUDE.md "Times are `chrono::DateTime<Utc>`").

### Established patterns

- `Issue.extensions: BTreeMap<String, serde_yaml::Value>` (Phase 27) — backend-specific metadata lives here. `updated_at` is on the core `Issue` struct, not in extensions.
- Backend tests use `wiremock` for HTTP fixtures (see Phase 11 `reposix-confluence`). Real-backend tests gated `#[ignore]` until creds present.
- SQLite WAL + append-only audit triggers from Phase 31 — extend with the new `delta_sync` op without schema migration.

### Integration points

- Phase 31's `reposix-cache` consumes the new trait method during `Cache::sync_delta()`.
- Phase 32's helper invokes `Cache::sync_delta()` on every `command=fetch` request before the protocol-v2 tunnel kicks in.
- Phase 35 will exercise the delta path against real backends; Phase 33 only validates against `SimBackend`.

</code_context>

<specifics>
## Specific Ideas

- Simulator REST surface adds: `GET /projects/<id>/issues?since=<ISO8601>` filters its in-memory set. Absent `since` → returns all (backwards-compatible).
- Audit row schema reuse: same `audit` table introduced in Phase 31, just a new `op="delta_sync"` value. Columns `since_ts` and `items_returned` use the existing free-form text/blob columns or the meta JSON column — Claude's discretion.
- ISO8601 strings round-trip through `chrono::DateTime<Utc>::to_rfc3339()` / `parse_from_rfc3339()`. No second-precision rounding losses.
- The "exactly one path in `git diff --name-only origin/main`" assertion is the headline ground-truth test. Two scratch git working trees, one shared simulator instance, mutation in agent_a, fetch in agent_b — the test must be deterministic (no timing flakiness; agent_a's `updated_at` is strictly greater than the prior `last_fetched_at`).
- Confluence / Jira / GitHub real-backend tests gated `#[ignore]` until Phase 35; Phase 33 only ships the wiremock contract tests for those backends.

</specifics>

<deferred>
## Deferred Ideas

- Multi-commit synthesis history (one commit per delta sync) — v0.10.0 observability/maintenance milestone.
- Real-backend delta validation against TokenWorld / `reubenjohn/reposix` / JIRA `TEST` — Phase 35.
- Delta-sync push-side mirror: backend mutation made by *this* helper updating `last_fetched_at` to skip a round-trip — Phase 34 may revisit; default is "let the next fetch pick it up" for simplicity.
- Cache eviction (LRU/TTL/quota) — architecture-pivot-summary §7 Q1, deferred indefinitely.
- Backwards-incompatible removal of `list_issues` — kept for one release cycle past v0.9.0; never removed in this phase.

</deferred>
</content>
