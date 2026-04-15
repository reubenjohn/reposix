---
phase: 21
plan: E
subsystem: ci / reposix-fuse
tags: [hardening, ci, macos, macfuse, parity, HARD-04, HARD-00]
dependency_graph:
  requires: [21-A]
  provides: [fuse-teardown-os-agnostic, hooks-ci-step]
  affects: [.github/workflows/ci.yml, crates/reposix-fuse/tests]
tech_stack:
  added: []
  patterns: [env-var-conditional-teardown, github-actions-hooks-test]
key_files:
  created: []
  modified:
    - .github/workflows/ci.yml
    - crates/reposix-fuse/tests/nested_layout.rs
    - crates/reposix-fuse/tests/sim_death_no_hang.rs
decisions:
  - "unmount() helper uses REPOSIX_UNMOUNT_CMD env var (default: fusermount3 -u) so same binary works on Linux and macOS when a self-hosted runner is available"
  - "macOS CI matrix deferred: gythialy/macfuse action 404 + kext approval unavailable on GitHub-hosted runners — requires self-hosted macOS runner with macFUSE pre-approved"
  - "HARD-04 partially closed: OS-agnostic teardown refactor + hooks CI step shipped; FUSE matrix on macOS deferred to self-hosted runner work"
  - "HARD-00 closed: bash scripts/hooks/test-pre-push.sh now runs in CI test job"
metrics:
  duration: ~25m
  completed_date: "2026-04-15"
  tasks_completed: 3
  tasks_total: 3
  files_changed: 3
---

# Phase 21 Plan E: macOS CI Matrix — Summary

**One-liner:** OS-agnostic FUSE teardown via `REPOSIX_UNMOUNT_CMD` + credential hook CI step shipped; macOS FUSE matrix deferred (requires self-hosted runner with macFUSE pre-approved).

## Status: COMPLETE (macos-blocked path)

All three tasks resolved. E1 and E2 landed as planned. E3 took the `macos-blocked` path: macFUSE kext approval is unavailable on GitHub-hosted runners, and the `gythialy/macfuse` action does not exist. The macOS matrix has been reverted from ci.yml. The durable wins from E1 and E2 (teardown refactor + hooks step) remain.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| E1 | Parametrise FUSE teardown via `$REPOSIX_UNMOUNT_CMD` | `09e1459` | `nested_layout.rs`, `sim_death_no_hang.rs` |
| E2 | Add macOS matrix + hooks step to ci.yml | `7b4b4a6` | `.github/workflows/ci.yml` |
| E3 | macos-blocked — revert matrix, keep hooks + teardown refactor | `68be8a4` | `.github/workflows/ci.yml` |

## macOS CI first-run result

**Status: BLOCKED — `macos-blocked: no valid macFUSE action for hosted runners`**

Research finding (confirmed before E3): macFUSE on GitHub-hosted `macos-14` runners requires interactive kext approval via System Preferences, which is unavailable in headless CI. The `gythialy/macfuse` action does not exist (404). No workaround exists for GitHub-hosted runners.

**Resolution taken:**
- Reverted `strategy.matrix` from the `integration` job back to `ubuntu-latest` only
- Removed the `Install macFUSE (macOS)` conditional step
- Removed `REPOSIX_UNMOUNT_CMD` env var from the integration job (Linux default is hardcoded in the test helper anyway)
- Removed all `runner.os == 'macOS'` conditional steps from integration job
- Added deferral comment in ci.yml near the integration job
- Kept `bash scripts/hooks/test-pre-push.sh` step in the `test` job (HARD-00 closes)
- Kept E1 `unmount()` helper with `REPOSIX_UNMOUNT_CMD` in both FUSE test files (durable, ready for self-hosted runner)

**Follow-up required:** To enable macOS FUSE CI, a self-hosted macOS runner with macFUSE pre-approved via System Preferences is needed. The E1 teardown refactor is already in place so enabling the macOS leg is a pure ci.yml change when a runner is available.

## What Was Built

### E1: OS-agnostic FUSE test teardown (durable)

