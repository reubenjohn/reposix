# ADR-002: Confluence page to issue mapping

> **Superseded in part (2026-04-14).** The flat-layout decision in this
> ADR's §"Options" table (row A, chosen in v0.3) was replaced by
> [ADR-003](003-nested-mount-layout.md) (`pages/` bucket + `tree/` symlink
> overlay). The field-mapping, status-mapping, auth, pagination, and
> rate-limit decision sections below remain authoritative.

- **Status:** Accepted (layout section superseded by ADR-003)
- **Date:** 2026-04-13
- **Deciders:** reposix core team (overnight session 3)
- **Supersedes:** none
- **Superseded by:** [ADR-003](003-nested-mount-layout.md) (layout decision only; 2026-04-14)
- **Scope:** `crates/reposix-confluence` (v0.3, `ConfluenceReadOnlyBackend`) and
  any future read-only adapter that consumes Atlassian Confluence Cloud REST v2
  pages under the v0.3 flat-issue data model.

## Context

reposix models its domain as a flat list of `Issue` records
(`crates/reposix-core/src/issue.rs`): a numeric `IssueId`, a title, a 5-valued
`IssueStatus`, a Markdown-ish body, and a monotonic `version` for optimistic
concurrency. The FUSE layer renders each record as `<padded-id>.md` at the mount
root, the CLI lists them as JSON rows, and `git-remote-reposix` round-trips them
as fast-import blobs. All three layers assume a single namespace with no
hierarchy — mirroring Jira/GitHub issues closely but not Confluence pages.

Confluence is a different shape. Pages form a tree: every page lives in a
space, and inside the space it can have an arbitrary parent chain. The REST v2
surface exposes `parentId`, `parentType`, `spaceId`, `_links.webui`, and a
nested `version.createdAt` that doesn't match the top-level `updatedAt` GitHub
and the sim both carry. The raw body comes back as XHTML (`body.storage.value`)
or as Atlassian's proprietary `atlas_doc_format`, not Markdown. Auth is Basic
(`email:api_token`, base64-encoded) — there is no Bearer variant for
user-issued tokens.

Three options were considered (HANDOFF.md §3):

| Option | Shape | Pros | Cons |
| ------ | ----- | ---- | ---- |
| **A. Flatten** | `Issue` stays 1-D; Confluence metadata discarded | FUSE/CLI/remote code unchanged; one-crate blast radius; ship in hours, not days | Round-trip loses `parentId`, `spaceKey`, `_links.webui`; no `cd` into page tree on the mount |
| B. `PageBackend` trait | New trait alongside `IssueBackend`, modeled on a recursive node | Correct long-term; FUSE could render a real directory tree | Requires every layer (FUSE, CLI, remote) to learn a second trait; weeks of work |
| C. Optional `parent_id` on `Issue` | Add `parent_id: Option<IssueId>` to the core type | Smaller refactor than B | Introduces schematic ambiguity across backends — "what does `parent_id` mean for GitHub?" — and bleeds Confluence-specific semantics into the shared core |

**Option A ships in v0.3.** It reuses the entire FUSE + CLI + remote-helper
substrate unchanged; the only new code is one crate (`reposix-confluence`) and
four lines of dispatch in `list.rs` / `mount.rs` / `reposix-fuse/src/main.rs`.
Option B is the correct long-term answer and is flagged for v0.5+ once we have
two real hierarchy-heavy backends demanding it. Option C is rejected on the
grounds that a field defined for one backend and ignored by others is worse
than an explicit ADR about lost metadata.

## Decision

Map each Confluence page onto an `Issue` with the following field assignments.
Canonical source is `crates/reposix-confluence/src/lib.rs::translate_page`; this
table documents the intent.

| `Issue` field | Confluence source             | Type coercion / notes                                                         |
| ------------- | ----------------------------- | ----------------------------------------------------------------------------- |
| `id`          | `page.id`                     | `parse::<u64>`; non-numeric → `Err(Error::Other("invalid page id: ..."))`     |
| `title`       | `page.title`                  | UTF-8 passthrough                                                             |
| `status`      | `page.status`                 | See branch table under "Status mapping" below                                 |
| `body`        | `page.body.storage.value`     | Raw XHTML string; `""` when `?body-format=storage` is not in the request    |
| `created_at`  | `page.createdAt`              | ISO 8601 → `chrono::DateTime<Utc>`                                            |
| `updated_at`  | `page.version.createdAt`      | Nested under `version`, NOT a top-level field (Confluence quirk)              |
| `version`     | `page.version.number`         | `u64`; monotonic per-page, aligns with our optimistic-concurrency discipline  |
| `assignee`    | `page.ownerId`                | Atlassian account ID (opaque string); `None` when absent                     |
| `labels`      | `[]`                          | Deferred to v0.4 (Confluence labels are a separate endpoint)                  |

