# QUALITY-LEDGER — Quality Convergence audit ledger (reconstructed)

**Status: RECONSTRUCTION.** The original 170-row ledger (6 BLOCKER / 51 HIGH /
62 MED / 51 LOW; routes: eager-window 101, intake 49, catalog-row 9,
wontfix 11) lived at `/tmp/claude-1000/.../QUALITY-LEDGER.md` and was lost
when the OS shut down mid-run on 2026-07-04 before the planned copy into the
repo. This file reconstructs everything provable from durable artifacts
(commits, `quality/SURPRISES.md` D-CONV journal, intake files) and records
the recovery method for the unrecoverable remainder.

**Recovery method for lost rows:** rows already routed before the shutdown
survive as commits/intake entries (tabled below). Undrained eager-window rows
cannot be replayed row-by-row; instead, fresh-eyes read-only re-audits over
the changed surfaces (Round 1: 2026-07-04, two sonnet auditors — connector
residuals + quality-framework surfaces) regenerate anything still real.
A finding that no fresh audit can re-find is treated as fixed-or-moot; this
is the honest floor, not a claim of zero residual.

## BLOCKER dispositions (QL-001..006) — all dispositioned

| ID | Finding | Status |
|---|---|---|
| QL-001 | Push-planner path-shape mismatch (+2 compounding bugs) — `diff::plan` keys on bare `NNNN.md`, real trees use `issues/<id>.md` | INTAKE (BLOCKER) `75db262` — verified repro + 4-site path-spelling analysis in v0.13.0 SURPRISES-INTAKE; anchor gate `agent-ux/real-git-push-e2e` minted `5a70aed` (waived until 2026-07-31, goes green when P90/P91 fix lands) |
| QL-002 | CAPABILITIES claims diverged from impl (JIRA claimed read-only with writes implemented, +3 surfaces) | FIXED `ca7cb61`; regression rows `code/capabilities-match-impl` `5a70aed` |
| QL-003 | binstall pkg-url named a nonexistent release asset | FIXED `33dd41f` |
| QL-004 | `reposix` subcommands fail on attached trees (remote-name resolution not partialClone-aware) | FIXED `8fccaf8` (+ doc follow-through `8f81fe6`) |
| QL-005 | `audit_events` not wired into helper Confluence/JIRA dispatch (OP-3 violation) | FIXED `a0c84a3`; regression row `security/connector-audit-wired` `5a70aed` |
| QL-006 | Mirror push egress bypassed `REPOSIX_ALLOWED_ORIGINS` | FIXED `a4b0aff` |

Stage-0 items: Google-API-key (AIza) cred-hygiene pattern FIXED `5f2b261`
(runtime-built fixture); false-positive language softened `ebba644` (owner
confirmed the flagged Google key was a FALSE POSITIVE, 2026-07-03).

## Other rows provably routed before the shutdown

| ID / item | Route | Evidence |
|---|---|---|
| QL-027 cold-init latency doc drift (27 ms canonical) | FIXED | `ead506c` |
| QL-042 quality/SURPRISES.md dead journal | FIXED | `5bba77a` (D-CONV-8 revival) |
| D-CONV-1 pre-pr cadence never ran in CI | FIXED | `8fd28cc`, margin `a7fbbb8` |
| D-CONV-2 verdict badge lied on yellow | FIXED | `c8bd2ec` (+CI fix `290bb21`) |
| D-CONV-3 scripts/ shim sprawl | FIXED | `c0d5459` + inverse gate `structure/scripts-registry-complete` |
| D-CONV-4 secret scanning single-layer | FIXED | `27b6173` `369dfc3` `4df9c31` `d0af03e` |
| D-CONV-5 walker last_walked churn | FIXED | `b04fd8d` |
| D-CONV-6 test-pre-push dirty-tree hazard | FIXED | `b866c90` |
| D-CONV-7 CLAUDE.md over budget | FIXED | this window (compaction + `.planning/PRACTICES.md`) |
| 5 false-BOUND doc-alignment rows | FIXED | `934dd37` honest re-grade |
| Steward-window findings (bench PR anomaly, quality-weekly chronic yellow) | INTAKE | `e1b86f2` |
| Intake entries 11–14 (fragile Confluence contract test, JIRA_TEST_PROJECT in CI, CI annotation noise, stale check-quality-catalogs.py) | INTAKE | v0.13.0 SURPRISES-INTAKE (committed pre-shutdown) |
| Catalog-row route (9 planned) | CATALOG | 5 landed: real-git-push-e2e, capabilities-match-impl, connector-audit-wired, cli-subcommand-parity (all `5a70aed`+`24948ee`), scripts-registry-complete (`c0d5459`). Remaining 4 candidates were not named in any durable artifact — regenerable by future audits if the underlying gaps re-surface. |

## Connector-audit findings (25-finding sweep) — re-verified 2026-07-04

| Finding | Severity | Status |
|---|---|---|
| JIRA CAPABILITIES claims read-only, writes implemented (types.rs:35) | BLOCKER | FIXED `ca7cb61` (subsumed by QL-002) |
| GitHub lists PRs as issues — no `pull_request` filter in list_records / list_changed_since / get_record (reposix-github/src/lib.rs:399-543) | HIGH | OPEN → fix wave this window |
| JIRA backoff dead code — `arm_rate_limit_backoff` zero prod callers + lib.rs:33-36 doc claims unimplemented exponential-backoff behavior | HIGH | OPEN → fix wave this window |
| ~440 lines dead Confluence API (list_comments / list_attachments / list_whiteboards / download_attachment + backing types; larger than the ~340 estimate) | HIGH | INTAKE → v0.13.0 SURPRISES-INTAKE (wire-vs-delete belongs to P91/P92 real-backend charter) |
| Swarm harness: stale "Phase 17 read-only" claim; zero write-contention coverage despite live Confluence writes | HIGH | SPLIT: stale claim → fix wave (XS); write-contention workload → INTAKE (M) |
| Confluence missing capabilities self-check test (github+jira have one) | MED | OPEN → fix wave this window (XS) |
| Remaining ~19 MED/LOW connector rows | MED/LOW | LOST with the ledger; covered by the recovery method above (fresh audits re-find anything real) |

## Re-audit Round 1 (2026-07-04, fresh-eyes sonnet x2 over changed surfaces)

| Finding | Severity | Status |
|---|---|---|
| verdict.py:189 markdown `Verdict:` line still 2-state (GREEN/RED) — contradicts yellow badge on next weekly run; untested | HIGH | OPEN → fix wave this window (XS) |
| freshness-invariants.json:333 cred-hygiene assert text names stale `scripts/hooks` EXCLUDE_DIRS, omits real `quality/gates/structure` self-exclusion | MED | OPEN → fix wave this window (XS) |
| deferral-pointer-linter regex blind spot: "wired in Phase 29" (word form) escapes all patterns — proven non-hypothetical by the JIRA stale pointer | MED | INTAKE → GOOD-TO-HAVES (S) |
| ci.yml quality-pre-pr timeout 15m vs PROTOCOL.md <10min budget (in-comment justified) | LOW | WONTFIX — justified in-line where a reader meets it; cross-file duplication of the number would add drift risk, not remove it |
| PEM pattern d0af03e | — | verified correct (hand-traced) |
| All other audited surfaces (cli-subcommand-parity, orphan-scripts registry, walk() skip logic, dirty-tree guard, SECURITY.md vs gates, script hygiene) | — | CLEAN |

Round 2+ results are appended below as they complete.
