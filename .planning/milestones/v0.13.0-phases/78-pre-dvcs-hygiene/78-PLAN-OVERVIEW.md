---
phase: 78
title: "Pre-DVCS hygiene — gix bump, WAIVED-row verifiers, multi-source walker"
milestone: v0.13.0
requirements: [HYGIENE-01, HYGIENE-02, MULTI-SOURCE-WATCH-01]
depends_on: []
plans:
  - 78-01-PLAN.md  # HYGIENE-01: gix bump
  - 78-02-PLAN.md  # HYGIENE-02: 3 TINY verifier .sh + WAIVED→PASS flip
  - 78-03-PLAN.md  # MULTI-SOURCE-WATCH-01: walker schema migration
waves:
  1: [78-01, 78-02]
  2: [78-03]
---

# Phase 78 — Pre-DVCS hygiene (overview)

This is the entry-point phase for milestone v0.13.0 (DVCS over REST). It
holds three independent, mutually-parallelizable hygiene items that
must close BEFORE the POC + `reposix attach` work begins in P79:

- **78-01 / HYGIENE-01 — gix yanked-pin bump.** Bumps `gix = "=0.82.0"`
  (yanked from crates.io 2026-04-28) and the transitively yanked
  `gix-actor 0.40.1` to non-yanked successors; closes GitHub issues
  #29 + #30. Touches `Cargo.toml`, `Cargo.lock`, `CLAUDE.md` § Tech
  stack. Cargo work — sequential per CLAUDE.md "Build memory budget".
- **78-02 / HYGIENE-02 — 3 TINY verifier scripts + WAIVED → PASS.**
  Lands `quality/gates/structure/{no-loose-top-level-planning-audits,no-pre-pivot-doc-stubs,repo-org-audit-artifact-present}.sh`
  (TINY shell verifiers mirroring `quality/gates/docs-alignment/jira-adapter-shipped.sh`)
  and atomically flips the corresponding catalog rows in
  `quality/catalogs/freshness-invariants.json` from `WAIVED → PASS`
  before the 2026-05-15 expiry. Shell + JSON only — no cargo.
- **78-03 / MULTI-SOURCE-WATCH-01 — walker schema migration.**
  Closes the v0.12.1 P75 carry-forward via path-(b): `Row::source_hashes:
  Vec<String>` parallel-array, walker AND-compares per-source hashes,
  one-time backfill of the 388-row catalog via `serde(default)`,
  regression tests at `crates/reposix-quality/tests/walk.rs::walk_multi_source_*`.
  Cargo work — must serialize against 78-01.

## Wave plan

Per CLAUDE.md "Build memory budget" rule (the VM has crashed twice from
parallel cargo workspace builds), **at most one cargo invocation runs at a
time across the whole phase**. The wave plan honors this:

| Wave | Plans | Cargo? | File overlap | Parallelism                                   |
|------|-------|--------|--------------|------------------------------------------------|
| 1    | 78-01 | YES    | none         | 78-02 runs in parallel (no cargo lock contention) |
| 1    | 78-02 | NO     | none         | 78-01 holds cargo lock; 78-02 is shell + JSON |
| 2    | 78-03 | YES    | CLAUDE.md*   | runs after 78-01 frees cargo lock              |

\* 78-01 edits `CLAUDE.md` § Tech stack (one paragraph). 78-03 edits
`CLAUDE.md` § "v0.12.1 — in flight" → "P75 — bind-verb hash-overwrite fix"
(a different paragraph, ~150 lines away). Wave-2 ordering ensures 78-03 sees
78-01's CLAUDE.md edit and the two paragraph edits compose cleanly via `Edit`
tool runs (no merge conflict).

`files_modified` overlap audit (per gsd-planner same-wave-zero-overlap rule):

| Plan | Files                                                                                                                |
|------|----------------------------------------------------------------------------------------------------------------------|
| 78-01| `Cargo.toml`, `Cargo.lock`, `CLAUDE.md`                                                                              |
| 78-02| `quality/gates/structure/{no-loose-top-level-planning-audits,no-pre-pivot-doc-stubs,repo-org-audit-artifact-present}.sh`, `quality/catalogs/freshness-invariants.json` (and a 3-line comment edit on `quality/gates/structure/freshness-invariants.py`) |
| 78-03| `crates/reposix-quality/src/catalog.rs`, `crates/reposix-quality/src/commands/doc_alignment.rs`, `crates/reposix-quality/tests/walk.rs`, `quality/catalogs/doc-alignment.json`, `CLAUDE.md` |

