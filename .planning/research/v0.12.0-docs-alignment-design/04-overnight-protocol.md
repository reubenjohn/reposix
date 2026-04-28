# 04 — Overnight execution protocol

## Mission

Ship P64 (docs-alignment infrastructure) and P65 (docs-alignment backfill) into v0.12.0 before the human wakes up. Then dispatch the milestone-close verifier; if GREEN, the v0.12.0 tag is ready for the human to push (the orchestrator does NOT push the tag).

## Deadline

**08:00 local time** the morning of the run. The orchestrator should target completion by 07:30 to leave a 30-minute buffer for unexpected verifier loops or final ceremony.

## The suspicion-of-haste rule

**Finishing significantly ahead of schedule is a RED FLAG, not a green one.** If the orchestrator believes both phases are complete in under 5 hours of wall-clock work, that is evidence of one of:

- Hallucination of test pass / verdict GREEN without primary-source verification.
- Skipping the catalog-first commit per phase (v0.12.0 QG-06 rule).
- Skipping the verifier subagent dispatch (QG-06).
- Skipping the CLAUDE.md update per phase (QG-07).
- Skipping waves in the P65 backfill (e.g. "I extracted everything in one big subagent call").
- Cargo build/test runs that didn't actually run because cargo memory-budget rules were violated and the orchestrator silently degraded to "pretend it passed."

If wall-clock at the end of P65's verifier dispatch is < 5 hours from session start, the orchestrator MUST:

1. Halt before any tag-related action.
2. Re-run the milestone-close verifier subagent against fresh artifacts.
3. Re-run `quality/runners/run.py --cadence pre-push` and `--cadence pre-pr` and confirm exit 0 from observed stdout (not from cached state files).
4. Spot-check three random catalog rows: read the cited prose, read the cited test, manually verify alignment.
5. If anything inconsistent: pause, write a SURPRISES.md entry, await the human.

## Expected wall-clock

These are calibrated guesses. Use them as a sanity check, not a hard schedule.

| Phase | Expected wall-clock | Notable steps |
|---|---|---|
| Session bootstrap + read this bundle | 10–20 min | Read README → 02-architecture → 03-execution-modes → 04-this-doc → 05-p64 → 06-p65. Do NOT skim. |
| P64 — docs-alignment infrastructure | 4–6 hours | New crate, ~10 subcommands, `syn`-based hash binary, skill files, slash commands, hook wiring, ~3 catalog seed rows, golden test for `merge-shards` dedup, CLAUDE.md update, verifier dispatch. |
| P65 — docs-alignment backfill | 2–3 hours | Top-level execution. plan-backfill, ~30 shards in waves of 8, merge-shards (likely some conflict resolution), punch-list summary, CLAUDE.md update, verifier dispatch. |
| Milestone-close verifier + tag-gate prep | 30–60 min | Read existing `tag-v0.12.0.sh`, ensure new gates pass, dispatch milestone-close verifier subagent, write `quality/reports/verdicts/milestone-v0.12.0/VERDICT.md`. |
| Buffer for surprises | 60+ min | Expect at least one real obstacle. Cargo build conflicts, hash binary bugs, shard conflicts that need real thought. |

**Total expected: 7–10 hours.** If a 10-hour wall-clock window is tight, the orchestrator extends by deferring the buffer items rather than skipping core steps.

## Per-phase non-negotiables (inherited from v0.12.0 QG rules)

For BOTH P64 and P65, every one of these MUST hold or the phase does not ship:

