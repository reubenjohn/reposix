# Phase 2: Simulator + audit log — Context

**Gathered:** 2026-04-13
**Status:** Ready for planning
**Source:** Auto-generated from PROJECT.md + ROADMAP.md + research/ (discuss step skipped per user instruction)

<domain>
## Phase Boundary

**In scope:**
- `reposix-sim` binary becomes a real REST issue tracker on `127.0.0.1:7878` (or env-overridable bind).
- Endpoints (GitHub-shaped paths, hybrid GitHub+Jira semantics):
  - `GET    /healthz`
  - `GET    /projects/:slug/issues` (list, returns JSON array of issues)
  - `GET    /projects/:slug/issues/:id` (returns single issue with `frontmatter` + `body`)
  - `POST   /projects/:slug/issues` (create — agent-friendly minimal body)
  - `PATCH  /projects/:slug/issues/:id` (update; honors `If-Match` header for optimistic concurrency → 409 on stale `version`)
  - `DELETE /projects/:slug/issues/:id` (delete)
  - `GET    /projects/:slug/issues/:id/transitions` (Jira-style: returns valid status transitions)
- Persistent SQLite store at `runtime/sim.db` (WAL mode). Two tables: `issues` (mutable) and `audit_events` (append-only, schema from `reposix-core::audit::SCHEMA_SQL`).
- Seed loader: `--seed-file crates/reposix-sim/fixtures/seed.json` populates a `demo` project with at least 3 issues including realistic statuses + bodies. Deterministic.
- Audit middleware (`axum::middleware::from_fn`): every HTTP request writes a row (timestamp, agent_id from `X-Reposix-Agent` header, method, path, status, request body hash, response status). Append-only enforced by Phase 1 triggers.
- Rate limiting via `governor` (or `tower_governor`): 100 req / sec / agent default; 429 with `Retry-After` on overflow. Configurable via `--rate-limit`.
- 409 conflict path: `PATCH` with stale `If-Match: "<version>"` returns 409 with current state in body. This is what Phase S's `git push` will turn into a merge conflict.
- All HTTP clients used internally must come from `reposix_core::http::client()` (none expected in this phase, but the discipline matters for future contributor sanity).

**Out of scope:**
- Authentication beyond the `X-Reposix-Agent` header (no JWT, no OAuth — v0.2).
- Pagination (returns full list — fine for v0.1 with ~3-30 issues).
- Webhook delivery (no event emission to external listeners).
- Bulk-delete cap on the API (lives on the push path, Phase S).
- Multi-tenant isolation beyond the URL slug (no per-tenant DB).
- Jira-style workflow rule enforcement at the API layer (status validation is best-effort; the v0.1 sim accepts any of the 5 `IssueStatus` values; "must transition through in_progress" is deferred).

</domain>

<decisions>
## Implementation Decisions

### Storage
- `rusqlite` (workspace dep), `bundled` feature. WAL mode (`PRAGMA journal_mode=WAL`).
- `parking_lot::Mutex<rusqlite::Connection>` as `AppState` field. Single global connection. The audit-write happens inline with the response; if it ever becomes a hot spot, move to a `tokio::spawn` background writer.
- DDL for `issues` table lives in `crates/reposix-sim/src/db.rs` (next to the code that uses it). DDL for `audit_events` is loaded via `reposix_core::audit::load_schema(&conn)`.

### Routing
- `axum` 0.7 (workspace dep).
- `Router::new()` with all routes registered in `crates/reposix-sim/src/routes.rs`.
- Layer ordering (outermost first): `audit_middleware` → `rate_limit_middleware` → handlers. Audit captures even rate-limited requests so the audit log is the source of truth.
- Errors use a custom `ApiError` enum that implements `IntoResponse` returning consistent JSON error bodies: `{"error": "kind", "message": "human readable", "details": {...}}`.

### Seed data (`fixtures/seed.json`)
```json
{
  "project": {"slug": "demo", "name": "Demo Project", "description": "Reposix walkthrough fixtures"},
  "issues": [
    {"id": 1, "title": "database connection drops under load", "status": "open", "labels": ["bug","p1"], "body": "..."},
    {"id": 2, "title": "add `--no-color` flag to CLI", "status": "in_progress", "assignee": "agent-alpha", "labels": ["enhancement"], "body": "..."},
    {"id": 3, "title": "document the new auth flow", "status": "in_review", "labels": ["docs"], "body": "..."}
  ]
}
```
Filenames will be `0001.md`, `0002.md`, `0003.md` (Phase 1's `validate_issue_filename` requires `<digits>.md`; Phase 3's FUSE rendering uses 4-digit zero-padded). `id` is `IssueId(u64)`.

