# P91 — attach + sync real-backend wiring + QL-001 — VERIFIER VERDICT

**Verdict: GREEN**
**Graded:** 2026-07-05 · **Verifier:** unbiased phase verifier (zero session context)
**HEAD:** 8e9115b · **origin/main:** 8e9115b (in sync — push landed)
**Scope:** ROADMAP SC1–SC9 + QL-001 criteria 1–6 + 5 in-scope catalog rows + the D91-07 REOPEN gate + OP-8 honesty spot-check

Grading is from committed artifacts + the fresh 2026-07-05T02:23Z cadence run. The one P0
transport row (`real-git-push-e2e`) is env-gated NOT-VERIFIED on this box (git 2.25 < 2.34, the
designed honest behavior); its PASS is read from CI run 28726703296 (git ≥ 2.34) at 01cd552, not
inferred. Mechanical `ql-001` re-run locally (exit 0). One cargo build was avoided for
`dvcs-third-arm` (CI PASS + fresh local artifact both dated this session make a redundant workspace
link unnecessary per the memory budget).

---

## Catalog-row grades (5 in-scope rows)

| Row | Dim | P | Evidence | Grade |
|---|---|---|---|---|
| agent-ux/ql-001-canonical-path-shape | agent-ux | P1 | re-ran `ql-001-canonical-path.sh` → exit 0, 16 asserts (grep-proven: zero `format!("{:04}/{:011}")` outside reposix-core, shared `record_path`/`issue_id_from_path` in core, QL-157 dup deleted from `main.rs`, all 16 regression fns present); CI pre-pr PASS (0.04s) at 01cd552 | **PASS** |
| agent-ux/attach-sync-real-backend | agent-ux | P1 (real-backend) | fresh transcript `attach-sync-real-backend-2026-07-05T02-23-26Z.txt`: argv `cargo test -p reposix-cli --test agent_flow_real -- --ignored attach_real_confluence sync_real_confluence` (real Confluence, creds+`REPOSIX_ALLOWED_ORIGINS` in env), `2 passed`, exit 0; env_keys are NAMES-only (no `=value`); artifact `asserts_failed:[]` | **PASS** |
| agent-ux/real-git-push-e2e | agent-ux | **P0** | local artifact exit 75 `git_too_old` (2.25.1) → NOT-VERIFIED **by design** (row assert 3); CI run 28726703296 pre-pr log at 01cd552: `[PASS] agent-ux/real-git-push-e2e (P0, 0.60s)` — single-PATCH round-trip + no-op idempotency proven on git ≥ 2.34 | **PASS** (CI-authoritative) |
| agent-ux/dvcs-third-arm | agent-ux | P1 | fresh artifact `dark-factory-dvcs-third-arm.json` (2026-07-05T00:07Z) exit 0, all 20 asserts incl. `matched=3 no_id=1 backend_deleted=1 mirror_lag=2` (4 non-zero) + duplicate-id hard-abort; CI pre-pr PASS (0.54s) at 01cd552 | **PASS** |
| agent-ux/milestone-close-vision-litmus-real-backend | agent-ux | **P0** (real-backend) | fresh transcript `...-2026-07-05T02-23-17Z.txt`: real vanilla-clone + `reposix attach confluence::REPOSIX` + edit + `git push reposix main` round-trip against TokenWorld page 2818063, REST-confirmed marker, dual-table audit (cache=5, events=1), refs/mirrors advanced, gitleaks clean; artifact 11/11 asserts PASS incl. D90-06 sanctioned-target assertion; env_keys NAMES-only; exit 0 | **PASS** |

All P0 (`real-git-push-e2e`, `milestone-close-vision-litmus-real-backend`) and P1 rows PASS. No FAIL, no PARTIAL. `real-git-push-e2e`'s local NOT-VERIFIED is the row's own designed env-gap path, PASS in CI.

## Success-criteria grades

