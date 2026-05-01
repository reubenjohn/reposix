# Extractor subagent prompt — docs-alignment backfill

> Loaded by the `reposix-quality-doc-alignment` skill in `backfill` mode, prepended to each shard's per-Task prompt. Orchestrator appends the manifest line (input files + row-id namespace) and dispatches one Haiku Task per shard.

## Your role

You have ZERO session context beyond this prompt and your manifest line. You are an unbiased extractor. Read the cited files, identify behavioral claims with file:line citations, find binding tests, and mint catalog state via the `reposix-quality` binary — never write JSON by hand.

## What counts as a claim

A behavioral claim is a statement specific enough that **a test could fail**. It is a contract between docs and implementation. If the prose is just background, narrative, or motivation, do NOT extract a row.

- BAD (too vague): "Reposix is fast." / "The simulator is the default."
- GOOD: "`reposix init sim::demo /tmp/repo` completes in <100ms on a warm cache."
- GOOD: "When `REPOSIX_ALLOWED_ORIGINS` is unset, only `http://127.0.0.1:*` origins resolve."
- GOOD: "`git push` to a backend that drifted returns the standard git 'fetch first' error."
- GOOD: "JIRA stories have a `parent` symlink pointing to their epic."

## Procedure

1. **Read each input file in your shard.** Do not skim; read line-by-line.

2. **For each candidate claim:**
   - Compose a stable row ID: `<dimension-area>/<verb>-<short-noun>` (e.g. `jira-init-story-parent-symlink`, `allowlist-default-localhost-only`).
   - Cite the source as `<file>:<line-start>-<line-end>` covering the smallest contiguous range that contains the claim.
   - Search `tests/`, `crates/*/tests/`, `crates/*/src/**/tests.rs` for a binding test:
     ```
     grep -rln "<keyword from claim>" crates/ tests/ scripts/
     ```
   - If a candidate test exists, READ THE TEST BODY. Verify the test actually asserts the claim (assertion text contains the value, not just calls a function whose name happens to match). When uncertain, treat as MISSING_TEST — false BOUND is the worst failure mode.

3. **Mint state via the binary** (one call per claim):

   - **Found a binding test:**
     ```
     reposix-quality doc-alignment bind \
       --row-id <stable-id> \
       --claim "<one-line claim text>" \
       --source <file>:<lstart>-<lend> \
       --test <test-file>::<fn-name> \
       --grade GREEN \
       --rationale "<file>:<line> of the test asserts <quoted assertion text>"
     ```
     The binary validates citations, computes hashes, and persists. If it errors, the citation is wrong — fix and retry.

   - **No binding test exists:**
     ```
     reposix-quality doc-alignment mark-missing-test \
       --row-id <stable-id> \
       --claim "<one-line claim text>" \
       --source <file>:<lstart>-<lend> \
       --rationale "IMPL_GAP: <where impl exists or what's needed>"
     ```
     The binary derives `next_action` from the rationale prefix
     (`IMPL_GAP:` -> `FIX_IMPL_THEN_BIND`, `DOC_DRIFT:` -> `UPDATE_DOC`,
     otherwise `WRITE_TEST`). Pass `--next-action <value>` only when you
     need to override the heuristic. See the "next_action field" section
     below for the full mapping.

   - **Claim is clearly superseded by a documented architecture decision:**
     ```
     reposix-quality doc-alignment propose-retire \
       --row-id <stable-id> \
       --claim "<one-line claim text>" \
       --source <file>:<lstart>-<lend> \
       --rationale "Superseded by <supersession-doc>:<line>; <one-sentence why>"
     ```
     RETIREMENT IS THE MOST EXPENSIVE OPTION. Only propose retirement when the supersession is documented in a checked-in artifact (e.g. `architecture-pivot-summary.md` retiring FUSE behavior). "I couldn't find a test" is NOT a reason to propose retirement — that's `mark-missing-test`. **Before proposing retirement, read the "Retirement vs implementation-gap" section below.**

4. **Output is the cumulative effect of your `reposix-quality` calls.** Do NOT write a prose summary, do NOT write JSON files yourself, do NOT update the catalog directly. The binary is the only legitimate writer.

5. **When done**, print a one-line summary on stdout:
   ```
   shard <NNN>: <total> rows, <bound> BOUND, <missing> MISSING_TEST, <retire> RETIRE_PROPOSED
   ```

## Retirement vs implementation-gap

**Retirement requires the FEATURE to be intentionally dropped with a documented decision.** Transport / implementation-strategy changes do NOT retire claims about a user-facing surface — those remain `MISSING_TEST` (gap-closure work for the next implementation strategy). If the prose describes WHAT the user sees (directory layout, filename pattern, CLI verb output, round-trip behavior) and only HOW the project produces that surface changed, the row is `MISSING_TEST` with rationale prefix `IMPL_GAP:` — NOT `RETIRE_PROPOSED`.

Decision tree (default to `mark-missing-test`):

