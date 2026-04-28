# 06 — P65 implementation brief: docs-alignment backfill audit

## Phase identity

- **Phase number:** P65
- **Milestone:** v0.12.0
- **Title:** Docs-alignment backfill — surface the punch list
- **Execution mode:** **`top-level`** (NOT under `/gsd-execute-phase`; see `03-execution-modes.md`)
- **Goal:** Run the extractor across all current docs and archived REQUIREMENTS.md to populate the doc-alignment catalog. Output is a reviewable punch list of `MISSING_TEST` and `RETIRE_PROPOSED` rows that v0.12.1 closes.
- **Requirements:** DOC-ALIGN-08 through DOC-ALIGN-10.
- **Prerequisite:** P64 has shipped (catalog schema + binary + skill + slash commands all live).

## Why top-level

The backfill is orchestration-shaped: dispatch ~30 subagents, aggregate results, resolve merge conflicts that need semantic judgment. `gsd-executor` lacks the `Task` tool and depth-2 spawning is forbidden. The orchestrator IS the executor for this phase. See `03-execution-modes.md`.

## Inputs (the corpus the extractor mines)

The chunker (`reposix-quality doc-alignment plan-backfill`) walks these globs:

```
docs/**/*.md
.planning/milestones/v0.6.0-phases/REQUIREMENTS.md
.planning/milestones/v0.7.0-phases/REQUIREMENTS.md
.planning/milestones/v0.8.0-phases/REQUIREMENTS.md
.planning/milestones/v0.9.0-phases/REQUIREMENTS.md
.planning/milestones/v0.10.0-phases/REQUIREMENTS.md
.planning/milestones/v0.11.0-phases/REQUIREMENTS.md
README.md
```

Output: `MANIFEST.json` at `quality/reports/doc-alignment/backfill-<ts>/MANIFEST.json` with shard assignments. Directory-affinity sharding, ≤3 files per shard, alphabetical fallback within a directory.

## Protocol the orchestrator follows

```
1. Run: reposix-quality doc-alignment plan-backfill
   → produces MANIFEST.json under quality/reports/doc-alignment/backfill-<ts>/
   → expect ~25–35 shards depending on docs growth
   → COMMIT MANIFEST.json (catalog-first analog: the chunker output is the contract for what work happened).

2. For each wave of 8 shards:
   For each shard in the wave (parallel via Task tool):
     prompt = read .claude/skills/reposix-quality-doc-alignment/prompts/extractor.md
              + manifest_line for this shard
     spawn Task(subagent_type="general-purpose", model="haiku", prompt=prompt)
     subagent must:
       - read each cited file in its manifest
       - identify behavioral claims with file:line citations
       - for each claim, attempt to find a binding test (search tests/ via grep)
       - call `reposix-quality doc-alignment bind --row-id ... --grade GREEN ...` if a test is found AND the subagent verifies alignment
       - call `reposix-quality doc-alignment mark-missing-test ...` if no binding test exists
       - call `reposix-quality doc-alignment propose-retire ...` ONLY if the claim is clearly superseded by a documented architecture decision (cite the supersession source)
       - write a summary to quality/reports/doc-alignment/backfill-<ts>/shards/NNN.json (the tool calls do this implicitly via state mutation; the JSON is computed from tool outputs)
   Wait for all 8 to complete (Task tool blocks until done).
   The orchestrator's context only sees per-shard summary strings (tool stdout: "shard 005: 22 rows, 3 MISSING_TEST, 0 RETIRE_PROPOSED").

3. After all waves complete:
   Run: reposix-quality doc-alignment merge-shards quality/reports/doc-alignment/backfill-<ts>/
   If exit 0:
     The catalog is written. Proceed to step 4.
   If exit non-zero:
     Read CONFLICTS.md. For each conflict:
       - read the cited prose in each shard
       - read the cited test
       - decide which shard had it right (or whether both citations belong as multi-source on one row)
       - edit the relevant shards/NNN.json directly (deterministic dedup will then succeed)
     Re-run merge-shards.
     If conflicts persist after 2 rounds of edits, pause and checkpoint — do not commit.

4. Run: reposix-quality doc-alignment status
   Capture the alignment_ratio, claims_total, claims_bound, claims_missing_test, claims_retire_proposed.
   If alignment_ratio < 0.50 (the floor):
     This is expected for a first backfill — many claims, few existing tests.
     Do NOT lower the floor. Instead, immediately commit a WAIVER row in
     quality/catalogs/doc-alignment.json's summary.floor_waiver field
     with until=2026-07-31 (matches v0.12.1 milestone close target),
     reason="initial backfill; gap closure phased in v0.12.1",
     dimension_owner="reuben".
   The walker honors floor_waiver: when present and unexpired, alignment_ratio < floor does not BLOCK.

5. Generate the punch-list report at quality/reports/doc-alignment/backfill-<ts>/PUNCH-LIST.md:
   Cluster MISSING_TEST rows by user-facing surface
     (e.g. "Confluence backend parity", "JIRA epic/story shape",
      "ease-of-setup happy path", "outbound HTTP allowlist behavior").
   For each cluster, list the rows with their claim text + source citation.
   List RETIRE_PROPOSED rows separately for human review.
   This is the input v0.12.1's gap-closure phases consume.

6. Commit cadence (one commit per logical step):
   - MANIFEST.json
   - The full backfill-<ts>/ run dir (all shard JSONs + MERGE.md or CONFLICTS.md resolutions)
   - The populated quality/catalogs/doc-alignment.json (written by merge-shards)
   - quality/reports/doc-alignment/backfill-<ts>/PUNCH-LIST.md
   - CLAUDE.md update (P65 H3 subsection)
   - quality/SURPRISES.md update (if any obstacles arose)
   - Phase-close VERDICT (after verifier subagent dispatch)

7. Dispatch milestone-close verifier subagent (Path A — top-level orchestrator HAS Task tool).
   Verdict written to quality/reports/verdicts/p65/VERDICT.md.
   If GREEN: phase ships.
   If RED: read findings, address them, re-dispatch. Do not negotiate down.

8. After P65 verdict GREEN:
   Dispatch milestone-close verifier for v0.12.0 itself
   (writes quality/reports/verdicts/milestone-v0.12.0/VERDICT.md).
   Verifier checks:
     - All P56–P65 catalog rows GREEN or WAIVED.
     - Tag-gate guards in .planning/milestones/v0.12.0-phases/tag-v0.12.0.sh all pass.
     - CHANGELOG [v0.12.0] section finalized.
   If GREEN: STOP. Do not push the tag. Update STATE.md cursor to "v0.12.0 ready-to-tag (re-verified after P64+P65); owner pushes tag."
   If RED: write findings, await human.
```

