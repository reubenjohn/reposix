---
title: "reposix: edit issue trackers with cat, grep, and git push"
date: 2026-04-25
authors:
  - reubenjohn
description: >
  Why we wrapped Jira, GitHub Issues, and Confluence in a real git working
  tree — and what 8ms cached reads buy an autonomous agent.
tags:
  - launch
  - agents
  - git
  - mcp
  - dark-factory
---

# reposix: edit issue trackers with `cat`, `grep`, and `git push`

> **TL;DR.** reposix exposes REST issue trackers (Jira, GitHub Issues,
> Confluence) as a real git working tree. Agents `git clone`, `cat`, `grep`,
> `sed`, and `git push` — no MCP schema, no SDK. Cached reads land in **8 ms**;
> bootstrap in **24 ms**. Simulator is the default backend; real backends are
> opt-in behind an egress allowlist. v0.9.0 shipped April 24; this is the
> launch post.

## The 100k-token tax

Picture the most boring thing an LLM agent does all day: closing a ticket.

An agent on the MCP path opens a session, the runtime advertises its tool
catalog, and the model loads ~100k tokens of schema discovery before it has
read a single byte of actual work. Fetching one issue is a tool dispatch
that round-trips through the protocol, the runtime, and the LLM context
window — typically **200–500 ms** of wall-clock and a non-trivial slice
of the model's attention. Multiply by the dozen issues in a triage queue,
and the agent has spent more tokens *deciding how to read* than it has
spent reading.

Here is the same operation under reposix:

```bash
$ cat issues/2444.md
---
id: 2444
title: Avatar upload returns 500 on > 4 MiB
status: in_progress
assignee: alice@acme.com
labels: [backend, regression]
---
## Description
S3 PutObject is rejecting multipart bodies above 4 MiB...
```

That `cat` is **8 ms** end-to-end against the simulator
([benchmark](../benchmarks/v0.9.0-latency.md)). It loads roughly 2k
tokens — the file's bytes, nothing else. The agent did not learn a new
protocol. It already knew `cat`. It already knew Markdown. It already
knew YAML frontmatter. The only thing reposix taught it is *where* the
file lives.

The cost differential is the headline. Everything else in this post is
the engineering required to make that one `cat` honest.

## The pattern: substrates a human would never read

Simon Willison spent an hour on Lenny's Podcast in April describing how
StrongDM ships security software with **nobody reading the code**. Their
twist: a swarm of thousands of agent "employees" file simulated tickets
24/7, against in-process fakes of every external dependency, exercising
the real product overnight at roughly $10k/day in tokens. Code review is
absorbed into invariants the agents enforce on each other.

We distilled the relevant patterns from that conversation into a
[reference doc](../research/agentic-engineering-reference.md). The piece
that matters for this post is the framing: when no human reads the
output, you are no longer designing for human comprehension. You are
designing for **substrates a human would never read** — log streams,
synthetic chat channels, audit tables, and, in our case, a git working
tree of Markdown files that no operator will ever cd into.

reposix is the same pattern applied to issue trackers. The working tree
is a *substrate* — uniform, append-mostly, diff-able, version-controlled,
with twenty years of off-the-shelf tooling. An agent is at home there
because every model in production was pre-trained on millions of git
repos. The substrate is in the model's bones.

We did not invent this framing; we are an exemplar of it.

## What an agent actually runs

Here is a complete triage loop. No reposix-specific commands except
`init`:

```bash
# One-time bootstrap — points a partial-clone working tree at the helper.
reposix init sim::demo /tmp/triage
cd /tmp/triage && git checkout origin/main

# Find every open issue mentioning "database".
grep -rl '^status: open' issues/ \
  | xargs grep -l 'database' \
  | head -5

# Triage one of them.
sed -i 's/^status: .*/status: in_progress/'        issues/2444.md
sed -i 's/^assignee: .*/assignee: alice@acme.com/' issues/2444.md
echo $'\n## Comment\nReproduced locally. Owning.' >> issues/2444.md

git commit -am 'PROJ-2444: triage to alice'
git push
```

