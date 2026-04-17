# Crates overview

reposix is a Cargo workspace of eight crates. `reposix-core` is the seam: every other crate depends on it; it depends on nothing internal.

## reposix-core

The contracts. Every type and function below is tested.

### Types

| Type | Purpose |
|------|---------|
| `Issue` | Single issue. Serialized as Markdown+YAML frontmatter. |
| `IssueId(u64)` | Project-scoped unique id. `IssueId(0)` is valid; sentinel reservation is a v0.2 concern. |
| `IssueStatus` | `Open`, `InProgress`, `InReview`, `Done`, `WontFix`. |
| `ProjectSlug` | URL- and path-safe identifier (`^[A-Za-z0-9._-]{1,64}$`, rejects `.` and `..`). |
| `Project` | Container for issues. |
| `RemoteSpec` | Parsed reposix remote URL (`origin + project`). |
| `Tainted<T>` / `Untainted<T>` | Type-level taint tracking (SG-05). |
| `HttpClient` | Sealed wrapper around `reqwest::Client`. The **only** legal way to make outbound HTTP in this workspace. |
| `Error`, `Result<T>` | Shared error enum and alias. |

### Functions

| Function | Purpose |
|----------|---------|
| `http::client(opts)` | Construct an `HttpClient` honoring `REPOSIX_ALLOWED_ORIGINS`. |
| `HttpClient::request(method, url)` | Per-call allowlist recheck + 5s timeout + no redirects. |
| `HttpClient::request_with_headers(...)` | Same, with extra headers (used for `X-Reposix-Agent`). |
| `HttpClient::request_with_headers_and_body(...)` | Same, with body — used for PATCH/POST. |
| `parse_remote_url` | Parse `reposix::http://host/projects/slug` into a `RemoteSpec`. |
| `frontmatter::render(&Issue)` | Serialize to on-disk `---\n<yaml>\n---\n<body>` form. |
| `frontmatter::parse(&str)` | Inverse of `render`. |
| `sanitize(Tainted<Issue>, ServerMetadata)` | Strip server-controlled fields, return `Untainted<Issue>`. |
| `path::validate_issue_filename(&str)` | Return `IssueId` iff name matches `^[0-9]+\.md$`. |
| `path::validate_path_component(&str)` | Reject `/`, `\0`, `.`, `..`, empty. |
| `audit::SCHEMA_SQL`, `audit::load_schema(&conn)`, `audit::open_audit_db(path)` | SQLite audit-log setup with `BEFORE UPDATE/DELETE` triggers + defensive-mode open. |

### Clippy lint

`clippy.toml` at workspace root bans `reqwest::Client::new`, `reqwest::Client::builder`, and `reqwest::ClientBuilder::new` outside `crates/reposix-core/src/http.rs`. `scripts/check_clippy_lint_loaded.sh` verifies the lint actually fires.

## reposix-sim

In-process axum REST simulator. Standalone binary (`reposix-sim`) or library callable from integration tests.

