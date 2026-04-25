# Confluence backend reference

`reposix-confluence` is a **read-only** adapter for Atlassian Confluence Cloud
via its [REST v2 API](https://developer.atlassian.com/cloud/confluence/rest/v2/intro/).
It implements the
[`IssueBackend`](https://github.com/reubenjohn/reposix/blob/main/crates/reposix-core/src/backend.rs)
trait the FUSE daemon and `reposix list` CLI consume, so the same kernel path
and command-line surface that works against the simulator and GitHub also works
against a real Confluence space. Ships in **v0.3.0**.

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
Capped at 500 pages per `list_issues` call. See
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

## FUSE mount layout (v0.4+)

After Phase 13 (v0.4), pages live under a `pages/` bucket rather than at the
mount root:

```
mount/
├── pages/
│   ├── 00000131192.md           # page body — writable target
│   ├── 00000131192.comments/    # read-only comment overlay (Phase 23)
│   │   ├── 00000012345.md       # inline/footer comment as frontmatter+body
│   │   └── 00000012346.md
│   └── 00000065916.md
├── tree/                        # read-only symlink hierarchy (Confluence only)
│   └── reposix-demo-space-home/
│       ├── _self.md             -> ../../pages/00000360556.md
│       └── architecture-notes.md -> ../../pages/00000065916.md
└── .gitignore                   # synthesized; contains /tree/
```

`pages/<id>.comments/` directories are lazy-fetched — the backend round-trip
for a page's comments only occurs when that directory is first accessed.
Comment files are read-only; writes return `EROFS`.

See [ADR-003](../decisions/003-nested-mount-layout.md) for the full layout
specification, including slug algorithm and collision resolution.

## What's NOT supported

The adapter deliberately loses the following Confluence concepts on the
round-trip; see
[ADR-002 §Lost metadata (deliberate)](../decisions/002-confluence-page-mapping.md)
for the full list:

- **Space metadata.** `spaceId` / `spaceKey` are discarded after the
  key-to-id resolver runs at the start of `list_issues`.
- **Browser links.** `_links.webui` / `_links.editui` / `_links.tinyui` are
  discarded.
- **Atlassian rich-doc format.** `body.atlas_doc_format` is ignored — the
  body is raw XHTML from `body.storage.value`, not rendered Markdown.
- **Labels.** Confluence labels live at a separate endpoint; v0.3 returns
  `labels: []` unconditionally.
- **Write path.** `create_issue` / `update_issue` / `delete_or_close` all
  return `not supported`. v0.4 will add the write path (with
  server-field sanitization, mirroring SG-03).

## Known failure modes

| Symptom                                                                  | Cause                                                                                              | Fix                                                                                                                                |
| ------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `401` + `x-failure-category: FAILURE_CLIENT_AUTH_MISMATCH`               | `ATLASSIAN_EMAIL` does not match the account the token was issued under.                           | Re-read **Credential setup** step 2. The email at top-right of id.atlassian.com is authoritative.                                  |
| `403 Forbidden`                                                          | Auth succeeded but the account lacks permission on the requested space.                            | Check the space's permissions in Confluence admin, or use a space the account can read.                                            |
| `Error::Other("invalid confluence tenant subdomain: ...")`               | `REPOSIX_CONFLUENCE_TENANT` contains characters outside `[a-z0-9-]`, is empty, or exceeds 63 chars. | Fix the env var to be just the subdomain (e.g. `mycompany`, not `mycompany.atlassian.net` or `https://...`).                       |
| `Error::Other("origin not allowlisted: ...")`                            | `REPOSIX_ALLOWED_ORIGINS` does not include `https://<tenant>.atlassian.net`.                       | Export `REPOSIX_ALLOWED_ORIGINS="http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net"` before invoking `reposix`. |
| `429` repeatedly                                                         | Rate-limit gate keeps re-arming; the tenant has a very tight quota or you're on a hot loop.        | Let the adapter back off (it will sleep until `Retry-After` elapses). Consider increasing the delay between bulk invocations.      |

## Demos

- **Tier 3B parity demo** —
  [`scripts/demos/parity-confluence.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/parity-confluence.sh)
  runs `reposix list` against the simulator and `reposix list --backend
  confluence` against your tenant, then `jq`-diffs the normalized
  `{id, title, status}` shape. Skips cleanly if any Atlassian env var is
  unset.
- **Tier 5 live-mount demo** —
  [`scripts/demos/06-mount-real-confluence.sh`](https://github.com/reubenjohn/reposix/blob/main/scripts/demos/06-mount-real-confluence.sh)
  mounts your Confluence space as a POSIX directory via FUSE, `cat`s the
  first page's Markdown, and unmounts. Skips cleanly with `SKIP:` if any
  Atlassian env var is unset. The token and email are never echoed — only
  the tenant host, space key, and allowlist appear on stdout.

See the [demo suite index](../demos/index.md) for how these fit into the
broader Tier 1–5 story.

## See also

- [ADR-002 — Confluence page to issue mapping](../decisions/002-confluence-page-mapping.md)
- [ADR-001 — GitHub state mapping](../decisions/001-github-state-mapping.md)
  (structural sibling)
- [Write your own connector](../guides/write-your-own-connector.md) — how to write a
  third adapter following the same pattern as `reposix-github` and
  `reposix-confluence`.