1. Does the claim describe a user-facing shape (directory layout, blob path, verb output, round-trip behavior, error string)? → if YES, the feature is alive even if the transport changed. Use `mark-missing-test` with rationale `IMPL_GAP: <where impl exists or what's needed>`.
2. Does the claim describe an implementation detail tied to a specific transport (e.g. "FUSE write callback", "page-tree symlink mount") that is NOT externally observable beyond what (1) covers? → likely `mark-missing-test` with rationale `DOC_DRIFT: <prose names old transport; rebind to current transport>`.
3. Is there an ADR, CHANGELOG entry, or research note that explicitly says "we are no longer providing X" where X is the user-facing surface? → only then `propose-retire`, citing the document in the rationale.
4. Default if uncertain: `mark-missing-test`. Retirement is the most expensive option.

**Rationale-prefix convention** for `mark-missing-test` (downstream filtering depends on it):

- `IMPL_GAP: <details>` — feature alive, implementation strategy changed or never landed; resolution = bind to existing/new code or write ADR retiring.
- `DOC_DRIFT: <details>` — prose names a stale shape; resolution = update doc OR rebind to current shape.
- (no prefix) — straightforward "test missing for a clear claim."

## next_action field (W4 / v0.12.1 P68)

Every row carries a structured `next_action` field that names the action that closes the gap. Cluster-closure phases (P72-P80) filter by this field, so getting it right is what turns "166 MISSING_TEST rows" into a small set of actionable cluster phases.

The five enum variants:

- `WRITE_TEST` — test missing for a clear claim; write the test. **Default** for plain `mark-missing-test` (no rationale prefix).
- `FIX_IMPL_THEN_BIND` — implementation regressed or never landed; fix impl, then bind.
- `UPDATE_DOC` — prose names a stale shape; update the doc, then rebind.
- `RETIRE_FEATURE` — feature was intentionally dropped; needs `RETIRE_PROPOSED` -> `RETIRE_CONFIRMED`.
- `BIND_GREEN` — already bound to a green test; nothing to do.

**Heuristic** — when calling `mark-missing-test`, the binary derives `next_action` from the rationale prefix automatically:

| Rationale prefix | Implied `next_action` |
| --- | --- |
| `IMPL_GAP: <details>` | `FIX_IMPL_THEN_BIND` |
| `DOC_DRIFT: <details>` | `UPDATE_DOC` |
| (no prefix) | `WRITE_TEST` |

`bind` always sets `next_action=BIND_GREEN`. `propose-retire` always sets `next_action=RETIRE_FEATURE`. You do NOT pass `--next-action` to those verbs; the value is implicit.

**Override flag** — `mark-missing-test --next-action <value>` overrides the heuristic when the rationale prefix would guess wrong (e.g. rationale begins `IMPL_GAP:` but the doc itself drifted, so `UPDATE_DOC` is the closer). The override is rare; reach for it only when you've thought about the closure path and the prefix would guess wrong.

### Canonical examples (v0.12.1 audit corrections, commit `24b2b62`)

Four rows initially proposed `RETIRE_PROPOSED` but flipped to `MISSING_TEST` — user-facing surface persists across the FUSE → git-native pivot. Lesson: prose naming an old TRANSPORT is not a retirement signal; the signal is a documented decision to drop a user-facing SURFACE.

| Row | Prose (abridged) | Wrong call | Right prefix + resolution |
| --- | --- | --- | --- |
| `req-confluence-create-page-via-write` | edit `.md`, page updates upstream | retire (FUSE dropped) | `IMPL_GAP:` — impl at `confluence/lib.rs:280` + `remote/main.rs:513`; bind test in `agent_flow_real`. |
| `req-tree-index-overview-blob` | `cat mount/_INDEX.md` returns overview | retire (FUSE synthesizer removed) | `IMPL_GAP:` — user-facing shape, not transport; reimplement in `reposix-cache` OR ADR-retire. |
| `req-confluence-init-page-tree-mount` | `spaces/<KEY>/` exists after init | retire ("FUSE-mount" in prose) | `IMPL_GAP:` — implement multi-space init materializing `spaces/<KEY>/` OR ADR-retire. |
| `req-jira-readonly-phase28` (counter-ex: `DOC_DRIFT:`) | JIRA writes return `not supported` | retire (prose stale) | `DOC_DRIFT:` — Phase 29 shipped writes (`jira/lib.rs:8,197,279,334`); update `docs/reference/jira.md` §Limitations, rebind to `dark_factory_real_jira`. |

## Hard rules

- Read every cited file. Don't extract from filenames or section headers alone.
- Citations must be the smallest contiguous line range containing the claim. Don't cite an entire file or a 50-line block.
- Row IDs are stable across re-runs. Use the same kebab-case ID for the same claim. Conflicts surface in `merge-shards`; the orchestrator resolves them.
- Never call `confirm-retire`. That's human-only and env-guarded.
- If you hit a binary error you don't understand, surface it on stderr and continue with the next claim. Do not silently skip rows.

## Cross-references

- Catalog schema: `quality/catalogs/doc-alignment.json` + `quality/catalogs/README.md`
- Hash semantics + row state machine: `.planning/research/v0.12.0-docs-alignment-design/02-architecture.md`
