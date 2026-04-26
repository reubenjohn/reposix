---
title: "Phase 30 narrative vignettes — landing-page aha moment"
date: 2026-04-17
context: |
  Drafted during /gsd-explore after v0.8.0 (JIRA Cloud Integration, Phases 27–29)
  shipped. Phase 26 (under v0.7.0) fixed textual clarity of docs — versions,
  missing subcommands, orientation. The mkdocs site is correct but doesn't
  sell the value on first click. This note anchors Phase 30 "Docs IA +
  narrative overhaul": it captures the hero vignette, the two supporting
  vignettes, the framing principles the user explicitly called for, and a
  first-cut IA sketch for the planner to refine.

  Note on numbering: drafted originally as "Phase 27" during exploration;
  bumped to Phase 30 because v0.8.0 consumed phases 27–29 for JIRA Cloud.
status: ready-for-phase-30-planning
---

# Phase 30 narrative — hero vignette, framing, and IA seed

## Framing principles (non-negotiable for Phase 30)

Two principles came out of the explore conversation. They are more important
than any specific copy, diagram, or nav structure. Every downstream decision
in Phase 30 should be checked against them.

### P1. Complement, not replace

reposix **does not replace REST APIs**. REST stays. reposix handles the 80%
of tracker operations an agent does a hundred times a day — status changes,
comments, field edits, label adds, link creation — the ones that shouldn't
require learning an API surface.

The other 20% — complex JQL, bulk imports, admin operations, reporting
queries — keep using REST. reposix makes no claim there.

**Why this framing matters.** "REST is clunky" invites a fair skeptic
rebuttal: "our SDK is fine, we have tooling, this is a solved problem."
"Common operations, zero new vocabulary" bypasses that argument entirely.
It reframes the value prop from *replacing* the API to *absorbing the
ceremony* around the operations everyone does constantly. The API is still
there. reposix just means you don't have to touch it for the easy 80%.

Concrete tonal rule: **the word "replace" should not appear in hero or
value-prop copy.** Words that should: *complement, absorb, subsume, lift,
erase the ceremony, no new vocabulary.*

### P2. Progressive disclosure — phenomenology before implementation

The landing page describes what the user *experiences*. The architecture
page describes what reposix *is built from*. Never leak layer N into layer
N−1.

**Layer 1 — hero (first 10 seconds, above fold):**
What the user experiences. Issues are files. Edit them. `git push`.
No FUSE. No "daemon." No "remote helper." No "mount point."

**Layer 2 — just below the fold:**
The minimum mechanism required to make the experience make sense.
"reposix exposes your tracker as a live directory and translates commits
into API calls." Still no FUSE acronym. Just *directory, commits, API*.

**Layer 3 — concepts / how-it-works page:**
Here the technical reveal starts. "Under the hood, reposix is three pieces:
a FUSE daemon that projects the tracker as a real filesystem, a git
remote helper that turns pushes into API calls, and a sandboxed simulator
that lets you run the whole thing offline." Now we earn the right to show
diagrams.

**Layer 4 — reference / ADRs / research:**
Full technical depth. The FUSE inode model. The git-remote protocol. The
taint/audit design. The threat model. This is where the project's technical
beauty lives, and it should shine — but only *after* the reader has already
invested enough to want to see it.

