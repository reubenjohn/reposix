# Confluence backend reference

`reposix-confluence` is an adapter for Atlassian Confluence Cloud via its
[REST v2 API](https://developer.atlassian.com/cloud/confluence/rest/v2/intro/).
It implements the
[`IssueBackend`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-core/src/backend.rs)
trait that `reposix-cache` (the bare-repo cache backing `git-remote-reposix`)
and the `reposix list` CLI consume, so the same partial-clone working-tree
surface that works against the simulator and GitHub also works against a real
Confluence space. The read path ships in **v0.3.0**; the `git push` write
path lands later in the v0.4 milestone series.

## CLI surface

```bash
# List all readable Confluence spaces (key + name + URL)
reposix spaces --backend confluence

# List pages in a Confluence space (SPACE_KEY = e.g. REPOSIX, not a numeric id)
reposix list --backend confluence --project <SPACE_KEY>

# List pages and fail if the backend would truncate at the 500-page cap
reposix list --backend confluence --project <SPACE_KEY> --no-truncate

# Bootstrap the space as a partial-clone working tree (v0.9.0+)
reposix init confluence::<SPACE_KEY> <dir>
```

`--project` takes the **space key** (the short uppercase identifier visible in
every Confluence URL, e.g. `REPOSIX` in
`https://<tenant>.atlassian.net/wiki/spaces/REPOSIX/overview`). The adapter
internally resolves the key to Confluence's numeric `spaceId` via
`GET /wiki/api/v2/spaces?keys=<SPACE_KEY>`.

`--no-truncate` causes `reposix list` to fail with a non-zero exit code when
the space has more than 500 pages (the per-invocation cap). Without this flag,
`list` silently returns the first 500 pages and logs a `WARN`. Use
`--no-truncate` in scripts where a truncated result would be incorrect.

## Required env vars

All four must be set before `reposix list --backend confluence` or
`reposix init confluence::<SPACE_KEY>` will run. Any missing variable causes
the CLI to fail fast with a single error message listing every missing name.

| Var                         | What                                                              | Where to get it                                                                                           |
| --------------------------- | ----------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| `ATLASSIAN_API_KEY`         | Atlassian user API token (starts with `ATATT3xF...`, ~192 chars) | <https://id.atlassian.com/manage-profile/security/api-tokens> → **Create API token**                      |
| `ATLASSIAN_EMAIL`           | Atlassian account email the token was issued under                | The email shown at the top-right of the same API-tokens page. MUST match the token's issuing account.     |
| `REPOSIX_CONFLUENCE_TENANT` | Tenant subdomain (e.g. `mycompany` for `mycompany.atlassian.net`) | URL bar of any Confluence page in your tenant                                                             |
| `REPOSIX_ALLOWED_ORIGINS`   | SG-01 egress allowlist; MUST include `https://<tenant>.atlassian.net` | Set explicitly per invocation — the default is loopback-only                                              |

## Credential setup

The step-by-step happy path:

1. Navigate to
   <https://id.atlassian.com/manage-profile/security/api-tokens> in a
   browser, logged in as the Atlassian account that has permission on the
   target space.
2. **Note the email address shown at top-right of that page** — this is
   your `ATLASSIAN_EMAIL`. Do not guess it from `git config user.email`;
   the token is account-scoped and a mismatch surfaces as a 401 with
   `x-failure-category: FAILURE_CLIENT_AUTH_MISMATCH`.
3. Click **Create API token**, give it a descriptive label (e.g.
   `reposix-dev-local`), and copy the resulting string.
4. Copy `.env.example` to `.env` at the repo root and fill in all three
   Atlassian vars plus the allowlist:

   ```bash
   cp .env.example .env
   $EDITOR .env    # set ATLASSIAN_API_KEY, ATLASSIAN_EMAIL, REPOSIX_CONFLUENCE_TENANT
   ```

5. Source the file before running reposix commands:

   ```bash
   set -a; source .env; set +a
   export REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"
   reposix list --backend confluence --project YOUR_SPACE_KEY --format table
   ```

## Auth

Basic auth only: `Authorization: Basic base64(email:api_token)`. No Bearer
path — user-issued tokens cannot be used as OAuth 2.0 Bearer tokens. See
[ADR-002 §Auth decision](../decisions/002-confluence-page-mapping.md)
for the full rationale.

## Pagination

Cursor-in-body: the response body carries `_links.next` as a relative path
(e.g. `/wiki/api/v2/spaces/360450/pages?cursor=...`). The adapter prepends
the tenant base URL to turn it into a fully-qualified URL — it does NOT
trust `_links.base` from the response (that would be an SSRF vector).
Capped at 500 pages per `list_records` call. See
[ADR-002 §Pagination decision](../decisions/002-confluence-page-mapping.md).

## Rate limiting

Atlassian responds to rate-limit exhaustion with `429 Too Many Requests` plus
a `Retry-After` header (integer seconds). The adapter also watches
`x-ratelimit-remaining`: when it hits zero, the shared rate-limit gate is
armed with `Instant::now() + Retry-After` and every subsequent call parks
until the gate elapses. See the
[Atlassian rate-limiting docs](https://developer.atlassian.com/cloud/confluence/rate-limiting/).

This differs from the GitHub backend, which uses `x-ratelimit-reset` (unix
epoch); the two patterns are documented side-by-side in
[ADR-002 §Rate-limit decision](../decisions/002-confluence-page-mapping.md).

## Working tree layout

Once `reposix init confluence::<SPACE_KEY> <dir>` finishes, `<dir>` is a
plain git checkout backed by the partial-clone helper. The shape is flat —
each Confluence page maps to exactly one Markdown file under `pages/`,
keyed by its stable numeric `id`:

```
<dir>/
├── .git/                       # real git directory; partial clone (extensions.partialClone=origin)
└── pages/
    ├── 00000131192.md          # page body (YAML frontmatter + storage-format XHTML)
    ├── 00000065916.md
    └── 00000360556.md
```

YAML frontmatter carries server-controlled fields (`id`, `version`,
`spaceId`, `parentId`, `createdAt`, `updatedAt`); the body is the raw
storage-format XHTML returned by `body.storage.value`. Filenames use the
zero-padded numeric page id so renames and reparenting on the Confluence
side never rewrite the working-tree path — they show up as frontmatter
diffs only.

The blob behind each `pages/<id>.md` is fetched lazily: the tree (the
filename list) is materialized eagerly by `git fetch --filter=blob:none
origin`, and individual page bodies download on first read via
`git-remote-reposix`'s `stateless-connect` capability. Agents that
operate on a subset use `git sparse-checkout set 'pages/000001312*.md'`
before the first `git checkout`, so the helper sees a single batched
fetch turn for exactly the blobs they need.

Comment overlay (the v0.4-era `pages/<id>.comments/` directory),
read-only-via-`EROFS` semantics, and the synthesized `tree/` symlink
hierarchy were a FUSE-mount affordance that did not survive the v0.9.0
git-native pivot — Confluence parent/child structure is now reachable
via `parentId` in frontmatter, and inline-comment access is deferred
to a future milestone. See the
[v0.9.0 architecture-pivot summary](https://github.com/reubenjohn/reposix/tree/main/.planning/research/v0.9-fuse-to-git-native)
and [Git layer](../how-it-works/git-layer.md) for the partial-clone
shape in full, and [ADR-003](../decisions/003-nested-mount-layout.md)
(superseded) for the historical FUSE-era layout.

## What's NOT supported

The adapter deliberately loses the following Confluence concepts on the
round-trip; see
[ADR-002 §Lost metadata (deliberate)](../decisions/002-confluence-page-mapping.md)
for the full list:

- **Space metadata.** `spaceId` / `spaceKey` are discarded after the
  key-to-id resolver runs at the start of `list_records`.
- **Browser links.** `_links.webui` / `_links.editui` / `_links.tinyui` are
  discarded.
- **Atlassian rich-doc format.** `body.atlas_doc_format` is ignored — the
  body is raw XHTML from `body.storage.value`, not rendered Markdown.
- **Labels.** Confluence labels live at a separate endpoint; v0.3 returns
  `labels: []` unconditionally.
- **Write path on v0.3.0.** In the initial release, `create_record` /
  `update_record` / `delete_or_close` all returned `not supported`.
  Subsequent v0.4-series phases added the write path (with
  server-field sanitization mirroring SG-03), so a `git push` from a
  partial-clone working tree now translates each changed `pages/<id>.md`
  into the matching REST PATCH/POST/DELETE call against Confluence. The
  push round-trip — including push-time conflict detection that rejects
  with the standard `fetch first` git error when the remote drifted —
  is documented in [Git layer → push round-trip](../how-it-works/git-layer.md#the-push-round-trip-happy-path-and-conflict).

## Known failure modes

| Symptom                                                                  | Cause                                                                                              | Fix                                                                                                                                |
| ------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `401` + `x-failure-category: FAILURE_CLIENT_AUTH_MISMATCH`               | `ATLASSIAN_EMAIL` does not match the account the token was issued under.                           | Re-read **Credential setup** step 2. The email at top-right of id.atlassian.com is authoritative.                                  |
| `403 Forbidden`                                                          | Auth succeeded but the account lacks permission on the requested space.                            | Check the space's permissions in Confluence admin, or use a space the account can read.                                            |
| `Error::Other("invalid confluence tenant subdomain: ...")`               | `REPOSIX_CONFLUENCE_TENANT` contains characters outside `[a-z0-9-]`, is empty, or exceeds 63 chars. | Fix the env var to be just the subdomain (e.g. `mycompany`, not `mycompany.atlassian.net` or `https://...`).                       |
| `Error::Other("origin not allowlisted: ...")`                            | `REPOSIX_ALLOWED_ORIGINS` does not include `https://<tenant>.atlassian.net`.                       | Export `REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"` before invoking `reposix`. |
| `429` repeatedly                                                         | Rate-limit gate keeps re-arming; the tenant has a very tight quota or you're on a hot loop.        | Let the adapter back off (it will sleep until `Retry-After` elapses). Consider increasing the delay between bulk invocations.      |

## Regression coverage

The Confluence backend's end-to-end regression is the
`dark_factory_real_confluence` ignored test in
[`crates/reposix-cli/tests/agent_flow_real.rs`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-cli/tests/agent_flow_real.rs).
It drives a clone+grep+edit+push cycle against the TokenWorld space (see
[testing-targets.md](./testing-targets.md)) and skips cleanly if any
Atlassian env var is unset.

(The historical `scripts/demos/parity-confluence.sh` parity demo, the
pre-pivot FUSE live-mount demo, and the demo-suite index page were
all removed alongside the v0.9.0 architecture pivot — most recently
in v0.11.1 §7-F2.)

## See also

- [ADR-002 — Confluence page to issue mapping](../decisions/002-confluence-page-mapping.md)
- [ADR-001 — GitHub state mapping](../decisions/001-github-state-mapping.md)
  (structural sibling)
- [Write your own connector](../guides/write-your-own-connector.md) — how to write a
  third adapter following the same pattern as `reposix-github` and
  `reposix-confluence`.
