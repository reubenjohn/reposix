---
phase: 11-confluence-adapter
plan: D
subsystem: demos
tags: [demo, confluence, tier-3b, tier-5, fuse, skip-contract]
requires:
  - reposix binary with `--backend confluence` support (Phase 11-B; not yet in binary at time of 11-D execution — live path verified in 11-F)
  - scripts/demos/_lib.sh (unchanged by this plan)
  - scripts/demos/parity.sh (structural template)
  - scripts/demos/05-mount-real-github.sh (structural template)
provides:
  - tier-3b sim-vs-Confluence parity demo with four-env-var SKIP contract
  - tier-5 FUSE-mount-real-Confluence demo with four-env-var SKIP contract
  - docs/demos/index.md coverage of both new demos
affects:
  - docs/demos/index.md (Tier 3 + Tier 5 tables updated; Tier 5 section header generalized from "real GitHub" to "real backend")
tech-stack:
  added: []
  patterns:
    - four-var SKIP check (ATLASSIAN_API_KEY / ATLASSIAN_EMAIL / REPOSIX_CONFLUENCE_TENANT / REPOSIX_CONFLUENCE_SPACE) placed immediately after `require` so dev hosts with binaries built can SKIP without auth
    - tenant origin dynamically composed into REPOSIX_ALLOWED_ORIGINS (`http://127.0.0.1:*,https://${REPOSIX_CONFLUENCE_TENANT}.atlassian.net`)
    - double-quote every `$REPOSIX_CONFLUENCE_SPACE` expansion to defuse shell injection via adversarial space keys
    - cat the first file from the listing (Confluence page IDs aren't 1-based like GitHub issue numbers, so hardcoded `0001.md` doesn't apply)
    - hard mountpoint assertion after `fusermount3 -u` (no zombie FUSE mount leak into next run)
key-files:
  created:
    - scripts/demos/parity-confluence.sh
    - scripts/demos/06-mount-real-confluence.sh
  modified:
    - docs/demos/index.md
decisions:
  - "SKIP is exit-0 with the `== DEMO COMPLETE ==` marker — matches the convention that smoke.sh's assert.sh looks for, keeps a future addition of these demos to Tier 1 frictionless"
  - "Port 7805 for parity-confluence.sh's sim (parity.sh uses 7804) so the two Tier 3 demos can (in principle) run concurrently"
  - "cat the first listed page rather than a hardcoded ID — Confluence page IDs are per-space numerics assigned server-side, not stable across tenants"
  - "Never echo ATLASSIAN_API_KEY or ATLASSIAN_EMAIL anywhere; tenant host + space key (non-secrets) are the only Confluence-identifying strings on stdout"
  - "Kept `require` BEFORE the SKIP check per plan spec — matches 05-mount-real-github.sh ordering. Note: the plan's verify commands therefore require release binaries on PATH (\"PATH=\\\"$PWD/target/release:$PATH\\\"\") for the SKIP path to exit 0"
metrics:
  duration: ~15 minutes
  tasks: 3
  files_created: 2
  files_modified: 1
  commits: 3
  completed: 2026-04-13
---

# Phase 11 Plan D: Confluence Demos Summary

Ship two demo scripts — `scripts/demos/parity-confluence.sh` (Tier 3B
sim-vs-Confluence shape diff) and `scripts/demos/06-mount-real-confluence.sh`
(Tier 5 FUSE-mount → `ls` → `cat` → unmount) — both with a four-env-var
SKIP contract that keeps CI and credential-less dev hosts green.

## What shipped

### scripts/demos/parity-confluence.sh (153 lines, executable)

Tier 3B structural parity demo. Mirrors `scripts/demos/parity.sh` but
talks to a real Atlassian tenant instead of `gh api`:

1. `[1/4]` boots a reposix-sim on 127.0.0.1:7805 (offset from parity.sh's
   7804 so the two can run concurrently).
2. `[2/4]` runs `reposix list --origin ... --project demo --format json`
   and normalizes to `{id, title, status}` via jq.
