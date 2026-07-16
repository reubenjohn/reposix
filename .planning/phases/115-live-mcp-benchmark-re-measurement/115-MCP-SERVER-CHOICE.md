# 115-MCP-SERVER-CHOICE.md — ratified MCP-server choice for the P115 token benchmark

**Author:** P115 Task-4 capture executor (L0 workhorse #38) · **Date:** 2026-07-16 (UTC) ·
**Status:** ratified-with-blocker (the server is chosen; it cannot execute the benchmark
task as specified — see § Blockers, escalated to owner).

This is the Task-1 residual formal note. It is **grounded, not a rubber-stamp**: the
ratified choice is recorded together with the reality captured while wiring it, including
the reasons it does not currently satisfy the T4 mcp-arm capture.

## Ratified choice

**Official Atlassian remote MCP server ("Rovo") — `atlassian-rovo`.**

| Field | Value |
|---|---|
| Server | `atlassian-rovo` |
| URL | `https://mcp.atlassian.com/v1/mcp` |
| Transport | streamable-HTTP |
| serverInfo | `atlassian-mcp-server` v1.0.0 |
| Auth | Bearer **API token** (Personal API Token; org-admin-gated) — token value never recorded |
| Scope in Claude Code | Local config, private to the reposix repo (`claude mcp get atlassian-rovo`) |
| Connection status | `✔ Connected` (`claude mcp list`) — handshake succeeds |

Chosen over the community `sooperset/mcp-atlassian` because the official remote requires
no self-hosting and the existing `.env` API token authenticates it (auth refuted the
rotation-#34 "API-token-endpoint blocker" — see `115-ROVO-AUTH-CHECK.md`).

## Evidence the tools LOAD (positive)

- **Smoke session** (mandatory pre-capture check): a nested `claude -p "List every tool
  available to you" --output-format json --dangerously-skip-permissions` run from
  `cwd=<reposix repo>` (where `atlassian-rovo` is local-scoped) listed exactly three
  `mcp__atlassian-rovo__*` tools. Session `52b94471-2c68-4d96-9dcf-7b4674b325a7`; usage
  committed at `benchmarks/captures/mcp-kan-smoke.json`; JSONL under the reposix project
  hash. This proves the MCP context (tool schemas) loads and the tool-loading cost is real.
- The three tools and their exact schemas are captured at
  `benchmarks/fixtures/mcp_jira_catalog.json` (replaces the prior synthetic 35-tool file).

## Blockers — why the T4 mcp-arm capture cannot run as specified (escalated to owner)

Three independent findings, each verified this rotation:

1. **No Jira issue-CRUD tool on this server.** `atlassian-rovo` advertises **only 3
   Teamwork Graph tools**: `getTeamworkGraphContext` (read), `getTeamworkGraphObject`
   (read/hydrate), and `addTeamworkGraphContext` (adds a *relationship* link only —
   blocks/links/tracks). There is **no** `editJiraIssue` / `createJiraIssue` /
   `updateJiraIssue` / JQL-search tool. The server's own instructions say "Do not use
   Teamwork Graph tools for basic CRUD operations." → The benchmark task's **"edit 1
   issue"** step has no tool. (The synthetic fixture assumed a full-CRUD server —
   `sooperset/mcp-atlassian` — which is a *different* server.)
2. **The API token is permission-denied on actual invocation.** A real `tools/call`
   (`getTeamworkGraphContext` on `JiraSpace KAN`, `cloudId=https://reuben-john.atlassian.net`)
   returned: `"You don't have permission to connect via API token. Please ask your
   organization admin for access."` The token authenticates the `initialize` handshake
   (200, tools listed) but is **not authorized to run the tools**. This resolves the
   explicit open caveat (1) in `115-ROVO-AUTH-CHECK.md` ("tool-level authorization scopes
   were NOT verified") — with a negative result.
3. **Jira project KAN has 0 issues.** `reposix init jira::KAN` (which uses reposix's own
   Jira REST v3 basic-auth path — a *different* credential path that works) synced KAN and
   produced an empty tree: commit message `sync(jira:KAN): 0 issues`, `git ls-tree HEAD`
   empty. → Neither arm can "read 3 issues"; there are none. This also blocks the
   reposix-mediated arm, independent of the MCP findings.

Net: the ratified server is reachable and its schemas load (so the *tool-loading* half of
the token economy is measurable, and is captured), but the **end-to-end read+edit+push
task** cannot be executed via `atlassian-rovo` with the current credential, and KAN has no
content to operate on.

## Recommendation for the owner (decision required before spending more of the 50-session budget)

Any ONE of these unblocks a real, comparable capture; all are owner/charter calls:

- **(A) Grant the API token Teamwork-Graph access** (per the error's own instruction: "ask
  your organization admin"), AND **redefine the mcp-arm task** to what the graph tools can
  do — e.g. "read 3 work items + add one relationship link" — since issue-field edits are
  not possible on this server. Requires accepting that the two arms measure a
  read+link workflow, not read+field-edit.
- **(B) Switch the mcp-arm to `sooperset/mcp-atlassian`** (self-hosted, API-token-only,
  full Jira CRUD) so the ratified benchmark task ("read 3, edit 1, push") runs unchanged on
  both arms. This is the server the synthetic fixture was modeled on. Needs setup +
  egress-allowlist review.
- **(C) Seed KAN with ≥3 issues** (via reposix push, sanctioned per OP-6 "if you create a
  throwaway test issue, note it") to unblock the reposix arm regardless — but this alone
  does not fix the MCP arm.

The capture executor did **not** unilaterally pick a server-swap, redefine the ratified
task, or seed KAN, because each changes what the benchmark measures and/or spends the
capped session budget on a comparison that is currently impossible. Honest partial over
fabricated completion.

## Cross-references

- `benchmarks/fixtures/mcp_jira_catalog.json` — real captured tool surface (evidence for finding 1).
- `benchmarks/captures/mcp-kan-smoke.json` — the one real session's scrubbed usage.
- `benchmarks/bench-session-ledger.md` — session 1 recorded (running_total 1/50).
- `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` — BLOCKER entry (2026-07-16).
- `115-ROVO-AUTH-CHECK.md` — prior auth probe; its open caveat (1) is closed here (negative).
