---
phase: 115-live-mcp-benchmark-re-measurement
verified: 2026-07-16T17:15:00Z
status: GREEN-CHECKPOINT
score: 7/7 must-haves verified (agent-side); 1 checkpoint open (human-only confirm-retire, 11 rows)
overrides_applied: 0
human_verification:
  - test: "Run the FINAL consolidated confirm-retire batch (11 rows) from a real TTY"
    expected: "quality/catalogs/doc-alignment.json shows 11 rows flip RETIRE_PROPOSED -> RETIRE_CONFIRMED; git status shows only that one file modified; commit+push lands"
    why_human: "confirm-retire is env-guarded (refuses under $CLAUDE_AGENT_CONTEXT / non-TTY stdin) by design — no agent can execute this, per quality/PROTOCOL.md Principle B and the tool's own guard"
---

# Phase 115: Live MCP Benchmark Re-measurement Verification Report

**Phase Goal:** Live MCP benchmark re-measurement + honest headline re-anchoring — replace
stale/synthetic latency and token-economy figures with fresh, real-backend-measured numbers,
and retire the old claims from the doc-alignment/perf catalogs.
**Verified:** 2026-07-16T17:15:00Z
**Status:** GREEN-CHECKPOINT (all agent-side work complete and verified against reality; one
human-only checkpoint — confirm-retire — remains open by design, not by omission)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from 115-PLAN.md `must_haves.truths`)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Fresh latency measurements (cold init + cached read) for sim AND all 3 real backends exist in `docs/benchmarks/latency.md`, timestamped after 2026-07-15 | VERIFIED | Frontmatter `last_measured_at: 2026-07-15T21:45:40Z`; table has 4 populated columns (sim/github/confluence/jira) with cold-init, list, get, patch, capabilities rows; canonical source cited as CI run `29452237641` |
| 2 | 24ms-vs-27ms cold-init discrepancy resolved to ONE authoritative figure | VERIFIED | `docs/benchmarks/latency.md` carries a "Corrected: 2026-07-15" banner superseding the noise-outlier figure; canonical `278 ms` used consistently in the table and cross-checked by `headline-numbers-cross-check.py` (12/12 + 26/26 pytest passing per 115-T6-CLOSEOUT.md) |
| 3 | Fresh live-MCP token-economy figures from a REAL MCP server against sanctioned real backends (not synthetic 35-tool manifest) | VERIFIED | `docs/benchmarks/token-economy.md` methodology names `github/github-mcp-server` (GA, official) as `github-probe`, backend `reubenjohn/reposix` GitHub issues (sanctioned OP-6 target); `benchmarks/bench-session-ledger.md` records 6 real sessions (rows 2-7) with real timestamps and capture artifacts |
| 4 | `benchmarks/fixtures/reposix_session.txt` is honest, current, no `/mnt` FUSE paths, no `scripts/demo.sh` provenance | VERIFIED | `grep -n "/mnt\|demo.sh\|FUSE"` on the file returns zero matches; file dated Jul 15 23:10, 8041 bytes |
| 5 | Session-spend ledger records every live-MCP session, running total ≤50 | VERIFIED | `benchmarks/bench-session-ledger.md`: "Sessions spent: 7 / 50", 7 rows, running_total increments correctly 1→7, outlier guard documented |
| 6 | `docs/benchmarks/token-economy.md` provenance names the real capture method, not "modeled on" / `scripts/demo.sh` literal-output | VERIFIED | Header reads "Measured: 2026-07-16, from 6 live agentic sessions captured during P115 Task 4" / "Source: committed session-usage records in `benchmarks/captures/*.json`" |
| 7 | A documented un-waive path names the exact perf-targets rows + script/line a future code phase must wire | VERIFIED | `115-UNWAIVE-PATH.md` (349 lines) enumerates all 21 originally-waived rows by ID, live state, exit route, and who acts; both perf rows (`perf/token-economy-bench`, `perf/headline-numbers-cross-check`) are now un-waived and PASS (confirmed live in catalog, see below) |

