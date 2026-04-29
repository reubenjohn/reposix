# quality/SURPRISES.md — append-only pivot journal

Per `.planning/research/v0.12.0-autonomous-execution-protocol.md` § "SURPRISES.md format": append one line per unexpected obstacle + its one-line resolution. **Required reading for every phase agent at start of phase.** The next agent does NOT repeat investigations of things already journaled here.

Format: `YYYY-MM-DD P<N>: <obstacle> — <one-line resolution>`.

Anti-bloat: ≤200 lines. When the file crosses 200 lines, archive the oldest 50 entries to `quality/SURPRISES-archive-YYYY-QN.md` and start fresh — see `quality/PROTOCOL.md` § "Anti-bloat rules per surface".

## Ownership

P56 seeded this file at phase close (5 entries; commit `87cd1c3`). **P57 takes ownership 2026-04-27** as part of the Quality Gates skeleton landing. From P57 onward, this file is referenced by `quality/PROTOCOL.md` § "SURPRISES.md format" as the canonical pivot journal.

**Archive rotations:** past rotation history retained in git log. The 2026-04-29 prune deleted `quality/SURPRISES-archive-2026-Q2.md` after auditing every archived entry as codified into CLAUDE.md / `quality/PROTOCOL.md` / catalog rows / verifier scripts / v0.12.1 REQUIREMENTS.md carry-forwards. Recoverable via `git show <pre-prune-sha>:quality/SURPRISES-archive-2026-Q2.md` if forensic detail is needed.

---

_Compressed 2026-04-29 from multi-paragraph entries; per-line format per file's own header rule._

2026-04-28 P64 Wave 3: walker writes `summary.last_walked` on every invocation, mutating `doc-alignment.json` even when rows == [] — accepted for v0.12.0; v0.12.1 MIGRATE-03 either moves field to artifact or extends `catalog_dirty()` to ignore it.
2026-04-28 P65: backfill shard 016 wrote 17 rows to live catalog instead of shard catalog (default-path bug) — recovered via jq move + reset; v0.12.1 MIGRATE-03 (j) makes binary refuse default-path mutation when invoked via skill.
2026-04-28 P65: walker waiver scope mismatch — `summary.floor_waiver` only covers `alignment_ratio<floor` BLOCK, not `MISSING_TEST`/`RETIRE_PROPOSED` rows; resolved via separate row-level waiver on `docs-alignment/walk` in `freshness-invariants.json` (TTL 2026-07-31); v0.12.1 should consolidate behind one initial-backfill grace period.
2026-04-28 P65: backfill envelope overshoot (388 rows vs planned 100-200) driven by glossary doc extracting 24 RETIRE_PROPOSED rows (one per term) — under "wildly off" threshold, no halt; future backfills should bias-conservative or filter definitional/glossary docs from the manifest.
2026-04-28 v0.12.1: orchestrator falsely logged a safeguard breach (`confirm-retire` running from agent context) — retracted; owner had run the bulk-confirm script from a real TTY in parallel. Lesson: check git reflog and ask the owner before logging a safeguard breach.
