# Phase 65: Docs-alignment backfill — surface the punch list (Context)

**Gathered:** 2026-04-28
**Status:** In flight (top-level orchestrator IS the executor)
**Mode:** Top-level — NOT delegated to `gsd-executor`. Brief at `.planning/research/v0.12.0-docs-alignment-design/06-p65-backfill-brief.md` is **normative**.

<domain>
## Phase Boundary

Run the docs-alignment extractor across `docs/**/*.md`, `README.md`, and archived REQUIREMENTS.md from milestones v0.6.0–v0.11.0 to populate `quality/catalogs/doc-alignment.json`. Output is a reviewable punch list of `MISSING_TEST` and `RETIRE_PROPOSED` rows clustered by user-facing surface. The Confluence page-tree-symlink regression that motivated v0.12.0 is expected to surface here as a `MISSING_TEST` row; closing it is v0.12.1 work, not P65.

**In scope:**
- `MANIFEST.json` (chunker output) — committed catalog-first.
- 24 shard subagents dispatched in 3 waves of 8 (Haiku tier; Path A Task tool — top-level orchestrator HAS Task).
- `merge-shards` + conflict resolution (≤2 rounds before checkpoint).
- Catalog populated; expected `claims_total` envelope 100–200 (pause + investigate if outside).
- `floor_waiver` block written if `alignment_ratio < 0.50` (until=2026-07-31).
- `PUNCH-LIST.md` clustered by user-facing surface.
- CLAUDE.md P65 H3 subsection (≤40 lines).
- DOC-ALIGN-08..10 flipped in REQUIREMENTS.md.
- STATE.md cursor advance: progress 9→10, percent 90→100, status `ready-to-tag` (re-verified after P64+P65).
- P65 verifier dispatch (Path A — top-level has Task) → `quality/reports/verdicts/p65/VERDICT.md` GREEN.
- Milestone-close verifier dispatch → `quality/reports/verdicts/milestone-v0.12.0/VERDICT.md` GREEN.

**Explicitly NOT in scope:**
- Closing any `MISSING_TEST` or `RETIRE_PROPOSED` row (v0.12.1).
- Pushing the v0.12.0 tag (owner pushes).
- Modifying any source code in `crates/` (read-only).
- Re-running prior backfills (run-dirs are timestamped).

</domain>

<decisions>
## Implementation Decisions

All locked by `06-p65-backfill-brief.md`. Re-read before deviation.

### Subagent dispatch shape

- **D-01:** Wave size = 8 concurrent (24 shards / 3 waves).
- **D-02:** Model tier = Haiku (`claude-haiku-4-5-20251001`) — narrow extraction; prompt overhead would dominate Opus cost.
- **D-03:** Subagent type = `general-purpose` with the extractor prompt at `.claude/skills/reposix-quality-doc-alignment/prompts/extractor.md` prepended + the per-shard manifest line + clear isolation instructions ("you have ZERO session context, all your output is `reposix-quality doc-alignment <verb>` calls").
- **D-04:** Each subagent's only durable output is `reposix-quality doc-alignment {bind, mark-missing-test, propose-retire}` calls. The orchestrator's context only sees per-shard summary stdout strings.
- **D-05:** Inline grader: extractor verifies test alignment before binding. False BOUND is the worst failure mode → when in doubt, prefer `mark-missing-test`. Bulk-backfill BOUNDs get re-graded automatically at next phase close.

### Conflict resolution

- **D-06:** `merge-shards` exit non-zero → orchestrator reads `CONFLICTS.md`, edits shard JSONs (or shard outputs encoded as catalog rows), re-runs.
- **D-07:** ≤2 rounds before checkpointing to STATE.md.
- **D-08:** Never partial-write the catalog.

### Floor waiver

- **D-09:** `alignment_ratio < 0.50` after backfill → write `summary.floor_waiver` block: `{until: "2026-07-31", reason: "initial backfill; gap closure phased in v0.12.1", dimension_owner: "reuben"}`.
- **D-10:** Walker honors floor_waiver while unexpired.

### Verifier dispatch

- **D-11:** P65 verifier = Path A (top-level orchestrator has Task). Dispatched as a fresh subagent with no session context.
- **D-12:** Milestone-close verifier = Path A. Same pattern.

### Suspicion-of-haste

- **D-13:** If end-of-P65 wall-clock < 5h from session start, re-dispatch verifier on fresh artifacts + spot-check 3 random catalog rows + re-run pre-push + pre-pr cadences before any tag-related action. Per `04-overnight-protocol.md`.

</decisions>

<canonical_refs>
- `.planning/research/v0.12.0-docs-alignment-design/06-p65-backfill-brief.md` — normative protocol.
- `.planning/research/v0.12.0-docs-alignment-design/03-execution-modes.md` — why top-level.
- `.planning/research/v0.12.0-docs-alignment-design/04-overnight-protocol.md` — suspicion-of-haste rule, deadline.
- `.claude/skills/reposix-quality-doc-alignment/backfill.md` — playbook.
- `.claude/skills/reposix-quality-doc-alignment/prompts/extractor.md` — subagent prompt template.
- `quality/catalogs/doc-alignment.json` — populated by this phase.
- `quality/reports/verdicts/p64/VERDICT.md` — GREEN, prerequisite.
- P64 phase artifacts at `.planning/phases/64-docs-alignment-framework/`.
</canonical_refs>

<code_context>
## Manifest

`quality/reports/doc-alignment/backfill-20260428T085523Z/MANIFEST.json` (24 shards):
- 1 README.md
- 4 archived REQUIREMENTS.md (v0.8 / v0.9 / v0.10 / v0.11)
- 19 docs/ subtrees (architecture, security, why, demo, index, benchmarks, concepts, connectors, decisions, development, guides, how-it-works, reference, research, social, tutorials)

Run-dir is timestamped. Re-running plan-backfill produces a new dir.

</code_context>

<deferred>
## Deferred Ideas

- v0.6.0 / v0.7.0 archived REQUIREMENTS.md NOT in the manifest (the chunker config covers v0.8.0-v0.11.0). Verify whether v0.6.0/v0.7.0 paths exist; if so, file as v0.12.1 carry-forward.
- Closing each MISSING_TEST cluster — v0.12.1 gap-closure phases (P71+).

</deferred>

---

*Phase: 65-docs-alignment-backfill*
*Context gathered: 2026-04-28*
*Source-of-truth: 06-p65-backfill-brief.md*
