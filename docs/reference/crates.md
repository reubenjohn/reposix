# Crates overview

reposix is a Cargo workspace of nine crates. `reposix-core` is the seam: every other crate depends on it; it depends on no internal crate. All crates are currently path-only — none publish to crates.io yet (release readiness tracked under v0.11.0).

## reposix-core

The contracts. Shared types, traits, and security-critical primitives.

### Public API entry points

- `BackendConnector` (trait, `backend.rs`) — the seam every adapter implements: `list_records`, `get_record`, `create_record`, `update_record`, `delete_or_close`, `list_changed_since`, plus `supports(BackendFeature)` for capability queries.
- `Record`, `RecordId`, `RecordStatus`, `Project`, `ProjectSlug`, `RemoteSpec`, `parse_remote_url` — the core record vocabulary.
- `Tainted<T>` / `Untainted<T>` / `sanitize` — type-level taint tracking enforcing SG-05.
- `HttpClient` (sealed) — the single legal way to make outbound HTTP. Honors `REPOSIX_ALLOWED_ORIGINS` per call.
- `audit::SCHEMA_SQL`, `audit::open_audit_db` — append-only audit log with `BEFORE UPDATE/DELETE` triggers (SG-06).
- `frontmatter::render` / `parse` — deterministic Markdown+YAML serialization for on-disk records.
- `path::validate_record_filename`, `slug_or_fallback`, `dedupe_siblings` — working-tree filename hygiene.

A workspace-wide `clippy.toml` bans `reqwest::Client::new` outside `core/src/http.rs`; `scripts/check_clippy_lint_loaded.sh` proves the lint fires.

### Used by

Every other crate.

## reposix-cache

Backing bare-repo cache built from `BackendConnector` responses. The substrate of the v0.9.0 git-native architecture.

### Public API entry points

- `Cache::open(backend, backend_name, project)` — open or create the cache at a deterministic path under `$XDG_CACHE_HOME/reposix/`.
- `Cache::build_from(...)` → `SyncReport` — list all issues, write a tree object with one entry per record. Tree metadata is eager; blobs are not.
- `Cache::read_blob(oid)` — materialize a single blob on demand (the lazy-blob invariant — see Phase 31 plans).
- `audit::*` — append-only audit table inside `cache.db`; vocabulary includes `helper_push_*` and `blob_limit_exceeded` per Phase 32 / 34.

### Used by

`reposix-remote` (the helper reads + writes through the cache), integration tests in `reposix-cli`.

## reposix-sim

In-process axum REST simulator. Standalone binary `reposix-sim` and a library callable from integration tests.

### Public API entry points

- `reposix_sim::run(...)` — start an `axum` server on a configured `SocketAddr`.
- Routes: `GET /healthz`, `GET/POST /projects/:slug/issues`, `GET/PATCH/DELETE /projects/:slug/issues/:id`, `GET /projects/:slug/issues/:id/transitions`. PATCH honors `If-Match: "<version>"` for optimistic concurrency.
- Middleware order: `audit` → `rate-limit` → handlers. Audit captures rate-limited 429s.

### Used by

The default backend for tests, demos, the `dark-factory-test.sh` regression, and the `BackendConnector` parity contract tests in every adapter.

## reposix-remote

