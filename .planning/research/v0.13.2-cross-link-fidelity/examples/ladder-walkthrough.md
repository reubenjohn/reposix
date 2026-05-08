# Ladder walkthrough: one edge through L0–L3

> **Purpose.** Concrete trace of how a single edge gets graded at each level of the scrutiny ladder, with sample inputs and outputs.
> **See also.** [`../02-architecture.md`](../02-architecture.md) for the conceptual ladder; [`../03-schemas.md`](../03-schemas.md) for tracker shape.

## The edge under test

**Source:** `docs/README.md` (line 47)
**Target:** `docs/concepts/dvcs-topology.md#mirror-lag-refs`
**Scope:** `anchor-readme` (matched on source-glob `**/README.md`)
**Configured `max_level`:** L3

The link in the source looks like:

```markdown
The cache writes [mirror-lag refs](./concepts/dvcs-topology.md#mirror-lag-refs)
that measure observable lag between SoT edits and mirror syncs.
```

## L0 — link resolves (mechanical, ~0.1ms)

**Check.** Does the file `docs/concepts/dvcs-topology.md` exist?

**Verifier:**
```bash
test -f docs/concepts/dvcs-topology.md
```

**Outcome.** Pass. File exists.

**On fail.**
```
[cross-link] BROKEN — target file missing.
  source: docs/README.md:47
  target: docs/concepts/dvcs-topology.md (NOT FOUND)
recovery:
  - cross-link rebind <edge-id> --target <new-path>
  - or restore the file at the original path
```

## L1 — anchor resolves (mechanical, ~1ms)

**Check.** Does `docs/concepts/dvcs-topology.md` contain a heading whose mkdocs slug equals `mirror-lag-refs`?

**Verifier (pseudo):**
```python
slugs = mkdocs_slug_set(parse_markdown("docs/concepts/dvcs-topology.md"))
assert "mirror-lag-refs" in slugs
```

**Outcome.** Pass. The doc has `## Mirror-lag refs` → slug `mirror-lag-refs`.

**On fail.**
```
[cross-link] BROKEN — anchor not in target.
  source: docs/README.md:47
  target: docs/concepts/dvcs-topology.md
  anchor: #mirror-lag-refs (NOT FOUND)
helper:
  $ cross-link find-similar-heading docs/concepts/dvcs-topology.md "#mirror-lag-refs"
  candidates:
    1. #mirror-lag (distance 5)
    2. #lag-refs (distance 5)
recovery:
  $ cross-link rebind <edge-id> --target docs/concepts/dvcs-topology.md#<correct-slug>
```

## L2 — hash-drift since last grade (mechanical, ~5ms)

**Check.** Has the AST subtree under `## Mirror-lag refs` changed since `last_graded_hash`?

**Algorithm:**
1. Parse `docs/concepts/dvcs-topology.md` to AST.
2. Find subtree rooted at heading `## Mirror-lag refs`.
3. Compute SHA-256 of canonical AST representation (heading-text-stripped, whitespace-normalized).
4. Compare against `tracker[edge_id].last_graded_hash`.

**Outcome.**
- If `current_hash == last_graded_hash` → state remains `GRADED`. No L3 dispatch.
- If `current_hash != last_graded_hash` → state becomes `STALE`. Schedule L3 dispatch (subject to `max_l3_per_push` cap).
- If `tracker[edge_id]` is missing → state is `UNGRADED`. Schedule L3 dispatch IF scope's enforcement_mode is `block-newedge` AND edge `introduced_in_pr` is the current PR.

**Tracker entry pre-L3:**
```json
{
  "id": "docs/README.md@root->docs/concepts/dvcs-topology.md@mirror-lag-refs",
  "state": "STALE",
  "last_graded_hash": "sha256:abc123...",
  "current_target_hash": "sha256:def456...",
  "last_graded_at": "2026-04-22T09:14:00Z",
  "drift_detected_at": "2026-05-08T14:32:11Z"
}
```

## L3 — LLM-graded fidelity (~30s, ~$0.05)

**Check.** Does the source's framing of the target still adequately forecast what the target teaches?

