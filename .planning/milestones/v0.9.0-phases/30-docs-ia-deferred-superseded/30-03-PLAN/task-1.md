← [back to index](./index.md)

# Task 1: Rewrite docs/index.md with V1 hero + complement line + three-up + where-to-go-next

<task type="auto">
  <name>Task 1: Rewrite docs/index.md with V1 hero + complement line + three-up + where-to-go-next</name>
  <files>docs/index.md</files>
  <read_first>
    - `docs/index.md` (current — 84 lines; WILL BE REPLACED wholesale)
    - `.planning/notes/phase-30-narrative-vignettes.md` lines 109-181 (the EXACT source of the V1 hero — copy the bash blocks verbatim; copy the complement-line blockquote verbatim)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §docs/index.md (grid-cards syntax pattern from old index.md lines 44-62)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md` §"Menu-style recap — which 3 to steal" (confirms competitor choices B+D+G)
    - `docs/why.md` (for the token-economy link target `#token-economy-benchmark`)
  </read_first>
  <action>
    Replace the ENTIRE contents of `docs/index.md` with the following markdown. This is the final hero — not a stub. Do NOT add "Stub" markers; this file ships as-is.

```markdown
# reposix

> Close a ticket with `sed` and `git push`. Keep your REST API for the 20% that needs it.

reposix makes tracker operations — status changes, field edits, labels, comments, links — feel like editing text files in a git working tree. The ceremony of learning an API goes away; your agent works with primitives (`cat`, `sed`, `grep`, `git`) it already knows from pre-training.

## Before — REST from an agent

\`\`\`bash
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
\`\`\`

Five round trips. Three ID formats. The ADF document schema for what should be one line of text. No audit trail you can grep.

## After — the same change with reposix

\`\`\`bash
cd ~/work/acme-jira        # a folder that is your Jira project

sed -i -e 's/^status: .*/status: Done/' \
       -e 's/^assignee: .*/assignee: alice@acme.com/' \
       issues/PROJ-42.md

cat >> issues/PROJ-42.md <<'EOF'

## Comment — 2026-04-17
Shipped in v0.7.1.
EOF

git commit -am "close PROJ-42" && git push
\`\`\`

One commit. The audit trail is `git log`. No schemas to learn, no SDKs to vendor, no tokens threaded through prompts.

> You still have full REST access for the operations that need it — JQL
> queries, bulk imports, admin config. reposix just means you don't have
> to reach for it for the hundred small edits you'd otherwise make every
> day.

## What reposix gives you

<div class="grid cards" markdown>

-   :material-file-document-edit: **Common operations, no new vocabulary**

    Edit YAML frontmatter for fields, write markdown for comments, `git commit` to record intent, `git push` to sync. Every verb is one your agent already knows.

-   :material-source-branch: **`git log` is the audit trail**

    Every field change is a commit. Every conflict is a text-file merge. Two agents racing on the same ticket get ordinary merge markers, not a silent 409.

-   :material-shield-check: **Safe by default**

    Outbound HTTP is allowlisted. The bulk-delete cap fires before an agent can erase your backlog. The audit ledger is append-only SQLite. The simulator lets you test offline with zero credentials.

</div>

## Where to go next

<div class="grid cards" markdown>

-   :material-play-circle: **[Try it in 5 minutes](tutorial.md)**

    Edit a simulated ticket with `sed`, `git push`, and watch the version bump from 1 to 2. Runs offline against the bundled simulator.

-   :material-lightbulb: **[Mental model in 60 seconds](mental-model.md)**

    Three ideas you already know: git working tree, YAML frontmatter, `git push` as a sync verb.

-   :material-graph: **[How it works](how-it-works/index.md)**

    The filesystem layer, the git layer, and the trust model — one diagram per page.

-   :material-scale-balance: **[reposix vs MCP and REST SDKs](vs-mcp-sdks.md)**

    Where each tool earns its keep. Honest comparison with measured token numbers.

</div>

!!! success "v0.8 — eight autonomous overnight sessions, 2026-04-13 to 2026-04-16"
    Every line of code in this repository was written by a coding agent across eight overnight sessions: v0.1 (simulator + guardrails), v0.2 (GitHub read-only), v0.3 (Confluence Cloud), v0.4 (nested tree layout), v0.5 (`_INDEX.md` sitemaps), v0.6 (Confluence write path + labels), v0.7 (hardening + benchmarks + docs reorg), v0.8 (JIRA Cloud integration). A red-team subagent critiques the design; a planner verifies each phase. See [Why reposix](why.md) for the token economics that started all this.
```

After writing, run Vale on the file:

```bash
~/.local/bin/vale --config=.vale.ini docs/index.md
```

Expected: exit 0. The word "replace" has zero occurrences (P1). The terms "FUSE / inode / daemon / helper / kernel / mount / syscall" appear only inside fenced code blocks where Vale's `IgnoredScopes = code, code_block` exempts them; in prose, only phenomenology words appear.

Also run `mkdocs build --strict` from the repo root. Expected: may still fail on nav (plan 30-04 not yet run), but should NOT fail due to broken internal links from `docs/index.md` (every link in the Where-to-go-next grid points at a file that exists from plan 30-02).
  </action>
  <verify>
    <automated>~/.local/bin/vale --config=.vale.ini docs/index.md && grep -cE '\breplace\w*\b' docs/index.md | grep -q '^0$' && grep -c 'git commit -am "close PROJ-42"' docs/index.md | grep -q '^1$' && grep -c 'You still have full REST access' docs/index.md | grep -q '^1$'</automated>
  </verify>
  <acceptance_criteria>
    - `grep -cE '\breplace\w*\b' docs/index.md` returns `0` (P1 ban).
    - `grep -cE '\b(FUSE|inode|daemon|kernel|syscall)\b' <(python3 -c "import re; print(re.sub(r'\`\`\`.*?\`\`\`', '', open('docs/index.md').read(), flags=re.DOTALL))")` returns `0` (P2 ban, code-fence-excluded; verify via the same strip-code-fences logic used by `scripts/check_phase_30_structure.py`).
    - `grep -c 'git commit -am "close PROJ-42"' docs/index.md` returns `1` (after-block V1 signature present).
    - `grep -c 'You still have full REST access' docs/index.md` returns `1` (complement line present).
    - `grep -c 'no new vocabulary' docs/index.md` returns `>= 1` (P1 positive marker).
    - `grep -cE '(tutorial\.md|mental-model\.md|how-it-works|vs-mcp-sdks\.md)' docs/index.md` returns `>= 4` (Where-to-go-next grid points at all four next-step pages).
    - `grep -cE '(empower|revolutioniz|next-generation)' docs/index.md` returns `0` (creative-bans enforced per source-of-truth note §"Creative license notes").
    - `grep -c '!!! success' docs/index.md` returns `1` (version/autonomy callout preserved as admonition, not feature-grid table).
    - `~/.local/bin/vale --config=.vale.ini docs/index.md` exits 0.
  </acceptance_criteria>
  <done>
    docs/index.md rewritten as a narrative Layer-1 hero. The V1 before/after hero, the complement line, the three-up value props, and the where-to-go-next grid are all present. Vale clean. Creative-bans enforced. Ready for doc-clarity-review in plan 30-09.
  </done>
</task>
