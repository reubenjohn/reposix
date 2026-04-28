# 02 — Architecture, principles, and surface

## Two project-wide principles (write into `quality/PROTOCOL.md`)

These were named in the design session and apply across every gate, dimension, and tool — not just docs-alignment.

### Principle A: Subagents propose with citations; tools validate and mint

LLM agents (extractor, grader, verifier subagents) must NEVER emit machine-checkable state directly. They produce proposals with file:line citations and invoke deterministic tools that:

1. Validate that the cited file exists, the cited lines are valid, the cited symbol resolves.
2. Compute the canonical hash, verdict, or row-state from the cited primary source.
3. Refuse to mint state if validation fails.

This eliminates the hallucination surface. A subagent that hallucinates a hash, verdict, or test binding cannot persist that hallucination — the tool catches it at the validation step.

Cross-tool examples (already in the codebase or shipping in P64):
- Test verdicts come from `cargo test` exit codes, not LLM judgment (already true).
- Subjective rubric grades come from the rubric's score-to-verdict mapping, not LLM phrasing (P61 pattern).
- Catalog row bindings come from `reposix-quality doc-alignment bind`, which validates and computes hashes (P64).

### Principle B: Tools fail loud, structured, agent-resolvable

Deterministic tools assert preconditions and emit machine-readable failure when preconditions don't hold. They never silently pick a default, never auto-resolve ambiguity, never log-warn-and-continue. Result interpretation and ambiguity resolution belong to the agent that called them — because that agent is the only actor in the loop with the context to decide.

Cross-tool examples:
- `merge-shards` writes `CONFLICTS.md` on ambiguity, exits non-zero, never partially writes the catalog. Agent reads CONFLICTS.md, edits shard JSON files, re-runs.
- `bind` refuses to write if the cited test fn doesn't resolve.
- `confirm-retire` env-guards: refuses to run if `$CLAUDE_AGENT_CONTEXT` is set. Only human shells can confirm.
- The hash walker reports `STALE_*` states; never tries to refresh hashes itself.

## Catalog row schema

Lives at `quality/catalogs/doc-alignment.json`. Each row:

```jsonc
{
  "id": "jira-init-story-parent-symlink",
  "claim": "stories/<key>/parent symlinks to its epic",
  "source": "docs/reference/jira.md:43-44",
  "source_hash": "sha256:8a3f...",      // of the exact line range, computed at bind time
  "test": "tests/agent_flow_real.rs::dark_factory_real_jira",
  "test_body_hash": "sha256:b71e...",   // syn token-stream hash of the fn body, comments normalized away
  "rationale": "Line 91 of test fn asserts symlink target == epic key",
  "last_verdict": "BOUND",
  "last_run": "2026-04-28T03:00:00Z",
  "last_extracted": "2026-04-28T01:30:00Z",
  "last_extracted_by": "extractor-shard-005"
}
```

Catalog summary block (single object at top, computed on every walk):

```jsonc
{
  "claims_total": 147,
  "claims_bound": 93,
  "claims_missing_test": 42,
  "claims_retire_proposed": 0,
  "claims_retired": 12,
  "alignment_ratio": 0.73,
  "floor": 0.50,
  "trend_30d": "+0.04"
}
```

`alignment_ratio = claims_bound / (claims_total - claims_retired)`. Floor starts at 0.50. The floor only ratchets up via deliberate human commit, never auto. Pre-push BLOCKS if `alignment_ratio < floor`.

## Row state machine

| State | Set by | Pre-push block? |
|---|---|---|
| `BOUND` | tool, after grader returns GREEN | no |
| `MISSING_TEST` | tool, after extractor finds new claim with no test | **yes** |
| `STALE_DOCS_DRIFT` | hash walker, when source line range hash changes | **yes** (must run refresh) |
| `STALE_TEST_DRIFT` | hash walker, when test body hash changes | no (soft; re-grade at next phase close) |
| `STALE_TEST_GONE` | hash walker, when cited test path/symbol no longer resolves | **yes** |
| `TEST_MISALIGNED` | tool, after grader returns RED | **yes** |
| `RETIRE_PROPOSED` | tool, from extractor's proposal | **yes** until human confirms |
| `RETIRE_CONFIRMED` | tool, human-only (`confirm-retire` env-guarded) | no |

## Hash semantics

Two hashes per row, computed by the tool, never by the LLM:

1. **`source_hash`** — `sha256` of the exact line range from the cited markdown file. Cheap. Drift detection is a `sha256sum` walk.
2. **`test_body_hash`** — `syn::ItemFn::to_token_stream().to_string()` then `sha256`. Token-stream hashing normalizes whitespace and comments away. Only semantic changes (renames, body edits, attribute changes) trigger drift. A small Rust binary at `quality/gates/docs-alignment/hash_test_fn.rs` (~50 LOC) implements this.

**The hash IS the grading certificate.** A row's hashes update if and only if a grading pass said "this prose and this test still align." The hash walker NEVER refreshes hashes; only `bind` does, and only after the grader returns GREEN. Mechanical hash refresh would silently approve drift — the failure mode the gate exists to prevent.

## Binary surface — `reposix-quality`

