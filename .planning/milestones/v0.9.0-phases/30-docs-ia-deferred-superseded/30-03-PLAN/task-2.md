← [back to index](./index.md)

# Task 2: Fill docs/mental-model.md + docs/vs-mcp-sdks.md

<task type="auto">
  <name>Task 2: Fill docs/mental-model.md with 300-400 words + docs/vs-mcp-sdks.md with comparison table + P1 paragraph</name>
  <files>docs/mental-model.md, docs/vs-mcp-sdks.md</files>
  <read_first>
    - `docs/mental-model.md` (current skeleton from plan 30-02; three H2s are LOCKED — do not change them)
    - `docs/vs-mcp-sdks.md` (current skeleton from plan 30-02)
    - `.planning/notes/phase-30-narrative-vignettes.md` §"IA sketch" (lines 350-355 for mental-model framing)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md` §"Mental Model in 60 seconds — format guidance" (lines 747-763) and §"Competitor Narrative Scan → Pattern F (Turso)" (length precedent)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §docs/mental-model.md, §docs/vs-mcp-sdks.md (analog voices from docs/why.md)
    - `docs/why.md` §"token-economy-benchmark" (~line 55 — the 4,883 vs 531 token numbers)
  </read_first>
  <action>
    Replace the ENTIRE contents of `docs/mental-model.md` with (~350 words — the three locked H2s are preserved verbatim; the prose + snippets are new):

```markdown
# Mental model in 60 seconds

reposix is three ideas you already know, snapped together. Read the three sections below in order, then either [try it](tutorial.md) or [open the mechanism](how-it-works/index.md).

## mount = git working tree

The folder reposix gives you is a git working tree. Each tracker item — a Jira issue, a Confluence page, a GitHub issue — is a file. `ls` lists them. `git diff` shows pending edits. `git log` is the audit trail you already know how to query.

\`\`\`bash
cd ~/work/acme-jira
ls issues/
# PROJ-42.md  PROJ-43.md  PROJ-44.md ...
git log --oneline issues/PROJ-42.md
# 3a1f9b2 close PROJ-42
# 2c0d4e8 bump priority to high
\`\`\`

The folder is not a view over a database. It is the source of truth of what you intend — `git push` is what makes intent real.

## frontmatter = schema

Every tracker has a schema — issue types, custom fields, labels, statuses, assignees, story points, sprints. reposix puts the whole schema in the YAML frontmatter at the top of each file. Changing a field is changing one line of text.

\`\`\`yaml
---
id: PROJ-42
status: In Progress
assignee: alice@acme.com
labels: [backend, needs-review]
story_points: 5
custom_fields:
  customer_impact: medium
---
\`\`\`

Custom fields are just more YAML keys. Comments and worklogs are markdown sections below the frontmatter. A stranger reads the schema in 30 seconds — no endpoint table, no ADF body format, no `customfield_10034` lookup.

## `git push` = sync verb

`git push` is the one verb that syncs local edits with the remote tracker. Under the hood it is not magic: reposix computes the diff between your working tree and the upstream state, dispatches `PATCH` / `POST` / `DELETE` calls against the tracker's REST API, and writes every dispatch to an append-only audit table.

\`\`\`bash
git commit -am "close PROJ-42"
git push
# → syncs frontmatter diff + markdown body edits to the tracker
# → writes one audit row per API call (grep-able in sqlite)
\`\`\`

Optimistic concurrency — two agents racing on the same ticket — surfaces as an ordinary text-file merge conflict, not a silent 409. See [how it works / the git layer](how-it-works/git.md) for the round-trip.

---

> Now what: [try it in 5 minutes](tutorial.md) or [open the mechanism](how-it-works/index.md).
```

Replace the ENTIRE contents of `docs/vs-mcp-sdks.md` with:

