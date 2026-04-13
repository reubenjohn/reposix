# Phase 1: Core contracts + security guardrails — Context

**Gathered:** 2026-04-13
**Status:** Ready for planning
**Source:** Auto-generated from PROJECT.md + ROADMAP.md + research/ (discuss step skipped per user instruction at 12:55 PDT)

<domain>
## Phase Boundary

**In scope (this phase ships):**
- `reposix_core::http::client()` factory: the only legal way to construct a `reqwest::Client` in this workspace. Honors `REPOSIX_ALLOWED_ORIGINS` env var (default: `http://127.0.0.1:*,http://localhost:*`). Refuses redirects.
- `Tainted<T>` / `Untainted<T>` newtype pair + `sanitize()` that strips `id`, `created_at`, `version`, `updated_at` from inbound frontmatter.
- `validate_issue_filename(name: &str) -> Result<IssueId, Error>`: rejects `/`, `\0`, `.`, `..`, anything that isn't `<digits>.md`.
- `crates/reposix-core/fixtures/audit.sql`: SQLite DDL for the `audit_events` table with `BEFORE UPDATE`/`BEFORE DELETE` triggers that `RAISE(ABORT, …)`.
- `examples/show_audit_schema.rs`: prints the DDL.
- `clippy.toml` at workspace root with `disallowed-methods` rule banning `reqwest::Client::new` and `reqwest::ClientBuilder` outside `crates/reposix-core/src/http.rs`.
- Tests for every guardrail above. Including a `trybuild` compile-fail test for the `Tainted`/`Untainted` discipline.

**Out of scope (other phases):**
- Actually wiring `Tainted<T>` into the simulator response path → Phase 2.
- Enforcing `validate_issue_filename` at the FUSE boundary → Phase 3.
- Writing audit rows into the table → Phase 2.
- Bulk-delete cap (lives on the push path, no MVD surface) → Phase S.
- Demo recording showing guardrails firing → Phase 4.

</domain>

<decisions>
## Implementation Decisions

### HTTP client factory
- **Lives at:** `crates/reposix-core/src/http.rs`, exported as `reposix_core::http::client(opts: ClientOpts) -> Result<Client>`.
- **Allowlist source:** `REPOSIX_ALLOWED_ORIGINS` env var, comma-separated, glob-style (`http://127.0.0.1:*` matches any port). When unset, defaults to `http://127.0.0.1:*,http://localhost:*`.
- **Wildcard semantics:** Only `*` for port is supported in v0.1. Path globbing is out of scope. Scheme matters (`http://` and `https://` are different origins).
- **Redirects:** disabled (`reqwest::redirect::Policy::none()`). A redirect to a non-allowlisted host would be a one-shot data exfil channel.
- **Timeouts:** 5-second total request timeout (matches SG-07).
- **Enforcement at construction:** `client()` does not gate per-request; instead, every HTTP request goes through a thin `request(client, method, url) -> Result<Response>` wrapper that re-checks the URL against the allowlist before sending. Belt-and-braces because `reqwest::Client` lets callers override the URL.
- **Disallowed-methods clippy lint:** `clippy.toml` lists `reqwest::Client::new`, `reqwest::Client::builder`, `reqwest::ClientBuilder::new`. The lint runs on the whole workspace; the only file allowed to use those constructors is `crates/reposix-core/src/http.rs` (which uses an `#[allow(clippy::disallowed_methods)]` line at the construction site, with a comment explaining why).