What the agent sees: nine commands, all POSIX, all in its training
corpus. What the helper does behind the scenes:

1. `reposix init` runs `git init`, sets `extensions.partialClone=origin`,
   and points `remote.origin.url` at the `git-remote-reposix` binary.
   Total wall-clock: **24 ms** against the simulator.
2. The first `grep -r` materializes the blobs that match — each first-time
   touch is one REST GET, cached on disk thereafter. Subsequent reads
   are **8 ms** local.
3. `git push` pipes a fast-import stream into the helper. The helper
   parses it, fetches the current backend version of every changed file,
   compares against the agent's commit base, and either applies the
   writes (PATCH/POST) or rejects with `error refs/heads/main fetch first`.

The agent never learned that the remote is a REST API. The helper never
learned what an `assignee` field means. The contract between them is git
protocol; the contract between the helper and the backend is REST. Each
side speaks a vocabulary it already knew.

The architecture is documented end-to-end in
[How it works → filesystem](../how-it-works/filesystem-layer.md) and
[How it works → git](../how-it-works/git-layer.md), each with a mermaid
diagram.

## Hard numbers

The v0.9.0 latency envelope, measured against the in-process simulator
on a stock laptop ([reproducer](../benchmarks/v0.9.0-latency.md)):

| Step                                              | sim (ms) |
|---------------------------------------------------|---------:|
| `reposix init <backend>::<project> <path>` cold  |       24 |
| List issues (REST round-trip)                    |        9 |
| Get one issue (REST round-trip)                  |        8 |
| PATCH issue (REST round-trip)                    |        8 |
| Helper `capabilities` probe                      |        5 |

For comparison, characterized from public-API behaviour:

| Step                                              | MCP        | Raw REST SDK |
|---------------------------------------------------|-----------:|-------------:|
| Tokens before first useful op                    | ~100k      | ~5k          |
| Cached read latency                              | 200–500 ms | 100–300 ms   |

