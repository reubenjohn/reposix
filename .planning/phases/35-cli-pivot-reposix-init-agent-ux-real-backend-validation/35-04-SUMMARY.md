---
phase: 35
plan: 04
title: "Latency benchmark + docs/benchmarks/v0.9.0-latency.md + docs/reference/testing-targets.md"
status: complete
requirements: [ARCH-17, ARCH-18]
---

# Phase 35 Plan 04 — Summary

## What shipped

- **`scripts/v0.9.0-latency.sh`** (executable, `set -euo pipefail`):
  - Builds the workspace bins, spawns `reposix-sim --ephemeral` on
    `127.0.0.1:7780` with the bundled `crates/reposix-sim/fixtures/seed.json`.
  - Times five golden-path steps using millisecond `date +%s%N` deltas:
    `reposix init` cold, list-issues REST round-trip, get-one-issue
    REST round-trip, PATCH-issue REST round-trip, helper `capabilities`
    probe.
  - Soft thresholds: emits `WARN:` to stderr if sim cold init > 500ms
    or list > 500ms. Never `exit 1` on threshold breach.
  - Regenerates `docs/benchmarks/v0.9.0-latency.md` in place; the
    script is the canonical regenerator.
  - EXIT trap kills the sim child and removes `/tmp/v090-latency-$$`.

- **`docs/benchmarks/v0.9.0-latency.md`** populated from a real run:
  - Header records generation time + git short-SHA for replay.
  - Two-paragraph "How to read this" intro names what's measured and
    what's NOT measured (network jitter, runner variance, TLS reuse).
    Cross-links the v0.7.0 token-economy benchmark.
  - Markdown table: 5 rows × 4 backend columns. Sim populated; real
    backends empty in this run with an explicit "(creds unavailable)"
    footnote.
  - Soft-thresholds section restates the non-CI-blocking contract.
  - "Reproduce" block points at `bash scripts/v0.9.0-latency.sh`,
    plus the env-var bundle for capturing real-backend columns.
  - Cross-links `docs/reference/testing-targets.md`.

- **`docs/reference/testing-targets.md`** per ARCH-18:
  - Top-of-file blockquote with the verbatim permission statement
    ("TokenWorld is for testing — go crazy, it's safe.") attributed
    and dated.
  - Three H2 sections (TokenWorld Confluence, `reubenjohn/reposix`
    GitHub, JIRA project `TEST`). Each H2 contains: env-var setup table
    (names only — never values, per T-11B-01); rate-limit notes;
    cleanup procedure ("do not leave junk issues lying around — clear
    `kind=test` after each run"); a one-liner repeat of the permission
    statement.
  - JIRA section explicitly names both `JIRA_TEST_PROJECT` and
    `REPOSIX_JIRA_PROJECT` overrides with the resolution precedence
    (`JIRA_TEST_PROJECT` → `REPOSIX_JIRA_PROJECT` → `TEST`).
  - Trailing "Running real-backend tests" section ships the canonical
    cargo-test command bundle for the three backends.
  - Cross-link block: "Linked from: project CLAUDE.md (Phase 36 wires
    the cross-link from the 'Commands you'll actually use' section)."

- **CHANGELOG `[Unreleased]` `### Added`** cross-references all four
  Phase 35 artifacts (latency doc, testing-targets doc, dark-factory
  test, real-backend integration tests).

## Tests added

This plan adds documentation + script artifacts; no new `cargo test`
invocations. Validation by direct invocation:

- `bash scripts/v0.9.0-latency.sh` → exits 0; latency rows produced.
  Sim numbers from the recovery run: init=24ms, list=9ms, get=8ms,
  patch=8ms, cap=5ms — all well under the 500ms soft threshold.
- `test -f docs/benchmarks/v0.9.0-latency.md` and
  `test -f docs/reference/testing-targets.md` → both succeed.
- `grep -c "go crazy, it's safe" docs/reference/testing-targets.md`
  → 5 (top + three section repeats + one extra in the
  reubenjohn/reposix permission carry-over).
- `grep -c 'JIRA_TEST_PROJECT\|REPOSIX_JIRA_PROJECT' docs/reference/testing-targets.md`
  → 5.
- `cargo build --workspace` → clean.

## Acceptance criteria — status

Plan 35-04 acceptance criteria from `35-04-PLAN.md`:

**Task 35-04-A:**
- `bash scripts/v0.9.0-latency.sh` runs end-to-end against sim with exit 0. **Met.**
- Output Markdown matches the §R5 format. **Met.**
- Soft thresholds produce `WARN:` lines; do not fail the script. **Met by code; not triggered in this run.**

**Task 35-04-B:**
- File exists at `docs/benchmarks/v0.9.0-latency.md`. **Met.**
- Contains a Markdown table with 5 rows and 4 backend columns. **Met (5 rows × 4 cols).**
- Reproducer command points at `scripts/v0.9.0-latency.sh`. **Met.**
- CHANGELOG references the doc. **Met.**

**Task 35-04-C:**
- File exists at `docs/reference/testing-targets.md`. **Met.**
- Contains the literal string `go crazy, it's safe`. **Met (5 occurrences).**
- Contains both `JIRA_TEST_PROJECT` and `REPOSIX_JIRA_PROJECT`. **Met.**
- CHANGELOG references the doc. **Met.**

## Notes for downstream phases

- **Phase 36 CI** wires three integration jobs that run the latency
  script with each backend's secret pack to populate the empty
  columns. The artifact file is regenerated on each push to `main`.
- **Phase 36 docs sweep** will add an explicit cross-link from
  CLAUDE.md's "Commands you'll actually use" section to
  `docs/reference/testing-targets.md` — the doc itself anticipates
  this with a "Linked from" footer.
- **Phase 36 skill** (`.claude/skills/reposix-agent-flow`) will
  invoke `bash scripts/dark-factory-test.sh sim` as the autonomous-mode
  default; the latency script is parallel infrastructure.
- The "real-backend column" population is gated on the helper learning
  multi-backend dispatch (currently SimBackend-only per Phase 32 limitation).
  Phase 35-03 ships the gated test harness; the empty columns in the
  latency table will populate in lockstep with that future phase.