```markdown
# reposix vs MCP and REST SDKs

reposix complements MCP servers and REST SDKs — it does not stand in for them. The claim is narrower and more honest: reposix absorbs the ceremony around the 80% of tracker operations an agent does a hundred times a day. Your MCP server and your SDK keep earning their keep for the other 20%.

## When to reach for which

| Scenario | reposix | MCP server | REST SDK |
|----------|:-------:|:---:|:--------:|
| Edit a ticket's status, assignee, labels, or comments | ✓ | | |
| Bulk-import 10 000 tickets from a spreadsheet | | | ✓ |
| Run arbitrary JQL / CQL / GraphQL query | | ✓ | ✓ |
| Subscribe to tracker webhooks | | ✓ | ✓ |
| Let an agent review + comment across 50 tickets in one loop | ✓ | | |
| Export an audit report for compliance | ✓ (git log) | ✓ | ✓ |
| First-time project / space / board setup | | ✓ | ✓ |

reposix wins wherever the operation is (a) a small frontmatter edit, (b) a markdown comment, or (c) a state transition expressible as a diff. Everything else stays on REST.

## Why this framing matters

"REST is clunky" invites a fair rebuttal: the SDK is fine, the team has tooling, this is a solved problem. "Common operations, no new vocabulary" sidesteps that argument entirely. It reframes the value from subsuming the API to absorbing the ceremony around the operations everyone does constantly. The API is still there. reposix just means you do not have to touch it for the easy 80%.

## Measured numbers

For the tracker-edit workload measured in `docs/why.md`:

| Scenario | Real tokens (count_tokens) |
|----------|---------------------------:|
| MCP-mediated (tool catalog + schemas) | ~4,883 |
| **reposix** (shell session transcript) | **~531** |

~92.3% reduction on the same end state.[^1] The ratio holds because reposix replaces the schema tokens (which every turn loads) with shell tokens (which the model already knows). The ratio is not universal — it collapses on workloads that are mostly JQL or bulk ops, where MCP or the REST SDK is the right tool.

[^1]: Methodology and caveats: [Why reposix § token-economy-benchmark](why.md#token-economy-benchmark).
```

Run Vale:

```bash
~/.local/bin/vale --config=.vale.ini docs/mental-model.md docs/vs-mcp-sdks.md
```

Expected: exit 0. mental-model.md uses "mount" in its H2 — exempted by `.vale.ini`'s `[docs/mental-model.md]` rule. vs-mcp-sdks.md is Layer-2 (ProgressiveDisclosure active + NoReplace not scoped there) — zero banned Layer-3 terms in prose, zero "replace".

Also run:

```bash
wc -w docs/mental-model.md
```

Expected: 280-420 words (per Turso /concepts precedent).
  </action>
  <verify>
    <automated>~/.local/bin/vale --config=.vale.ini docs/mental-model.md docs/vs-mcp-sdks.md && grep -c '^## mount = git working tree$' docs/mental-model.md | grep -q '^1$' && grep -c '^## frontmatter = schema$' docs/mental-model.md | grep -q '^1$' && grep -Pc '^## `git push` = sync verb$' docs/mental-model.md | grep -q '^1$' && grep -cEi '(complement|absorb|subsume)' docs/vs-mcp-sdks.md | awk '{exit !($1 >= 1)}' && grep -c '#token-economy-benchmark' docs/vs-mcp-sdks.md | grep -q '^1$'</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c '^## ' docs/mental-model.md` returns exactly `3` (H2 count).
    - Three locked H2s present verbatim: `mount = git working tree`, `frontmatter = schema`, `` `git push` = sync verb ``.
    - `wc -w docs/mental-model.md` returns between 280 and 420.
    - `grep -c '^\`\`\`' docs/mental-model.md` returns `>= 6` (3 opening + 3 closing code fences, one per H2 section).
    - `grep -c 'Now what' docs/mental-model.md` returns `>= 1` (closing pointer present).
    - `grep -cEi '(complement|absorb|subsume)' docs/vs-mcp-sdks.md` returns `>= 1` (P1 positive marker).
    - `grep -c '#token-economy-benchmark' docs/vs-mcp-sdks.md` returns `1` (citation target link).
    - `grep -c '^\[\^1\]:' docs/vs-mcp-sdks.md` returns `1` (footnote definition).
    - `grep -cE '\breplace\w*\b' docs/vs-mcp-sdks.md` returns `0` (P1 ban).
    - `grep -cE '\bFUSE\b|\bdaemon\b|\bkernel\b|\bsyscall\b' docs/vs-mcp-sdks.md` returns `0` (P2 ban in Layer-2 page).
    - `~/.local/bin/vale --config=.vale.ini docs/mental-model.md docs/vs-mcp-sdks.md` exits 0.
  </acceptance_criteria>
  <done>
    Mental-model page is 300-400 words with three locked H2s, one code snippet per section, and a closing "Now what" pointer. vs-mcp-sdks page has comparison table, P1-grounded paragraph with complement/absorb, and a footnote citing the token-economy benchmark. Vale clean. Wave 4 doc-clarity-review can validate comprehension in <60s.
  </done>
</task>
