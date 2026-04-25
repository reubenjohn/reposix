# ADR-008 — Helper URL-scheme backend dispatch

| | |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-04-24 |
| **Phase** | v0.10.0 (closes v0.9.0 Phase 32 carry-forward debt) |
| **Supersedes / amends** | none |

## Context

Phase 32 shipped the `stateless-connect` read path against
`reposix-cache`, but the helper's `Cache::open(...)` site hardcoded
`SimBackend` and the cache-key prefix `"sim"`. Every backend — GitHub,
Confluence, JIRA — was wedged onto the same backend instance, which
meant `git fetch` against a real backend silently fell back to talking
to the local simulator (or failed with a connection-refused if no sim
was running). The defect is documented in
`.planning/v0.9.0-MILESTONE-AUDIT.md` §5 and was the single item that
tilted the milestone verdict from `passed` to `tech_debt`.

We need the helper to look at `argv[2]` (the remote URL) and
instantiate the matching `BackendConnector` so real-backend fetches
actually hit `api.github.com` / `<tenant>.atlassian.net` / etc.

## Decision

The helper performs **URL-scheme dispatch** at startup. A new module
`crates/reposix-remote/src/backend_dispatch.rs` exposes:

```rust
pub enum BackendKind { Sim, GitHub, Confluence, Jira }

pub fn parse_remote_url(url: &str) -> Result<ParsedRemote>;
pub fn instantiate(parsed: &ParsedRemote) -> Result<Arc<dyn BackendConnector>>;
pub fn sanitize_project_for_cache(project: &str) -> String;
```

Dispatch is keyed off the **origin** (host + scheme) plus an optional
**path-segment marker** for the two backends that share an Atlassian
Cloud origin:

| URL form | BackendKind | Notes |
|---|---|---|
| `reposix::http://127.0.0.1:<port>/projects/<slug>` | `Sim` | matches any loopback host (`127.0.0.1`, `localhost`, `[::1]`) |
| `reposix::https://api.github.com/projects/<owner>/<repo>` | `GitHub` | project carries `owner/repo` literally |
| `reposix::https://<tenant>.atlassian.net/confluence/projects/<space>` | `Confluence` | `/confluence/` marker required |
| `reposix::https://<tenant>.atlassian.net/jira/projects/<key>` | `Jira` | `/jira/` marker required |

`reposix init` emits the canonical form of each. The leading
`reposix::` prefix is optional (git strips it before invoking the
helper, but `assert_cmd` test harnesses pass it verbatim, and we
defensively tolerate an accidental double-strip).

### Cache-slug naming

`Cache::open(backend, backend_slug, project)` joins to a filesystem
path:

```
<cache-root>/reposix/<backend_slug>-<project>.git
```

