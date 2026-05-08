# 04 — CLI verbs and workflow

> **Read first.** [`03-schemas.md`](./03-schemas.md) for what the verbs read/write.
> **Read next.** [`05-edge-cases.md`](./05-edge-cases.md) for failure-mode recovery.

## Verb set (mirrors `reposix-quality doc-alignment`)

The CLI surface intentionally mirrors `reposix-quality doc-alignment {bind, walk, status, plan-refresh, ...}`. Same mental model = lower learning cost for agents already trained on docs-alignment.

### Discovery and inventory

| Verb | Reads | Writes | Purpose |
|---|---|---|---|
| `cross-link suggest-scopes` | repo files | stdout (suggested config) | Brownfield onboarding aid; emits a starting `.cross-link-fidelity` |
| `cross-link walk` | config + repo files | tracker | The core operation. Discovers edges, classifies state. Pure (no L3). |
| `cross-link status` | tracker | stdout | Human-readable summary: counts per state per scope, coverage, floor. |
| `cross-link list <state>` | tracker | stdout | List edges in a state. `cross-link list STALE`, `cross-link list BROKEN`. |
| `cross-link show <edge-id>` | tracker + frontmatter | stdout | Full inventory entry for one edge, with grading context applied. |

### Grading and refresh

| Verb | Reads | Writes | Purpose |
|---|---|---|---|
| `cross-link plan-refresh [--batch N] [--target STATE] [--age-min Xd]` | tracker | tracker | Selects edges needing L3, dispatches subagents, writes verdicts. |
| `cross-link bootstrap [--all] [--target-coverage 0.5]` | config + repo | tracker | Greenfield/brownfield bootstrap. `--all` does everything; `--target-coverage` stops when reached. |
| `cross-link grade <edge-id>` | tracker + grading-context | tracker | One-edge regrade. Used by humans/agents responding to a STALE BLOCK. |

### Recovery and repair

| Verb | Purpose |
|---|---|
| `cross-link find-similar-heading <path> <broken-anchor>` | Suggests heading slugs in `<path>` close to the broken anchor. Helper for renamed-heading recovery. |
| `cross-link rebind <edge-id> --target <new-path>[#<new-anchor>]` | Updates an edge's target after a rename. Preserves last_graded state if content_hash unchanged. |
| `cross-link mark-broken <edge-id> --reason <text>` | Tombstone an edge that's intentionally broken (target deleted, merged into another doc). |
| `cross-link reset-floor <scope> --reason <text>` | Resets a scope's coverage floor (e.g., scope renamed, edge population shifted). Requires `--reason`. |

### Reporting

| Verb | Purpose |
|---|---|
| `cross-link badge <scope>` | Emits shields.io endpoint JSON for the scope's coverage @ L3. |
| `cross-link report-pr` | Emits a PR-comment-friendly summary: edges added/removed/drifted/regraded, coverage delta. |
| `cross-link audit` | Verifies tracker consistency: every tracker edge still discoverable, no orphaned tracker rows, schema valid. |

## A dark-factory walkthrough

Scenario: an agent is editing `docs/concepts/dvcs-topology.md` and adds a new section "## Cache desync recovery." This drifts the content hash for any anchor that includes the new section.

### Step 1: agent edits + commits

```bash
$ git add docs/concepts/dvcs-topology.md
$ git commit -m "docs(concepts): add cache desync recovery section"
[main abc1234] docs(concepts): add cache desync recovery section
```

Pre-commit hook is fast (just runs L0 + L1 + structural checks). Passes.

### Step 2: agent runs `git push`

Pre-push hook fires `cross-link walk` then `cross-link plan-refresh --since HEAD~1`:

```
$ git push
[cross-link] walk: 233 edges discovered
[cross-link] state classification:
  GRADED:    228
  STALE:     3   ← drift on docs/concepts/dvcs-topology.md
  BROKEN:    0
  UNGRADED:  2
[cross-link] STALE edges:
  docs/README.md@root -> docs/concepts/dvcs-topology.md@root
  docs/architecture.md@dvcs -> docs/concepts/dvcs-topology.md@root
  docs/guides/troubleshooting.md@dvcs-issues -> docs/concepts/dvcs-topology.md@root
[cross-link] dispatching 3 L3 judges (within max_l3_per_push=10)...
  edge 1/3: PASS — README still forecasts the topology adequately; new section is internal mechanic
  edge 2/3: PASS — architecture chapter framing still accurate
  edge 3/3: BLOCK — troubleshooting guide promises "all desync recovery in this section" but desync recovery is now in concepts/dvcs-topology.md§"Cache desync recovery"

[cross-link] FAIL — 1 STALE edge graded BLOCK.
  edge: docs/guides/troubleshooting.md -> docs/concepts/dvcs-topology.md
  rationale: troubleshooting guide claims authoritative coverage of desync recovery
            but new section in target now owns that material
  recovery options:
    a) update docs/guides/troubleshooting.md to defer to dvcs-topology.md§"Cache desync recovery"
    b) move the new section back to troubleshooting.md (rebind concepts edge)
    c) explicit override: cross-link mark-broken <id> --reason "intentional split, both docs cover"
push aborted by pre-push hook.
```

### Step 3: agent fixes + re-pushes

```bash
$ # update troubleshooting guide to point to the new authoritative section
$ vim docs/guides/troubleshooting.md
$ git add docs/guides/troubleshooting.md
$ git commit -m "docs(guides): defer desync recovery to concepts/dvcs-topology.md"
$ git push
[cross-link] walk: 233 edges
[cross-link] STALE: 1 (docs/guides/troubleshooting.md edited; inbound-from-architecture is now stale)
[cross-link] dispatching 1 L3 judge...
  edge 1/1: PASS — architecture's framing of troubleshooting guide is still accurate
[cross-link] all gates pass.
[main def5678] docs(guides): defer desync recovery to concepts/dvcs-topology.md
remote: pushing...
```