| SC | Grade | Evidence (file:line) |
|---|---|---|
| SC1 — attach reconciles by frontmatter id (5 cases) | **PASS** | litmus transcript `attach: matched=3 no_id=1 backend_deleted=0 mirror_lag=0`; `dvcs-third-arm` artifact covers matched/no_id/backend_deleted/mirror_lag + duplicate-id hard-abort (case 4) |
| SC2 — `agent_flow_real` attach/sync real family GREEN w/ creds, skip w/o | **PASS** | 6 fns present `agent_flow_real.rs:343-437` (attach/sync × confluence/github/jira); confluence pair `2 passed` in fresh transcript; cadence `pre-release-real-backend` default-skips (`cadence-pre-release-real-backend.json` `skipped_real_backend:true` when `REPOSIX_ALLOWED_ORIGINS` unset) |
| SC3 — no `P\d+-\d+` token in attach stderr | **PASS** | `structure/banned-production-tokens` CI pre-pr PASS (0.18s) at 01cd552; enforced from P89 RBF-FW-04 |
| SC4 — dvcs-third-arm populates tree; ≥3 non-zero reconciliation counts | **PASS** | `dark-factory-dvcs-third-arm.json`: `matched=3 no_id=1 backend_deleted=1 mirror_lag=2` (4 non-zero, closes p86 F13) |
| SC5 — T2 re-runs, 5/5 boxes pass | **PASS** | litmus transcript = T2 flow (clone→attach→edit→commit→push→REST-confirm), all boxes PASS |
| SC6 — REOPEN gate + D90-06 sanctioned-target litmus body | **PASS** | litmus artifact exit 0, 11/11; run-2 `HIGH=0` (gate PASS); `milestone-close-vision-litmus.sh:70-72` STEP-1 sanctioned-target hard-FAIL (exit 1, OD-2), not a second weaker allowlist |
| SC7 — catalog rows + REQUIREMENTS flip + CLAUDE.md land first / same PR | **PASS** | 710211d (early P91 commit) minted both rows NOT-VERIFIED before impl; `REQUIREMENTS.md:45-50,132-135` honest "P79 sim / P91 real-backend" flip w/ RBF-A-06 coverage note; CLAUDE.md revised in-phase (d34ad75) |
| SC8 — push origin main; verifier GREEN; verdict written | **PASS** | origin/main == HEAD 8e9115b; this verdict at `quality/reports/verdicts/p91/VERDICT.md` |
| SC9 — QL-001 lands; real-git-push-e2e waiver retired not renewed | **PASS** | catalog row `waiver: null` (retired at c9e2b8f, not renewed); all 6 QL-001 criteria PASS (below) |

## QL-001 acceptance criteria (1–6)