3. `[3/4]` runs `reposix list --backend confluence --project
   $REPOSIX_CONFLUENCE_SPACE --format json`, normalizes the same way.
4. `[4/4]` `diff -u` the two normalized files, asserts identical key
   sets, emits `shape parity: confirmed` + `== DEMO COMPLETE ==`.

SKIP path (any of the four env vars missing): prints the names of the
missing vars, the `== DEMO COMPLETE ==` marker, and exits 0.

### scripts/demos/06-mount-real-confluence.sh (140 lines, executable)

Tier 5 FUSE-mount demo. Mirrors `scripts/demos/05-mount-real-github.sh`
but adapted for Confluence's Basic-auth + tenant-scoped model:

1. `[1/4]` `reposix mount "$MOUNT_PATH" --backend confluence --project
   $REPOSIX_CONFLUENCE_SPACE` in the background; `wait_for_mount`
   polls for 30s (Atlassian cold-connection latency + MountProcess
   watchdog budget).
2. `[2/4]` snapshots the listing once, asserts `COUNT >= 1`.
3. `[3/4]` cats the FIRST file from the listing (not a hardcoded
   `0001.md` — Confluence IDs are per-space numerics).
4. `[4/4]` `fusermount3 -u`, then polls `mountpoint -q` for 3s and
   fails loudly if the mount is still alive. `cleanup_trap` is
   belt-and-braces on EXIT.

SKIP path (any of the four env vars missing): identical shape to
parity-confluence.sh.

### docs/demos/index.md

Added a row to the Tier 3 table for `parity-confluence.sh` and to
the Tier 5 table for `06-mount-real-confluence.sh`. Generalized the
Tier 5 section header from "FUSE mount real GitHub end-to-end" to
"FUSE mount real backend end-to-end" now that GitHub is no longer
the only real backend. Documented the four-env-var SKIP contract
under both new entries, with `.env.example` / `MORNING-BRIEF-v0.3.md`
breadcrumbs.

## SKIP-path run transcripts

**parity-confluence.sh** (with `ATLASSIAN_*` unset, release binaries on PATH):

```
SKIP: env vars unset: ATLASSIAN_API_KEY ATLASSIAN_EMAIL REPOSIX_CONFLUENCE_TENANT REPOSIX_CONFLUENCE_SPACE
      Set them (see .env.example and MORNING-BRIEF-v0.3.md) to
      run the Confluence half of parity.
== DEMO COMPLETE ==
```

Exit code: 0.

**06-mount-real-confluence.sh** (with `ATLASSIAN_*` unset, release binaries on PATH):

```
SKIP: env vars unset: ATLASSIAN_API_KEY ATLASSIAN_EMAIL REPOSIX_CONFLUENCE_TENANT REPOSIX_CONFLUENCE_SPACE
      Set them (see .env.example and MORNING-BRIEF-v0.3.md) to
      run this demo.
== DEMO COMPLETE ==
```

Exit code: 0.

## Regression check — smoke.sh stays 4/4

```
smoke suite: 4 passed, 0 failed (of 4)
```

Confirmed that `scripts/demos/smoke.sh` still references only the four
original Tier 1 demos (`01-edit-and-push`, `02-guardrails`,
`03-conflict-resolution`, `04-token-economy`) and neither of the two
new Tier 3B / Tier 5 scripts.

## Success criteria — all 11 pass

