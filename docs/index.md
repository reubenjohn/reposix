---
title: reposix
---

# reposix

> **Agents already know `cat` and `git`. They don't know your JSON schema.**

reposix exposes REST-based issue trackers (Jira, GitHub Issues, Confluence) as a **real git working tree**. An autonomous LLM agent can `git clone`, `cat`, `grep`, edit, and `git push` tickets without learning a single Model Context Protocol (MCP) tool schema or REST SDK surface. It complements REST — the other 20% of operations (complex JQL, bulk imports, admin) keep using the API directly.

## Before — five round trips through the REST API

```bash
# Transition PROJ-42 to Done, reassign, comment.
curl -s -u "$E:$T" /rest/api/3/issue/PROJ-42/transitions \
  | jq -r '.transitions[] | select(.name=="Done") | .id'           # 1. lookup id
curl -s -u "$E:$T" -X POST .../transitions -d '{"transition":...}' # 2. transition
curl -s -u "$E:$T" /rest/api/3/user/search?query=alice@acme.com    # 3. resolve user
curl -s -u "$E:$T" -X PUT .../PROJ-42 -d '{"fields":{"assignee":...}}'  # 4. assign
curl -s -u "$E:$T" -X POST .../comment -d '{"body":{"type":"doc",...}}' # 5. ADF comment
```

## After — one commit

```bash
cd ~/work/acme-jira
sed -i -e 's/^status: .*/status: Done/' \
       -e 's/^assignee: .*/assignee: alice@acme.com/' issues/PROJ-42.md
echo $'\n## Comment\nShipped in v0.7.1' >> issues/PROJ-42.md
git commit -am "close PROJ-42" && git push
```

The audit trail is `git log`. No SDK to vendor; no schemas to load.

## Three measured numbers

<div class="grid cards" markdown>

-   **`8 ms`** — read one issue from the local cache after first fetch ([`v0.9.0-latency.md`](benchmarks/v0.9.0-latency.md)).
-   **`0`** — MCP schema tokens an agent loads before the first useful op.
-   **`1`** — command to bootstrap (`reposix init sim::demo /tmp/repo`).

</div>

[Mental model in 60 seconds →](concepts/mental-model-in-60-seconds.md){ .md-button .md-button--primary }
[How it complements MCP and SDKs →](concepts/reposix-vs-mcp-and-sdks.md){ .md-button }

---

## Tested against

reposix's `8 ms` cache read is measured against the in-process simulator, but the architecture is exercised end-to-end against three real backends sanctioned by the project owner for aggressive testing:

- **Confluence — [TokenWorld space](reference/testing-targets.md#confluence--tokenworld-space)** (Atlassian Cloud).
- **GitHub — [`reubenjohn/reposix` issues](reference/testing-targets.md#github--reubenjohnreposix-issues)** (this project's own tracker).
- **JIRA — [project `TEST`](reference/testing-targets.md#jira--project-test-overridable)** (overridable via `JIRA_TEST_PROJECT`).

Latency for each backend is captured in [`docs/benchmarks/v0.9.0-latency.md`](benchmarks/v0.9.0-latency.md). Sim cold init is `24 ms` (soft threshold `500 ms`); list-issues `9 ms`; capabilities probe `5 ms`. Real-backend cells fill in once CI secret packs are wired (Phase 36).

## Five-line quickstart

```bash
git clone https://github.com/reubenjohn/reposix && cd reposix
cargo build --release --workspace --bins
export PATH="$PWD/target/release:$PATH"
reposix init sim::demo /tmp/reposix-demo
cd /tmp/reposix-demo && git checkout origin/main && cat issues/0001.md
```

After `init`, agent UX is pure git: `cat`, `grep -r`, edit, `git commit`, `git push`. The bootstrap takes ≤ `24 ms` against the simulator on a stock laptop.

## Where to go next

<div class="grid cards" markdown>

-   :material-lightbulb-on: **[Mental model in 60 seconds](concepts/mental-model-in-60-seconds.md)** — three keys to the design (clone = snapshot · frontmatter = schema · `git push` = sync verb).
-   :material-compare: **[reposix vs MCP and SDKs](concepts/reposix-vs-mcp-and-sdks.md)** — positioning, with measured numbers per row.
-   :material-graph: **How it works** — [the filesystem layer](how-it-works/filesystem-layer.md), [the git layer](how-it-works/git-layer.md), and [the trust model](how-it-works/trust-model.md). One diagram each.
-   :material-chart-line: **[Latency envelope](benchmarks/v0.9.0-latency.md)** — the v0.9.0 measured numbers.

</div>

## What it looks like underneath

reposix has three pieces — a local bare git repository built from REST responses (with file content fetched lazily), a `git` remote that handles both reads and pushes by translating to API calls, and `reposix init` (a one-shot bootstrap). Two guardrails are load-bearing for autonomous agents: **push-time conflict detection** rejects stale-base pushes with the standard git "fetch first" error so an agent recovers via `git pull --rebase`; the **fetch size limit** caps `git fetch` and emits a stderr message that names `git sparse-checkout` as the recovery move. An agent unfamiliar with reposix observes the error, runs `sparse-checkout`, and recovers with no human prompt engineering.

The detail of how each piece works lives in [How it works](how-it-works/filesystem-layer.md). The reference material — frontmatter schema, simulator HTTP surface, testing targets — is in [Reference](reference/simulator.md).

---

*Honest scope: built across autonomous coding-agent sessions; v0.9.0 architecture pivoted from a virtual filesystem to git-native partial clone (2026-04-24). Treat as alpha — but every demo on this site is reproducible on a stock Ubuntu host in under five minutes. The v0.7 token-economy benchmark measured a 92.3% input-context-token reduction vs MCP for the same task.*