### Optimistic concurrency
- Each `Issue` row has a `version BIGINT NOT NULL DEFAULT 1` column.
- `PATCH` reads request header `If-Match: "<version>"` (RFC 7232 quoted etag). If absent, treat as wildcard match (allow). If present and != current version, return 409 with body `{"error":"version_mismatch","current":<n>,"sent":<n>}`.
- On success, increment `version`, update `updated_at`, return new state.

### Rate limiting
- Per-agent (from `X-Reposix-Agent` header; default `"anonymous"` if absent).
- `governor::Quota::per_second(NonZeroU32::new(100).unwrap())` default. CLI flag `--rate-limit <rps>` overrides.
- 429 response includes `Retry-After` in seconds.

### CLI surface
- `reposix-sim --bind 127.0.0.1:7878 --db runtime/sim.db --seed-file <path> [--rate-limit 100]`
- `--no-seed` skips seeding (resume against existing DB).
- `--ephemeral` uses `:memory:` for DB (overrides `--db`).

### Tests
- Unit tests next to handlers.
- Integration test in `crates/reposix-sim/tests/api.rs` that:
  - boots the sim on `127.0.0.1:0` (random port)
  - lists issues, asserts ≥ 3
  - patches one with bogus `If-Match`, asserts 409
  - reads audit DB, asserts the row count grew
  - attempts UPDATE on `audit_events`, asserts trigger error
- One `#[ignore]`-gated property test for the optimistic-concurrency invariant if time permits (not blocking).

### Claude's discretion
- Whether to use `tower_governor` crate or hand-roll a `tower::Layer` over `governor::DefaultDirectRateLimiter`. Hand-roll is fine; whichever lets us pass the integration test in <1 hour.
- Exact JSON shape for issue resource (mirror `reposix_core::Issue` Serialize output; if the `id` zero-padding is needed at the JSON level, document it).
- Whether to expose a `/audit` endpoint that streams recent rows (nice for the demo dashboard but not blocking — Phase 4 can add it).

</decisions>

<canonical_refs>
## Canonical References

- `.planning/research/simulator-design.md` — the primary blueprint. Especially §3 (state model), §4 (audit), §5 (seeder), §7 (axum skeleton + layer order).
- `.planning/research/threat-model-and-critique.md` — confirms append-only audit + 409-as-merge-conflict are non-negotiable.
- `crates/reposix-core/src/audit.rs` — Phase 1 schema fixture; Phase 2 calls `audit::load_schema(conn)`.
- `crates/reposix-core/src/issue.rs` — `Issue`, `IssueId`, `IssueStatus` types this phase serializes.
- `crates/reposix-core/src/path.rs` — `validate_issue_filename`; if the sim ever returns filenames in payloads, validate them through this.
- [axum 0.7 docs](https://docs.rs/axum/0.7/)
- [governor crate](https://docs.rs/governor/)
- [rusqlite WAL mode](https://www.sqlite.org/wal.html)

</canonical_refs>

<specifics>
## Specific Ideas

- The audit middleware should compute and store a SHA-256 hash of the request body (truncated to 16 hex chars) instead of the body itself for non-obvious-PII bodies — but for v0.1 simplicity, store the first 256 chars of the body verbatim. Document this in CLAUDE.md as a v0.2 hardening item.
- Seed data should include at least one issue with a body containing a `<script>` tag and another with markdown that includes a fake `version: 999` line — these become the test corpus for "frontmatter-stripping survives adversarial bodies."

</specifics>

<deferred>
## Deferred Ideas

- HTTP/2, TLS (rustls is in deps but plaintext is fine for `127.0.0.1`).
- A `/dashboard` web UI (Phase 4 might add a tiny one).
- Listing endpoint pagination + filtering.
- Workflow transition enforcement in PATCH (Jira-style — out of scope for v0.1).

</deferred>

---

*Phase: 02-simulator-audit-log*
*Context: 2026-04-13 via auto-mode*