| # | Criterion | Result |
|---|---|---|
| 1 | `test -x scripts/demos/parity-confluence.sh` | OK |
| 2 | `test -x scripts/demos/06-mount-real-confluence.sh` | OK |
| 3 | `grep -q 'ASSERTS: "shape parity" "DEMO COMPLETE"'` on parity-confluence.sh | OK |
| 4 | `grep -q 'ASSERTS: "DEMO COMPLETE"'` on 06-mount-real-confluence.sh | OK |
| 5 | Env-stripped parity-confluence.sh exits 0 | OK |
| 6 | Env-stripped 06-mount-real-confluence.sh exits 0 | OK |
| 7 | parity-confluence.sh SKIP banner mentions `ATLASSIAN_API_KEY` | OK |
| 8 | 06-mount-real-confluence.sh SKIP banner mentions `ATLASSIAN_API_KEY` | OK |
| 9 | Neither new script appears in `scripts/demos/smoke.sh` | OK |
| 10 | No `echo` of `$ATLASSIAN_API_KEY` / `$ATLASSIAN_EMAIL` in either script | OK |
| 11 | `PATH="$PWD/target/release:$PATH" bash scripts/demos/smoke.sh` = 4 passed, 0 failed | OK |

## Commits

| Hash | Subject |
|---|---|
| `dd3abff` | feat(11-D-1): parity-confluence.sh Tier 3B demo with skip path |
| `5dfb1e4` | feat(11-D-2): 06-mount-real-confluence.sh Tier 5 demo with skip path |
| `2786b72` | docs(11-D): demo index lists Confluence Tier 3B + Tier 5 rows |

## Deviations from Plan

**None** on the Rule-1/2/3 auto-fix path. The plan executed verbatim.

One spec clarification worth surfacing to 11-F's MORNING-BRIEF (see
"Notes for 11-F" below): the plan's Task-1/2 verify commands work
only when the release binaries are on `PATH`, because `require
reposix-sim` / `require reposix` precede the SKIP check per the plan's
ordering. If a reviewer runs the verify on a clean checkout, they
will need to `cargo build --release --workspace --bins` and prepend
`target/release` to `PATH` first. Task 3's verify already does this
chaining, so the plan is internally consistent — this is a hint for
the user-facing morning brief, not an error.

## Auth gates

None. This plan was executable entirely without live Atlassian
credentials because the SKIP path is the only path we exercised.
The happy-path live-mount verification is scheduled for Phase 11-F
per the plan's `<verification>` block and `00-CREDENTIAL-STATUS.md`.

## Notes for 11-F's MORNING-BRIEF-v0.3.md

Surface these to the user:

- **Required env vars** (exact names): `ATLASSIAN_API_KEY`,
  `ATLASSIAN_EMAIL`, `REPOSIX_CONFLUENCE_TENANT` (e.g. `reuben-john`
  — the subdomain, not the full host), `REPOSIX_CONFLUENCE_SPACE`
  (e.g. `REPOSIX` — the space key, not the numeric ID).
- **Pre-req before running the two new demos**:
  ```bash
  cargo build --release --workspace --bins
  export PATH="$PWD/target/release:$PATH"
  ```
  otherwise `require reposix{,-sim}` will exit 2 before the SKIP
  check sees anything.
- **Temp paths used** (for awareness in case of a stale-state bug):
  - parity-confluence.sh: `/tmp/reposix-demo-parity-conf-sim.db{,-wal,-shm}`,
    `/tmp/parity-conf-sim.json`, `/tmp/parity-conf-confluence.json`,
    `/tmp/parity-conf-diff.txt`, sim bind `127.0.0.1:7805`.
  - 06-mount-real-confluence.sh: `/tmp/reposix-conf-demo-mnt` (mount
    point, cleaned up by trap), `/tmp/reposix-conf-demo-mnt.log`
    (mount daemon stderr — NOT in the trap's rm list, intentional
    for post-mortem).

## Known Stubs

None. Both scripts are feature-complete for their stated scope. The
live-mount golden path is explicitly deferred to 11-F by plan design,
not stubbed out here.

## Self-Check: PASSED

- `scripts/demos/parity-confluence.sh` — FOUND, executable.
- `scripts/demos/06-mount-real-confluence.sh` — FOUND, executable.
- `docs/demos/index.md` — FOUND, contains both new demo rows.
- Commit `dd3abff` — FOUND in `git log`.
- Commit `5dfb1e4` — FOUND in `git log`.
- Commit `2786b72` — FOUND in `git log`.