### Status mapping

| `page.status`                              | `IssueStatus` | Rationale                                                                 |
| ------------------------------------------ | ------------- | ------------------------------------------------------------------------- |
| `current`                                  | `Open`        | Live page                                                                 |
| `draft`                                    | `Open`        | Draft is still "active work" from the agent's point of view              |
| `archived`, `trashed`, `deleted`          | `Done`        | Terminal states — don't lie and say the page is still `Open`             |
| anything else (unknown-value forward-compat) | `Open`      | Pessimistic fallback: treat unknown statuses as live rather than closed   |

### Lost metadata (deliberate)

The v0.3 flattening DOES NOT carry the following Confluence fields through onto
the `Issue`. Round-tripping a page through `reposix mount --backend confluence`
therefore loses all of them. This is the Option-A tradeoff, called out
explicitly so that a future agent reading this ADR doesn't mistake the absence
for a bug:

- `parentId`, `parentType` — page-hierarchy position
- `spaceId`, `spaceKey` — which space the page lives in
- `_links.webui`, `_links.editui`, `_links.tinyui` — human-browser links
- `position` — sibling ordering within the parent
- `body.atlas_doc_format` — Atlassian's proprietary rich-doc format

The v0.4 extension path is either `Issue.extensions: BTreeMap<String,
serde_json::Value>` (bag of backend-specific fields) or a full `PageBackend`
trait (Option B). Whichever ships, this ADR and the crate-level module
documentation will be updated to link to it.

### Auth decision

Basic auth (`Authorization: Basic base64(email:api_token)`) is the only
supported scheme. Rationale:

- User-issued API tokens from
  <https://id.atlassian.com/manage-profile/security/api-tokens> are
  account-scoped and only work under the Atlassian Account email they were
  issued under. They cannot be used as OAuth 2.0 Bearer tokens.
- OAuth 2.0 3LO (the only way to get a legitimate Bearer on a Confluence
  endpoint) requires registering an Atlassian Forge app, a redirect URI, and
  an interactive consent screen — which is out of scope for a CLI adapter.
- The common failure mode is `401` with header
  `x-failure-category: FAILURE_CLIENT_AUTH_MISMATCH`, which means the
  `ATLASSIAN_EMAIL` env var does not match the account the token was issued
  under. See
  [`.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md)
  for the debugging history.

### Pagination decision

Confluence v2 uses **cursor-in-body** pagination: the response body carries
`_links.next`, a relative path like
`/wiki/api/v2/spaces/360450/pages?cursor=...`. The adapter prepends its
tenant base URL (`https://<tenant>.atlassian.net`) to turn the relative path
into a fully-qualified URL. **Critically, the adapter does NOT trust
`_links.base` from the body** — using the server-supplied base would be an
SSRF vector (a compromised or malicious Confluence tenant could redirect the
cursor to an arbitrary origin).

A `MAX_PAGES_PER_LIST = 500` cap matches GitHub's cap in
`reposix-github`. At a page size of 100 that's 5 round-trips per `list_issues`
call — enough for every reasonable space, bounded for pathological ones.

### Rate-limit decision

Atlassian signals rate-limit state differently from GitHub:

- **GitHub:** `x-ratelimit-remaining` + `x-ratelimit-reset` (unix epoch).
- **Confluence:** `x-ratelimit-remaining` + `Retry-After` (seconds from now).

The adapter watches both. If a response is `429` or carries
`x-ratelimit-remaining: 0`, the shared `rate_limit_gate:
Arc<parking_lot::Mutex<Option<Instant>>>` is armed with `Instant::now() +
Retry-After`. Every subsequent call parks at `await_rate_limit_gate` until
the gate elapses. The gate is shared across `Clone`s so a single exhausted
token cannot be bypassed by cloning the backend.

### Read-path only (v0.3)

`create_issue`, `update_issue`, and `delete_or_close` all return
`Err(Error::Other("not supported: ..."))`. `IssueBackend::supports(feature)`
returns `false` for every `BackendFeature` variant (no `Delete`, no
`Transitions`, no `StrongVersioning`, no `BulkEdit`, no `Workflows`).

The write path is deferred to v0.4, where it must also acquire a sanitize
step: Confluence's server-authoritative fields (`id`, `version.number`,
`createdAt`, `_links`) must be stripped from any outbound PATCH or POST body,
the same way `Tainted::sanitize` strips sim/GitHub server fields today.

