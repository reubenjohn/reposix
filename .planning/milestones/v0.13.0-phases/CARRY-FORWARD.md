# v0.13.0 carry-forward intake

Items deferred from prior milestones that v0.13.0 phases should pick up.
One H2 per item; cite the originating phase + requirement.

## MULTI-SOURCE-WATCH-01 — walker hashes every source from `Source::Multi`

**Source:** v0.12.1 P75 (`BIND-VERB-FIX-01`) shipped path (a) — preserve
first-source hash on `Source::Single → Source::Multi` promotion. Path (b)
(walker iterates every source citation in a `Multi` row, hashes each,
ANDs the results) was deferred to keep P75 single-phase.

**Why deferred:** path (b) requires a schema migration (`source_hash:
Option<String>` → `source_hashes: Vec<String>`) with a parallel-array
invariant on `Multi` rows, plus migration of the populated 388-row
catalog and a walker compare-loop refactor. Out of scope for a
single-phase fix.

**Acceptance:**

- `Row::source_hashes: Vec<String>` parallel-array to `source.as_slice()`.
- `verbs::walk` hashes each source citation against its corresponding
  `source_hashes[i]`; row enters `STALE_DOCS_DRIFT` on ANY index drift.
- `verbs::bind` writes/preserves all entries on the parallel array
  (Single result → 1-element vec; Multi append → push the new hash;
  Multi same-source rebind → refresh that index only).
- Existing single-source-hash field migrates via `serde(default)` +
  a one-time backfill (read `source_hash` if present, push it into
  `source_hashes[0]`).
- Regression tests in `crates/reposix-quality/tests/walk.rs` exercise
  the path-(b) "non-first source drift fires STALE" case.

**Carries from:** v0.12.1 phase 75 (`BIND-VERB-FIX-01`); see
`.planning/phases/75-bind-verb-hash-fix/PLAN.md` and
`.planning/phases/75-bind-verb-hash-fix/SUMMARY.md`.

**Owner:** unassigned. Pick up under v0.13.0 docs-alignment dimension.

## GIX-YANKED-PIN-01 — bump gix off yanked 0.82.0 baseline

**Source:** GitHub issues #29 (`gix 0.82.0 yanked`) + #30 (`gix-actor 0.40.1 yanked`),
filed 2026-04-28; surfaced 2026-04-30 by CI-monitor subagent during v0.13.0 kickoff.

**Why P0:** the `=`-pin on gix is load-bearing per `CLAUDE.md` § Tech stack
(gix is pre-1.0 → semver surprises possible). A yanked pin in a load-bearing
dep is a structural risk for the DVCS milestone, which extends the cache
materialization path significantly. Bumping cleanly NOW means v0.13.0 builds
against a stable baseline; bumping LATER under bus-remote pressure invites
conflated debugging.

**Acceptance:**
- `crates/*/Cargo.toml` `gix = "=0.82.0"` → next non-yanked release.
- `gix-actor` and any other `=`-pinned gix-family crates aligned.
- `cargo check --workspace` GREEN (single invocation per CLAUDE.md "Build memory budget").
- `cargo nextest run --workspace` GREEN (per-crate if memory pressure).
- CLAUDE.md § Tech stack updated to cite the new version.
- Issues #29 + #30 closed with the bump SHA.

**Owner:** v0.13.0 P0 (Phase 0 — pre-attach hygiene). Decision ratified by
owner 2026-04-30; see `.planning/research/v0.13.0-dvcs/decisions.md` § "gix
yanked-pin".

## WAIVED-STRUCTURE-ROWS-03 — land 3 verifier scripts before 2026-05-15

**Source:** `quality/catalogs/freshness-invariants.json` — three rows in
`status: WAIVED` with `waiver.until: 2026-05-15T00:00:00Z`. Tracked in P62
Wave 3 (POLISH-ORG); waiver is short-lived by design.

**Why P0/P1:** waiver auto-renewal would defeat the catalog-first principle
(rows defining a green contract whose verifier never lands). With v0.13.0
shipping in a multi-week timeline and the waiver expiring 2026-05-15
(~15 days from kickoff), landing the verifiers inline is the path of least
process drift.

**Acceptance:** verifier scripts exist under `quality/gates/structure/` for:
- `no-loose-top-level-planning-audits` — fail if any
  `.planning/milestones/audit*.md` or top-level audit doc exists outside
  `.planning/milestones/audits/` or `.planning/archive/`.
- `no-pre-pivot-doc-stubs` — fail if any `docs/<slug>.md` exists at
  top-level docs/ with size <500 bytes (catches stub leftover from pre-v0.9
  era).
- `repo-org-audit-artifact-present` — pass if the canonical repo-org-audit
  artifact exists at the catalog-cited path.

Each verifier:
- Catalog row flips `status: WAIVED` → `status: PASS` (waiver block deleted).
- 5-30 line shell script (TINY shape per docs-alignment dimension precedent).
- Tested via the standard runner (`python3 quality/runners/run.py`).

**Owner:** v0.13.0 P0 or P1 (alongside gix bump or attach work). Decision
ratified by owner 2026-04-30; see `decisions.md` § "WAIVED structure rows".

## POC-DVCS-01 — exercise the three innovations before phase plans

**Source:** `.planning/research/v0.13.0-dvcs/kickoff-recommendations.md` rec #2;
v0.9.0 precedent (`research/v0.9-fuse-to-git-native/poc/`) — ~1 day spent
saved 3-4 days of mid-phase rework on a thesis-level shift.

**Why before phase decomposition:** the architecture sketch's open
questions include reconciliation cases that may require revisiting the
algorithm shape (Q1.x), the bus-remote sequencing (Q3.x), and the
mirror-lag ref semantics (Q2.x). Surfacing integration issues in a 1-day
exploration is cheaper than discovering them in phase 3 of a 6-phase
milestone.

**Acceptance:**
- `research/v0.13.0-dvcs/poc/` directory exists; throwaway code only
  (NOT v0.13.0 implementation).
- A `run.sh` or equivalent that exercises end-to-end against the simulator:
  - `reposix attach` against a working tree with mixed `id`-bearing +
    `id`-less files (deliberately mangled).
  - A bus-remote push that observes mirror lag (SoT writes succeed, mirror
    trailing).
  - The cheap-precheck path (refuse fast when SoT version mismatches local
    cache).
- A `POC-FINDINGS.md` lists what the POC surfaced — algorithm-shape
  decisions, integration friction, design questions the architecture
  sketch did not anticipate. Feeds directly into Phase 1's PLAN.md.
- Time budget: ~1 day; if exceeding 2 days, surface as a SURPRISES-INTAKE
  candidate before continuing.

**Owner:** v0.13.0 pre-Phase-1 (after milestone container exists, before
PLAN.md drafted). Decision ratified by owner 2026-04-30; see `decisions.md`
§ "POC scope".
