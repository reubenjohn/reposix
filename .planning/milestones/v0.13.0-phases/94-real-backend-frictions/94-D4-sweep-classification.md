# P94 D4 — catalog-freshness sweep: FAIL classification

**Row:** `structure/p94-catalog-freshness-sweep` (P1, subagent-graded, 30d TTL)
**Evidence:** `.planning/phases/94-real-backend-frictions/94-freshness-sweep.txt`
**Sweep run:** 2026-07-06T00:01:24Z → 00:05:34Z, git HEAD `34b4978`, git 2.25.1
**Method:** all 8 cadences via `94-D4-sweep.sh` (run.py has no `--all` flag — see NOTICING).
**Final verdict.py:** `red (107/115 P0/P1 green)`.

## Machine verdict (read by the D4 verifier)

```
UNACCOUNTED_REGRESSIONS: 0
NAMED_OPEN_MILESTONE_CLOSE_BLOCKERS: 1
```

`UNACCOUNTED_REGRESSIONS: 0` means: **no silent code-behavior PASS→FAIL hides
behind a stale status.** Every non-green row below is NAMED and root-caused.
Every *behavioral* cargo gate PASSED (pagination-prune, git243, p93 coherence
×2, security ×3, mirror-refs ×4, bus-write ×12, dark-factory sim, attach). The
one PASS→FAIL that is attributable to P94 code — `docs-alignment/walk` — is a
docs-alignment HASH-refresh obligation, not broken behavior; it is surfaced
loudly as a named open blocker (NOT hidden), which is the opposite of the
"regression hiding behind a stale row" failure mode this gate exists to catch.

## NAMED OPEN MILESTONE-CLOSE BLOCKER (must resolve before v0.13.0 close)

### `docs-alignment/walk` (P0, PASS→FAIL) — P94-induced docs-alignment drift

- **Reason:** `STALE_DOCS_DRIFT` on `crates/reposix-core/src/backend.rs`
  (`sources_drifted=[0]`) for 8 `docs/connectors/guide/*` BackendConnector-trait
  claims (`trait-method-count-eight`, `backendconnector-{create,get,
  list-changed-since,list-records,name,root-collection-name,supports}-method`).