Both `crates/reposix-fuse/tests/nested_layout.rs` and `crates/reposix-fuse/tests/sim_death_no_hang.rs` now:

1. Include an `unmount()` helper that reads `$REPOSIX_UNMOUNT_CMD` (default: `fusermount3 -u`)
2. Call `unmount()` as belt-and-suspenders explicit unmount called before `drop(mount)` in teardown
3. Linux behaviour is unchanged — `REPOSIX_UNMOUNT_CMD` unset defaults to `fusermount3 -u`

### E2: ci.yml hooks step (durable)

`test` job in `.github/workflows/ci.yml` gains:
```yaml
- name: Test pre-push credential hook
  run: bash scripts/hooks/test-pre-push.sh
```
This closes the HARD-00 regression gap: the credential pre-push hook is now regression-tested in CI on every push.

### E3: macOS matrix reverted, deferral comment added

`integration` job reverted to Linux-only `ubuntu-latest`. Comment added:
```yaml
# macOS CI matrix deferred: macFUSE requires interactive kext approval
# (unavailable on GitHub-hosted runners). Use a self-hosted macOS runner
# with macFUSE pre-approved to enable FUSE integration testing on macOS.
# HARD-04 partial: hooks step ships here; FUSE matrix requires self-hosted.
```

## HARD requirement status

| Requirement | Status | Notes |
|-------------|--------|-------|
| HARD-00 (hooks in CI) | CLOSED | `test-pre-push.sh` step in CI `test` job |
| HARD-04 (macOS FUSE parity) | PARTIAL | Teardown refactor ready; FUSE matrix deferred to self-hosted runner |

## Deviations from Plan

### Auto-discovered Issues

**1. [Rule 1 - Discovery] gythialy/macfuse action does not exist**
- **Found during:** Task E2, Step 2 (pin verification)
- **Issue:** `GET https://api.github.com/repos/gythialy/macfuse` returns HTTP 404. The action referenced in RESEARCH.md as "ASSUMED" does not exist at that repository path.
- **Resolution:** Took the `macos-blocked` path at E3 — macOS matrix reverted, deferral comment added.
- **Files modified:** `.github/workflows/ci.yml`
- **Commit:** `68be8a4`

**2. [Observation] macFUSE kext approval fundamentally incompatible with hosted runners**
- macFUSE requires interactive System Preferences approval to load the kernel extension. GitHub-hosted runners (including `macos-14`) are headless and cannot complete this flow. This is a platform limitation, not a fixable CI configuration issue.

**3. [Observation] Test files had no direct fusermount3 Command calls**
- The plan described replacing `Command::new("fusermount3")` teardown calls, but neither test file used that pattern — teardown was `drop(mount)` (fuser UmountOnDrop). The `unmount()` helper was added as belt-and-suspenders explicit unmount called before drop, which is the correct approach for macOS where `umount -f` may be needed to unblock a stale mount.

## Known Stubs

None — E1 teardown refactor is fully wired and correct on Linux. E2 hooks step is fully wired. macOS FUSE CI is deferred by documented design decision, not a stub.

## Threat Flags

None. The macOS matrix removal eliminates T-21-E-01 (unpinned third-party action) since that step no longer exists.

## Self-Check: PASSED

- `nested_layout.rs`: FOUND (`/home/reuben/workspace/reposix/crates/reposix-fuse/tests/nested_layout.rs`)
- `sim_death_no_hang.rs`: FOUND (`/home/reuben/workspace/reposix/crates/reposix-fuse/tests/sim_death_no_hang.rs`)
- `ci.yml`: FOUND (`.github/workflows/ci.yml`)
- Commit `09e1459` (E1 teardown refactor): FOUND
- Commit `7b4b4a6` (E2 hooks step): FOUND
- Commit `68be8a4` (E3 macos-blocked revert): FOUND
- `cargo check --workspace`: PASSED
- YAML parse: PASSED (`python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`)
- No `macos-14`, `gythialy`, or `strategy.matrix` in integration job: CONFIRMED
- `test-pre-push.sh` step in test job: CONFIRMED