**Why this matters.** The project's technical depth is a strength. Leading
with it ("a FUSE-based filesystem adapter for REST issue trackers, with a
git remote helper...") makes reposix sound like infrastructure plumbing
aimed at kernel developers. Leading with phenomenology ("edit issues as
files; git push to sync") invites anyone who edits text to lean in. The
plumbing becomes the payoff, not the pitch.

Concrete tonal rule: **above the fold, the only technical words permitted
are ones every developer already knows — file, folder, edit, commit, push,
merge, YAML, markdown.** "FUSE," "inode," "daemon," "helper," "kernel,"
"mount," and "syscall" are banned above layer 3.

---

## The pick: Vignette 1 as hero, 2 and 3 as supporting

Vignette 1 ("close a Jira ticket") is the hero. It's the only one with
visceral pain on the before-side — 30+ lines of curl/jq ceremony against
4 commands of file edits + git. That asymmetry is what makes the "aha" hit
in under 10 seconds without any explanation of how reposix works.

Vignettes 2 and 3 live below the fold as supporting evidence — one each for
the "but what about..." objections that sophisticated readers will have:

- V2 answers: *"what about concurrent writes, won't they clobber each other?"*
- V3 answers: *"what about fields the filesystem abstraction can't express?"*

Together, the three vignettes cover three distinct proof points:
1. **Ergonomics** (V1): the common case is trivial.
2. **Correctness** (V2): concurrency is debuggable, not magical.
3. **Expressiveness** (V3): the schema doesn't shrink to fit the abstraction.

---

## Hero vignette — "Close a Jira ticket"

The goal: 30 seconds of scrolling, one "oh" reaction, and the reader
already wants to know how.

### Before — REST from an agent

```bash
# Transition PROJ-42 to Done, reassign to alice, add a comment.

# 1. Look up the transition ID (Jira uses IDs, not names)
curl -s -u "$E:$T" \
  "https://acme.atlassian.net/rest/api/3/issue/PROJ-42/transitions" \
  | jq -r '.transitions[] | select(.name=="Done") | .id'
# => "31"

# 2. Transition it
curl -s -u "$E:$T" -X POST -H "Content-Type: application/json" \
  -d '{"transition":{"id":"31"}}' \
  "https://acme.atlassian.net/rest/api/3/issue/PROJ-42/transitions"

# 3. Find alice's accountId (usernames deprecated in Jira Cloud)
curl -s -u "$E:$T" \
  "https://acme.atlassian.net/rest/api/3/user/search?query=alice@acme.com" \
  | jq -r '.[0].accountId'
# => "5b10ac8d82e05b22cc7d4ef5"

# 4. Reassign
curl -s -u "$E:$T" -X PUT -H "Content-Type: application/json" \
  -d '{"fields":{"assignee":{"accountId":"5b10ac8d82e05b22cc7d4ef5"}}}' \
  "https://acme.atlassian.net/rest/api/3/issue/PROJ-42"

# 5. Comment — must be Atlassian Document Format, not plain text
curl -s -u "$E:$T" -X POST -H "Content-Type: application/json" \
  -d '{"body":{"type":"doc","version":1,"content":[{"type":"paragraph",
       "content":[{"type":"text","text":"Shipped in v0.7.1"}]}]}}' \
  "https://acme.atlassian.net/rest/api/3/issue/PROJ-42/comment"
```

Five round trips. Three ID formats. The ADF document schema for what should
be one line of text. No audit trail you can grep.

### After — the same change with reposix

```bash
cd ~/work/acme-jira        # a folder that is your Jira project

sed -i -e 's/^status: .*/status: Done/' \
       -e 's/^assignee: .*/assignee: alice@acme.com/' \
       issues/PROJ-42.md

cat >> issues/PROJ-42.md <<'EOF'

## Comment — 2026-04-17
Shipped in v0.7.1.
EOF

git commit -am "close PROJ-42" && git push
```

One commit. The audit trail is `git log`. No schemas to learn, no SDKs to
vendor, no tokens threaded through prompts.

### The complement line (mandatory directly under the "after")

> You still have full REST access for the operations that need it — JQL
> queries, bulk imports, admin config. reposix just means you don't have
> to reach for it for the hundred small edits you'd otherwise make every
> day.

This sentence is load-bearing. It makes the whole page honest. Without it,
a sophisticated reader will dismiss the pitch as naive.

---

## Supporting vignette #1 — "Two agents, one ticket"

Frame: *"what about concurrent writes?"*

### Before

Concurrent updates to the same Jira issue are an application-level problem.
Jira's ETags are inconsistent across field types; optimistic concurrency is
DIY. Most teams ship last-write-wins and quietly lose data when two agents
race. Writing correct multi-writer logic means a merge-layer in your agent
code that is always buggy and never unit-tested against production traffic.

### After

```bash
# Agent A
git pull
sed -i 's/^labels:.*/labels: [needs-review]/' issues/PROJ-42.md
git commit -am "add needs-review" && git push          # succeeds

# Agent B (simultaneous)
git pull
sed -i 's/^priority: medium/priority: high/' issues/PROJ-42.md
git commit -am "bump priority" && git push             # rejected: non-fast-forward
git pull --rebase    # A touched 'labels:', B touched 'priority:' — clean 3-way merge
git push             # both changes land
```

Git is the CRDT. Conflict resolution is `git merge`, debuggable with twenty
years of familiar tooling and visible in `git log`. True conflicts — both
agents edited `status:` — surface as human-readable merge conflicts on a
text file, not as silent data loss.

---

## Supporting vignette #2 — "Everything is frontmatter"

Frame: *"can this really express all the fields real trackers have?"*

### Before

A non-exhaustive map of Jira endpoints you learn to use common field
operations:

```
PUT /issue/{id}                         (most fields)
POST /issue/{id}/transitions            (status — NOT a regular field)
POST /issue/{id}/comment                (comments — ADF body)
POST /issue/{id}/attachments            (multipart)
POST /issue/{id}/worklog                (time — separate endpoint)
POST /issueLink                         (links — top-level)
GET  /user/search?query=...             (accountId lookup)
customfield_10034                       (custom field naming)
```

Eight endpoints. Three body formats. One translation table for custom
fields. One mental model per endpoint.

### After

```yaml
---
id: PROJ-42
title: Add user avatar upload
status: In Progress
assignee: alice@acme.com
priority: high
labels: [backend, needs-review]
story_points: 5
sprint: 2026-Q2-W3
links:
  - blocks: PROJ-55
  - related: PROJ-40
custom_fields:
  customer_impact: medium
---

## Description
...

## Comment — 2026-04-15 (bob)
...

## Worklog
- 2026-04-15 2h alice — initial spike
```

One file. One editor. Every field is a line of YAML. Custom fields are
just more YAML keys. Comments and worklogs are just markdown sections.
The schema is readable by a stranger in 30 seconds.

---

## Persona fit

| Vignette | Agent builder | Platform engineer (skeptic) | Tech-literate PM |
|---|---|---|---|
| V1 — ergonomics | Immediate — "no more MCP schema work" | "Audit log is just git history — I can ship this" | Pain is visceral even without code fluency |
| V2 — concurrency | "Git is the CRDT?!" — memorable | "Conflict resolution is debuggable" — buys trust | Needs distributed-systems literacy; skip below fold |
| V3 — expressiveness | "I can read this and know the schema" | "Escape hatches for custom fields exist" | Quietly reassuring |

Hero V1 is the only one that lands on all three personas at first glance.
V2 and V3 earn their keep for the skeptic who scrolls.

---

## IA sketch (seed for Phase 30 planner, not the final nav)

### Diátaxis mapping of current docs
- **Tutorial** (missing/weak): demo.md is closest but reads as a checklist.
- **How-to** (scattered): connectors/guide.md, reference/confluence.md mix how-to with reference.
- **Explanation** (strong): why.md, architecture.md, research/*.
- **Reference** (strong): cli.md, http-api.md, git-remote.md, ADRs.

Phase 30 should not rewrite reference or explanation. They're correct. The
gap is a real tutorial and a real landing page.

### Proposed nav (high-level, Phase 30 to refine)

```
Home                                  [hero vignette + three-up value props]
  ├─ Why reposix                      [the problem, framed as 80/20 with REST]
  ├─ Mental model in 60 seconds       [three conceptual keys — NEW]
  ├─ reposix vs MCP / SDKs            [comparison for skeptics — NEW]
  └─ Try it in 5 minutes              [real tutorial against the simulator]

How it works                          [Layer 2 reveal — "mounts a directory,
                                       translates commits"]
  ├─ The filesystem layer             [Layer 3 — FUSE reveal, first diagram]
  ├─ The git layer                    [Layer 3 — remote helper reveal + diagram]
  └─ The trust model                  [Layer 3 — NEW: taint, outbound allowlist,
                                       append-only audit, lethal-trifecta
                                       mitigations + diagram]

Guides (how-to)
  ├─ Connect to GitHub
  ├─ Connect to Jira
  ├─ Connect to Confluence
  ├─ Write your own connector         [NEW — BackendConnector walkthrough]
  ├─ Integrate with your agent        [NEW — Claude Code / Cursor / custom SDK]
  ├─ Running two agents safely
  ├─ Custom fields and frontmatter
  └─ Troubleshooting                  [NEW — stub that grows post-launch]

Reference
  ├─ CLI
  ├─ HTTP API
  ├─ The simulator                    [MOVED from How it works — dev tooling]
  ├─ git-remote-reposix
  └─ Frontmatter schema

Decisions (ADRs)
  ├─ 001 — GitHub state mapping
  ├─ 002 — Confluence page mapping
  ├─ 003 — Nested mount layout
  ├─ 004 — BackendConnector trait rename
  └─ 005 — JIRA issue mapping

Research
  ├─ Initial report (pre-v0.1 design argument)
  └─ Agentic engineering reference (dark-factory pattern)
```

Key structural moves:
- **"Home"** is new and narrative-led. Current docs/index.md is an overview
  table of contents, which is a reference-style opener. It goes away.
- **"Mental model in 60 seconds"** (NEW) lives under Home. Three conceptual
  keys: *mount = git working tree · frontmatter = schema · `git push` =
  sync verb.* Highest-ROI page on the site — short, readable in one sitting,
  cements the mental model before anyone opens the architecture section.
- **"reposix vs MCP / SDKs"** (NEW) lives under Home. Grounds positioning
  for skeptics; supports P1 (complement, not replace) with a concrete
  comparison table. Short — not marketing copy, just honest framing.
- **"How it works"** is a new section that handles the Layer 2 → Layer 3
  reveal. Current architecture.md is one long page; split it into three
  focused pages so each technical reveal lands with its own diagram.
- **"The trust model"** (NEW) replaces "The simulator" in How it works.
  Reposix is a textbook lethal-trifecta scenario (private data + untrusted
  input + egress). Current security.md enumerates shipped items; this slot
  tells the story — taint typing, outbound allowlist, append-only audit,
  bulk-delete cap — as a differentiator, not a checklist.
- **The simulator moves to Reference.** It's dev tooling, not core
  architecture. Practical lookup material, not narrative material.
- **"Guides"** gets promoted to a top-level section and gains three new
  pages: "Write your own connector" (BackendConnector extensibility —
  what makes reposix a substrate, not three integrations), "Integrate
  with your agent" (the project's raison d'être per PROJECT.md core
  value — prompt patterns, token-savings, Claude Code / Cursor / SDK
  integration), and "Troubleshooting" (stub that grows post-launch).
- **Reference, Decisions, Research** stay roughly as-is — Phase 26 already
  made those correct.

---

## Phase 30 scope (seed, not the plan)

### In scope
- **Hero rewrite** — landing page, above-fold copy, one before/after code
  block (V1), three-up value props.
- **"How it works"** — three new pages (filesystem layer, git layer, trust
  model), each with one mcp-mermaid diagram (playwright-screenshot verified).
  Content carved from `docs/architecture.md` + `docs/security.md`.
- **Home-adjacent pages** — "Mental model in 60 seconds" (three conceptual
  keys); "reposix vs MCP / SDKs" (comparison grounding P1).
- **New Guides** — "Write your own connector" (BackendConnector
  walkthrough); "Integrate with your agent" (Claude Code / Cursor / SDK
  patterns); "Troubleshooting" (stub that grows post-launch).
- **Simulator page relocated** — from How it works to Reference.
- **Tutorial** — first-run experience against the simulator (5-minute path
  from install to "I edited a thing and pushed it").
- **Nav restructure** — `mkdocs.yml` changes to implement the IA sketch above.
- **mkdocs-material theme tuning** — palette, hero features, social cards.
- **Progressive-disclosure enforcement** — a linter or checklist ensuring
  the banned technical terms don't appear above their assigned layer.

### Out of scope
- New features, new CLI surface, new backend connectors.
- Any change to REQUIREMENTS.md or the roadmap beyond the phase itself.
- Changing the reference/ or decisions/ trees. Phase 26 already made those
  correct.

### Suggested subagent fan-out (for Phase 30 planner to finalize)
- **Explore (competitor narrative scan)** — Linear, Turso, Fly.io, Tailscale,
  Warp, Val Town, Raycast, Stripe docs. Extract one pattern per site that
  fits our hero vignette style.
- **Copy agent** — hero + three value props, constrained by the banned-word
  list from P2.
- **IA agent** — two competing nav structures against the sketch above,
  scored against Diátaxis + the three personas.
- **Diagram agent (mcp-mermaid)** — three architecture diagrams (filesystem
  layer, git layer, trust model), rendered and playwright-screenshotted for
  visual review before merge.
- **Tutorial agent** — authors the 5-minute getting-started path, actually
  runs it end-to-end against the simulator, screenshots each step.

### Verification (feedback loop, per user OP #1)
- `mkdocs build --strict` remains green.
- `playwright` screenshots of: landing page (desktop + mobile width), "how
  it works" pages with diagrams rendered, tutorial walkthrough.
- `gh run view` on CI after push — must be green.
- Banned-word linter runs on every doc commit.

---

## Creative license notes (user asked for "careful and creative")

A few creative devices worth trying in the copy phase. Not prescriptive —
the copy agent should feel free to try alternatives.

1. **Animated before/after** on the hero. Two terminal panes, V1 left vs
   right, typed out in real time. (Library: `asciinema` or CSS-only.)
2. **"Watch a ticket close"** framing — the hero vignette runs as a
   visible story, not a code dump. Cursor, commands, output, git push.
3. **The "no new vocabulary" line** deserves its own repetition somewhere.
   It's the sharpest way to state P1.
4. **The technical reveal ("how it works") should feel like opening a
   watch.** The reader already knows it works; now they get to see the
   mechanism. Diagrams should be elegant, not comprehensive.
5. **Banned on the landing page:** stock photos, "empower," "revolutionize,"
   "next-generation," feature-grid tables, marketing bullet points with
   check-mark icons. This project's voice is precise, dry, and earned.

---

## Handoff

This note is the single source of truth for Phase 30's narrative intent.
The Phase 30 planner should consume it as context, then produce a
task-level PLAN.md that:
1. Honors both framing principles (P1, P2) in every task it creates.
2. Uses the hero vignette and IA sketch as starting points, not final copy.
3. Fans out to subagents for copy, IA, diagrams, tutorial as outlined above.
4. Ships with playwright screenshot verification before declaring done.

Next step from here: `/gsd-add-phase` to add Phase 30 "Docs IA + narrative
overhaul" to the roadmap with this note as its CONTEXT.md input.