Wave 1 file overlap: NONE between 78-01 and 78-02. Safe parallel.
Wave 2 file overlap: 78-03 ↔ 78-01 share `CLAUDE.md` only; 78-03 runs in
Wave 2 (sequential after 78-01); the two CLAUDE.md edits target different
paragraphs. Safe.

## Phase-close protocol

Per CLAUDE.md OP-7 + REQUIREMENTS.md § "Recurring success criteria across
every v0.13.0 phase":

1. **All commits pushed.** Each plan terminates with `git push origin main`
   (per CLAUDE.md "Push cadence — per-phase, codified 2026-04-30, closes
   backlog 999.4"). Pre-push gate-passing is part of each plan's close
   criterion.
2. **Pre-push gate GREEN** for each plan's push. If pre-push BLOCKS:
   treat as plan-internal failure (fix, NEW commit, re-push). NO
   `--no-verify` per CLAUDE.md git safety protocol.
3. **Verifier subagent dispatched.** After 78-03 pushes and Wave 2
   completes, the orchestrator dispatches an unbiased verifier subagent
   per `quality/PROTOCOL.md` § "Verifier subagent prompt template"
   (verbatim copy). The subagent grades ALL P78 catalog rows from
   artifacts with zero session context.
4. **Verdict at `quality/reports/verdicts/p78/VERDICT.md`.** Format per
   `quality/PROTOCOL.md`. Phase loops back if verdict is RED.
5. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "Phase 78 in flight" → "Phase 78 SHIPPED 2026-MM-DD"
   (commit SHA cited). Update `progress` block: `completed_phases: 1`,
   `total_plans: 3`, `completed_plans: 3`, `percent: 9`.
6. **CLAUDE.md updated in same PR-equivalent.** 78-01 updates § Tech
   stack; 78-03 updates § "v0.12.1 P75". 78-02 does NOT update
   CLAUDE.md — its convention (TINY shell verifiers under
   `quality/gates/structure/`) is already documented in CLAUDE.md
   "Quality Gates — dimension/cadence/kind taxonomy" (the existing
   section already names `<dim>/` directory structure as canonical;
   78-02 is an instance of the existing convention, not a new one).
   No CLAUDE.md surface gap.

## Risks + mitigations

| Risk                                                                          | Likelihood | Mitigation                                                                                                                                                                                                                                                                          |
|------------------------------------------------------------------------------|------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **gix bump introduces broken-API at consumer call site** (rare on minor; possible) | LOW        | 78-01 T03 runs cargo check / clippy / nextest serially after the bump; failures surface immediately. Diagnose via gix CHANGELOG; port forward; cite the entry in code comment. If the only non-yanked release introduces a major-shape change, fall back to last-known-non-yanked + flag as HIGH SURPRISES entry. |
| **gix bump triggers transitive bump of unrelated workspace dep** (e.g., `cargo update -p gix` cascades) | LOW        | Use `cargo update -p gix` (scoped); do NOT run `cargo update` without `-p`. Verify via `git diff Cargo.lock` shows only gix-family entries.                                                                                                                                          |
| **HYGIENE-02 runner doesn't dispatch `.sh` verifiers** (currently dispatches Python via `freshness-invariants.py` for these rows) | MEDIUM     | 78-02 T04 inspects `quality/runners/run.py` BEFORE the catalog flip. If runner needs a one-line dispatch update for `.sh` extension, Eager-resolve in 78-02 if < 30 min; else SURPRISES entry + DEFER to P87. The Python branches in `freshness-invariants.py` STAY as fallback regression net. |
| **HYGIENE-02 `repo-org-audit-artifact-present` artifact missing**             | LOW        | 78-02 T03 smoke-test detects this immediately. If artifact missing, STOP the flip; SURPRISES entry HIGH; either re-author the artifact in P78 (Eager-resolution if < 1hr) or defer to P87.                                                                                          |
| **MULTI-SOURCE-WATCH-01 walker migration surfaces drift on the live 388-row catalog** | MEDIUM     | 78-03 T05 runs the walker on the live catalog post-migration; drift here is the CORRECT behavior (path-(a) was hiding it). Refresh affected rows via `/reposix-quality-refresh <doc>` (top-level slash command per CLAUDE.md). If >5 rows surface, defer the rest to P87 SURPRISES.                |
| **Schema migration breaks downgrade rollback** (someone reverts past P78)     | LOW        | The `source_hash` legacy field STAYS for one release cycle (78-03 T03 keeps both fields in sync). Pre-P78 binary loads post-P78 catalog cleanly because `source_hash` still carries the first-source hash.                                                                          |
| **Cargo memory pressure** (the load-bearing CLAUDE.md rule)                  | LOW        | Strict serial cargo: 78-01 cargo runs first; 78-03 cargo runs in Wave 2 only after 78-01 completes. Per-crate fallback (`cargo nextest run -p reposix-quality`) is documented in 78-01 T03 + 78-03 T05.                                                                              |
| **Pre-push hook BLOCKs on a pre-existing drift unrelated to P78**            | LOW        | Per CLAUDE.md § "Push cadence — per-phase": treat as phase-internal failure. Diagnose, fix, NEW commit (NEVER amend), re-push. Do NOT bypass with `--no-verify`.                                                                                                                                       |
| **CLAUDE.md edit conflict between 78-01 and 78-03**                          | LOW        | Wave-2 ordering puts 78-03 after 78-01; the two edits target different paragraphs (~150 lines apart); `Edit` tool serializes. No merge conflict possible since they happen in sequence within the same working tree.                                                                |
| **GH `gh issue close` requires interactive auth**                            | LOW        | 78-01 T04 documents the human-action fallback: emit a stderr note; the cargo gates remain the success contract; closing #29/#30 is housekeeping that can complete out-of-band. Phase still ships.                                                                                    |

## +2 reservation: out-of-scope candidates

Initialize `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` and
`GOOD-TO-HAVES.md` lazily — only when an entry surfaces during a plan's
execution. P78 is the FIRST phase of v0.13.0; if no surprises arise the
files don't yet exist.

Anticipated candidates the plans flag (per OP-8):

- **HIGH** — repo-org-audit artifact missing (78-02 T03 fail mode). Eager-resolve if simple; else P87.
- **HIGH** — gix multi-version jump (78-01 T01 fail mode). Always SURPRISES; phase still ships against the floor (any non-yanked).
- **MEDIUM** — runner doesn't dispatch `.sh` (78-02 T04). Eager-resolve if < 30 min.
- **LOW** — walker migration surfaces real drift on live catalog (78-03 T05). Eager-resolve via `/reposix-quality-refresh` if < 5 rows.
- **LOW** — `cargo fmt` drift on the migration commits. Always Eager-resolve in-phase.

Items NOT in scope for P78 (deferred per the v0.13.0 ROADMAP):

- POC + `reposix attach` core (P79). Do not attempt attach scaffolding here.
- Any DVCS-* surface (P80+). Do not pre-stage bus-remote scaffolding.
- L2/L3 cache-desync hardening (deferred to v0.14.0).
- `source_hash` legacy field removal (post-v0.14.0; back-compat for one release cycle).

## Subagent delegation

Per CLAUDE.md "Subagent delegation rules" + the gsd-planner spec
"aggressive subagent delegation":

| Plan / Task             | Delegation                                                                                                       |
|-------------------------|------------------------------------------------------------------------------------------------------------------|
| 78-01 T01 (gix research)| `gsd-phase-researcher` to find the latest non-yanked gix + gix-actor versions (cargo search + crates.io API).    |
| 78-01 T02-T05 (cargo)   | `gsd-executor` (single subagent, holds the cargo lock for Wave 1).                                               |
| 78-02 T01-T03 (3 .sh)   | `gsd-executor` (parallel with 78-01; shell + JSON only; no cargo).                                               |
| 78-02 T04-T05 (catalog flip + push) | Same 78-02 subagent (continues sequentially).                                                                    |
| 78-03 T01-T06 (cargo)   | `gsd-executor` (Wave 2; runs after 78-01 completes to honor the cargo serialization rule).                       |
| Phase verifier (P78 close) | Unbiased subagent dispatched by orchestrator per `quality/PROTOCOL.md` § "Verifier subagent prompt template" (verbatim). Zero session context; grades catalog rows from artifacts.|

Phase verifier subagent's verdict criteria (extracted for P78):

- HYGIENE-01: `cargo metadata` shows non-yanked gix; `gh issue view 29 30` returns CLOSED (or human-action note); CLAUDE.md cites new version.
- HYGIENE-02: 3 .sh files exist, executable, 5-30 lines each; catalog rows status=PASS waiver=null verifier ends in `.sh`; runner exits 0.
- MULTI-SOURCE-WATCH-01: `Row::source_hashes` field present in catalog.rs; walker `walk_multi_source_non_first_drift_fires_stale` test exists + passes; live catalog walks GREEN; CLAUDE.md cites P78-03 SHA in the v0.12.1 P75 paragraph; new catalog row `doc-alignment/multi-source-watch-01-non-first-drift` is BOUND in `quality/catalogs/doc-alignment.json`.
- Recurring (per phase): catalog-first ordering preserved; per-phase push completed; verdict file at `quality/reports/verdicts/p78/VERDICT.md`.
