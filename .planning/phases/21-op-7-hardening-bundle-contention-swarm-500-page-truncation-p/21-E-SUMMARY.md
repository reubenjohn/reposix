---
phase: 21
plan: E
subsystem: ci / reposix-fuse
tags: [hardening, ci, macos, macfuse, parity, HARD-04, HARD-00]
dependency_graph:
  requires: [21-A]
  provides: [macos-ci-matrix, fuse-teardown-os-agnostic, hooks-ci-step]
  affects: [.github/workflows/ci.yml, crates/reposix-fuse/tests]
tech_stack:
  added: []
  patterns: [env-var-conditional-teardown, github-actions-matrix]
key_files:
  created: []
  modified:
    - .github/workflows/ci.yml
    - crates/reposix-fuse/tests/nested_layout.rs
    - crates/reposix-fuse/tests/sim_death_no_hang.rs
decisions:
  - "unmount() helper uses REPOSIX_UNMOUNT_CMD env var (default: fusermount3 -u) so same binary works on Linux and macOS"
  - "macos-14 pinned over macos-latest to avoid Sequoia kext approval changes (RESEARCH Pitfall 3)"
  - "fail-fast: false on matrix so Linux signal survives macOS flakes"
  - "gythialy/macfuse@v1 referenced as action pin — UNVERIFIED (repo 404 as of 2026-04-15, flagged for E3)"
metrics:
  duration: ~15m
  completed_date: "2026-04-15"
  tasks_completed: 2
  tasks_total: 3
  files_changed: 3
---

# Phase 21 Plan E: macOS CI Matrix — Summary

**One-liner:** OS-agnostic FUSE teardown via `REPOSIX_UNMOUNT_CMD` + macOS-14 matrix in CI (blocked at E3 for action verification).

## Status: STOPPED AT CHECKPOINT E3

Tasks E1 and E2 are complete and committed. Task E3 is a `checkpoint:human-verify` — execution stopped here intentionally. The human must push the branch, observe the first macOS-14 CI run, and signal one of the resume codes.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| E1 | Parametrise FUSE teardown via `$REPOSIX_UNMOUNT_CMD` | `09e1459` | `nested_layout.rs`, `sim_death_no_hang.rs` |
| E2 | Add macOS matrix + hooks step to ci.yml | `7b4b4a6` | `.github/workflows/ci.yml` |

## Task E3: Pending Human Verification

**Status:** AWAITING HUMAN ACTION — do not push yet.

**Critical blocker discovered during E2:** The `gythialy/macfuse` GitHub Action referenced in the research (`gythialy/macfuse@v1`) **does not exist**. `GET https://api.github.com/repos/gythialy/macfuse` returns 404. The action is not published at that repository path.

The ci.yml was committed with `uses: gythialy/macfuse@v1` and a `NOTE(E3)` comment flagging this. The macOS-14 CI leg **will fail** at the "Install macFUSE (macOS)" step until the correct action reference is resolved.

### What the human needs to do at E3

**Step 1 — Find a valid macFUSE GitHub Action.** Options to investigate:

| Option | Notes |
|--------|-------|
| `mstksg/macfuse-action` | Search GitHub for active macFUSE actions |
| `homebrew/actions` + `brew install --cask macfuse` | NOT safe — kext install hangs CI |
| Direct `curl` + `hdiutil` install | Complex, fragile, not recommended |
| Accept macOS is blocked | Revert matrix from ci.yml; keep E1 refactor + hooks step |

**Step 2 — Push to a throwaway branch first** (recommended):
```bash
git checkout -b phase-21-E-macos-trial
git push -u origin phase-21-E-macos-trial
```
Watch the Actions tab. If the ubuntu-latest leg passes and macos-14 fails at the action step, that confirms the action reference is the only blocker.

**Step 3 — Signal one of the resume codes:**
- `"macos-green"` — both legs green; plan complete
- `"macos-partial: <description>"` — specific fixable failure; file follow-up
- `"macos-blocked: <reason>"` — revert matrix from ci.yml; keep E1 + hooks step
- `"retry with pin @<tag-or-sha>"` — executor re-pins action and re-verifies

## What Was Built

### E1: OS-agnostic FUSE test teardown

Both `crates/reposix-fuse/tests/nested_layout.rs` and `crates/reposix-fuse/tests/sim_death_no_hang.rs` now:

1. Compile on both Linux and macOS (`#[cfg(any(target_os = "linux", target_os = "macos"))]`)
2. Include an `unmount()` helper that reads `$REPOSIX_UNMOUNT_CMD` (default: `fusermount3 -u`)
3. Call `unmount()` as belt-and-suspenders before `drop(mount)` in teardown

Linux behaviour is unchanged — `REPOSIX_UNMOUNT_CMD` unset → defaults to `fusermount3 -u`.

### E2: ci.yml matrix + hooks step

`.github/workflows/ci.yml` changes:
- `integration` job now has `strategy.matrix.os: [ubuntu-latest, macos-14]` with `fail-fast: false`
- Conditional FUSE install: `apt-get install fuse3` on Linux, `gythialy/macfuse@v1` on macOS (UNVERIFIED)
- `REPOSIX_UNMOUNT_CMD` env var set per `runner.os` in the integration test step
- `test` job gains `bash scripts/hooks/test-pre-push.sh` step (closes HARD-00 regression gap)

## macOS CI first-run result

**Status: PENDING** — E3 checkpoint not yet passed.

_(This section will be updated after the human signals a resume code at E3.)_

## Deviations from Plan

### Auto-discovered Issues

**1. [Rule 1 - Discovery] gythialy/macfuse action does not exist**
- **Found during:** Task E2, Step 2 (pin verification)
- **Issue:** `GET https://api.github.com/repos/gythialy/macfuse` returns HTTP 404. The action referenced in RESEARCH.md as "ASSUMED" does not exist at that repository path.
- **Fix applied:** ci.yml committed with `gythialy/macfuse@v1` plus a `NOTE(E3)` comment explicitly calling out the verification gap. The macOS CI leg will fail until resolved.
- **Action required at E3:** Find the correct macFUSE GitHub Action or choose the "macos-blocked" path.
- **Files modified:** `.github/workflows/ci.yml`
- **Commit:** `7b4b4a6`

**2. [Observation] Test files had no direct fusermount3 Command calls**
- The plan described replacing `Command::new("fusermount3")` teardown calls, but neither test file used that pattern — teardown was `drop(mount)` (fuser UmountOnDrop). The `unmount()` helper was added as belt-and-suspenders explicit unmount called before drop, which is the correct approach for macOS where `umount -f` may be needed to unblock a stale mount.

## Known Stubs

None — E1 is fully wired. E2 has a known-broken action reference documented explicitly.

## Self-Check: PASSED

- `nested_layout.rs`: FOUND
- `sim_death_no_hang.rs`: FOUND
- `ci.yml`: FOUND
- Commit `09e1459` (E1): FOUND
- Commit `7b4b4a6` (E2): FOUND