The `<project>` segment must be filesystem-safe. GitHub's `owner/repo`
form contains a path separator that would create a nested directory,
so the helper sanitizes via `sanitize_project_for_cache` (replace
`/`, `\`, `:` with `-`) before reaching `Cache::open`. The
**unsanitized** `owner/repo` string is still passed to
`BackendConnector` methods so `GithubReadOnlyBackend` can assemble
`repos/{owner}/{repo}/...` URLs correctly. Concretely:

- `github::reubenjohn/reposix` → cache dir `github-reubenjohn-reposix.git`,
  backend project string `reubenjohn/reposix`.
- `confluence::TokenWorld` → cache dir `confluence-TokenWorld.git`,
  backend project string `TokenWorld`.
- `jira::TEST` → cache dir `jira-TEST.git`, backend project string
  `TEST`.

### Credential resolution

Each non-sim backend reads its credentials from environment variables
documented in `docs/reference/testing-targets.md`:

| Backend | Required env vars |
|---|---|
| `Sim` | (none) |
| `GitHub` | `GITHUB_TOKEN` |
| `Confluence` | `ATLASSIAN_API_KEY`, `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` |
| `Jira` | `JIRA_EMAIL`, `JIRA_API_TOKEN`, `REPOSIX_JIRA_INSTANCE` |

Missing-creds errors list every absent env var on its own line and
include a pointer to `docs/reference/testing-targets.md`. The
`REPOSIX_ALLOWED_ORIGINS` egress allowlist is enforced by
`reposix_core::http::client()` — the helper does no extra check.

### Audit signal

A new `op='helper_backend_instantiated'` row is appended to
`audit_events_cache` once per `git-remote-reposix` invocation that
opens the cache. The row records `(backend_slug, project_for_cache,
project_for_backend)` so forensics can reconstruct the original
backend project string even when the cache directory uses the
sanitized form.

## Why dispatch lives in the helper, not the CLI alone

The CLI's `reposix init` writes `remote.origin.url` and exits — git
itself invokes the helper on subsequent `git fetch` / `git push`
calls. Once init has happened, the CLI is no longer in the call
stack. The helper is the only process git talks to about transport,
so it must own dispatch.

The CLI's existing `translate_spec_to_url` (in `init.rs`) and the
helper's `parse_remote_url` are duals: the CLI translates
`<backend>::<project>` (the friendly form a human types) into a URL,
and the helper translates the URL back into a `(BackendKind, project)`
tuple. The friendly form is **not** stored in `git config` — only the
URL is — so the helper has no choice but to reconstruct the kind from
the URL.

## Alternatives considered

1. **Encode backend kind in a query string** (`?backend=jira`).
   Rejected because the existing `parse_remote_url` in
   `reposix-core/src/remote.rs` (used by other code paths) splits at
   `/projects/` without query parsing; adding query handling would
   ripple. The path-segment marker is more visually obvious in
   `git config remote.origin.url` output.
2. **Use a fully-tagged URL scheme** (`reposix::sim::demo`,
   `reposix::github::owner/repo`). Rejected because dozens of existing
   tests assert the `reposix::<scheme>://<host>/projects/<slug>`
   shape; flipping the format would have invalidated the entire
   integration test surface in one phase.
3. **Probe the backend at startup** (try a sim port, then GitHub,
   etc.). Rejected because the network-probing model would couple
   helper startup to liveness of every potential backend, and the
   probe order would leak information about credentials.

## Consequences

### Positive

- Real-backend `git fetch` actually works against the right adapter.
  Phase 35's `agent_flow_real` tests (which were stuck verifying CLI
  init only) now exercise the full helper path when creds are
  present.
- `pending-secrets` CI jobs (`integration-contract-confluence-v09`
  etc.) become real coverage as soon as secret packs decrypt.
- Cache directories are unambiguous per backend — no more risk of
  Confluence and JIRA colliding on `<tenant>-TokenWorld.git` (they
  now bear distinct `confluence-` / `jira-` prefixes).

### Negative

- The Atlassian URL form changed shape (added `/confluence/` /
  `/jira/` markers). Any external doc or script that hardcoded the
  old form needs to update. We mitigate by:
  - Keeping the change scoped to `reposix init`'s emitted URL.
  - Treating the old marker-less form as a parse error (with a
    clear message naming the required marker), so nothing silently
    dispatches to the wrong backend.

### Neutral

- The helper now depends on three backend crates
  (`reposix-github`, `reposix-confluence`, `reposix-jira`) that were
  previously CLI-only. The compile-time blast radius grows, but
  workspace builds and CI cache hits are unchanged at observable
  granularity (~4s incremental).

## References

- `.planning/v0.9.0-MILESTONE-AUDIT.md` §5 — the tech-debt entry that
  motivated this work.
- `crates/reposix-remote/src/backend_dispatch.rs` — implementation.
- `crates/reposix-remote/src/main.rs::real_main` — the dispatch
  call-site.
- `docs/reference/testing-targets.md` — credential matrix the
  missing-env error message points at.
