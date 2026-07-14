---
phase: 79
plan: 01
subsystem: research/v0.13.0-dvcs/poc
tags: [poc, dvcs, attach, bus-remote, cheap-precheck, throwaway]
requires: []
provides: [POC-FINDINGS.md, run.sh, scratch crate, fixture set]
affects: [79-02 plan revision decision (REVISE tags F01 + F04)]
tech-stack:
  added: ["reposix-poc-79 (standalone, NOT a workspace member)"]
  patterns:
    - "shell + scratch-Rust split (shell drives sim subprocesses + assertions; Rust does frontmatter parse + classification)"
    - "two-sim staged reproduction of bus SoT-first sequencing (no production bus URL parser)"
    - "single-record GET as POC-tier cheap-precheck (production uses list_changed_since per P81 L1)"
key-files:
  created:
    - research/v0.13.0-dvcs/poc/README.md
    - research/v0.13.0-dvcs/poc/POC-FINDINGS.md
    - research/v0.13.0-dvcs/poc/.gitignore
    - research/v0.13.0-dvcs/poc/run.sh
    - research/v0.13.0-dvcs/poc/path-a.sh
    - research/v0.13.0-dvcs/poc/path-b.sh
    - research/v0.13.0-dvcs/poc/path-c.sh
    - research/v0.13.0-dvcs/poc/scratch/Cargo.toml
    - research/v0.13.0-dvcs/poc/scratch/src/main.rs
    - research/v0.13.0-dvcs/poc/fixtures/mangled-checkout/issues/0001.md
    - research/v0.13.0-dvcs/poc/fixtures/mangled-checkout/issues/0042-a.md
    - research/v0.13.0-dvcs/poc/fixtures/mangled-checkout/issues/0042-b.md
    - research/v0.13.0-dvcs/poc/fixtures/mangled-checkout/issues/0099.md
    - research/v0.13.0-dvcs/poc/fixtures/mangled-checkout/notes/freeform.md
    - research/v0.13.0-dvcs/poc/logs/path-a-reconciliation.log
    - research/v0.13.0-dvcs/poc/logs/path-b-bus-mirror-lag.log
    - research/v0.13.0-dvcs/poc/logs/path-c-cheap-precheck.log
  modified: []
decisions:
  - "F01 (REVISE): walker should accept --ignore glob (default .git/.github) — feeds 79-02 T03"
  - "F04 (REVISE): Cache::log_attach_walk audit-hook signature should be regular (event_type + jsonblob) — feeds 79-02 T03"
  - "5-row reconciliation table from architecture-sketch is COMPLETE; no 6th case surfaced"
  - "No HTTP /audit endpoint on the simulator; production bus must own cache-side audit trail (consistent with OP-3)"
  - "Naming consistency: 'record' is the abstract type (used in code); URL paths say '/issues' (sim specialization)"
metrics:
  completed: 2026-05-01
  duration_minutes: 9
  tasks_completed: 5
  tasks_total: 5
  commits: 5
  budget: "1d (CARRY-FORWARD POC-DVCS-01)"
  budget_status: "well under (~1% of budget)"
---

# Phase 79 Plan 01: POC of three v0.13.0 innovations (POC-01) Summary

End-to-end POC at `research/v0.13.0-dvcs/poc/` exercising three integration paths against the simulator: (a) `reposix attach`-shaped reconciliation against a deliberately-mangled 5-fixture checkout, (b) bus-remote SoT-first sequencing with simulated mirror failure + recovery, (c) cheap-precheck refusing fast on SoT version mismatch — all three exit 0 in `run.sh`, transcripts captured, FINDINGS.md authored with INFO/REVISE routing tags.

## What was delivered

