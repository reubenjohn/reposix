# v0.12.1 HANDOVER — entry point for the next session

> **Eventually delete this file.** It's a session-bridge artifact. The
> next session reads it cold, executes the queued work, and the file
> deletes itself when the queued items all close (last phase commit
> includes `git rm .planning/HANDOVER-v0.12.1.md`).

**Created:** 2026-04-28T07:53Z by overnight orchestrator.
**Deadline:** 2026-04-28 17:00 PT (next business day close-of-day).
**Owner:** reuben.

---

## TL;DR for the next session

1. **v0.12.0 is held at the tag boundary.** Owner decides G1 (workspace `Cargo.toml` 0.11.3 vs tag-gate Guard 3 expects 0.12.0). See `quality/reports/verdicts/milestone-v0.12.0/VERDICT.md` § Gap-block G1.
2. **v0.12.1 is in flight.** P66 (coverage_ratio metric) shipped this session. P67-P71 placeholders renumbered. P72+ cluster phases TBD per scoping below.
3. **Pre-push correctly BLOCKs** on misaligned rows (since hook fix `fdb4d24`). You can't push until cluster-closure phases lift `alignment_ratio` ≥ 0.50 AND clear blocking row states.
4. **Local is ahead of origin by 9 commits.** Owner should `git pull --rebase` BEFORE pushing — the previous attempted push failed with stale-base ref-lock (origin moved during this session via release-plz or similar).

## Live state snapshot

```
HEAD = db7366d  (P66 SHIPPED)
local ahead of origin by 9 commits

quality/catalogs/doc-alignment.json:
  claims_total          388
  claims_bound          171
  claims_missing_test   166  ← BLOCKING walker
  claims_retire_proposed 41  ← BLOCKING walker; 24 glossary + 17 audit candidates + 1 corrected
  claims_retired          0  ← becomes 24 after glossary bulk-confirm
  alignment_ratio       0.4407  ← BLOCKING (floor 0.5000)
  coverage_ratio        0.2055  ← PASS  (floor 0.1000)

pre-push:    21 PASS, 1 FAIL, 3 WAIVED  →  exit=1  (push blocks)
```

---

## Queued work, in dependency order

### Immediate (≤30 min, no fresh subagent dispatch needed)

#### W1 — Bulk-confirm 24 glossary retirements

Owner runs from a real TTY:

```bash
bash scripts/v0.12.1-confirm-glossary-retires.sh
```

Script lives at that path; reads the 24 row IDs via `jq` from the live catalog. Loops `confirm-retire`. Uses `--row-id` per call. After: commit the catalog mutation. Expected effect: `claims_retire_proposed` 41 → 17, `claims_retired` 0 → 24, `alignment_ratio` slightly bumps (denominator drops by 24).

#### W2 — Apply 17-row audit recommendations from RETIRE-AUDIT.md

A subagent (Opus, Path A) is reviewing the 17 non-glossary RETIRE_PROPOSED rows in background as of 2026-04-28T07:50Z. Output lands at:

```
quality/reports/doc-alignment/backfill-20260428T085523Z/RETIRE-AUDIT.md
```

The doc will contain per-row recommendations (CONFIRM_RETIRE / FLIP_TO_MISSING_TEST_IMPL_GAP / etc.) PLUS two ready-to-run scripts:
- `confirm-retire` script for legitimate retirements (owner runs from TTY).
- `mark-missing-test` script for over-retired rows (orchestrator runs from this session; not env-guarded).

Estimated 3-5 confirm-retires + 12-14 flips. Apply both scripts. Commit.

### Short (1-2h each; subagent-dispatchable)

#### W3 — P67: RETIRE audit + extractor prompt update

Currently a renumbered placeholder. Repurpose: this phase OWNS W2 (apply audit findings) plus updates `.claude/skills/reposix-quality-doc-alignment/prompts/extractor.md` to teach the transport-vs-feature distinction:

> Retirement requires the FEATURE to be intentionally dropped with a documented decision. Transport / implementation-strategy changes do NOT retire claims about user-facing surface — those remain MISSING_TEST and become gap-closure work for the next implementation strategy.

Proposed phase scope: catalog correction (W2 above) + extractor prompt update + 1 regression test (re-run `plan-refresh` on a doc with FUSE-era prose; assert no new RETIRE_PROPOSED proposals).

#### W4 — P68: `next_action` field schema extension

