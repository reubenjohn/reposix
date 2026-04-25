# Testing targets — sanctioned real-backend test fixtures

> "TokenWorld is for testing — go crazy, it's safe."
> — project owner, 2026-04-24

reposix's architecture claims (pure-git agent UX, sub-second sim
latencies, FUSE-free transport) are not satisfied by simulator-only
coverage per project CLAUDE.md OP-6. This document enumerates the three
real-backend targets the project considers safe-to-mutate during
automated tests, the env vars each target requires, the rate-limit
notes, and the cleanup procedure each target expects.

The literal phrase "go crazy, it's safe" appears verbatim above and at
the end of each section so any future agent reading this doc has zero
hesitation about running automated tests that mutate state on these
specific targets.

> **Linked from:** project `CLAUDE.md` (Phase 36 wires the cross-link
> from the "Commands you'll actually use" section).

---

## Confluence — `TokenWorld` space

The `TokenWorld` Confluence space at
`https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net/wiki/spaces/TokenWorld`
is the project owner's test-only Confluence space. Tests that create
pages, mutate bodies, attach files, and delete content are explicitly
sanctioned.

### Env vars

| Name | Purpose |
|------|---------|
| `ATLASSIAN_API_KEY` | Atlassian Cloud API token. NEVER logged. |
| `ATLASSIAN_EMAIL` | The account the API token is associated with. |
| `REPOSIX_CONFLUENCE_TENANT` | Atlassian Cloud subdomain, e.g. `reuben-john`. |
| `REPOSIX_ALLOWED_ORIGINS` | Must include `https://${tenant}.atlassian.net`. |

### Rate-limit notes

Atlassian Cloud applies per-tenant rate limits (see
`crates/reposix-confluence/src/rate_limit.rs`). The reposix-confluence
adapter honors `Retry-After` and falls back to exponential backoff with
a 4-attempt cap. For aggressive test loops (>10 mutations per second)
serialize the test run via `--test-threads=1`.

### Cleanup

Tests that create pages SHOULD tag them with a `kind=test` label so the
Phase 36 cleanup procedure (deferred per the v0.9.0 plan) can sweep
them. For now, manually delete leftover pages from
`https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net/wiki/spaces/TokenWorld`
after a test session. Do not leave junk pages lying around.

> "TokenWorld is for testing — go crazy, it's safe." — project owner, 2026-04-24

---

## GitHub — `reubenjohn/reposix` issues

The project's own GitHub repo at
<https://github.com/reubenjohn/reposix>. Tests that create issues,
update labels, close issues, and add comments are explicitly sanctioned
(but DO NOT push code to branches or modify releases).

### Env vars

| Name | Purpose |
|------|---------|
| `GITHUB_TOKEN` | Personal access token (or `gh auth token`). NEVER logged. |
| `REPOSIX_ALLOWED_ORIGINS` | Must include `https://api.github.com`. |

### Rate-limit notes

GitHub applies a 5000 req/hr limit for authenticated requests (and
1000 req/hr for the issues endpoint specifically). The reposix-github
adapter parks the next call until reset, capped at 60s. For aggressive
test loops serialize via `--test-threads=1`.

### Cleanup

Tests that create issues SHOULD tag them with a `kind:test` label so
they can be located via `is:issue label:kind:test` and bulk-closed at
session end. The Phase 36 cleanup automation will handle this; for
now manual cleanup at <https://github.com/reubenjohn/reposix/issues>.

> "TokenWorld is for testing — go crazy, it's safe." — project owner, 2026-04-24
> (the same permission applies to `reubenjohn/reposix` issues — they
> are owner-controlled and safe to mutate during tests.)

---

## JIRA — project `TEST` (overridable)

JIRA Cloud project key `TEST` is the default test target. The project
key can be overridden per-test via `JIRA_TEST_PROJECT` or
`REPOSIX_JIRA_PROJECT`; if both are unset, tests fall back to `TEST`.

### Env vars

| Name | Purpose |
|------|---------|
| `JIRA_EMAIL` | The Atlassian account the token is associated with. |
| `JIRA_API_TOKEN` | Atlassian Cloud API token. NEVER logged. |
| `REPOSIX_JIRA_INSTANCE` | Atlassian Cloud subdomain, e.g. `reuben-john`. |
| `JIRA_TEST_PROJECT` | (optional) Project key override. |
| `REPOSIX_JIRA_PROJECT` | (optional) Alternative project key override. |
| `REPOSIX_ALLOWED_ORIGINS` | Must include `https://${instance}.atlassian.net`. |

The project key resolution precedence is:
`JIRA_TEST_PROJECT` → `REPOSIX_JIRA_PROJECT` → `TEST`.

### Rate-limit notes

JIRA Cloud applies the same per-tenant rate limits as Confluence; the
reposix-jira adapter honors `Retry-After` (Phase 28). For aggressive
test loops (>5 mutations per second) serialize via
`--test-threads=1`.

### Cleanup

Tests that create issues SHOULD apply the `kind=test` label and a
descriptive summary prefix (e.g. `[reposix-test]`) so they can be
located via the JIRA query
`project = TEST AND labels = kind=test`. Bulk-close at session end.

> "TokenWorld is for testing — go crazy, it's safe." — project owner, 2026-04-24
> (the same permission applies to JIRA project `TEST` — it is the
> owner's test-only project and safe to mutate during tests.)

---

## Running real-backend tests

Each backend's test surface is `#[ignore]`-gated and additionally
`skip_if_no_env!`-guarded. Without env vars, all tests skip cleanly with
`SKIP: env vars unset: …` to stderr. With env vars, tests exercise the
real backend.

> **As of v0.10.0 the helper actually dispatches by URL scheme** — `git
> fetch` against a `reposix::https://api.github.com/...` remote hits
> GitHub, not the local sim. Pre-v0.10.0 the helper hardcoded
> `SimBackend` and only `reposix init` exercised the real adapter; see
> [ADR-008](../decisions/008-helper-backend-dispatch.md) for details.
> Note the Atlassian URL form picked up a `/confluence/` or `/jira/`
> path-segment marker so the helper can tell the two adapters apart.

```bash
# Default cargo test stays green without any secrets:
cargo test --workspace

# Real-backend exercise (creds present):
export GITHUB_TOKEN=…
export ATLASSIAN_API_KEY=… ATLASSIAN_EMAIL=… REPOSIX_CONFLUENCE_TENANT=…
export JIRA_EMAIL=… JIRA_API_TOKEN=… REPOSIX_JIRA_INSTANCE=…
export REPOSIX_ALLOWED_ORIGINS='https://api.github.com,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net'

cargo test -p reposix-confluence --features live -- --ignored
cargo test -p reposix-github --features live -- --ignored
cargo test -p reposix-jira --features live -- --ignored

# v0.9.0 dark-factory + init flow:
cargo test -p reposix-cli --test agent_flow_real -- --ignored
```

Phase 36 wires three CI integration jobs
(`integration-contract-confluence-v09`, `-github-v09`, `-jira-v09`) that
decrypt the relevant secret pack and run these test commands on every
push to `main`.