| # | Grade | Evidence |
|---|---|---|
| 1 — real push round-trips ONE edit as single PATCH, misclass==0 | **PASS** | CI `real-git-push-e2e` Assertion 1 `PASS: real push round-tripped as exactly 1 PATCH (0 Create, 0 Delete)` (intake 2026-07-05 line 568); canonical path fix d6e1411/1c03da0 |
| 2 — no-op push writes zero backend mutations | **PASS** | CI `real-git-push-e2e` Assertion 2 GREEN at 01cd552 after oid_map fix (01cd552) + server-field-drift planner fix (4feb5f6); regression `noop_push_server_field_drift.rs` + `oid_map_returns_current_oid.rs` |
| 3 — full seeded tree push, zero Deletes | **PASS** | ql-001 regression `full_seeded_tree_push_emits_zero_deletes` + `pages_full_tree_push_emits_zero_deletes` present & PASS |
| 4 — `.reposix/` metadata pushes without invalid-blob reject | **PASS** | ql-001 regression `reposix_metadata_paths_are_ignored_not_rejected` present & PASS (issues/*.md-only plan filter, 4bebfa3) |
| 5 — REAL git push e2e at agent-ux gate; asserts version, fails loud on old git | **PASS** | `real-git-push-e2e` exit 75 loud NOT-VERIFIED on git 2.25 (not silent skip); PASS on CI git ≥ 2.34 |
| 6 — all 4 path sites one canonical spelling via shared helper; QL-157 dedup | **PASS** | ql-001 asserts: zero `format!` record-path outside core, shared `issue_id_from_path`/`record_path` in reposix-core, QL-157 duplicate deleted from `main.rs` |

## D91-07 REOPEN gate

| Layer | Grade | Evidence |
|---|---|---|
| (a) mechanical litmus exit 0 | **PASS** | `milestone-close-vision-litmus-real-backend.json` status PASS, `asserts_failed:[]`, 11/11, exit 0 (fresh 2026-07-05T02:23:17Z) |
| (b) fresh-agent T2 run-1 (HIGH=4) all fixed | **PASS** | SURPRISES-INTAKE 2026-07-04 22:45: H1/H2 RESOLVED 854586b, H3 RESOLVED 807ec7a/d34ad75, H4/MED6 RESOLVED d6f7966 (helper fix-line 726f277) |
| (b) fresh-agent T2 run-2 (HIGH=0, MED=3, LOW=1) gate PASS | **PASS** | SURPRISES-INTAKE 2026-07-04 23:00: `HIGH=0, MED=3, LOW=1`, all 5 boxes ticked, gate PASSES; 4 MEDs/LOW honestly ROUTED to post-P97 docs drain |

Both runs are recorded honestly, run-1 HIGH items carry fix SHAs, run-2 records gate PASS with MEDs ROUTED (not silently dropped).

## OP-8 honesty spot-check

**PASS.** SURPRISES-INTAKE is NOT empty (~50 entries); every P91 pivot is journaled WITH a disposition + SHA:
confluence mass-delete BLOCKER (line 440) RESOLVED d6e1411 (bucket-aware paths + id-keyed planner);
second-push mass-delete BLOCKER (line 475) RESOLVED 5612fa6 (`saw_commit` guard + `second_push_no_commit.rs`);
QL-001 fetch-path 3-layer fix (line 537) RESOLVED 5c758fb/a4bb090; Assertion-2 oid_map (line 582) RESOLVED
(`find_oid_for_record ORDER BY rowid DESC`, RED-proven); minted_at landmine (line 572) RESOLVED 09e10c1
(honest write-once anchor, explicitly NOT the restore-before-commit workaround). No finding-without-disposition
flagged. The deferred P96/P97 audit-field-drift item (line 514) is filed honestly with cadence rationale.

Two P2 honesty nits (non-blocking, do not affect any grade): (i) the Assertion-2 oid_map RESOLVED entry
(line 600) closes with "Commits: see the P91 final-red-row pushes" — a pointer rather than the concrete
hash (the fix is 01cd552, confirmed in `git log`); every sibling RESOLVED entry cites an explicit SHA.
(ii) the T2-REOPEN run-1 entry (line 491) journals `HIGH=4` with per-finding fix SHAs but does not record a
`MED/LOW` breakdown for run-1 (ROADMAP:180 + the run-2 entry do record run-2's `HIGH=0 MED=3 LOW=1`). Neither
weakens the load-bearing evidence — all 4 run-1 HIGH items carry fix SHAs and run-2 gate PASS is fully tallied.

## CLAUDE.md (QG-07)

**PASS.** `git diff 32ba856..HEAD -- CLAUDE.md` reflects all P91 conventions: attach real-backend dispatch
via `backend_dispatch` factory + `remote.pushDefault` + missing-helper warning (attach bullet, D91-03);
bucket-aware paths `issues/`/`pages/` via `reposix_core::path` (D91-13); litmus is now a real round-trip not
an exit-75 stub (9th-probe paragraph, D91-06, "11/11 asserts"); mirror-refs reality correction (91-06). The
`dvcs-third-arm` command + its "cache audit" description were already documented pre-P91 and remain accurate.

---

## Overall verdict: **GREEN**

Every in-scope catalog row is PASS: both P0 rows (`real-git-push-e2e` PASS via CI at 01cd552 with the local
NOT-VERIFIED being its own designed git-version gate; `milestone-close-vision-litmus-real-backend` PASS with a
genuine, non-synthetic real-TokenWorld round-trip — real `reposix` binary, real REST-confirmed page marker,
dual-table audit, NAMES-only env_keys) and all three P1 rows. All nine success criteria and all six QL-001
acceptance criteria hold, each with a committed-artifact or read-not-inferred-CI citation. The D91-07 REOPEN
gate is genuinely closed — mechanical litmus 11/11 exit 0 plus two honestly-recorded fresh-agent T2 runs
(run-1 HIGH=4 all fixed with SHAs, run-2 HIGH=0 gate PASS, MEDs ROUTED). The two catastrophic mass-delete
BLOCKERs surfaced during the phase were both resolved in-phase with RED-proven regressions, and the
SURPRISES-INTAKE honestly journals every pivot with a disposition and SHA — the honesty check passes rather
than showing the empty-intake-beside-skipped-findings anti-pattern. CLAUDE.md is updated in the same PR.
Non-blocking note: the newest main CI run (8e9115b, a catalog-JSON-only commit) was still in progress at grade
time; the authoritative full pre-pr sweep at 01cd552 was 61 PASS / 0 FAIL / 0 PARTIAL / 3 WAIVED (only the
pre-existing `structure/file-size-limits`, unrelated) / 0 NOT-VERIFIED. Phase 91 ships GREEN.