- **5 commits, all pushed:** scaffold (660bae0) → path-a (9dc9afa) → path-b (29c4cba) → path-c (df21dbf) → FINDINGS finalize (4e6de2b).
- **Throwaway code under `research/v0.13.0-dvcs/poc/`:** zero `crates/` touch verified across all 5 commits (`git diff --stat HEAD~5 -- crates/` empty).
- **Standalone Rust scratch crate** (`research/v0.13.0-dvcs/poc/scratch/`) with empty `[workspace]` table — NOT a member of the parent workspace; depends on `reposix-core` via local path for `frontmatter::parse`.
- **5 reconciliation cases** all observed cleanly via `run.sh`'s path-a invocation: MATCH (id=1), BACKEND_DELETED (id=99), NO_ID (notes/freeform.md), DUPLICATE_ID (id=42 a+b), MIRROR_LAG (id=2 — backend has, local lacks).
- **POC-FINDINGS.md (~20KB)** with three substantive path sections + § Implications for 79-02 routing block (5 INFO + 2 REVISE + 0 SPLIT).

## How tasks landed

| Task | Goal                                              | Commit  | Artifact                                                     |
| ---- | ------------------------------------------------- | ------- | ------------------------------------------------------------ |
| T01  | Scaffold + fixtures + run.sh skeleton             | 660bae0 | README, FINDINGS skeleton, 5 fixtures, run.sh, .gitignore   |
| T02  | Path (a) reconciliation                           | 9dc9afa | path-a.sh, scratch/Cargo.toml, scratch/src/main.rs           |
| T03  | Path (b) bus SoT-first + mirror lag               | 29c4cba | path-b.sh                                                    |
| T04  | Path (c) cheap precheck on version mismatch       | df21dbf | path-c.sh                                                    |
| T05  | Finalize FINDINGS + transcripts + terminal push   | 4e6de2b | POC-FINDINGS.md + 3 path transcript logs                     |

## POC-FINDINGS routing (BLOCKING for 79-02)

7 findings recorded in `POC-FINDINGS.md` § Implications for 79-02:

- **REVISE × 2** — both small spec tightenings, no scope expansion:
  - **F01:** reconciliation walker should accept `--ignore` glob (default: `.git, .github`) so production attach produces clean reconciliation tables on real checkouts with vendored docs. Feeds 79-02 T03.
  - **F04:** `Cache::log_attach_walk` audit-hook signature should be regular (`event_type + jsonblob`) rather than per-event-typed-args, anticipating sibling hooks in P83. Feeds 79-02 T03.
- **INFO × 5** — informational, no plan revision required:
  - **F02:** keep "record" as abstract type in code (already true); document URL templates explicitly in `attach` help text + topology doc.
  - **F03:** 5-row reconciliation table is COMPLETE (no 6th case surfaced) — green signal for early-warning trigger from `vision-and-mental-model.md` § "Risks".
  - **F05:** production bus must own cache-side audit trail; SoT's `audit_events` table is SQLite-only (no HTTP `/audit` route) — consistent with OP-3.
  - **F06:** reject-message text should fork by topology in P82; `git pull --rebase` is wrong-ish in `bus://` mode.
  - **F07:** initialize `last_fetched_at` to NOW on attach (not None) so first push after attach doesn't degrade to full `list_records` walk; relevant to P81 L1 migration.

**Highest-severity tag: REVISE.** Orchestrator decides whether to re-engage planner for an in-place 79-02 revision covering F01 + F04, or treat them as "noted, will-fix during execution" with FINDINGS as the executor's context.

## Acceptance criteria — verified