The agent never had to read CLAUDE.md to recover. The error message named the recovery verbs.

## Orchestration integration (orchestration-agnostic)

Per ADR-23, the framework is orchestration-agnostic. The CLI takes `--tags <a,b>` and `--exclude-tags <c>`; the user wires the filter into whatever orchestration they have. Three example wirings, none of them the contract:

### Example 1: Pre-push hook (local)

Wire `cross-link walk --tags pre-push` into `.githooks/pre-push`. L0+L1+L2 run always; L3 runs only on STALE edges (capped per-scope by `max_l3_per_push`). Fast path: no STALE → ~50ms. Slow path: 3 STALE → ~$0.15 + 30s. Block path: cap exceeded → BLOCK with `cross-link plan-refresh --batch N --since HEAD~K` directive.

### Example 2: GitHub Actions cron (bootstrap)

```yaml
# .github/workflows/cross-link-bootstrap.yml
on:
  schedule:
    - cron: "0 4 * * *"  # 04:00 UTC daily
  workflow_dispatch:

jobs:
  bootstrap-batch:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install reposix-quality
      - name: Grade 20 oldest edges
        env:
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
        run: cross-link plan-refresh --tags weekly --batch 20 --target UNGRADED --target STALE
      - name: Commit tracker updates
        run: |
          git config user.name "cross-link-bot"
          git config user.email "bot@reposix.dev"
          git add quality/state/cross-link-fidelity-tracker.json
          git diff --staged --quiet || git commit -m "cross-link: daily batch grade ($(date +%F))"
          git push
```

### Example 3: prek / pre-commit / Claude Code hooks / MCP / plain cron

Same shape: invoke `cross-link walk --tags <whatever-tag-the-scope-carries>`. The framework provides only the CLI; you provide the trigger. See [`03-schemas.md`](./03-schemas.md) § "Two layers — framework vs reposix integration" for the boundary.

### 3. PR-comment integration

```yaml
# .github/workflows/cross-link-pr.yml
on:
  pull_request:

jobs:
  pr-summary:
    steps:
      - uses: actions/checkout@v4
      - run: cross-link walk  # no L3 calls
      - run: cross-link report-pr > /tmp/pr-comment.md
      - uses: marocchino/sticky-pull-request-comment@v2
        with:
          path: /tmp/pr-comment.md
```

Output looks like:

```markdown
## Cross-link fidelity

**Coverage @ L3:** 87% → 87% (no change)
**Floor:** 85% (default scope)

### Edges in this PR
- ✅ Added: 2 (UNGRADED — will be graded by background bootstrap)
- ⚠️ Drifted: 3 (STALE — pre-push will dispatch judges)
- ✅ Resolved: 1 (was STALE, now GRADED PASS)
- ❌ Broken: 0

[Full report](./cross-link-pr-report.md)
```

## Workflow patterns

### Pattern: routine push (greenfield repo)

Most pushes drift 0 edges → walk is mechanical-only → no L3 cost → pre-push completes in <100ms. The 95th-percentile case is "I edited prose without changing any heading or section structure" → no STALE edges → fast path.

### Pattern: refactoring a concept doc (drift cascade)

Renaming a heading or restructuring sections drifts every inbound edge. The cap (`max_l3_per_push: 10`) catches this:

```
[cross-link] STALE: 23 edges (drift on docs/concepts/dvcs-topology.md)
[cross-link] FAIL — STALE count (23) exceeds max_l3_per_push (10).
recovery: run `cross-link plan-refresh --since HEAD~1` out-of-band, then re-push.
push aborted.
```

The agent runs the refresh out-of-band (it's slow but doesn't block the push hook), reviews the BLOCK verdicts, fixes them, re-pushes. The refresh step is checkpoint-friendly (each judgement commits independently).

### Pattern: brownfield onboarding (bootstrap from zero)

See [`09-brownfield-and-onboarding.md`](./09-brownfield-and-onboarding.md) § "The onboarding journey." Summary: `suggest-scopes` → review config → commit → daily cron grades batches → coverage climbs → owner promotes enforcement_mode incrementally.

### Pattern: archive an old concept doc

```bash
$ cross-link mark-broken docs/old-concept.md --reason "deprecated; superseded by new-concept.md"
$ cross-link rebind <edge-id> --target docs/new-concept.md
```

`mark-broken` doesn't delete the edge from the tracker — it tombstones it. The walker still discovers references to the old path; they're tagged `MARKED_BROKEN` and excluded from coverage. This preserves audit history.

## Output discipline

Every CLI verb produces:

- **stderr:** human-readable progress (color, animations).
- **stdout:** machine-parseable summary on success (JSON if `--json`, table otherwise).
- **Exit codes:** 0 = success, 1 = fail/block, 2 = config error, 3 = transient error (retryable).

This matters for CI: `cross-link walk` returning exit 1 on BLOCK is what fires the GitHub Actions failure. Exit 3 (transient) lets CI auto-retry.

## What we're NOT building

To keep v1 scope tight:

- **No interactive REPL.** All CLI is one-shot; agents drive it from scripts.
- **No GUI / web dashboard.** Tracker JSON + shields.io badge is the UX. Anyone can build a dashboard from the tracker; we don't ship one.
- **No auto-fix.** L3 produces verdicts + rationale + recovery options; the human/agent decides. Auto-rewriting parent READMEs is out of scope (sketched in [`08-open-questions.md`](./08-open-questions.md)).
- **No multi-repo / monorepo cross-references.** Each repo has its own config + tracker. Cross-repo edges (e.g., to GitHub external URLs) are L0 only.