`git-remote-reposix` binary. Speaks the [git remote helper protocol](https://git-scm.com/docs/gitremote-helpers) on stdin / stdout.

### Public API entry points

- `main.rs` advertises `import`, `export`, `refspec refs/heads/*:refs/reposix/*`, and `stateless-connect`. The helper dispatches RPC turns to:
  - `stateless_connect.rs::handle_stateless_connect` — the v0.9 read path; tunnels protocol-v2 over the cache.
  - `fast_import.rs::emit_import_stream` / `parse_export_stream` — backend ↔ git stream conversion using `frontmatter::render` for deterministic blob bytes.
  - `diff.rs::plan(prior, parsed)` — per-issue PATCH / POST / DELETE planning. Refuses >5 deletes (SG-02) unless the commit message contains `[allow-bulk-delete]`.

`#![deny(clippy::print_stdout)]` is set; only `protocol::send_line` and `send_raw` may write to stdout (it is protocol-reserved).

### Used by

`git fetch` / `git push` against a `reposix::...` remote URL, end-to-end via `reposix init`.

## reposix-cli

Top-level `reposix` binary. `clap`-derive CLI; the orchestrator described in [reference/cli.md](cli.md).

### Public API entry points

- Subcommands: `init`, `sim`, `list`, `refresh`, `spaces`, `version`.
- `reposix_cli::list`, `reposix_cli::refresh`, `reposix_cli::spaces` re-exported as a library so integration tests can drive the same code paths the CLI does.

### Used by

End users, CI workflows, and the `agent_flow` regression suite.

## reposix-github

Read-only `BackendConnector` adapter for the GitHub REST v3 Issues API.

### Public API entry points

- `GithubReadOnlyBackend::new(token)` — constructs a backend whose `list_records` / `get_record` hit `https://api.github.com`.
- Status mapping decided in [ADR-001](../decisions/001-github-state-mapping.md); the backend translates GitHub's two-valued `state` + `state_reason` + `status/*` labels into the five-valued `RecordStatus`.

### Used by

`reposix init github::<owner>/<repo>` and the `integration-contract-github-v09` CI job.

## reposix-confluence

`BackendConnector` adapter for Atlassian Confluence Cloud REST v2. Read + write per Phase 22 / 24; comments overlay per Phase 23.

### Public API entry points

- `ConfluenceBackend::new(creds, tenant)` — backend bound to one Atlassian tenant.
- `list_records`, `get_record`, `create_record`, `update_record`, `delete_or_close` — the `BackendConnector` surface, mapped page → record per [ADR-002](../decisions/002-confluence-page-mapping.md).
- `list_records_strict` — variant of `list_records` that errors instead of capping at 500 pages (used by `reposix list --no-truncate`).
- `list_spaces` — backs `reposix spaces`.
- `list_comments(page_id)` — inline + footer comments.
- `adf` module — Markdown ↔ Atlassian Document Format converter, shared with `reposix-jira`.

Requires `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`, plus the tenant origin in `REPOSIX_ALLOWED_ORIGINS`. See [reference/confluence.md](confluence.md).

### Used by

`reposix init confluence::<space>`, the live tenant smoke against `TokenWorld`.

## reposix-jira

`BackendConnector` adapter for Atlassian JIRA Cloud REST v3. Read (Phase 28) + write (Phase 29).

### Public API entry points

- `JiraBackend::new(creds, instance)` — backend bound to one JIRA Cloud instance.
- Read path: `POST /rest/api/3/search/jql` with cursor pagination via `nextPageToken`; `GET /rest/api/3/issue/{id}` for single fetch.
- Write path: `create_record` (POST `/rest/api/3/issue`), `update_record` (PUT `/rest/api/3/issue/{id}`), `delete_or_close` (transitions API with DELETE fallback).
- Issue → record mapping documented in [ADR-005](../decisions/005-jira-issue-mapping.md).
- `adf` module — shared with `reposix-confluence`.

Honors HTTP 429 `Retry-After`; falls back to exponential backoff with jitter (max 4 attempts).

### Used by

`reposix init jira::<key>`, the live JIRA smoke against the `TEST` project.

## reposix-swarm

Adversarial swarm harness for load testing and concurrency validation. Not a user-facing binary; used in CI and `cargo test -p reposix-swarm`.

### Public API entry points

- Modes: `sim-direct` (HTTP to the simulator), `confluence-direct` (HTTP to a real Confluence tenant), `contention` (N clients patching the same record via `If-Match`, proving 409 determinism).
- Emits a Markdown summary with P50 / P95 / P99 per op type, total requests, error rate, and an audit-row invariant check (SG-06 append-only).

Motivation: the "10k agent QA team" pattern from the StrongDM dark-factory playbook (see `docs/research/agentic-engineering-reference.md` §1).

### Used by

The `chaos_audit` CI job and ad-hoc local soak tests.