| Criterion                                                                                        | Status                                                                                  |
| ------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- |
| `research/v0.13.0-dvcs/poc/run.sh` exists, executable, exits 0                                   | ✓ — verified end-to-end run 2026-05-01T06:28:42Z, exit 0                                |
| 5 fixture files exercising 5 reconciliation cases, comments naming each case                     | ✓ — issues/{0001,0042-a,0042-b,0099}.md + notes/freeform.md                             |
| `POC-FINDINGS.md` with 5 sections (Header, Path-a/b/c, Implications, Time spent)                  | ✓ — all sections substantive; § Implications has 7 routing-tagged items                  |
| `logs/` with at least one transcript per path                                                    | ✓ — path-{a,b,c}-*.log committed; sim-*.log gitignored as transient noise               |
| Standalone scratch crate at `scratch/Cargo.toml` (NOT a workspace member)                         | ✓ — empty `[workspace]` table; cargo check from inside `scratch/` succeeds              |
| No catalog rows minted for the POC                                                               | ✓ — production rows land in 79-02; verified `git diff -- quality/catalogs/` empty        |
| No `crates/` touch in any plan commit                                                            | ✓ — `git diff --stat HEAD~5 -- crates/` empty                                            |
| Per-phase push: terminal commit pushed, pre-push GREEN                                           | ✓ — 26 PASS / 0 FAIL on every push; final push 4e6de2b lands at origin/main             |
| Time annotation in FINDINGS § Time spent                                                         | ✓ — started 2026-05-01T06:20:31Z, finished 2026-05-01T06:29:16Z, ~9 min wall-clock      |

## Deviations from plan

- **Rule 3 (blocking) — `/projects/<p>/records` vs `/projects/<slug>/issues`.** Plan prose for path (a) referenced `/projects/<p>/records` as the simulator's REST route. Actual simulator route is `/projects/<slug>/issues` (verified `crates/reposix-sim/src/routes/issues.rs`). Used the actual route in `path-a.sh` + `scratch/src/main.rs`. The naming inconsistency itself is captured as Path (a) finding F02 (INFO).
- **Rule 1 (bug fix) — `use reposix_core::record::frontmatter` is private.** Initial scratch `main.rs` imported `reposix_core::record::frontmatter`, but `record` is a private module. The compiler suggested `use reposix_core::frontmatter` (the public re-export), which is what `reposix-core/src/lib.rs` exposes. Applied the suggested fix; no plan-prose update needed.
- **Logs `.gitignore` extension** — added `logs/sim-*.log`, `logs/run.log`, and `path-*-seed*.json` patterns to filter transient sim subprocess noise + per-path scratch seed JSONs from git tracking. The meaningful verifier evidence (`logs/path-{a,b,c}-*.log`) IS tracked. This was not in the plan but is consistent with the plan's spirit ("logs/ for transcripts captured by run.sh — these are the verifier's evidence trail").

No deviations triggered Rule 4 (architectural change).

## SURPRISES-INTAKE candidates

**None.** The POC ran in well under the 2-day surprises threshold (~9 minutes vs 1-day budget); no architectural-shape decisions emerged that warrant SURPRISES — the architecture sketch's open questions were already pre-resolved in `decisions.md` and the POC simply confirmed them. The 7 routing-tagged findings in POC-FINDINGS.md are the deliverable, not a defect.

## Stub tracking

No stubs introduced. The POC is throwaway research code; no production data wiring, no UI components, no placeholder content.

## Self-Check: PASSED

Verified each created file exists:
- ✓ `research/v0.13.0-dvcs/poc/README.md`
- ✓ `research/v0.13.0-dvcs/poc/POC-FINDINGS.md`
- ✓ `research/v0.13.0-dvcs/poc/run.sh`
- ✓ `research/v0.13.0-dvcs/poc/path-a.sh` / `path-b.sh` / `path-c.sh`
- ✓ `research/v0.13.0-dvcs/poc/scratch/Cargo.toml` / `src/main.rs`
- ✓ `research/v0.13.0-dvcs/poc/fixtures/mangled-checkout/{issues/0001,0042-a,0042-b,0099}.md` + `notes/freeform.md`
- ✓ `research/v0.13.0-dvcs/poc/logs/path-{a,b,c}-*.log`

Verified each commit hash is in `git log --all`:
- ✓ 660bae0 (T01)
- ✓ 9dc9afa (T02)
- ✓ 29c4cba (T03)
- ✓ df21dbf (T04)
- ✓ 4e6de2b (T05)

All pushed to `origin/main`.