**Inputs assembled:**
1. **Source content:** the paragraph(s) around line 47 of `docs/README.md` that contain the link, plus the heading hierarchy above (so the judge knows the README's local context).
2. **Target content:** full content of `docs/concepts/dvcs-topology.md` § "Mirror-lag refs" subtree.
3. **Grading context (merged target ⊕ edge ⊕ source):**
   - Target frontmatter `cross_link_fidelity.grading_audience` + `grading_context` + `must_forecast`.
   - Edge override (none — this edge has no `[edge_overrides]` entry).
   - Source scope's `grading_contexts.anchor-readme` block.
4. **Scope's `max_level`** (L3) and current state (STALE).

**Prompt skeleton (illustrative; full template ships with the binary):**

```
You are grading a markdown link assertion for fidelity.

# The link
The source doc `docs/README.md` (an anchor README) contains a link to
`docs/concepts/dvcs-topology.md#mirror-lag-refs`. Your job is to grade
whether the source's framing of the linked target is still accurate.

# Grading audience
{from target frontmatter: "agent or developer planning a v0.14+ phase that touches the cache or helper"}

# Grading context — target self-description
{from target frontmatter grading_context}

# Grading context — source scope (anchor-readme)
{from grading_contexts.anchor-readme.context}

# Must-forecast (BLOCK if any concept missing from source's framing)
{from target frontmatter must_forecast: ["three DVCS roles", "mirror-lag refs invariant"]}

# Source content (with hierarchical context)
{path: docs/README.md, lines 30–60}
[markdown excerpt with the link in context]

# Target content
{path: docs/concepts/dvcs-topology.md § Mirror-lag refs}
[markdown subtree]

# Output schema
Respond with JSON:
{
  "verdict": "PASS" | "FLAG" | "BLOCK",
  "rationale": "one paragraph explanation",
  "missing_concepts": [list of must_forecast concepts not addressed],
  "noteworthy_drift": "anything that changed since last grade that may matter"
}
```

**Sample output:**

```json
{
  "verdict": "PASS",
  "rationale": "README's framing of mirror-lag refs covers the lag-measurement-not-state invariant explicitly, including the sentence 'measure observable lag between SoT edits and mirror syncs.' Target doc's new section 'Cache desync recovery' is internal mechanic; not load-bearing for the reader's mental model from the README level. The must-forecast concepts ('three DVCS roles', 'mirror-lag refs invariant') are both present in the README. PASS.",
  "missing_concepts": [],
  "noteworthy_drift": "Target added a 'Cache desync recovery' subsection. README does not currently forecast it, but this is acceptable — recovery procedures are operator concerns, not architecture concepts."
}
```

**Tracker entry post-L3:**
```json
{
  "id": "docs/README.md@root->docs/concepts/dvcs-topology.md@mirror-lag-refs",
  "state": "GRADED",
  "last_graded_hash": "sha256:def456...",
  "last_graded_at": "2026-05-08T14:32:41Z",
  "last_verdict": "PASS",
  "last_judge_rationale": "[full rationale above]",
  "last_judge_model": "claude-opus-4-7",
  "last_judge_tokens_in": 4823,
  "last_judge_tokens_out": 287,
  "regrade_count": 4
}
```

## What happens at each verdict

| Verdict | Pre-push behavior | Tracker change |
|---|---|---|
| PASS | No block | `last_graded_hash` updated, `state: GRADED`, `last_verdict: PASS` |
| FLAG | No block; warning emitted | Same as PASS but `last_verdict: FLAG` |
| BLOCK | Push BLOCKs with rationale | `last_graded_hash` NOT updated; `state` remains STALE; recovery directive emitted |

## Cost summary for this one edge

| Level | Latency | Cost |
|---|---|---|
| L0 | ~0.1ms | 0 |
| L1 | ~1ms | 0 |
| L2 | ~5ms | 0 |
| L3 | ~30s | ~$0.05 |

**Total per-push fast path** (no drift): ~6ms total, $0.

**Total per-push slow path** (this edge drifted): ~30s + ~$0.05.

**Cap protection:** `max_l3_per_push: 10` ensures total per-push slow-path cost ≤ ~5min and ~$0.50.
