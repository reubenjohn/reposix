# Phase 11: Confluence Cloud read-only adapter — Context

**Gathered:** 2026-04-13 ~21:00 PDT (overnight session 3)
**Status:** Ready for planning
**Source:** Orchestrator-written (discuss-phase skipped per `.planning/config.json`)

<domain>
## Phase Boundary

Ship `crates/reposix-confluence` — a read-only `IssueBackend` adapter for Atlassian Confluence Cloud REST v2. After this phase, `reposix list --backend confluence --project <SPACE_KEY>` and `reposix mount --backend confluence --project <SPACE_KEY>` behave end-to-end against a real Atlassian tenant (once valid credentials are provided — see §credentials below). Parity demo + Tier 5 live-mount demo + ADR-002 + `v0.3.0` release ship alongside.

**In scope:** read path (`list_issues`, `get_issue`), wiremock unit tests, wiremock+live contract test, CLI dispatch, two demo scripts, ADR-002, docs, `.env.example` rename, CHANGELOG, v0.3.0 tag.

**Out of scope (deferred to v0.4 or later):**
- Write path (`create_issue` / `update_issue` / `delete_or_close` return `NotSupported`).
- `PageBackend` trait refactor (Option B from HANDOFF §3) — staying with Option A flattening tonight.
- Rendering Confluence's `atlas_doc_format` to Markdown — v3 ships raw `storage` HTML in body and documents the cosmetic limitation.
- Comments, attachments, restrictions, labels — not mapped onto `Issue` yet.
- Jira adapter (separate phase).
</domain>

<decisions>
## Implementation Decisions

### Architecture
- **Crate name:** `reposix-confluence`, sibling of `reposix-github` under `crates/`.
- **Backend struct:** `ConfluenceReadOnlyBackend` implementing `reposix_core::backend::IssueBackend`.
- **Constructor:** `new(creds: ConfluenceCreds, tenant: &str)` where `ConfluenceCreds { email, api_token }`. Also `new_with_base_url(creds, base_url)` for wiremock tests (mirrors `reposix-github`).
- **HTTP client:** `reposix_core::http::client(ClientOpts::default())` — sealed HttpClient enforces SG-01 allowlist. No direct reqwest.
- **Thread-safety:** `Arc<HttpClient>`, cloneable, all methods `&self` (matches github pattern).

### Protocol mapping (Option A flattening)
- **Page → Issue:**
  - `Issue.id = IssueId(parse_u64(confluence_page.id))` — Confluence page IDs are numeric strings.
  - `Issue.title = page.title`.
  - `Issue.status = match page.status { "current" => Open, "draft" => Open, "archived" | "trashed" | "deleted" => Done, _ => Open }` (pessimistic fallback).
  - `Issue.body = page.body.storage.value` (raw HTML) OR empty if body not requested.
  - `Issue.created_at = page.createdAt`, `updated_at = page.version.createdAt` (Confluence returns the latter on the page doc).
  - `Issue.version = page.version.number` (monotonic per-page, matches our `u64`).
  - `Issue.labels = []` (deferred — Confluence labels are a separate endpoint).
  - `Issue.assignee = page.ownerId` (Atlassian accountId) if present, else `None`.
  - Frontmatter-extension fields (carried via `body`-preamble or an ADR-documented future `Issue.extensions` bag; for tonight, we DO NOT extend `Issue` — we render a Confluence-specific frontmatter block into `Issue.body` at the CLI layer, keeping the trait surface clean).
- **Actually revised decision (for clarity to the planner):** The adapter returns canonical `Issue` shape. `parent_id`, `space_id`, `space_key`, `webui_link` are **not** fields on `Issue`. Instead, the FUSE/CLI layer already renders frontmatter from `Issue`; we document in ADR-002 that v0.3 loses Confluence-specific metadata on the round-trip. This keeps this phase's blast radius to one crate + the CLI's `--backend` enum.

### Auth
- **Scheme:** Basic auth — `Authorization: Basic base64(email:token)`. The env var at runtime is `ATLASSIAN_API_KEY` (renamed from `TEAMWORK_GRAPH_API`); email comes from `ATLASSIAN_EMAIL` env var (NEW — must be added to `.env.example`); tenant subdomain from `REPOSIX_CONFLUENCE_TENANT` (NEW).
- **Origin:** Computed as `https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net`. Must be present in `REPOSIX_ALLOWED_ORIGINS` or SG-01 refuses the call.
- **No Bearer fallback** — `id.atlassian.com` tokens can't be used as OAuth. Document this in ADR-002.

### Pagination
- Confluence v2 uses **cursor-based pagination** with `_links.next` pointing to the next page URL (fully-qualified or a relative path under `/wiki/api/v2/`). This is DIFFERENT from GitHub's `Link: rel="next"` header.
- Follow cursor links until exhausted or `MAX_PAGES_PER_LIST` hit (start with 500 pages = 5 requests at page size 100; matches GitHub cap).

