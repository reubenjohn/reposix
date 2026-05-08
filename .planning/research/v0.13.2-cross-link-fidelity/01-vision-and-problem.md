# 01 — Vision and problem

> **Read first.** [`index.md`](./index.md) for 1-page summary.
> **Read next.** [`02-architecture.md`](./02-architecture.md) for the technical design.

## The problem in one sentence

Markdown links assert fidelity ("A's framing of B matches what B currently teaches"); the project has 233+ such assertions and zero machinery to detect when they go stale.

## The failure mode this gate catches

Today, when a doc-author updates `docs/concepts/dvcs-topology.md` to add a new section on "mirror-lag refs," nothing checks that the parent README.md (which previously claimed authority over that concept) is still adequate. The reader of `docs/README.md` sees a forecast of `dvcs-topology.md` that's now out-of-date. They don't know what they don't know.

This is the **unknown-unknowns** failure mode: a reader following progressive disclosure trusts the parent doc to forecast its children, and the parent doc silently lies because nobody re-graded it after the children drifted.

Mechanical link checkers (lychee, markdown-link-check) catch the *resolution* failure mode (broken link → 404). They don't catch the *fidelity* failure mode (link resolves, but the asserter's framing is wrong). That's the gap.

## Why now

Three forces converged:

1. **The dark-factory ethos.** This project treats docs as adversarial input. Drift in a parent README means an autonomous agent gets a misleading map of the codebase before it ever touches code. That's a security-of-grounding concern, not a polish concern.

2. **Subagent grading is now cheap and reliable enough.** The `subjective-rubrics.json` catalog already dispatches Sonnet judges for cold-reader, install-positioning, and headline-numbers. The infrastructure is reusable. We're not inventing a new mechanism; we're applying an existing mechanism to a new edge type.

3. **Edge count is tractable.** Baseline measurement: 233 md→md edges in docs/, 25 README files, ≈50 markdown files in tracked scope. Even at L3-default with weekly cadence, this is ≈10 Sonnet judge calls per week (post-bootstrap). The cost ladder makes this affordable.

## What success looks like

A cold agent — one that has never seen this project — should be able to read `README.md`, follow exactly the children it forecasts, and find no surprises. If the agent ever encounters knowledge in a child file that the parent README didn't even hint at, that's a failed fidelity assertion. Pre-push catches this before it lands.

Concretely:

- **Coverage % @ L3 ≥ 90%.** At least 90% of in-scope edges have been L3-graded within the freshness TTL.
- **Orphan list is empty.** Every in-scope markdown file is either a designated root or has ≥1 inbound edge.
- **Pre-push gate fires on drift.** Editing a child file that has stale parent forecasts BLOCKs the push with a directed recovery verb.
- **L0–L2 zero-cost.** Mechanical checks run pre-commit; subagent grading runs only on hash-drift.

## Why this is hard

Three things make it harder than docs-alignment:

1. **Edges, not nodes.** The unit of grading is `(source, target)`, not a single file. A file participates in many edges; a single edit can stale many edges with one keystroke.

2. **Subjective grading needs grounding context.** "Does A forecast B?" is meaningless without "for what reader, with what scope?" The grading_context schema (target frontmatter + edge override + source default) is the load-bearing answer to this — without it, the judge hallucinates "this README could mention X" feedback that's noise.

3. **Anchor stability under heading rename.** mkdocs auto-slugs anchors from heading text. A typo fix in a heading silently breaks every cross-link to that section. The walker has to detect "did you mean `#new-slug`?" and surface helper verbs to the agent without doing magical inference.

## What we're explicitly NOT trying to do

- **Nav reachability.** Cross-link-audit (`structure/cross-link-audit.py`) checks every doc is reachable from mkdocs nav. That's a node-property check, not an edge-property check. Keep it as a smaller, separate gate.
- **Mechanical link-resolves only.** Existing tools do this fine. We layer on top, not replacing.
- **Generic doc-graders.** The L3 prompt is scoped to "does the asserter forecast the target?" Not "is this doc well-written" — that belongs in the subjective rubrics catalog.
- **Auto-fix.** L3 produces verdicts + rationale; the human or agent decides how to fix. Auto-rewriting the parent README is out of scope.

## The vision in one paragraph

A future agent opens `quality/catalogs/cross-link-fidelity.json`, sees every markdown link in the project graded with a verdict and a freshness timestamp, knows exactly which edges are stale, and runs `cross-link plan-refresh` to dispatch judges for the stale ones. Pre-push gates the rigor: a content edit that stales 3 inbound forecasts BLOCKs until those forecasts are re-graded. The dark-factory regression test proves that a cold agent can navigate the doc graph without surprises. Eventually, this becomes its own OSS tool — `cross-link-fidelity` — that any project with progressive-disclosure docs can adopt.