## Consequences

- **No `cd`-into-hierarchy UX on the mount.** Every page is a sibling
  `<padded-id>.md` at the mount root. Agents who want to follow parent chains
  must use the Confluence web UI; reposix v0.3 cannot represent it.

- **Body is raw XHTML, not Markdown.** `cat 131192.md` returns an
  HTML-with-`<ac:structured-macro>` soup. This is ugly but honest — rendering
  `atlas_doc_format → Markdown` is a v0.4 enhancement, not a correctness
  issue.

- **Space resolver adds a round-trip.** `list_issues` first calls
  `GET /wiki/api/v2/spaces?keys=<SPACE_KEY>` to resolve the caller's
  human-readable space key into the numeric `spaceId` Confluence's page
  endpoints demand. No cache in v0.3 — every `list_issues` pays the resolver
  cost. Acceptable for a read-only demo backend; a candidate for optimization
  if someone starts hammering it.

- **No OAuth path.** Users whose workflow demands OAuth (because their
  Atlassian admin disabled token auth) cannot use v0.3. Documented as a known
  limitation in `docs/reference/confluence.md`.

- **Page IDs must parse as `u64`.** Confluence page IDs are numeric strings
  (`"131192"`), so this holds in practice. A future Confluence schema change
  that introduces non-numeric IDs would surface as a clean `Err` at the
  `translate_page` boundary, not a silent corruption downstream.

- **Forward compatibility via Phase 12.** The long-term answer to "how do
  third parties add their own backends?" is the subprocess / JSON-RPC
  connector ABI tracked in
  [ROADMAP.md §Phase 12](https://github.com/reubenjohn/reposix/blob/main/.planning/ROADMAP.md).
  Until that ships, the crates.io + fork path documented in
  [`docs/connectors/guide.md`](../connectors/guide.md) is the supported
  route — with `reposix-github` and `reposix-confluence` as twin worked
  examples. The subprocess model supersedes this ADR's flatten-via-core-trait
  approach the moment it lands; ADR-003 will capture the transition.

## References

- [ADR-001 GitHub state mapping](001-github-state-mapping.md) — structural
  sibling; the read-path-only approach and pessimistic-fallback philosophy
  are inherited verbatim.
- [`crates/reposix-confluence/src/lib.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-confluence/src/lib.rs)
  — the canonical source of truth for every mapping rule above. If this
  ADR and the code disagree, the code wins and this ADR is stale.
- [`crates/reposix-core/src/backend.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-core/src/backend.rs)
  — the `IssueBackend` trait this adapter implements.
- [`.planning/phases/11-confluence-adapter/11-CONTEXT.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/phases/11-confluence-adapter/11-CONTEXT.md)
  — the phase scope document.
- [`.planning/phases/11-confluence-adapter/11-RESEARCH.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/phases/11-confluence-adapter/11-RESEARCH.md)
  — pattern-delta research; every row in the mapping table above was derived
  from it.
- [`.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/phases/11-confluence-adapter/00-CREDENTIAL-STATUS.md)
  — the debugging record for the auth-mismatch failure mode.
- [Confluence REST v2 overview](https://developer.atlassian.com/cloud/confluence/rest/v2/intro/)
- [Confluence: pages endpoint](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-page/)
- [Confluence: spaces endpoint](https://developer.atlassian.com/cloud/confluence/rest/v2/api-group-space/)
- [Confluence: Basic auth for REST APIs](https://developer.atlassian.com/cloud/confluence/basic-auth-for-rest-apis/)
- [Confluence: rate limiting](https://developer.atlassian.com/cloud/confluence/rate-limiting/)
