# Deferred items — P115 (out-of-scope discoveries, not fixed in-lane)

Per the executor scope boundary: pre-existing warnings unrelated to the current task's
changes are logged here, not fixed. This file is phase-scoped, not milestone-scoped
(contrast `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md`, which is for
genuine surprises the discovering session chose not to fix eagerly).

## 2026-07-16 | discovered-by: P115-T6 Wave 2 item 2 executor (post-commit pre-commit-hook output)

**What:** `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` is **43,804 chars**,
more than double the `structure/file-size-limits` 20,000-char `*.md` ceiling. Surfaced as
a WARN (not a blocking FAIL — `structure/file-size-limits` is itself `WAIVED` until
2026-08-08 per `quality/catalogs/freshness-invariants.json`) on this lane's commit, but
the breach predates this lane's edit — my item-2 addition to the file was only ~2.4k
chars of the 43.8k total, so the file was already roughly double the limit before this
commit.

**Why out-of-scope:** This is a pre-existing file-size violation unrelated to this lane's
charter (write `115-UNWAIVE-PATH.md`, file one intake row, commit+push). The existing
`2026-07-14 21:00` SURPRISES-INTAKE entry already flags the SAME class of problem for
`GOOD-TO-HAVES.md` (v0.14.0 and v0.15.0 variants, 27629 / 23584 chars) with a sketched
progressive-disclosure split — but does NOT cover `SURPRISES-INTAKE.md` itself, which is
now the largest offender of the three. Splitting this live-append-only file (used by
every session, referenced by row IDs) is a bigger structural change than an item-2 lane
should make unilaterally.

**Sketched resolution:** Apply the same progressive-disclosure split already sketched
for `GOOD-TO-HAVES.md` in the `2026-07-14 21:00` entry — split into a lean live ledger +
a `SURPRISES-INTAKE-history.md` (or `-archive.md`) companion, moving RESOLVED/DEFERRED/
WONTFIX entries out while preserving cross-refs. Natural home: same Arc D v0.17
"bloat remediation" bucket the `GOOD-TO-HAVES.md` entry already targets — recommend
folding this into that same drain-phase item rather than opening a fourth separate one,
since it is the identical fix pattern applied to a third file.

**STATUS:** OPEN