Workspace crate at `crates/reposix-quality/`. Self-contained (no `reposix-runtime` imports — keeps a future spinoff a `cargo init` away). Rust binary, `clap` subcommands, `syn` for hash compute.

```
reposix-quality <dimension> <verb> [args]

# doc-alignment dimension transitions
reposix-quality doc-alignment bind             --row-id X --source f:l-l --test f::sym --grade GREEN --rationale "..."
reposix-quality doc-alignment propose-retire   --row-id X --rationale "..."
reposix-quality doc-alignment confirm-retire   --row-id X            # human-only; refuses if $CLAUDE_AGENT_CONTEXT is set
reposix-quality doc-alignment mark-missing-test --row-id X
reposix-quality doc-alignment plan-refresh     <doc-file>            # builds stale-row manifest, prints JSON
reposix-quality doc-alignment plan-backfill                          # writes MANIFEST.json for backfill fan-out
reposix-quality doc-alignment merge-shards     <run-dir>             # deterministic dedup; writes catalog OR fails with CONFLICTS.md
reposix-quality doc-alignment status

# generic across dimensions
reposix-quality run     --gate <name>                                # invoke one specific gate
reposix-quality run     --cadence pre-push                           # everything that runs at this cadence
reposix-quality verify  --row-id X                                   # read-only inspection
reposix-quality walk                                                 # hash drift walker; updates last_verdict only
```

The walker (`walk`) is what runs in pre-push. It is deterministic-only:
- For each row, compute `source_hash` and `test_body_hash` from current files.
- Compare to stored hashes.
- Set `last_verdict` to one of the `STALE_*` states or to `BOUND` (preserving the existing certificate if hashes match).
- Refuse to update the stored hashes themselves.
- Exit non-zero if any blocking state is present.

## Skill surface — `.claude/skills/reposix-quality-doc-alignment/`

Mirrors the existing `.claude/skills/reposix-quality-review/` shape (P61 pattern):

```
.claude/skills/reposix-quality-doc-alignment/
├── SKILL.md                       # when/how to invoke
├── refresh.md                     # orchestrator playbook for one-doc refresh
├── backfill.md                    # orchestrator playbook for full backfill
└── prompts/
    ├── extractor.md               # subagent prompt template (per shard or per doc)
    └── grader.md                  # subagent prompt template (per row)
```

Slash commands as the user-facing entry:

```
/reposix-quality-refresh <doc-file>     # one stale doc — orchestrator runs plan-refresh,
                                        # dispatches grader subagent via Task,
                                        # interprets RETIRE_PROPOSED / TEST_MISALIGNED,
                                        # surfaces blocking issues to user

/reposix-quality-backfill               # full audit — orchestrator runs plan-backfill,
                                        # dispatches N shard agents in waves of 8,
                                        # runs merge-shards,
                                        # reports summary
```

Both slash commands are **top-level only**. See `03-execution-modes.md`.

## Pre-push wiring

The hook (`scripts/hooks/pre-push`) calls `reposix-quality run --cadence pre-push`. Internally that includes:
- The hash walker for `doc-alignment`.
- The existing freshness invariants (structure dimension).
- Cargo verdicts.
- The alignment_ratio floor check.

On `STALE_DOCS_DRIFT` or `MISSING_TEST` or any other blocking state, exit code is non-zero. Stderr names the slash command:

```
docs-alignment: STALE_DOCS_DRIFT on docs/reference/confluence.md
push BLOCKED — open Claude and run /reposix-quality-refresh docs/reference/confluence.md
```

## Backfill / refresh dispatch shape

Detailed in `06-p65-backfill-brief.md`. Highlights:

- Sharding is deterministic, ≤3 files per agent, directory-affinity first.
- `plan-backfill` writes `MANIFEST.json` (no LLM); orchestrator dispatches one Task subagent per shard with that shard's manifest line.
- Each shard subagent's output is a per-shard JSON file under `quality/reports/doc-alignment/backfill-<ts>/shards/NNN.json`. Subagents call `bind` / `mark-missing-test` / `propose-retire` to mint state — they never write JSON directly.
- `merge-shards` is deterministic: dedup by `(claim_text_normalized, test)`, multi-citation rows get multiple `source` entries on one row. Conflicts → CONFLICTS.md, exit non-zero.
- Wave-based dispatch (~8 concurrent) to dodge rate limits.
- Two-tier model deployment: Haiku for extractors (per-shard, narrow scope), Opus for grader (per-row, semantic alignment call).

## Naming decisions (do not bikeshed)

- Dimension: **`docs-alignment`** (not docs-coverage; alignment is bidirectional in framing).
- Binary: **`reposix-quality`** (project-prefixed for now; future standalone-spinoff possibility kept open by self-contained design).
- Catalog: **`quality/catalogs/doc-alignment.json`**.
- Skill: **`.claude/skills/reposix-quality-doc-alignment/`**.
- Slash commands: **`/reposix-quality-refresh`** and **`/reposix-quality-backfill`**.
- Hash binary: **`quality/gates/docs-alignment/hash_test_fn`** (compiled Rust binary; lives next to its dimension's other verifiers).