**Score:** 7/7 plan must-have truths VERIFIED against live codebase content (not SUMMARY claims).

### Roadmap-contract checkpoint (human-only gate, not a truth failure)

The phase's `115-UNWAIVE-PATH.md` § "FINAL consolidated confirm-retire batch" names 11
doc-alignment rows that require `confirm-retire`, a tool that is **env-guarded to refuse
under `$CLAUDE_AGENT_CONTEXT` or non-TTY stdin** (`quality/PROTOCOL.md` Principle B, verified
in the tool's own `--help` text transcribed in `115-UNWAIVE-PATH.md`). This is a designed
human-only checkpoint, not an incomplete truth — every truth above is fully agent-completable
and IS complete. Per the launching orchestrator's explicit instruction, this is graded as a
CHECKPOINT condition, not a phase-close blocker.

**Verified count:** `grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` → **11** (matches expected exactly).

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `docs/benchmarks/latency.md` | Fresh 4-backend latency table, canonical cold-init | VERIFIED | Populated table, "Corrected" banner, CI-canonical provenance |
| `docs/benchmarks/token-economy.md` | Real-session token-economy figures + honest provenance | VERIFIED | 6-session median-of-3, real capture method named, offline-reproducible via `bench_token_economy.py --offline` |
| `benchmarks/fixtures/reposix_session.txt` | Honest git-native transcript | VERIFIED | No FUSE/demo.sh artifacts |
| `benchmarks/bench-session-ledger.md` | Session-spend ledger ≤50 | VERIFIED | 7/50, real timestamps |
| `115-UNWAIVE-PATH.md` | Un-waive path document | VERIFIED | 21-row inventory + exit routes + FINAL 11-row confirm-retire batch |
| `115-T6-CLOSEOUT.md` | T6 all-item evidence | VERIFIED | Wave 1/2 evidence log with verify-against-reality artifacts (pytest counts, walk rc=0, sha256 comparisons) |
| `P116-ADR-010-DECISION-PACKET.md` | Owner ruling packet | VERIFIED | Header reads "Status: RULED 2026-07-16"; both `[MANAGER]` ruling entries confirmed present in `.planning/CONSULT-DECISIONS.md` (lines 113, 140) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `perf/token-economy-bench` catalog row | `bench_token_economy.py` `_assert_headline_reduction()` | un-waive + mint | WIRED | Live catalog: `status=PASS`, un-waived, confirmed in weekly run output |
| `perf/headline-numbers-cross-check` catalog row | `headline-numbers-cross-check.py` | un-waive + mint | WIRED | Live catalog: `status=PASS`, un-waived; script confirmed present on disk (was dangling-verifier pre-phase) |
| doc-alignment `docs/index/latency-8ms-read` etc. | live doc content | bind | WIRED | Live `docs/index.md:18` and `README.md:25` read `6 ms`, not stale `8 ms` — content matches bound claim |
| `docs/index.md` / `README.md` headline prose | `docs/benchmarks/{latency,token-economy}.md` | direct read | WIRED | Confirmed by direct file read: hero cards show `6 ms` / `278 ms` / `~94% fewer output tokens` / `~75% cheaper`, not the retired `8 ms` / `27 ms` / `89.1%` |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Catalog-row grading, phase scope | `python3 quality/runners/run.py --cadence weekly` | `17 PASS, 0 FAIL, 0 PARTIAL, 2 WAIVED, 2 NOT-VERIFIED -> exit=0` | PASS (no FAIL/PARTIAL in scope) |
| 11-row human-gate count | `grep -c '"last_verdict": "RETIRE_PROPOSED"' quality/catalogs/doc-alignment.json` | `11` | PASS (matches documented exception exactly) |
| Push landed | `git rev-parse origin/main` after fetch | `2f96e69...` (descendant of required `8212373`) | PASS |
| Main CI green (post-push) | `gh run watch <latest run id> --exit-status` | Run `29518587955` (commit `2f96e69`) — all jobs `success`, run `completed/success` | PASS |
| Live headline content matches new figures | `grep`/`sed` on `docs/index.md`, `README.md` | `6 ms` / `278 ms` / `~94%` / `~75%` present; no `8 ms`/`89.1%` in hero sections | PASS |

### Requirements Coverage

No `Phase 115` literal entry found in `.planning/milestones/v0.15.0-phases/ROADMAP.md` (this
milestone's ROADMAP.md does not enumerate every phase by number at top level); the phase's own
`115-PLAN.md must_haves.truths` frontmatter is the authoritative contract used above and is
fully satisfied (7/7).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `quality/catalogs/docs-reproducible.json` | rows `benchmark-claim/8ms-cached-read`, `benchmark-claim/89.1-percent-token-reduction` | `NOT-VERIFIED`, `verifier.script: null` | Info (pre-existing, not a P115 regression) | These two legacy "manual"-kind rows predate P115 (minted P106, `804eedc`, well before this phase); their `sources` still cite the now-superseded `8 ms`/`89.1%` claim locations. They are explicitly documented as "intentionally NOT waived/softened... permanently yellow" (D-CONV-2, 2026-07-04) and routed to a future launch-readiness milestone (GOOD-TO-HAVES-04) — not P115's obligation. `weekly` cadence exit stays 0 because P2 NOT-VERIFIED rows don't gate exit. Flagged here as **noticing**, not a blocker: since P115 changed the exact content these rows cite, a follow-up could re-point or retire them so the catalog stops referencing dead claim text — worth a GOOD-TO-HAVES entry if not already filed. |

No blocker-severity anti-patterns found in phase-scope artifacts.

### Human Verification Required

### 1. Run the FINAL consolidated confirm-retire batch

**Test:** From a real terminal (NOT a Claude Code session, NOT under `$CLAUDE_AGENT_CONTEXT`),
run each of the 11 `target/release/reposix-quality doc-alignment confirm-retire --row-id <ID>`
commands listed verbatim in `115-UNWAIVE-PATH.md` § "FINAL consolidated confirm-retire batch".
**Expected:** All 11 rows flip `RETIRE_PROPOSED` → `RETIRE_CONFIRMED` in
`quality/catalogs/doc-alignment.json`; `git status` shows only that one file modified; commit +
push lands as the phase-close human-gate landing.
**Why human:** `confirm-retire` is deliberately env-guarded — it refuses to run under
`$CLAUDE_AGENT_CONTEXT` or non-TTY stdin (`quality/PROTOCOL.md` Principle B). No agent, however
capable, can pass this guard. This is by design, not a gap in the phase's work.

### Gaps Summary

No gaps found in agent-completable scope. All 7 plan must-have truths are verified true
against live file content (not SUMMARY narrative) — fresh 4-backend latency measurements,
resolved cold-init figure, real live-MCP token-economy capture, honest fixture/ledger, and a
complete un-waive path document all exist and are internally consistent with the live catalog
state. Push cadence satisfied (main descendant of required commit, latest CI green — verified
by watching the run to completion, not by trusting a prior status). The only open item is the
human-only `confirm-retire` batch (11 rows), which is a designed checkpoint external to what
any agent can close — not a defect in this phase's delivery.

**Noticing (ownership charter):** the two stale `docs-repro/benchmark-claim-*` rows above are
pre-existing catalog debt whose `sources` field now point at claim text this very phase
retired/rewrote elsewhere. Not a P115 defect (predates the phase by 10 days), but a
"next-agent" tripwire worth a one-line GOOD-TO-HAVES entry if the intake doesn't already carry
one, so a future doc-repro refresh knows these two rows' citations are dangling against moved
content.

---

_Verified: 2026-07-16T17:15:00Z_
_Verifier: Claude (gsd-verifier)_
