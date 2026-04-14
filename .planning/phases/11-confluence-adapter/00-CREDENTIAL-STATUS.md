# Phase 11 — credential status at phase start

**Run date:** 2026-04-13 ~21:00 PDT (session 3, overnight agent)

## What we tried

- **Token:** `ATLASSIAN_API_KEY` from `.env` (was named `TEAMWORK_GRAPH_API` in handoff; renamed by user). `ATATT3xF…` prefix, 192 chars — standard user API token issued from <https://id.atlassian.com/manage-profile/security/api-tokens>.
- **Email guess:** `reubenvjohn@gmail.com` from `git config user.email`.
- **Auth scheme attempted:** Basic (`email:token` base64) — correct for this token class; Bearer would require OAuth 2.0 3LO.
- **Probe subagent:** `ad26affc2d5b9bf24`, ran 19 tool calls / 237 s / ~70 tenant-subdomain candidates swept.

## What we found

| Endpoint | Result | Diagnosis |
|---|---|---|
| `GET api.atlassian.com/me` with Basic | **401** + header `x-failure-category: FAILURE_CLIENT_AUTH_MISMATCH` | Email and token don't pair to the same Atlassian Account ID. |
| `GET api.atlassian.com/oauth/token/accessible-resources` | 401 | Requires OAuth; expected. |
| ~70 tenant-subdomain probes (`reubenjohn`, `reubenvjohn`, `rjohn`, `rvj`, …) | bit-for-bit equal to unauth response | We're anonymous on every tenant that exists. |
| `rvj.atlassian.net/rest/api/3/serverInfo` | 200; `displayUrl: https://internal.jira.picmo.com.br` | That tenant belongs to a Brazilian company "picmo", not this user. |

## Most likely cause

The token was issued under a **different Atlassian login email** than `reubenvjohn@gmail.com`. Atlassian tokens are account-scoped, not email-scoped; the email used for Basic auth must exactly match the account the token was issued under. Work-email / alias / old-address are the common variants.

## Implications for Phase 11 scope

- **Can ship**: adapter crate, wiremock unit tests, contract test skeleton, CLI dispatch, Tier 3B + Tier 5 demo scripts that skip cleanly when env unset, ADR-002, docs, CHANGELOG, `v0.3.0` tag.
- **Cannot ship**: empirical proof that the adapter works against a real Confluence instance (HANDOFF §9 step B). This failure of Operating Principle #1 ("close the feedback loop") is a known gap.
- **Unblock path** (30 seconds of user time when they wake up):
  1. At <https://id.atlassian.com/manage-profile/security/api-tokens>, note the email address shown at the top-right.
  2. `export ATLASSIAN_EMAIL="that-email@example.com"` + `export REPOSIX_CONFLUENCE_TENANT="yourtenant"` + `export REPOSIX_CONFLUENCE_SPACE="SPACEKEY"`.
  3. Run the integration snippet from `MORNING-BRIEF-v0.3.md`.

## What the adapter WILL do correctly despite no live verification

- HTTP calls go through `reposix_core::http::HttpClient` (SG-01 allowlist enforcement).
- All ingress is wrapped in `Tainted<T>` (SG-05).
- Body sanitization will strip server-controlled fields on the write path (SG-03) — not relevant in v0.3 read-only, but the plumbing is there.
- Unit tests against `wiremock::MockServer` exercise the pagination, auth-header, hierarchy-flattening, and error paths.
- The contract test auto-skips live tests when `ATLASSIAN_API_KEY` is unset (exact same pattern as `reposix-github`'s `#[ignore]`-gated half).

Proceeding to `/gsd-plan-phase 11`.
