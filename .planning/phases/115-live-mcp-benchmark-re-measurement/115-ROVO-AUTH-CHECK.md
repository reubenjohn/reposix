# 115-ROVO-AUTH-CHECK.md — READ-ONLY Rovo MCP auth verification (P115 T4 pre-work)

**Rotation:** L0 workhorse #36 · **Date:** 2026-07-15 · **Mode:** READ-ONLY probe (no
live MCP connection wired, no capture session spent, zero backend writes).

**Charter:** confirm or refute rotation #34's noted "API-token-endpoint blocker" —
i.e. whether the EXISTING `ATLASSIAN_API_KEY` (+ `ATLASSIAN_EMAIL`) can authenticate
Atlassian's official remote ("Rovo") MCP endpoint, or whether that endpoint is
OAuth-2.1-only. Manager rotation-#10 refresh named this as the outstanding T4
pre-requisite (`MANAGER-HANDOVER.md` line 106-107). Findings recorded here with
evidence per the launch charter.

> Credential values are NEVER printed here. Requests below reference the credential
> only as `$ATLASSIAN_API_KEY` / `$ATLASSIAN_EMAIL`; the tenant is written generically
> as `<tenant>.atlassian.net`. Credential was sent ONLY to official `*.atlassian.com`
> hosts. No `tools/call` or any data-touching request was made — probing stopped at the
> `initialize` handshake.

## VERDICT

**#34's "API-token-endpoint blocker" is REFUTED (HIGH confidence).** The existing
`ATLASSIAN_API_KEY` DOES authenticate the official Atlassian remote MCP endpoint
(`atlassian-mcp-server` v1.0.0 at `https://mcp.atlassian.com/v1/mcp`) — via **either**
HTTP Basic (`email:token`) **or** raw `Authorization: Bearer <token>`. The MCP
`initialize` handshake returned **HTTP 200** with a valid JSON-RPC result and an issued
`mcp-session-id` under both credential forms, bracketed by a no-auth **401** control.

## Endpoints probed

| URL | Transport | Note |
|---|---|---|
| `https://mcp.atlassian.com/v1/mcp` | streamable-HTTP | current primary endpoint — **authenticated OK** |
| `https://mcp.atlassian.com/v1/mcp/authv2` | streamable-HTTP | OAuth-oriented alias; RFC 9728 protected-resource-metadata pointer observed |
| `https://mcp.atlassian.com/v1/sse` | SSE | NOT probed — official docs mark SSE unsupported after 2026-06-30 (already elapsed) |
| `https://<tenant>.atlassian.net/wiki/rest/api/space` | REST | token-validity baseline |

## Evidence (HTTP status codes + key headers)

| # | Request (creds as $VARS) | Host | HTTP | Key header / body | Interpretation |
|---|---|---|---|---|---|
| 1 | `GET /wiki/rest/api/space?limit=1 -u $ATLASSIAN_EMAIL:$ATLASSIAN_API_KEY` | `<tenant>.atlassian.net` | **200** | — | Token itself is valid (baseline) |
| 2 | `GET /v1/mcp` (no auth) | mcp.atlassian.com | **401** | `WWW-Authenticate: Bearer realm="OAuth", error="invalid_token"` | Endpoint is auth-protected |
| 3 | `OPTIONS /v1/mcp` (no auth) | mcp.atlassian.com | 204 | CORS preflight, no challenge | Preflight only |
| 4 | `GET /v1/mcp/authv2` (no auth) | mcp.atlassian.com | **401** | `WWW-Authenticate: Bearer resource_metadata="…/.well-known/oauth-protected-resource/v1/mcp/authv2"` | RFC 9728 metadata pointer present |
| 5 | `initialize` POST `-u $ATLASSIAN_EMAIL:$ATLASSIAN_API_KEY` | mcp.atlassian.com/v1/mcp | **200** | `mcp-session-id` issued; `serverInfo:{name:"atlassian-mcp-server",version:"1.0.0"}` | **API token (Basic) ACCEPTED** |
| 6 | `initialize` POST `Authorization: Bearer $ATLASSIAN_API_KEY` | mcp.atlassian.com/v1/mcp | **200** | same result incl. tool-usage `instructions` | **API token (Bearer) ACCEPTED** |
| 7 | `initialize` POST, no auth (control) | mcp.atlassian.com/v1/mcp | **401** | `{"error":"invalid_token","error_description":"Missing or invalid access token"}` | Confirms the 200s are auth-gated, not an open endpoint |
| 8 | Bare `GET`, auth'd, no session | mcp.atlassian.com/v1/mcp | 400 | `{-32600 "Request must be an initialize request if no session ID is provided."}` | Protocol-level (not auth) — resolved by a proper `initialize` POST |

## Official-docs finding

Atlassian's own docs confirm API-token auth is a supported path on the official remote
MCP server: **"Configuring authentication via API token"**
(`support.atlassian.com/atlassian-rovo-mcp-server/docs/configuring-authentication-via-api-token/`)
— Personal API Token via Basic (`base64(email:token)`) or Service-Account key via
Bearer, **"if enabled by your organization admin."** OAuth 2.1 remains the *recommended
default for interactive use*; API token is positioned for non-interactive /
machine-to-machine scenarios. Empirically confirmed **enabled for this tenant**.

## Official remote vs `sooperset/mcp-atlassian`

Both accept API tokens. `sooperset/mcp-atlassian` (self-hosted) has always been
API-token-only. The **official** `atlassian/atlassian-mcp-server` remote defaults to
OAuth 2.1 but also accepts API tokens when the org admin has enabled it — which is the
case for this tenant, per probe #5/#6.

## Recommendation for P115 Task 4 mcp-arm (recommendation, not a ratified choice)

**Official Rovo remote MCP via API token.** It works end-to-end with the existing
`.env` credential (200 + session established) — no OAuth browser flow, no self-hosted
fallback needed. The formal server choice for T4 remains the T4-executor / manager's
call; this note only removes the auth uncertainty that blocked it.

## Confidence + caveats

**HIGH** — directly observed 200s on the real official endpoint under two independent
auth-header forms, bracketed by a 401 control and a valid-token REST baseline. Caveats:
(1) tool-level authorization scopes were NOT verified (no `tools/call` was made — that
would be a T4 capture action, out of this read-only charter); (2) org-admin API-token
enablement could in principle be tenant-specific or later revoked; (3) `.env`'s
`REPOSIX_CONFLUENCE_TENANT` holds only the subdomain slug — the full host needs
`.atlassian.net` appended, per `docs/reference/testing-targets.md:63`.