### Rate limiting
- Atlassian returns `429` with `Retry-After` (seconds). Shared `rate_limit_gate: Arc<Mutex<Option<Instant>>>` (same shape as GitHub's) parks outbound calls until reset. Log WARN on any 429.
- Tokens are typically 1000-ish requests/minute soft cap; we'll never hit it in normal usage.

### Security
- SG-01 enforcement via `HttpClient`.
- SG-05: every decoded `Issue` is wrapped in `Tainted<Issue>` at the backend boundary before returning.
- SG-03: read-only path doesn't sanitize (no writes); document in ADR that v0.4 write path will sanitize.
- `#![forbid(unsafe_code)]` at crate root. `#![warn(clippy::pedantic, missing_docs)]`.

### CLI dispatch
- `list.rs` and `mount.rs` each have a `--backend` enum. Add `Confluence` variant.
- Required env vars for confluence: `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT`. CLI fails fast with a clear error listing all three if any missing.
- `--project` is the Confluence **space key** (e.g. `REPOSIX`), not the space numeric ID. The adapter resolves key→id internally via `GET /wiki/api/v2/spaces?keys=KEY`.

### Tests
- **Unit tests (wiremock) in `lib.rs`:** min 5 — list-with-cursor-pagination, get-by-id, 404-maps-to-not-found, status-mapping-current→Open, status-mapping-trashed→Done, auth-header-is-basic (⚠ custom `Match` impl — HANDOFF §8 gotcha).
- **Contract test at `tests/contract.rs`:** parameterized over `SimBackend` + wiremock-backed `ConfluenceReadOnlyBackend` (always runs) + live `ConfluenceReadOnlyBackend` (`#[ignore]`-gated).
- **Integration CI job:** `integration-contract-confluence` gated on `secrets.ATLASSIAN_API_KEY != ''` — DO NOT set the secret ourselves (HANDOFF §4); document the gh-cli command for user.

### Demos
- **Tier 3B:** `scripts/demos/parity-confluence.sh` — `reposix list --backend sim` vs `reposix list --backend confluence` for the same-shape diff, like existing `parity.sh`.
- **Tier 5:** `scripts/demos/06-mount-real-confluence.sh` — mount → ls → cat → unmount, structured like `05-mount-real-github.sh`.
- **Skip behavior:** both scripts must `exit 0` with a SKIP message when `ATLASSIAN_API_KEY` is unset. This is non-negotiable for `scripts/demos/smoke.sh` compatibility (if we add 06 to smoke) and for friendly-CI behavior (HANDOFF §4).

### Credentials gap (from tonight's probe)
- We cannot empirically verify live-mount tonight. See `00-CREDENTIAL-STATUS.md`.
- `MORNING-BRIEF-v0.3.md` (Phase 11-F) must tell the user exactly what to set and run when they fix the email.
- This is a documented deviation from Operating Principle #1 ("close the feedback loop"); we own it loudly.

### Claude's Discretion
- Exact module layout within the crate (prefer flat `lib.rs` like reposix-github unless it grows big).
- Exact cursor-parsing helper (probably a `fn parse_next_cursor(body: &Value) -> Option<String>`).
- Whether to unit-test the space-key→space-id resolver separately.
- Where to document the "lost metadata" tradeoff (ADR-002 primary, module doc comment secondary).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Pattern templates (READ-FIRST — this phase copies these structurally)
- `crates/reposix-github/src/lib.rs` — target struct + method shape + rate-limit gate pattern.
- `crates/reposix-github/tests/contract.rs` — contract test structure.
- `crates/reposix-cli/src/list.rs` — `--backend` enum + dispatch.
- `crates/reposix-cli/src/mount.rs` — `--backend` enum + IssueBackend construction.
- `scripts/demos/parity.sh` — Tier 3 demo template.
- `scripts/demos/05-mount-real-github.sh` — Tier 5 demo template.
- `docs/decisions/001-github-state-mapping.md` — ADR template; ADR-002 follows same structure.

### Trait seam
- `crates/reposix-core/src/backend.rs` — the `IssueBackend` trait, `BackendFeature`, `DeleteReason`.

### Infrastructure
- `crates/reposix-core/src/http.rs` — sealed `HttpClient`; every new backend goes through it.
- `crates/reposix-core/src/taint.rs` — `Tainted<T>` / `Untainted<T>` (SG-05).
- `crates/reposix-core/src/sanitize.rs` — (for write path, not needed v3-read-only).

### External docs
- <https://developer.atlassian.com/cloud/confluence/rest/v2/intro/> — Confluence REST v2 overview.
- <https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-space/> — spaces endpoints.
- <https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/> — pages endpoints.
- <https://developer.atlassian.com/cloud/confluence/rate-limiting/> — rate-limiting behavior.
- <https://developer.atlassian.com/cloud/confluence/basic-auth-for-rest-apis/> — Basic auth with email + API token.

### Today's artifacts
- `.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md` — tonight's probe findings.
- `HANDOFF.md` — original mission brief from session 2 → 3.
</canonical_refs>

<specifics>
## Specific Requirements

- `cargo test --workspace --locked` must pass ≥180 tests, 0 failures.
- `cargo clippy --workspace --all-targets -- -D warnings` must stay clean.
- `cargo fmt --all --check` must stay clean.
- `bash scripts/demos/smoke.sh` must stay 4/4 green (Tier 1 demos are load-bearing).
- `#![forbid(unsafe_code)]` at crate root.
- `#![warn(clippy::pedantic, missing_docs)]` at crate root.
- Every public item documented; `# Errors` section on every `Result`-returning fn.
- Demos exit 0 on missing env (SKIP path) — verified in smoke if added.
- No secrets committed. `.env` gitignored remains untouched.
- Each commit atomic with `feat(11-X-N):`, `test(...):`, `docs(...):`, `fix(...):` prefix.
</specifics>

<deferred>
## Deferred Ideas

- PageBackend trait (Option B). v0.4+ if someone actually needs directory-tree UX for Confluence pages.
- `atlas_doc_format` → Markdown renderer. Stretch for v0.4.
- Write path on Confluence. v0.4.
- Confluence labels → `Issue.labels`. Requires separate endpoint; small v0.4 item.
- Jira adapter. Separate future phase (probably Phase 12).
- `git-remote-reposix` rewire through `IssueBackend` (still hardcodes sim). v0.4 per HANDOFF §1.
- Writing page comments / attachments round-trip. Not a v0.3 goal.
</deferred>

---

*Phase: 11-confluence-adapter*
*Context gathered: 2026-04-13 via orchestrator (discuss-phase skipped)*
