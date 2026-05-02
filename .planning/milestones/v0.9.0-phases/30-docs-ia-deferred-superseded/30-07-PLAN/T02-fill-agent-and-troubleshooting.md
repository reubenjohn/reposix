← [back to index](./index.md)

# T2 — Fill docs/guides/integrate-with-your-agent.md + docs/guides/troubleshooting.md

<task type="auto">
  <name>Task 2: Fill docs/guides/integrate-with-your-agent.md + docs/guides/troubleshooting.md</name>
  <files>docs/guides/integrate-with-your-agent.md, docs/guides/troubleshooting.md</files>
  <read_first>
    - `docs/guides/integrate-with-your-agent.md` (current skeleton from plan 30-02)
    - `docs/guides/troubleshooting.md` (current skeleton from plan 30-02)
    - `docs/why.md` §token-economy-benchmark (target link for agent-integration)
    - `docs/demo.md` §"Limitations / honest scope" (voice reference for troubleshooting)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §docs/guides/integrate-with-your-agent.md, §docs/guides/troubleshooting.md
    - `crates/reposix-sim/src/audit.rs` (for the audit-log SELECT query shape in troubleshooting)
  </read_first>
  <action>
    Replace the ENTIRE contents of `docs/guides/integrate-with-your-agent.md` with:

```markdown
# Integrate reposix with your agent

reposix's core value lands when an autonomous LLM agent treats your tracker as a working directory. The agent uses primitives it already knows from pre-training — `cat`, `sed`, `grep`, `git` — instead of learning your REST API schema at inference time. This page shows how to wire reposix into the agents you actually use.

For the measured token-savings numbers that motivate this, see [Why reposix](../why.md#token-economy-benchmark).

## With Claude Code

Claude Code respects a project-level `CLAUDE.md`. To make the tracker folder discoverable to every session, add an entry like:

\`\`\`markdown
## Tracker working directory

The project's Jira tickets live at `/tmp/acme-jira`. Treat it as a git working tree:

- `ls issues/` to list; `cat issues/PROJ-*.md` to read.
- Edit YAML frontmatter (status, assignee, labels) as text.
- Edit markdown body for comments and descriptions.
- `git add`, `git commit`, `git push` to sync.

Never call the Jira REST API from a prompt — the agent has `git` and `sed`, which is all it needs for the common 80% of operations.
\`\`\`

The agent now has the tracker in its working set without any tool schemas loaded.

## With Cursor

Cursor reads `.cursorrules` at the project root. Same pattern as Claude Code:

\`\`\`
# .cursorrules
The project's tracker state lives at /tmp/acme-jira. It is a git working tree.
Read tickets with `cat`, edit with `sed` or direct file writes, commit, and push.
Do not call the Jira REST API directly; reposix handles that.
\`\`\`

## With a custom SDK

If you are driving the agent programmatically, the integration is ~20 lines of Python or TypeScript:

\`\`\`python
import subprocess

TRACKER_DIR = "/tmp/acme-jira"

def close_ticket(ticket_id: str, comment: str) -> None:
    """Close a ticket by editing the file and pushing the commit."""
    path = f"{TRACKER_DIR}/issues/{ticket_id}.md"

    # 1. Edit YAML frontmatter
    subprocess.run(
        ["sed", "-i", "-e", "s/^status: .*/status: Done/", path],
        check=True,
    )

    # 2. Append a comment
    with open(path, "a") as f:
        f.write(f"\n## Comment\n{comment}\n")

    # 3. Commit + push
    subprocess.run(["git", "-C", TRACKER_DIR, "commit", "-am", f"close {ticket_id}"], check=True)
    subprocess.run(["git", "-C", TRACKER_DIR, "push"], check=True)
\`\`\`

No API client library. No schema. The workflow is explainable to a non-programmer in two sentences.

## Gotchas

**Taint boundary.** Everything the tracker sends back is attacker-influenced text. If your agent loop echoes ticket bodies or titles into another system (a prompt, a shell, a `git push` to a different remote), that is a lethal-trifecta exfiltration path. reposix marks these bytes as tainted at the type level, but your agent code is the last line of defense. See [The trust model](../how-it-works/trust-model.md).

**`REPOSIX_ALLOWED_ORIGINS` is required for real backends.** The simulator works without it (localhost is always allowed). When connecting to a real GitHub / Jira / Confluence tenant, export:

\`\`\`bash
export REPOSIX_ALLOWED_ORIGINS='http://127.0.0.1:*,https://api.github.com'
\`\`\`

Missing or empty: every outbound HTTP call fails closed. Set deliberately — never `https://*`.

**Bulk-delete cap (SG-02).** A `git push` whose commit removes more than 5 issues is rejected. If the agent reached that state unintentionally, that is a feature — resolve before overriding. If intentional, add `[allow-bulk-delete]` to the commit message.

**Two agents, one ticket.** Run both against the same `reposix mount` (or against separate working trees that `git pull` between edits). Conflicts surface as ordinary text-file merge markers. See [The git layer](../how-it-works/git.md).
```

Replace the ENTIRE contents of `docs/guides/troubleshooting.md` with:

```markdown
# Troubleshooting