### Routes

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/healthz` | Liveness probe. |
| GET | `/projects/:slug/issues` | List issues. |
| GET | `/projects/:slug/issues/:id` | Fetch one. |
| POST | `/projects/:slug/issues` | Create. |
| PATCH | `/projects/:slug/issues/:id` | Update (optional `If-Match: "<version>"` for optimistic concurrency → 409 on stale). |
| DELETE | `/projects/:slug/issues/:id` | Delete. |
| GET | `/projects/:slug/issues/:id/transitions` | List legal status transitions. |

### Middleware

Layer ordering (outermost first): `audit` → `rate-limit` → handlers. Audit captures every request including rate-limited 429s. See `crates/reposix-sim/src/middleware/`.

### Storage

`parking_lot::Mutex<rusqlite::Connection>` in `AppState`. WAL mode, `synchronous=NORMAL`, 5s `busy_timeout`. Seed data loaded from `crates/reposix-sim/fixtures/seed.json` (6 demo issues with adversarial bodies).

## reposix-fuse

FUSE daemon. `fuser` 0.17 with `default-features = false` (no `libfuse-dev` / `pkg-config` required). Runtime mounting uses `fusermount3`.

### Filesystem operations

| Op | Status | Notes |
|----|--------|-------|
| `init` | ✓ | Allocates inode registry, builds HTTP client. |
| `getattr` | ✓ | Synthetic `st_mode = 0o100444` for issues; current `uid/gid`. |
| `lookup` | ✓ | Validates filename via `path::validate_issue_filename`. |
| `readdir` | ✓ | Lists issue files; refreshes inode map from `GET /issues`. |
| `read` | ✓ | Fetches + renders frontmatter, caches rendered string by inode. |
| `open` | ✓ | Accepts RW modes in v0.1 (mount conditionally-RO). |
| `write` | ✓ | Per-inode DashMap buffer. |
| `flush` | ✓ | Accepts and returns OK. |
| `release` | ✓ | Parses buffer → `Tainted::new` → `sanitize` → PATCH with `If-Match`. |
| `create` | ✓ | POST new issue. |
| `unlink` | ✓ | Local-only (git push materializes DELETE). |
| `setattr` | ✓ | Truncate supported (for `>` redirection). |
| everything else | EROFS / ENOTSUP | Non-issue ops intentionally refused. |

### Async bridge

The FUSE struct owns an `Arc<tokio::runtime::Runtime>`. Callbacks do `rt.block_on(async { ... })`. FUSE threads are not tokio workers, so this is deadlock-safe.

## reposix-remote

`git-remote-reposix` binary. Speaks the [git remote helper protocol](https://git-scm.com/docs/gitremote-helpers) on stdin/stdout.

### Capabilities

`import`, `export`, `refspec refs/heads/*:refs/reposix/*`.

### Modules

- `protocol.rs` — stdin/stdout framing. `#![deny(clippy::print_stdout)]` is set; stdout is protocol-reserved and no code outside `protocol::send_line` / `send_raw` writes to it.
- `main.rs` — helper entry point. Constructs an `Arc<SimBackend>` from the parsed `RemoteSpec` and dispatches every list / create / update / delete through the `IssueBackend` trait (Phase 14 rewire; the former `client.rs` sim-REST wrapper is deleted).
- `fast_import.rs` — `emit_import_stream` (backend → git) and `parse_export_stream` (git → backend). Uses `frontmatter::render` for deterministic blob bytes.
- `diff.rs` — `plan(prior, parsed)` computes per-issue `PATCH` / `POST` / `DELETE` actions. Returns `BulkDeleteRefused` on > 5 deletes (SG-02), unless commit message contains `[allow-bulk-delete]`.

## reposix-cli

Top-level `reposix` binary. `clap`-derive CLI.

### Subcommands

| Command | Purpose |
|---------|---------|
| `reposix sim [flags]` | Spawn `reposix-sim` as a child process. All flags plumb through. |
| `reposix mount <path> --backend <origin> --project <slug>` | Mount the FUSE daemon foreground. Ctrl-C unmounts. |
| `reposix demo [--keep-running]` | End-to-end orchestration: spawn sim → mount → scripted ls/cat/grep → tail audit log → cleanup. |
| `reposix version` | Print the version. |

### Guard struct

`demo` uses a top-level `Guard` owning sim child + mount child + tempdir. `Drop` tears them down in reverse order. A `tokio::signal::ctrl_c()` handler races the step sequence via `tokio::select!`, ensuring Ctrl-C runs the same cleanup. `fusermount3 -u` is wrapped in a 3-second watchdog to prevent hang on lazy unmount.

## reposix-github

Read-only `IssueBackend` adapter for the GitHub REST v3 Issues API. Ships in v0.2.

Maps GitHub's 2-valued `state` + `state_reason` + `status/*` label convention onto reposix's 5-valued `IssueStatus`. See [ADR-001](../decisions/001-github-state-mapping.md) for the full mapping table. Write path (`create_issue`, `update_issue`, `delete_or_close`) deferred to v0.2.

Requires `GITHUB_TOKEN` env var. Uses `reposix_core::http::HttpClient` (SG-01 allowlist enforced).

## reposix-confluence

Read-only `IssueBackend` adapter for Atlassian Confluence Cloud REST v2. Ships in v0.3. Comments overlay (Phase 23) ships in v0.5.

### Capabilities by version

| Version | Capability |
|---------|-----------|
| v0.3 | `list_issues`, `get_issue` — reads Confluence pages as flat `Issue` records |
| v0.4 | `pages/` + `tree/` mount layout (ADR-003 nested layout) |
| v0.5 | `pages/<id>.comments/` overlay — inline and footer comments as read-only Markdown files |

Maps Confluence pages to `Issue` via `translate_page`; see [ADR-002](../decisions/002-confluence-page-mapping.md) for field assignments and lost-metadata list.

Requires `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` env vars plus `REPOSIX_ALLOWED_ORIGINS` containing the tenant origin. See [Confluence backend reference](confluence.md) for credential setup.

Key methods beyond the `IssueBackend` trait:
- `list_issues_strict` — like `list_issues` but errors instead of truncating at the 500-page cap (used by `--no-truncate`).
- `list_spaces` — lists all readable Confluence spaces; used by `reposix spaces`.
- `list_comments(page_id)` — fetches inline + footer comments for a page; used by the FUSE `.comments/` overlay.

## reposix-swarm

Adversarial swarm harness for load testing and concurrency validation. Not a user-facing binary; used in CI and by `cargo test -p reposix-swarm`.

Runs N concurrent simulated agents against either the simulator (via HTTP) or a mounted FUSE tree. Each agent runs a realistic workload loop (list + reads + patch) and records per-operation latencies. Emits a Markdown summary with P50/P95/P99 per op type, total requests, error rate, and an audit-row invariant check (SG-06 append-only).

Motivation: the "10k agent QA team" pattern from the StrongDM dark-factory playbook. See `docs/research/agentic-engineering-reference.md` §1.