- **Root cause:** P94 Fork A commit `5cb9a14` ("gate prune_oid_map on connector
  completeness signal") added `Listing` + `list_records_complete` to
  `backend.rs`. That legitimate feature change altered the file's content hash,
  drifting the docs-alignment claims bound to it. The DOCS are (very likely)
  still accurate — a method was added, none removed — so this is a hash-refresh
  certificate obligation, not a lying-doc defect.
- **NOT:** a git<2.34 env-gate; a silent regression; a broken-behavior
  regression (the Fork A behavior itself is proven GREEN by
  `agent-ux/p94-pagination-prune-completeness-gate`, which PASSED).
- **Resolution (out of executor scope):** a top-level Claude session must run
  `/reposix-quality-refresh crates/reposix-core/src/backend.rs` (or
  `/reposix-quality-backfill`) to re-bind the 8 claims. Per `.planning/CLAUDE.md`
  ("Orchestration-shaped phases run at top-level") + the row's own owner_hint,
  this slash command CANNOT run from inside `/gsd-execute-phase` (depth-2
  fan-out unreachable in an executor). REPORTED to the coordinator/owner and
  filed to SURPRISES-INTAKE.

## Accounted-for env-gates (NOT regressions)

| Row | Blast | State | Classification |
|---|---|---|---|
| `agent-ux/real-git-push-e2e` | P0 | NOT-VERIFIED (exit 75) | **git 2.25.1 < 2.34** env-gate — stateless-connect partial-clone fetch needs git≥2.34; PASSES on CI git 2.54. |
| `agent-ux/t4-conflict-rebase-ancestry` | P0 | NOT-VERIFIED (exit 75) | **git 2.25.1 < 2.34** env-gate — same fetch path. PASSES on CI git 2.54. |
| `agent-ux/t4-conflict-rebase-ancestry-real-backend` | P0 | NOT-VERIFIED | **pre-release-real-backend env-gate** (no creds / non-default allowlist). Honest SLOT state per OD-2. |
| `agent-ux/p93-partial-failure-recovery-real-confluence` | P0 | NOT-VERIFIED | pre-release-real-backend env-gate (no creds). P93 row (out of D4 scope). |
| `release/cargo-binstall-resolves` | P1 | FAIL | **missing-tool env-gate** — "cargo-binstall is not installed in the verification environment"; kind:container, passes where cargo-binstall is present (CI). Was NOT-VERIFIED at HEAD (not a PASS→FAIL flip). |
| `benchmark-claim/8ms-cached-read` | P2 | NOT-VERIFIED | structurally-manual row (no `verifier.script`) — cannot mechanically PASS; honest NOT-VERIFIED. |
| `benchmark-claim/89.1-percent-token-reduction` | P2 | NOT-VERIFIED | structurally-manual row (no `verifier.script`). |

**Pre-accounted fact named in the GREEN contract:**
`agent-ux/p92-mid-stream-litmus-t1-t4` (the charter's pre-established git<2.34
exit-75 env-gate) **actually PASSED this run** (on-demand, 0.39s) — its
dark-factory arm does not hit the git≥2.34 stateless-connect path, so the
git<2.34 env-gate did not fire for it here. The git<2.34 env-gate DID fire for
`real-git-push-e2e` + `t4-conflict-rebase-ancestry` (the real-push / rebase
scenarios), which honestly exit 75 → NOT-VERIFIED (never fabricated to PASS).

## Pending milestone-close deliverables (P2 PASS→FAIL — NOT code regressions)

These four re-graded PASS→FAIL because they were carrying **stale PASSes from
the v0.12.0 close** (`last_verified: 2026-05-01T22:00Z`) and their v0.13.0
deliverable does not exist yet. They are forward-looking readiness gates for
work scheduled in the +2 absorption/close phases (P96/P97), not code
regressions. They go green as P96/P97 execute.

| Row | Deliverable | Scheduled |
|---|---|---|
| `agent-ux/p87-surprises-absorption` | SURPRISES-INTAKE.md drained (0 OPEN) | OP-8 Slot-1 (P96) |
| `agent-ux/p88-good-to-haves-drained` | GOOD-TO-HAVES.md terminal-status on every heading | OP-8 Slot-2 (P96/P97) |
| `agent-ux/v0.13.0-tag-script-present` | `tag-v0.13.0.sh` authored | P97 |
| `agent-ux/v0.13.0-retrospective-distilled` | RETROSPECTIVE v0.13.0 section (OP-9) | P97 |

## Subjective-rubric freshness (NOT a code regression)

| Row | Blast | State | Classification |
|---|---|---|---|
| `subjective/dvcs-cold-reader` | P1 | PARTIAL (exit 2) | subagent-graded rubric needs a fresh dispatch (`/reposix-quality-review`, top-level only). Not a code regression. |

## Active waivers observed (all until 2026-09-15, RENEWED P90 90-05)

`perf/latency-bench`, `perf/token-economy-bench`, `perf/headline-numbers-cross-check`,
`subjective/{headline-numbers-sanity,install-positioning,cold-reader-hero-clarity}`,
`cross-platform/{windows-2022,macos-14}-rehearsal`, `code/cargo-test-pass`,
`docs-repro/{tutorial-replay,example-01,02,04,05}` — all carry live waivers; not
FAILs.

## P94 rows — reverted to NOT-VERIFIED (own work; unbiased phase-close grades)

`docs-build/p94-badges-real-vs-transient`, `agent-ux/p94-pagination-prune-completeness-gate`,
`agent-ux/p94-git243-fallback-sentinel`, `structure/p94-catalog-freshness-sweep`
were flipped by the sweep runner and **reverted** (`git checkout HEAD --
quality/catalogs/`) so they remain NOT-VERIFIED for the unbiased phase-close
verifier.

## Runner self-mutation during this sweep

The sweep mutated 6 catalog JSON files in place (the recurring quality-runner
self-mutation bug — reverted; occurrence + exact rows appended to the existing
SURPRISES-INTAKE self-mutation entry, fired-count incremented to 4). A NEW,
worse manifestation was captured: bumping legacy rows' `last_verified` past P90
validation cutoffs made two catalogs **fail to load** on later cadences
(`release-assets.json` → on-demand SystemExit; `agent-ux.json` →
pre-release-real-backend SystemExit), truncating the sweep. Data for P96.

## NOTICING: `run.py --all` does not exist

The D4 row's `command` named `python3 quality/runners/run.py --all`, but run.py
has NO `--all` flag (argparse REQUIRES `--cadence`). The canonical all-rows
sweep today = every cadence once (this driver, `94-D4-sweep.sh`). Tracked as
GOOD-TO-HAVES-03 (`--row`/`--dimension`/all-rows scope flags, deferred runner
surgery). The D4 row's `command` field was corrected to point at the driver.
