# 05 — Edge cases and failure modes

> **Read first.** [`02-architecture.md`](./02-architecture.md) and [`04-cli-and-workflow.md`](./04-cli-and-workflow.md).
> **Read next.** [`06-decisions-log.md`](./06-decisions-log.md) for which choices were made deliberately.

These are the cases that bite first. Each has a named recovery path so the agent doesn't have to reason from scratch.

## 1. Heading rename → silent anchor break

**Scenario.** Author fixes a typo in a heading: `## Mirror lag refs` → `## Mirror-lag refs`. mkdocs auto-slugs both: `#mirror-lag-refs` was the slug for *both* spellings, so the link still resolves... unless the original was `## Mirror-Lag Refs` → slug `#mirror-lag-refs` *and* now it's `## Mirror-lag refs` → also slug `#mirror-lag-refs`. Often anchor stays. But sometimes (e.g., adding/removing a word) the slug changes and inbound links 404 silently in mkdocs render.

**How the gate catches it.** L1 verifier walks the target's heading slugs and checks each inbound `#anchor` against the slug set. Rename mismatches → `state: BROKEN`.

**Recovery.**

```bash
$ cross-link find-similar-heading docs/concepts/dvcs-topology.md "#mirror-lag-refs-old"
candidates (ranked by Levenshtein distance):
  1. #mirror-lag-refs   (distance 4; section starts at line 47)
  2. #mirror-mechanics   (distance 11; section starts at line 92)
  3. #lag-refs           (distance 5; section starts at line 31)

run `cross-link rebind <edge-id> --target docs/concepts/dvcs-topology.md#mirror-lag-refs` to fix.
```

The agent picks the right candidate, runs `rebind`, re-pushes.

**Why not auto-fix.** "Auto-rebind to closest match" is a magic-inference foot-gun. Ratifying suggestions explicitly is cheap; silently picking the wrong target is expensive (silently misforecast doc).

## 2. Bulk content-drift on a refactor

**Scenario.** Author rewrites a concept doc end-to-end. 23 inbound edges are now STALE.

**How the gate catches it.** Walker classifies all 23 STALE; pre-push counts `STALE > max_l3_per_push (10)` → BLOCK with directive:

```
[cross-link] FAIL — 23 STALE exceeds max_l3_per_push (10).
recovery options:
  1. cross-link plan-refresh --since HEAD~1   (out-of-band; reviews each verdict)
  2. cross-link plan-refresh --since HEAD~1 --skip-l3 --reason "doc fully rewritten; defer regrade"
                                              (records UNGRADED-with-reason; coverage floor protects)
  3. REPOSIX_CLF_SKIP_L3=1 git push           (emergency only; writes audit log entry)
push aborted.
```

**Recovery.** Path 1 is normal: out-of-band refresh dispatches all 23 judgements (slow, ~$0.70, ~5min) but each is checkpointable and reviewable. Path 2 is when the rewrite is so total that grading old framings is wasted — defer to next cron cycle. Path 3 is rare, audit-logged.

## 3. File rename (path change, content unchanged)

**Scenario.** `docs/concepts/dvcs-topology.md` → `docs/dvcs/topology.md`. No content changed.

**How the gate catches it.** Walker discovers edges to the new path. Old-path edges become BROKEN. New-path discoveries are NEW UNGRADED. Tracker's `last_graded_hash` from the old edge entries is lost.

**Recovery.** The smart move is preserving grade state across renames:

```bash
$ cross-link rebind --auto --since HEAD~1
detected file rename: docs/concepts/dvcs-topology.md → docs/dvcs/topology.md
  (content_hash unchanged: sha256:5678ef...)
proposed rebinds:
  3 edges: source_path docs/README.md → target docs/concepts/dvcs-topology.md
    rebind to docs/dvcs/topology.md? [Y/n] y
  ...

Rebound 14 edges. Preserved last_graded state where target content_hash unchanged.
Run `git diff quality/catalogs/cross-link-fidelity.json` to review.
```

The walker's content-hash comparison is the safety net: if the moved file content hasn't changed, the edge stays GRADED with its original verdict. Path-only renames don't trigger L3 regrades.

## 4. Heading collision (sub-section slug ambiguity)

**Scenario.** A doc has two `## Examples` sections under different parents. mkdocs renders the second as `#examples_1`. Inbound `[link](#examples)` always points at the first.

**How the gate catches it.** Walker records `(file, heading-path, rendered-slug)` triples. When two `heading-path`s yield the same `rendered-slug`, walker emits a warning and chooses the first by document order. Inbound edges with `target_anchor: "#examples"` resolve to first; second is reachable only via `#examples_1`.

**Recovery.** Author uses unique heading text. Or explicitly anchors with HTML: `<a id="examples-rest"></a>` then `[link](#examples-rest)`.

The gate can't disambiguate intent — but it can flag it:

```
[cross-link] WARN — heading slug collision in docs/foo.md:
  ## Examples (line 12) → slug "examples"
  ## Examples (line 87) → slug "examples_1"
  inbound edges with anchor #examples will resolve to line 12.
  rename one heading or add explicit anchor.
```

## 5. Sub-sub-heading anchors

**How they work.** mkdocs auto-slugs are flat — no hierarchy. `### Foo under Bar` produces `#foo-under-bar`. Two `### Foo`s under different `##` parents collide as in case 4.

**Hashing strategy.** AST subtree hash is rooted at the heading. For `[link](file.md#foo-under-bar)`:
1. Find heading `### Foo under Bar` in `file.md`.
2. Hash the AST subtree from that heading down to (but not including) the next heading at depth ≤ 3.
3. That hash is `last_graded_hash` for edges with this anchor.

**Why this works.** Editing prose under `### Foo` changes that subtree's hash → STALE. Editing prose elsewhere in the file → no change. Adding a new sub-sub-heading under `### Foo` extends the subtree → STALE.

**What this misses.** A `## Parent` rename doesn't change the `### Child` subtree's hash, but it might change reader expectations. Acceptable v1 limitation; revisit if it bites.

## 6. Non-md target drift

**Scenario.** README links to `[the install script](../scripts/install.sh)`. Script gets a major refactor.

**How the gate catches it.** Walker hashes the entire `install.sh` file (no AST for non-md). Hash-drift triggers L3. L3 judge gets:
- Source doc (the README)
- Target file content (the script)
- Grading context from edge override or scope (no target frontmatter possible)

**Recovery.** Same as md targets — refresh, rebind, or mark-broken.

**Limitation.** Non-md targets can't carry `cross_link_fidelity:` frontmatter. Their grading context must come from the edge or source — which is fine for low-volume cases (most non-md targets are scripts/configs referenced from one or two anchor docs).

## 7. Circular edges

**Scenario.** `A.md` links to `B.md`, which links back to `A.md`. Both forecast each other.

**How the gate handles it.** Edges are independent; circular doesn't matter. A↔B is two separate edges in the tracker. Each gets graded against its own scope's rules.

**Why this isn't a problem.** L3 judge prompt is per-edge, not graph-traversal. No infinite recursion possible.

**Health metric.** `cross-link audit --report-cycles` prints any cycles. Track but don't block — cycles are sometimes legitimate (cross-references between sibling concept docs).

## 8. Same edge, multiple scopes

**Scenario.** An edge matches both `default` and `anchor-readme` scope globs.

**How the gate handles it.** Last-match-wins (gitignore semantics). The order scopes appear in `config.toml` is the resolution order.

**Best practice.** Put `default` first; named scopes after. Scopes ordered most-specific-last.

```toml
[scopes.default]                     # 1st: catch-all
[scopes.nav-only]                    # 2nd: refines a subset of default
[scopes.anchor-readme]               # 3rd: more specific subset
[edge_overrides."README.md@root->LICENSE.md@root"]   # last: per-edge override beats all scopes (ADR-23 D-3 edge-ID format)
```

## 9. Tracker corruption / schema bump

**Scenario.** Owner runs `cross-link migrate-tracker 1.0.0-to-2.0.0` mid-work, gets interrupted.

**How the gate catches it.** Every CLI verb runs `audit` first — checks `schema_version` matches binary, checks all entries parse, checks no duplicate edge IDs. Any mismatch → CLI refuses to run with `cross-link audit --fix` directive.

**Recovery.** `cross-link audit --fix` rebuilds tracker from scratch (re-runs walk, regrades nothing — preserves last_graded data where possible). Worst case: lose all `last_graded_hash` and re-bootstrap.

**Mitigation.** Tracker is git-tracked. Owner can `git checkout HEAD~1 quality/catalogs/cross-link-fidelity.json` to roll back. Migration verbs commit before starting so rollback is always available.

## 10. Edge that's intentionally ungraded forever

**Scenario.** A README links to `THIRD-PARTY-LICENSE.md`. The license is verbatim — "fidelity" is not a meaningful concept.

**Configuration.** Carve out a scope:

```toml
[scopes.licenses]
source_glob = "**/*.md"
target_glob = "**/{LICENSE,LICENSE-*,*-LICENSE.md,THIRD-PARTY-*.md}"
max_level = "L0"  # link must resolve, nothing more
count_in_coverage = false  # don't pull coverage @ L3 down
```

Or per-edge override:

```toml
[edge_overrides."README.md@root->LICENSE.md@root"]
max_level = "L0"
reason = "license text is verbatim; no fidelity grading possible"
```

## 11. Pre-push hook stuck on Sonnet API outage

**Scenario.** Anthropic API is down. Pre-push tries to dispatch L3 judges, hangs, eventually times out.

**How the gate handles it.** Each L3 dispatch has a hard 60-second timeout. On timeout: edge is marked `STALE_REGRADE_FAILED` (a sub-state of STALE), push BLOCKs with directive:

```
[cross-link] L3 dispatch failed: Anthropic API timeout (60s).
options:
  1. wait + retry: git push (transient errors auto-retry up to 3×)
  2. defer: cross-link plan-refresh --batch 0 --skip-l3   (records intent, regrades next CI run)
  3. emergency: REPOSIX_CLF_SKIP_L3=1 git push  (audit-logged)
```

**Why not auto-skip.** Silent skip on outage = silent coverage erosion. Make the human/agent ack the skip.

## 12. Grading context contains a secret

**Scenario.** Author pastes a debug log into `grading_context:` and it contains `ATLASSIAN_API_KEY=...`.

**How the gate handles it.** Pre-commit hook scans `cross_link_fidelity.grading_context` blocks for known secret patterns (same regex set as `quality/gates/structure/cred-hygiene.sh`). Match → BLOCK commit.

L3 dispatcher additionally rejects grading_context that contains `${ENV_VAR}` syntax — no env-var substitution, ever. Context is a content-only field.

**Why this matters.** L3 ships grading_context to Anthropic. A secret leaks if it's in the prompt. The cred-hygiene check is the only safety net.

## 13. Bidirectional vs unidirectional grading

**Scenario.** A → B grades fine. Does B owe a backlink to A?

**How the gate handles it.** No. Edges are unidirectional. A→B asserts "A's framing of B is faithful." It does not imply B should know about A.

**Why this matters.** Concept docs are leaf authorities. They link to nothing; many things link to them. They don't "owe" backlinks to every how-to that references them. The walker discovers bidirectionally; grading is one-way.

**Exception.** Anchor READMEs are graded "does this README forecast its children?" — that's unidirectional too (README → child). Children don't owe forecast of their parent README.

## 14. PR introduces a new doc with no inbound edges

**Scenario.** Author adds `docs/new-concept.md` but nobody links to it yet.

**How the gate handles it.** Walker classifies `new-concept.md` as orphan (no inbound edges). Orphans BLOCK if scope's enforcement_mode is `block-floor` or stricter:

```
[cross-link] FAIL — new file with no inbound edge: docs/new-concept.md
recovery:
  1. add a link from a parent doc (e.g., docs/README.md) to docs/new-concept.md
  2. declare as a designated root: add to project.designated_roots in .cross-link-fidelity
  3. exclude from coverage: add to a scope with ignore=true
```

**Why this matters.** A doc with no inbound edge is unreachable via progressive disclosure. It's "knowledge-graph dead weight" until something links to it. The gate forces explicit framing of new docs.

## 15. Edge moves between scopes (config edit)

**Scenario.** Author renames a scope or changes a glob, and an edge that was in `default` is now in `anchor-readme`.

**How the gate handles it.** Edge identity is stable (path-based); only the `scope` field on the edge changes. The walker re-resolves scope membership on every walk and updates `tracker[edge].scope`.

**Grade preservation.** YES — `last_graded_hash`, `last_graded_at`, `last_verdict` carry over. The new scope's `max_level` and `enforcement_mode` apply going forward.

**Audit trail.** Walker writes a `scope_changed` audit event when an edge's scope changes between walks: `{edge_id, prior_scope, new_scope, walked_at}`.

## 16. BROKEN → GRADED transition (edge re-fixed)

**Scenario.** Author fixes a broken anchor (renames target heading back, restores a deleted file). Edge was BROKEN; now resolves.

**How the gate handles it.** L0+L1 pass on next walk → state transitions to `STALE` (because hash drifted while broken) or `UNGRADED` (if no prior grade). Does NOT auto-revert to GRADED with old verdict — the edge needs an L3 re-grade since either the target or the source likely changed during the BROKEN period.

**Implication.** A "fix the typo, no real content change" recovery still costs one L3 dispatch. Acceptable; the gate's safety is in not assuming.

## 17. Edge re-introduced after `mark-broken` tombstone

**Scenario.** An edge was tombstoned via `cross-link mark-broken --reason "deprecated"`. Months later, author re-adds the same `[link](path#anchor)` from the same source location.

**How the gate handles it.** Walker discovers the edge with same `edge_id` (path-based). Tracker has the tombstone entry — walker promotes the edge from `MARKED_BROKEN` → `UNGRADED` and writes audit event `mark_broken_revoked`. The original `last_graded_hash` is preserved if available; if hash unchanged, edge can fast-path back to GRADED.

**Implication.** Tombstones are reversible. `mark-broken` is not destruction.

## What's NOT covered (yet)

Cases we acknowledge but defer:

- **Markdown extensions** (admonitions, footnotes, definition lists). The AST walker uses CommonMark + select GFM extensions. Anything beyond gets a warning, not an error.
- **Image alt-text fidelity.** Out of scope. Use a different gate.
- **External URLs.** L0 only (HTTP HEAD); no L1+. Handled by `docs-build` already; we don't duplicate.
- **Auto-suggesting which level to promote a scope to.** Could analyze scope's edge count + churn rate + verdict-mix and suggest "this scope is stable enough for L3." Sketched in [`08-open-questions.md`](./08-open-questions.md).