The MCP and SDK cells are not measured by our harness; they are
characterized from documented tool-dispatch overhead and HTTPS round-trip
times. Our v0.7 token-economy benchmark
([RESULTS.md](https://github.com/reubenjohn/reposix/blob/main/benchmarks/RESULTS.md),
if you want to dig) put the input-context-token reduction at **92.3%**
for the same task vs MCP.

The v0.9.0 win is on the latency axis, not the token axis: a cached
working tree means an agent can `grep -r database issues/` and get every
hit instantly, with no per-match REST call. That is the unlock.

Real-backend cells are blank in the public envelope until CI secret
packs land in v0.11.0. The architecture is tested end-to-end against
three sanctioned targets — Confluence space *TokenWorld*, GitHub
`reubenjohn/reposix` issues, and JIRA project `TEST` — but the
performance numbers we publish are the ones we can reproduce in CI.

## The trifecta, addressed head-on

reposix is, by construction, exactly the kind of system the
lethal-trifecta literature warns about. Three legs, all present:

| Leg | Where it shows up here |
|---|---|
| **Private data** | Issue bodies, custom fields, attachments. |
| **Untrusted input** | Every comment, title, and label is attacker-influenced text. |
| **Exfiltration** | `git push` is a side-effecting verb; the helper makes outbound HTTP. |

You cannot build this without all three. So instead of pretending one
isn't there, we cut the path between them at every boundary.

The five cuts that matter:

1. **Egress allowlist.** Every HTTP client is built through
   `reposix_core::http::client()`; a clippy `disallowed_methods` lint
   rejects direct `reqwest::Client::new()`. Default
   `REPOSIX_ALLOWED_ORIGINS` is `http://127.0.0.1:*`. A real backend is
   one explicit env-var addition away — and one audit row per refusal.
2. **Tainted-by-default types.** Bytes from the network return as
   `Tainted<Vec<u8>>`. The only safe conversion to `Untainted<T>` is the
   `sanitize()` boundary, which strips the server-controlled frontmatter
   fields (`id`, `created_at`, `version`, `updated_at`). A trybuild
   compile-fail test asserts you cannot send `Tainted<T>` to an egress
   sink without crossing the boundary.
3. **Push-time conflict detection.** The helper re-fetches the backend
   version of every changed file inside the `export` handler. On drift,
   it emits `error refs/heads/main fetch first` and writes a
   `helper_push_rejected_conflict` audit row. The agent — even one that
   has never read reposix's docs — knows what that error means, because
   every git remote on Earth speaks it.
4. **Blob-limit guardrail.** The helper counts `want <oid>` lines per
   `command=fetch` and refuses past `REPOSIX_BLOB_LIMIT` (default 200).
   The stderr names `git sparse-checkout` as the recovery move by name.
   A misbehaving agent that tries to materialize a 10 000-issue tree
   reads the error, narrows scope, retries — no prompt engineering, no
   reposix-specific knowledge.
5. **Append-only audit log.** SQLite WAL with `BEFORE UPDATE` and
   `BEFORE DELETE` triggers on `audit_events_cache`. Every
   network-touching action writes one row: `materialize`,
   `egress_denied`, `helper_push_accepted`, `helper_push_rejected_conflict`,
   `blob_limit_exceeded`, and friends. `git log` is the agent's intent;
   the audit table is the system's outcome.

What is **not** mitigated: shell access on the dev host bypasses every
cut. reposix is a substrate for safer agent loops — it is not a sandbox.
The full honest list of residual risks is in
[trust-model.md](../how-it-works/trust-model.md).

## Why git was the right substrate

The lazy answer is "agents already know git." The longer answer is that
git's protocol surface had two seven-year-old extension points sitting
unused, both of which fit our problem exactly.

**Partial clone.** Stable since git 2.20 (2018). `--filter=blob:none`
asks the remote for the tree (filenames, directory structure, blob
OIDs) without the file contents. Blobs lazy-fetch on demand the first
time the working tree references them. To git, this is just a remote
that happens to be slow on first read — every subsequent read is local.
For us, it is the difference between paying a REST round-trip per `cat`
(the v0.1 design) and paying it once per file ever.

**`stateless-connect` capability.** Defined in `git-remote-helpers(7)`
since 2018 but rarely advertised by any helper outside the core
`git-remote-{http,https,ftp,ftps}` family. A helper that advertises
`stateless-connect` is telling git "I will tunnel protocol v2 traffic
to a backing repository" — which is exactly the shape of "give git
direct access to a local bare repo built from REST responses." Pair it
with the `export` capability (which has been in git since 2009) for
push, and you get a single helper binary that reads via protocol-v2
tunnel and writes via fast-import. The hybrid worked in our v0.9.0 POC
and ships in production.

**Refspec namespacing.** The helper uses `refs/heads/*:refs/reposix/*`
rather than the obvious `refs/heads/*:refs/heads/*`. The non-default
namespace is load-bearing: collapsing it makes `fast-export` emit an
empty delta because the private OID matches the local HEAD, and the
push silently succeeds with zero changes. We learned this the hard way.
The bug is documented in the
[v0.9.0 architecture-pivot summary](https://github.com/reubenjohn/reposix/tree/main/.planning/research/v0.9-fuse-to-git-native).

The result of all three: agents talk to a remote that *is* a
real git remote. Native conflicts surface as `git merge` conflicts on
text files. `git stash`, `git restore`, `git rebase --interactive`,
`.gitignore`, hooks — all work. We did not build a synthetic surface;
we plugged into the one git already has.

## Try it in five minutes

The simulator-first ethos is in
[CLAUDE.md OP-1](https://github.com/reubenjohn/reposix/blob/main/CLAUDE.md):
the in-process simulator is the default for every demo, every test, and
every autonomous loop. No real credentials needed.

```bash
git clone https://github.com/reubenjohn/reposix && cd reposix
cargo build --release --workspace --bins
export PATH="$PWD/target/release:$PATH"

reposix init sim::demo /tmp/reposix-demo
cd /tmp/reposix-demo && git checkout origin/main
cat issues/0001.md

# Make an edit, push, watch the audit row land.
sed -i 's/^status: .*/status: done/' issues/0001.md
git commit -am 'close 0001' && git push

sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
  "SELECT ts, op, decision FROM audit_events_cache ORDER BY ts DESC LIMIT 5"
```

Wall-clock: under five minutes on a stock Ubuntu host. The full
walk-through is in [tutorials/first-run.md](../tutorials/first-run.md).
There are five end-to-end agent loop examples in
[`examples/`](https://github.com/reubenjohn/reposix/tree/main/examples)
— one of them is a two-agent contention scenario where the second
agent recovers from a `fetch first` rejection without any in-context
learning.

To point at a real backend, set `REPOSIX_ALLOWED_ORIGINS` and the
relevant credential env vars. The
[testing-targets reference](../reference/testing-targets.md) names the
three sanctioned safe-to-mutate targets.

## What's next

v0.10.0 (this milestone) was a docs-only shine pass — the
[Diátaxis-structured site](../index.md), the
[5-minute first-run tutorial](../tutorials/first-run.md), the trio of
[How it works](../how-it-works/filesystem-layer.md) pages each with a
single mermaid diagram, and a banned-words linter that mechanically
enforces the progressive-disclosure framing. Full notes in the
[changelog](https://github.com/reubenjohn/reposix/blob/main/CHANGELOG.md#v0100--2026-04-25).

v0.11.0 themes, in priority order:

- **Real-backend benchmarks vs MCP.** The latency envelope's empty
  cells fill in. We will measure, not characterize.
- **Agent-SDK integration guides.** Claude Code, Cursor, and one
  custom-SDK pattern, each with a working-loop fixture.
- **Helper backend abstraction.** The `stateless-connect` handler
  currently hardcodes `SimBackend`; that is honest tech debt from the
  v0.9.0 pivot and ships before any benchmark commits.

We are deliberately not promising connector-count milestones. The
existing four (sim, GitHub, Confluence, JIRA) are enough to validate
the architecture; the open invitation is for the community to write
the next one.

## What we'd like from you

If any of this resonates:

- **Star the repo** — [github.com/reubenjohn/reposix](https://github.com/reubenjohn/reposix).
  Visibility is the bottleneck.
- **Try the simulator.** Five minutes, no credentials. The full
  five-line bootstrap is at the top of
  [the docs landing](../index.md).
- **Open a connector proposal** for your favourite tracker. Linear,
  Asana, Notion, GitLab Issues, ServiceNow — every REST tracker is
  fair game. File an issue with the API surface you want and the
  data-shape questions you'd like answered.
- **Write your own connector.** The
  [connector guide](../guides/write-your-own-connector.md) walks
  through the `BackendConnector` trait and the audit-row contract.
  A new backend is roughly 600 lines of Rust plus fixtures.
- **Push back.** If the trifecta cuts feel inadequate for your threat
  model, we want to hear why. The honest residual-risk list in
  [trust-model.md](../how-it-works/trust-model.md) is meant to invite
  argument, not close it.

reposix complements MCP and REST SDKs. It absorbs the ceremony around
the operations every agent does constantly; the API stays exactly where
it is for everything else. The thesis is one sentence —
[the rest of the site](../index.md) is the engineering required to make
that sentence honest.

---

*Suggested socials tags:* `#agents` `#llmtools` `#git` `#mcp`
`#darkfactory` `#rust`