Common failure patterns and fixes. This page grows post-launch as more issues are captured in the field.

## Symptom — the folder is empty after connecting

You ran `reposix mount /tmp/foo --backend ...` but `ls /tmp/foo/issues/` returns nothing.

**Likely cause:** `fuse3` is not installed, so the connect command silently exits before exposing the folder.

**Fix:**

\`\`\`bash
# Ubuntu / Debian
sudo apt install fuse3

# Verify
which fusermount3
\`\`\`

If `fusermount3` is missing, reposix's connect path cannot complete. After install, retry the `reposix mount` command.

## Symptom — `git push` is rejected with `bulk-delete`

You `git commit`ed a diff that removes >5 files, and `git push` refuses with an error mentioning SG-02.

**Likely cause:** The bulk-delete cap fired (guardrail). This is intentional protection against runaway agent loops erasing your backlog.

**Fix:**

If the deletion is accidental, `git reset --hard HEAD~1` and re-commit with a narrower scope.

If the deletion is intentional, override by appending `[allow-bulk-delete]` to the commit message:

\`\`\`bash
git commit --amend -m "$(git log -1 --format=%B)
[allow-bulk-delete]"
git push
\`\`\`

The override is visible in `git log` and in the audit row.

## Symptom — I want to audit what an agent did

You need to see every HTTP call reposix dispatched on your behalf.

**Likely cause:** you are looking for the audit log. reposix writes one row per API call to an append-only SQLite table.

**Fix:**

\`\`\`bash
# For the simulator, default db path:
sqlite3 /tmp/tutorial-sim.db \
    "SELECT ts, method, path, status FROM audit ORDER BY ts DESC LIMIT 20"

# For a real backend, the default db path is:
sqlite3 ~/.local/share/reposix/audit.db \
    "SELECT ts, method, path, status FROM audit ORDER BY ts DESC LIMIT 20"
\`\`\`

Every `PATCH`, `POST`, `DELETE`, and `GET` is there — committed before the HTTP call returns (append-only per SG-06). Correlate against `git log` to see which agent action produced which API call.
```

Run Vale on both pages:

```bash
~/.local/bin/vale --config=.vale.ini docs/guides/integrate-with-your-agent.md docs/guides/troubleshooting.md
```

Expected: exit 0. Both are Layer-2 How-to pages under `docs/guides/` — ProgressiveDisclosure rule is active, no banned terms in prose.

Word "mount" appears in both pages but only inside code fences (`reposix mount ...`) — Vale's IgnoredScopes exempts code. If Vale flags anything, rephrase prose to replace "mount" with "connect" or "attach" or "expose".
  </action>
  <verify>
    <automated>~/.local/bin/vale --config=.vale.ini docs/guides/integrate-with-your-agent.md docs/guides/troubleshooting.md && grep -c 'Claude Code\|Cursor\|Custom SDK\|\.cursorrules' docs/guides/integrate-with-your-agent.md | awk '{exit !($1 >= 3)}' && grep -c 'REPOSIX_ALLOWED_ORIGINS' docs/guides/integrate-with-your-agent.md | awk '{exit !($1 >= 1)}' && grep -c 'why.md#token-economy-benchmark' docs/guides/integrate-with-your-agent.md | grep -q '^1$' && grep -c '^## Symptom' docs/guides/troubleshooting.md | awk '{exit !($1 == 3)}' && grep -c 'sqlite3' docs/guides/troubleshooting.md | awk '{exit !($1 >= 1)}'</automated>
  </verify>
  <acceptance_criteria>
    - integrate-with-your-agent.md contains sections `## With Claude Code`, `## With Cursor`, `## With a custom SDK`, `## Gotchas`.
    - `grep -c '.cursorrules' docs/guides/integrate-with-your-agent.md` returns `>= 1`.
    - `grep -c 'REPOSIX_ALLOWED_ORIGINS' docs/guides/integrate-with-your-agent.md` returns `>= 1`.
    - `grep -c 'why.md#token-economy-benchmark' docs/guides/integrate-with-your-agent.md` returns `1` (cites, does not duplicate).
    - `grep -c 'subprocess.run' docs/guides/integrate-with-your-agent.md` returns `>= 1` (Python SDK example present).
    - `grep -c 'Taint boundary' docs/guides/integrate-with-your-agent.md` returns `1` (gotcha present).
    - troubleshooting.md contains exactly 3 `^## Symptom` sections.
    - `grep -c '^\*\*Fix:\*\*' docs/guides/troubleshooting.md` returns `3` (one fix per symptom).
    - `grep -c 'sqlite3' docs/guides/troubleshooting.md` returns `>= 1` (audit-log query).
    - `grep -c '\[allow-bulk-delete\]' docs/guides/troubleshooting.md` returns `>= 1` (SG-02 override documented).
    - `grep -c 'apt install fuse3' docs/guides/troubleshooting.md` returns `>= 1` (fuse3 install command).
    - Vale passes on both files.
  </acceptance_criteria>
  <done>
    integrate-with-your-agent.md and troubleshooting.md filled with real content — no more stub markers. Both Vale-clean.
  </done>
</task>