### Tainted / Untainted typing
- **Lives at:** `crates/reposix-core/src/taint.rs`, exported as `reposix_core::{Tainted, Untainted, sanitize}`.
- **Shape:** `pub struct Tainted<T>(T);` and `pub struct Untainted<T>(T);`. Both are `#[derive(Debug, Clone, PartialEq, Eq)]`. Both implement `AsRef<T>`. Neither implements `Deref` (deliberately; we don't want auto-deref to leak the inner value into untainted-only APIs).
- **Construction:** `Tainted::new(value)` is `pub`; `Untainted::new(value)` is `pub(crate)` for now. The only legal user-code path to `Untainted<T>` is `sanitize(tainted: Tainted<Issue>) -> Untainted<Issue>` (and analogous `sanitize_url`, `sanitize_string` if needed).
- **`sanitize` for Issue:** Strips `id` (replaces with whatever the server response said), `created_at`, `version`, `updated_at` from the inbound `Issue`. Returns `Untainted<Issue>` whose fields are the merged "client body + server-controlled metadata" view.
- **trybuild test:** `tests/compile-fail/tainted_into_untainted.rs` shows code that tries to call a function expecting `Untainted<Issue>` with a `Tainted<Issue>`; CI asserts compilation fails with the expected error.

### Filename + path validator
- **Lives at:** `crates/reposix-core/src/path.rs`, exported as `reposix_core::path::{validate_issue_filename, validate_path_component}`.
- **`validate_issue_filename(name: &str) -> Result<IssueId, Error>`:** name must match the regex `^([0-9]+)\.md$`. The numeric prefix parses to `IssueId(u64)`. Otherwise returns `Error::InvalidPath`.
- **`validate_path_component(name: &str) -> Result<&str, Error>`:** rejects empty, `.`, `..`, anything containing `/`, `\0`. Otherwise returns the input unchanged.
- **Why not just use `Path::file_name`?** Because `Path` doesn't reject `\0` and lets `..` through if the caller forgot to call `components()`. Wrapping in our own validator means the check is centralized and tested.

### Audit-log schema fixture
- **Lives at:** `crates/reposix-core/fixtures/audit.sql`. Loaded via `include_str!` from `crates/reposix-core/src/audit.rs`, exported as `reposix_core::audit::SCHEMA_SQL`.
- **Schema columns:** `id INTEGER PRIMARY KEY AUTOINCREMENT, ts TEXT NOT NULL, agent_id TEXT, method TEXT NOT NULL, path TEXT NOT NULL, status INTEGER, request_body TEXT, response_summary TEXT`. WAL-friendly: no `UNIQUE` constraints on body fields.
- **Triggers:** `CREATE TRIGGER audit_no_update BEFORE UPDATE ON audit_events BEGIN SELECT RAISE(ABORT, 'audit_events is append-only'); END;` and analogous for `BEFORE DELETE`. CI test asserts both triggers exist via `pragma trigger_list`.
- **`examples/show_audit_schema.rs`:** prints `audit::SCHEMA_SQL` to stdout. Used by ROADMAP success-criterion #3.

### Workspace clippy config
- **Lives at:** `clippy.toml` at the workspace root.
- **Contents:** `disallowed-methods = [{ path = "reqwest::Client::new", reason = "use reposix_core::http::client()" }, { path = "reqwest::Client::builder", reason = "use reposix_core::http::client()" }, { path = "reqwest::ClientBuilder::new", reason = "use reposix_core::http::client()" }]`.

### Claude's discretion
- Internal struct field names beyond the published surface above.
- Helper functions inside `reposix_core::http` for parsing the env var glob list.
- Whether to use `globset` crate or a hand-rolled matcher for `http://127.0.0.1:*`. (Hand-rolled is fine for the v0.1 grammar.)
- Test naming beyond the five names ROADMAP success-criterion #1 mandates.
- Whether `Tainted<T>`/`Untainted<T>` derive `serde::Serialize`/`Deserialize` — pragmatic: don't, callers must `.into_inner()` first.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents (planner, executor) MUST read these before planning or implementing.**

### Project-level
- `.planning/PROJECT.md` — scope, security guardrails (PROJECT.md `### Active` Security guardrails block).
- `.planning/ROADMAP.md` — Phase 1 success criteria and plan stubs (the orchestrator has copied them into PLAN.md task scaffolding; planner just fills tasks).
- `CLAUDE.md` — workspace-wide conventions (forbid_unsafe, pedantic clippy, docs-on-Result-fns).

### Research
- `.planning/research/threat-model-and-critique.md` — section "PART D" lists exact tests and constraints to add (already mostly folded into PROJECT.md). Phase-1 implementer should cross-check.
- `.planning/research/simulator-design.md` §4 (audit log shape — Phase 1 must publish a schema that matches what Phase 2 wants to consume).
- `.planning/research/fuse-rust-patterns.md` §6 (filename rules — what the FUSE boundary will need).

### External
- [reqwest::redirect::Policy](https://docs.rs/reqwest/latest/reqwest/redirect/) — for "redirects disabled" implementation.
- [trybuild](https://docs.rs/trybuild/) — for the compile-fail test on `Tainted<T>` → `Untainted<T>`.
- [SQLite RAISE](https://www.sqlite.org/lang_corefunc.html#raise) — trigger semantics.
- [clippy disallowed_methods](https://rust-lang.github.io/rust-clippy/master/index.html#disallowed_methods) — lint config format.

</canonical_refs>

<specifics>
## Specific Ideas

- The `client()` factory should accept a small `ClientOpts` struct so callers can opt into longer timeouts for known-slow paths without bypassing the allowlist. Default `ClientOpts::default()` is what 95% of callers use.
- `audit.sql` should be valid against SQLite 3.31+ (Ubuntu 20.04 baseline; matches CI runner).
- Use `chrono::DateTime<Utc>` everywhere for timestamps (already established convention in `reposix-core`).
- The `trybuild` test should live in `crates/reposix-core/tests/compile-fail/` and be invoked from a `#[test] fn compile_fail_taint_discipline()` in `tests/compile_fail.rs`. trybuild itself goes under `[dev-dependencies]`.

</specifics>

<deferred>
## Deferred Ideas

- Tainted-content propagation through Span/Tracing context (Phase 2/3 may want it; v0.1 is fine with the type wrapper alone).
- A higher-level `IssueDocument` type that wraps `Tainted<Issue>` and remembers the source URL — would be useful for the remote helper but not for MVD.
- A `globset`-backed allowlist matcher (port glob is the only thing v0.1 needs; a string matcher is fine).
- `audit_events.agent_id` trust model — Phase 2 will populate this from a header; Phase 1 just defines the column.

</deferred>

---

*Phase: 01-core-contracts-security-guardrails*
*Context gathered: 2026-04-13 via auto-mode (discuss step skipped per user instruction)*