1. **Catalog-first commit.** First commit of each phase writes the catalog rows or schema seed. For P64 that's `quality/catalogs/doc-alignment.json` (initial schema with summary block + zero rows is acceptable; rows added in subsequent commits) plus `quality/catalogs/orphan-scripts.json` waiver entries if any. For P65 the catalog gets populated by the backfill itself.
2. **Mandatory CLAUDE.md update in the same PR.** P64 adds the `docs-alignment` dimension to the matrix in CLAUDE.md, the orchestration-shaped-phases note (see `03-execution-modes.md`), and a P64 H3 subsection. P65 adds a P65 H3 subsection (≤40 lines) with backfill summary + carry-forward notes.
3. **Verifier subagent dispatch on phase close.** Unbiased subagent reads the catalog rows + the verification artifacts with zero session context and writes a verdict to `quality/reports/verdicts/p64/VERDICT.md` and `quality/reports/verdicts/p65/VERDICT.md`. Phase does not close on RED.
4. **CLAUDE.md size budget respected.** Total file ≤40 KB enforced by `~/.git-hooks/pre-commit`. If P64 or P65 would push over, archive an older H3 subsection to `.planning/milestones/v0.12.0-phases/ARCHIVE.md` per the existing pattern.
5. **All cadences GREEN.** Before the verifier subagent is dispatched, `quality/runners/run.py --cadence pre-push` and `--cadence pre-pr` must exit 0. If not, fix or waive (with TTL ≤ 90 days and explicit `dimension_owner`); never paper over.

## Cargo memory-budget rule (project-specific, load-bearing)

Per CLAUDE.md: the VM has crashed twice from RAM pressure caused by parallel cargo. **Never run more than one cargo invocation at a time.** Doc-only / planning-only subagents can run in parallel with one cargo subagent. Two cargo subagents running concurrently is forbidden.

For P64 this means:
- `cargo check -p reposix-quality` for incremental work.
- `cargo test -p reposix-quality` for unit tests.
- Workspace-wide `cargo check --workspace` only at phase end, once, before verifier.
- If the planner suggests parallelizing two cargo invocations, refuse.

## Verifier subagent dispatch (Path A vs Path B)

Per the project's existing P56–P63 precedent: `Task` is unavailable in `gsd-executor`. The verifier dispatch is therefore via Path B (in-session Claude grading) with explicit disclosure in the verdict artifact: `dispatched_via: P64-Path-B-in-session` (or P65 / milestone-close as appropriate). This is the documented pattern; do NOT attempt Path A from inside an executor session.

For P65 (top-level execution mode), the orchestrator IS at depth 0 and CAN dispatch via Path A (real Task tool). Use Path A for P65's verifier.

## Commit cadence

Atomic commits per logical step (consistent with `gsd-executor` defaults):
- P64 first commit: catalog schema + `crates/reposix-quality/Cargo.toml` + lib skeleton.
- P64 subsequent commits: one per `clap` subcommand, one for the hash binary, one for the skill files, one for hook wiring, one for tests, one for CLAUDE.md update, one for verifier verdict.
- P65 first commit: `MANIFEST.json` (the chunker output, deterministic).
- P65 subsequent commits: backfill run dir (per-shard JSONs + MERGE.md), then catalog written by `merge-shards`, then CLAUDE.md update, then verdict.

Pre-push must pass for every commit. Use `git commit --no-verify` ONLY if the failure is itself the catalog-first commit's known interim state, and explain in the commit message.

## When stuck

The project's `quality/SURPRISES.md` is an append-only journal. If a real obstacle appears (a design assumption breaks, a tool can't do what we need, a test pattern doesn't match the codebase), append a short entry naming the obstacle, the attempted workaround, and the resolution. Do not delete or revise prior entries.

If the obstacle is severe enough that completing on time is in doubt:

1. Stop forward motion.
2. Write the SURPRISES entry.
3. Pause the autonomous loop and write a checkpoint at `.planning/STATE.md` describing exactly where you stopped and what the next agent should do.
4. The human will resume in the morning.

A handed-off-with-honesty checkpoint is infinitely better than a hallucinated GREEN.

## What NOT to do

- Do NOT push any git tag, ever. The owner pushes tags.
- Do NOT push to remote unless atomic commits succeed AND pre-push passes for every commit.
- Do NOT fall back to `claude -p` for subagent dispatch — the owner is on a subscription.
- Do NOT "fix" the v0.12.1 ROADMAP renumbering by deleting placeholder phases. Bump them up by 2 (P64→P66, P65→P67, etc.) and add the doc-alignment gap-closure stubs at P71+.
- Do NOT attempt to fix the Confluence symlink regression itself. P65 surfaces it as a `MISSING_TEST` row. Closing it is v0.12.1 work.
- Do NOT silently expand scope. If you discover something interesting that is not in P64 or P65 brief, file it as a v0.12.1 carry-forward in `.planning/milestones/v0.12.1-phases/REQUIREMENTS.md` and continue.