Add `next_action: enum { WRITE_TEST, FIX_IMPL_THEN_BIND, UPDATE_DOC, RETIRE_FEATURE, BIND_GREEN }` to `Row` struct. Default `WRITE_TEST` (back-compat for existing populated rows). Update extractor prompt to set the field appropriately. Update `status` and `--json` to display. One-time backfill script walks existing 388 rows + reassigns `next_action` heuristically (RETIRE_PROPOSED → RETIRE_FEATURE; rationale prefix `IMPL_GAP:` → FIX_IMPL_THEN_BIND; default → WRITE_TEST).

#### W5 — P69: `confirm-retire --i-am-human` flag

Owner explicitly authorizes retirement via flag (audit-trailed in catalog row's `last_extracted_by` field). Lets human authorize from a Claude Code session without leaving for a fresh terminal. Small Rust change + test asserting the flag's audit trail is preserved.

#### W6 — P70: hook self-test extension

Extend `scripts/test-pre-push.sh` (currently only verifies PASS-path) to also force a runner FAIL and assert the hook propagates exit non-zero. The recent `fdb4d24` hook fix was invisible to the existing test because the test asserts behavior on the PASSING side only. Companion: audit `~/.git-hooks/pre-push` (personal global) for the same `if ! cmd; exit $?` pattern.

#### W7 — P71: Schema cross-cut consolidation (MIGRATE-03 (i))

Two cross-cuts in the binary surface:

1. `bind` writes `Row.source` as `SourceCite` object; `merge-shards` reads `Source` enum (Single|Multi). Reconciled mid-flight via jq during P65; should be unified at the type level.
2. `Row.test` is `String` but some shards emitted multi-test arrays. Should be `Vec<String>` to support multi-test claims first-class.

3. (NEW finding from this session) `Row.rationale` is `Option<String>` in the writer side but the walker's deserialize was failing on missing rationale. Verify whether this is a serde back-compat issue or a writer bug. Test required: walker round-trips a catalog with rows lacking `rationale`.

4. (NEW finding) `FloorWaiver` struct expects fields `{until, rationale}` but the design brief had `{until, reason, dimension_owner}`. Pick one consistent shape; update either the brief, the schema spec, or the struct.

#### W8 — P72+: Cluster-closure phases per PUNCH-LIST.md

The 14 clusters identified in `quality/reports/doc-alignment/backfill-20260428T085523Z/PUNCH-LIST.md` need closure. After W2 (audit corrections), some MISSING_TEST counts shift. Re-read PUNCH-LIST.md (or regenerate via `python3 scripts/gen_punch_list.py quality/reports/doc-alignment/backfill-20260428T085523Z/`) before scoping P72+ phases.

Likely cluster ordering by leverage:

- **P72 — Confluence backend parity (smoking gun).** ~15 rows including `docs/reference/confluence.md` FUSE-era stale section + ADR-002/003 nested-shape promises (after W2 flip). Two paths per row: fix impl OR update doc. Cluster phase scope is the resolution.
- **P73 — JIRA shape.** ~10 rows. ADR-005, `docs/reference/jira.md` Phase 28 read-only stale (W2 confirms retire), `parent` symlink claim if any.
- **P74 — Benchmark numbers.** 20 MISSING_TEST. Either Rust-port `quality/gates/perf/{latency-bench.sh, bench_token_economy.py}` so each row binds to `<file>::<fn>`, OR extend the binary to accept Python verifier paths. Includes drift fixes: `~92%` (social) → measured 89.1%; `24 ms cold init` (mental-model) → measured 27 ms.
- **P75 — Connector authoring guide.** 24 MISSING_TEST. Trait method contracts asserted in code without named test fns. Add `#[test] fn backendconnector_supports_required_methods()` style harnesses.
- **P76 — Tutorial first-run.** 6 MISSING_TEST steps 4-8 (checkout, edit, push, audit). Integration test extending `dark_factory_sim_happy_path`.
- **P77 — Developer workflow + invariants.** 17 rows. Policy invariants (`#![forbid(unsafe_code)]`, MSRV, cargo test count) needing bespoke verifiers. Some collapse into a single shell-grep verifier under `quality/gates/code/`.
- **P78 — Concepts (mental model).** 13 MISSING_TEST. Includes the 24ms vs 27ms drift.
- **P79 — Internals + research notes.** Mostly already BOUND; small cleanup.
- **P80 — Coverage chunker redirect-following.** New finding from P66: `docs/connectors/guide.md`, `docs/security.md`, `docs/why.md`, `docs/reference/crates.md` show 0 rows in coverage even though shards extracted from their redirect targets. Either the chunker should follow redirects, OR rows should track the prose-source file rather than the canonical-redirect file. Pick one.

After P72-P80 land, alignment_ratio + coverage_ratio both lift. Re-dispatch the milestone-close verifier; v0.12.1 ships.

### Long-tail

#### W9 — Walker BLOCK granularity (per-priority gating)

Currently the walker BLOCKs on ANY MISSING_TEST/RETIRE_PROPOSED row. Once `next_action` lands (W4), the walker could be smarter: e.g. exit non-zero ONLY on rows where the `next_action` is overdue (e.g. `FIX_IMPL_THEN_BIND` aged > 90 days), and treat fresh rows as warnings (P2). This prevents v0.12.x+ milestones from inheriting all 166 rows as P0 blockers.

Scope as v0.13.0 work — too aggressive for v0.12.1 unless the cluster phases finish faster than expected.

#### W10 — Walker `last_walked` artifact promotion (MIGRATE-03 (h))

Walker writes `summary.last_walked` on every invocation, mutating the catalog file. This produces git churn on every pre-push. Either move to the artifact at `quality/reports/verifications/docs-alignment/walk.json` OR extend `catalog_dirty()` to ignore `summary.last_walked` drift.

#### W11 — Subagent default-catalog refusal (MIGRATE-03 (j))

Shard 016 in P65 wrote 17 rows to the LIVE catalog instead of its shard catalog because the agent forgot the `--catalog <shard-path>` flag. Binary should refuse to mutate the default-path catalog when invoked under a known subagent context (env-guard or required-flag pattern).

---

## Action classification debate (W4 above)

The owner asked: *"does adding `next_action` add too much burden on the agent classifying or does it encourage more reasoning and accuracy?"*

**My recommendation: include it.** Reasoning:

- **Default-WRITE_TEST means inattentive extractors produce correct output by accident.** A careful extractor produces structured signal. Net positive.
- **Consumer-side win is large.** Punch list + cluster-phase scoping become filterable. v0.12.1 can scope phases as "close all FIX_IMPL_THEN_BIND" rather than reading 166 rationale fields by hand.
- **Existing failure mode the field would fix:** The `IMPL_GAP:` rationale-prefix convention this session is informal and grep-only. Structured field formalizes it.
- **Cost is one prompt instruction + one catalog field.** Trivial schema change.

The field is on the *Row*, not the verdict. Verdict (`last_verdict`) stays orthogonal — same row can be MISSING_TEST + WRITE_TEST (test missing, just write it) OR MISSING_TEST + FIX_IMPL_THEN_BIND (test missing because impl regressed; fix both). Both axes are needed.

---

## Time budget

| Block | Estimate | Cumulative |
|---|---|---|
| W1 (glossary bulk-confirm) | 5 min | 5 min |
| W2 (apply audit) | 15 min | 20 min |
| W3 P67 (audit + extractor prompt) | 1h | 1h20min |
| W4 P68 (next_action field) | 1.5h | 2h50min |
| W5 P69 (--i-am-human flag) | 30 min | 3h20min |
| W6 P70 (hook self-test) | 30 min | 3h50min |
| W7 P71 (schema cross-cut) | 1h | 4h50min |
| W8 P72-P80 (cluster phases) | 6-8h | 10h50min - 12h50min |
| Milestone-close verifier | 30 min | + |

5pm deadline = ~9h from session start. Realistic: W1-W7 + 2-3 cluster phases ship by 5pm. Remaining clusters become v0.12.1.x or are rolled into a follow-on session.

**Recommended phase order for next session:** W1, W2 (consume the audit), W3, W6 (small + unblocks future hook regressions). Then W4 (structured field) BEFORE the cluster phases, because each cluster phase benefits from the structured `next_action`.

---

## v0.12.0 G1 reminder

**The orchestrator does NOT push the v0.12.0 tag.** Status remains
`ready-to-tag-pending-owner-decision` until owner picks Path A or B
(see `quality/reports/verdicts/milestone-v0.12.0/VERDICT.md` § Gap-block
G1). v0.12.1 work proceeds in parallel; the tag-cut waits.

---

## Cleanup criterion

This file deletes itself when:
- W1 and W2 closed (catalog at expected ratios for v0.12.1).
- P67-P71 (W3-W7) shipped GREEN at per-phase verifier.
- A v0.12.1-specific HANDOVER (or none — STATE.md alone) replaces it.

The phase that ships W7 (or whichever closes the last item above) includes `git rm .planning/HANDOVER-v0.12.1.md` in its closing commit.
