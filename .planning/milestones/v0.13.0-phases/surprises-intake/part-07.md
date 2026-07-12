# v0.13.0 Surprises Intake (P96 source-of-truth) — Part 7 of 7

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-11 | S-260711-waiverclear-promotion — cleared-waiver container rows never auto-promote; post-release verdict reds on stale committed status (SECOND occurrence) | discovered-by: post-release verdict mint (cargo-binstall row) | severity: MEDIUM

**What:** When a `container`/release-assets row's waiver clears, nothing auto-promotes it
out of `NOT-VERIFIED`. Post-release `verdict.py` grades the COMMITTED status (by design,
commit 2359c63 — cadence runs don't self-mutate the catalog), NOT the fresh CI run that
just passed; and post-release `run.py` runs without `--persist`. So a cleared-waiver
container row sits `NOT-VERIFIED` and spuriously reds the whole post-release workflow until
a human manually mints it — exactly this incident (`release/cargo-binstall-resolves`, gate
genuinely PASSES in CI job 85772845548, ~1.93s dry-run). This is the SECOND time the
freshness-vs-committed-status split has bitten us.

**Why out-of-scope:** the fix is an architectural/process choice (checklist vs. verdict
consuming the fresh run artifact), beyond a single-row mint.

**Sketched resolution:** either (a) a documented "mint after waiver-clear" checklist step in
the release runbook, or (b) post-release `verdict.py` consumes the fresh run artifact for
`container` rows (grade the run that just executed, not committed status). Route v0.14.0
CI-honesty hardening; note recurrence.

**Default disposition:** MEDIUM — recurring spurious RED; masks real signal via alert fatigue.

**STATUS:** OPEN

## 2026-07-11 | S-260711-stale-binstall-artifact — local `cargo-binstall-resolves.json` records a FAIL "cargo-binstall not installed" beside a PASS row | discovered-by: post-release verdict mint (cargo-binstall row) | severity: LOW

**What:** `quality/reports/verifications/release/cargo-binstall-resolves.json` records FAIL
"cargo-binstall not installed" from a 2026-07-06 LOCAL run — misleading next to the now-PASS
catalog row. Confirmed NOT tracked in git (`git ls-files --error-unmatch` → not-tracked), so
it is local noise only, not a committed lying artifact. Left in place (not deleted).

**Why out-of-scope:** local untracked artifact; deleting it is out of scope for this dispatch
and it carries no committed-repo impact.

**Sketched resolution:** ensure local verification runs either skip-report or clearly mark
"local, cargo-binstall absent" rather than emitting a bare FAIL that outlives the run.

**Default disposition:** LOW — local-only cosmetic confusion.

**STATUS:** OPEN

## 2026-07-11 | S-260711-docalign-walk-mutation — doc-alignment walker self-mutates its own catalog at walk-time | discovered-by: post-release verdict mint (cargo-binstall row) | severity: LOW

**What:** `quality/catalogs/doc-alignment.json` shows persistent unstaged drift because the
docs-alignment walker MUTATES its own catalog during a walk/grade run (walk-time
side-effect). A read/grade run should be side-effect-free; self-mutation makes every walk
dirty the tree and muddies `git status` for unrelated work.

**Why out-of-scope:** fixing the walker's persistence behavior is a code change beyond a
catalog-mint + intake dispatch.

**Sketched resolution:** the walker should not self-mutate on a read/grade run — separate the
`walk` (grade, read-only) path from an explicit `--persist`/`plan-refresh` write path. Route
v0.14.0 docs-alignment hardening.

**Default disposition:** LOW — no data loss; recurring unstaged-drift noise.

**STATUS:** OPEN
