---
phase: 60-docs-build-migration
plan: 07
subsystem: quality-gates
tags: [polish-docs-build, broaden-and-deepen, sweep, runner]
requires:
  - quality/runners/run.py
  - all 8 P60-touched rows shipped via Waves A-F
provides:
  - 4 cadences GREEN (pre-push, pre-pr, weekly, post-release)
  - quality/runners/check_p60_red_rows.py (P60 RED-row sentry, reusable for Wave H + verifier subagent)
affects:
  - the entire docs-build / code / structure dimension health
key-files:
  created:
    - quality/runners/check_p60_red_rows.py
  unchanged:
    - quality/catalogs/docs-build.json (catalog rows already PASS from Waves B/C)
    - quality/catalogs/code.json
    - quality/catalogs/freshness-invariants.json
decisions:
  - "Zero RED at Wave G entry: Waves A-F left the dimension pristine; no in-phase fixes needed"
  - "Promoted ad-hoc P60-row check to quality/runners/check_p60_red_rows.py (CLAUDE.md §4 self-improving infra)"
  - "GH Pages already propagated within ~90s of Wave F push -- QG-09 endpoint URL HTTP 200 immediately"
metrics:
  cadences_green: 4
  p0_p1_red_count: 0
  p2_partial_count: 0
  pre_push_summary: "19 PASS / 0 FAIL / 0 PARTIAL / 0 WAIVED / 0 NOT-VERIFIED"
  pre_pr_summary: "1 PASS / 0 FAIL / 0 PARTIAL / 2 WAIVED / 0 NOT-VERIFIED"
  weekly_summary: "14 PASS / 0 FAIL / 0 PARTIAL / 3 WAIVED / 2 NOT-VERIFIED"
  post_release_summary: "0 PASS / 0 FAIL / 0 PARTIAL / 6 WAIVED / 0 NOT-VERIFIED"
  duration_minutes: 4
  completed_date: "2026-04-27"
---

# Phase 60 Plan 07: POLISH-DOCS-BUILD broaden-and-deepen sweep (Wave G)

## One-liner

All 8 P60-touched rows GREEN at Wave G entry (zero RED to fix); 4 runner cadences exit 0; GH Pages publish completed within ~90s of Wave F push so the QG-09 endpoint URL returns HTTP 200 immediately.

## Cadence sweep (Task 1)

```
pre-push:     19 PASS / 0 FAIL / 0 PARTIAL / 0 WAIVED / 0 NOT-VERIFIED -> exit=0
pre-pr:        1 PASS / 0 FAIL / 0 PARTIAL / 2 WAIVED / 0 NOT-VERIFIED -> exit=0
weekly:       14 PASS / 0 FAIL / 0 PARTIAL / 3 WAIVED / 2 NOT-VERIFIED -> exit=0
post-release:  0 PASS / 0 FAIL / 0 PARTIAL / 6 WAIVED / 0 NOT-VERIFIED -> exit=0
```

## P60-touched row table (Task 1)

```
$ python3 quality/runners/check_p60_red_rows.py
  P1 docs-build/mkdocs-strict: PASS
  P1 docs-build/mermaid-renders: PASS
  P2 docs-build/link-resolution: PASS
  P2 docs-build/badges-resolve: PASS
  P1 code/cargo-fmt-check: PASS
  P1 code/cargo-clippy-warnings: PASS
  P2 structure/badges-resolve: PASS
  P0 structure/cred-hygiene: PASS
P0+P1 RED count: 0
```

## In-phase fixes (Tasks 2-4)

**None required.** The Waves A-F closed every dimension cleanly. Each individual verifier exits 0:

- `bash quality/gates/docs-build/mkdocs-strict.sh` -> PASS
- `bash quality/gates/docs-build/mermaid-renders.sh` -> PASS
- `python3 quality/gates/docs-build/link-resolution.py` -> PASS
- `bash quality/gates/code/cargo-fmt-check.sh` -> PASS
- `bash quality/gates/code/cargo-clippy-warnings.sh` -> PASS

GH Pages publish telemetry confirms `https://reubenjohn.github.io/reposix/badge.json` resolves with HTTP 200, content-type `application/json`, last-modified within ~90s of the Wave F push commit.

## New artifact (CLAUDE.md §4 self-improving infrastructure)

`quality/runners/check_p60_red_rows.py` -- a 50-line stdlib-only Python sentry that reads the 3 P60-relevant catalogs and prints the per-row grade for the 8 P60-touched rows. Exits 1 if any P0+P1 row is NOT in {PASS, WAIVED}. Reusable by Wave H + the verifier subagent. Promotes ad-hoc-bash-pipeline pattern to a committed artifact.

## Commits

- `chore(p60): POLISH-DOCS-BUILD broaden-and-deepen sweep -- runner GREEN` (this commit)

## Self-Check: PASSED

- 4 runner cadences exit 0.
- 8/8 P60-touched rows PASS.
- `quality/runners/check_p60_red_rows.py` exists + executes + exits 0.
- Wave F GH Pages publish verified live via `curl -sIL`.