## Subagent prompt requirements

The prompt template `.claude/skills/reposix-quality-doc-alignment/prompts/extractor.md` (P64 deliverable) MUST instruct the subagent:

- The ONLY output is a series of `reposix-quality doc-alignment <subcmd>` calls. No prose summary, no JSON written by hand.
- Every `bind` call MUST include `--rationale` with a file:line citation that the tool will validate.
- Claims must be specific enough that a test could fail. "Reposix is fast" is not a claim. "Reposix init completes in <100ms for the simulator backend" is.
- When a claim is bound to a test, the subagent must verify the test actually asserts the claim by reading the test fn body. If unsure, mark as `MISSING_TEST` rather than `BOUND` (false BOUND is the worst failure mode).
- When a claim has no test, `mark-missing-test`. Do NOT propose retirement just because no test exists.
- Retirement is the most expensive option. Only propose retirement if the claim is clearly superseded by a written architecture decision (e.g., a `architecture-pivot-summary.md` doc retired the FUSE behavior). Cite the supersession source.

## Wave dispatch details

- Wave size: 8 concurrent subagents.
- Model tier: extractors run on Haiku (`claude-haiku-4-5-20251001`) — they're doing narrow extraction, prompt overhead would dominate Opus cost.
- The grader role is invoked by the extractor inline (the same subagent verifies test alignment when binding). For `/reposix-quality-refresh` post-P65, the grader role is a separate Opus dispatch — but for the bulk backfill, inline Haiku grading is acceptable given the catalog-first nature (every BOUND will be re-verified at the next phase close anyway).
- Rate limit: ~8 concurrent is well within tier limits. If 429s appear, halve the wave size and retry (the run dir state lets you resume).

## Expected output shape

After P65 ships, expect the catalog to look approximately:

```jsonc
{
  "summary": {
    "claims_total": 100–200,
    "claims_bound": 30–80,
    "claims_missing_test": 50–120,
    "claims_retire_proposed": 5–20,
    "claims_retired": 0,
    "alignment_ratio": 0.30–0.50,
    "floor": 0.50,
    "floor_waiver": { "until": "2026-07-31", "reason": "initial backfill; v0.12.1 gap closure", "dimension_owner": "reuben" }
  },
  "rows": [ ... ]
}
```

If the actual numbers diverge wildly (e.g. 800 claims, or 5 claims), pause. Either the extractor is too aggressive (granularizing every sentence) or too conservative (skipping prose). Investigate before committing.

## What P65 explicitly does not do

- Does NOT close any MISSING_TEST or RETIRE_PROPOSED row. That's v0.12.1.
- Does NOT push the v0.12.0 tag. Owner pushes.
- Does NOT bypass the verifier subagent (even if alignment_ratio looks bad — bad ratios are a milestone-progression signal, not a failure of P65).
- Does NOT delete or re-run prior backfills. The run dir is timestamped; multiple backfills coexist.
- Does NOT modify any source code in `crates/` (read-only against implementation). The catalog and run-dir artifacts are the only writes outside `quality/catalogs/doc-alignment.json` and CLAUDE.md.
