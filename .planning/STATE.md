---
gsd_state_version: 1.0
mode: serial-workstreams
status: v0.14.0-SHIPPED-public-b773c04-red-main-CLOSED-post-tag-queue-items-0-5-in-progress
last_updated: "2026-07-13"
last_activity: "2026-07-13 — b773c04 RED-main blocker CLOSED: docs-repro timeout-budget fix landed @ 8e2aae5 (origin/main), CI run 29302967371 SUCCESS (all 15 jobs) + quality-post-release run 29302973970 SUCCESS. v0.14.0 fully SHIPPED + public (crates.io 0.14.0, GitHub release marked 'Latest'); fix-first arc DONE. Post-tag queue items 0-5 resumed by successor #17 (item 0 = this GSD cursor refresh). Live runbook: .planning/SESSION-HANDOVER.md."
workstreams:
  workstream_a:
    milestone: v0.13.0
    milestone_name: DVCS over REST (extended)
    status: closed-green  # v0.13.0-extension CLOSED GREEN with owner-gated caveats 2026-07-05 (milestone verdict quality/reports/verdicts/milestone-v0.13.0/VERDICT.md, verdict commit 390ce31, graded HEAD 3c6d72f)
    phases_total: 20  # P78-P97 (P78-P88 shipped + P89-P97 extension)
    phases_completed: 20  # P78-P97 (workstream A / v0.13.0-extension COMPLETE)
    next_phase: P98  # v0.13.0 SHIPPED — tagged/released 2026-07-07 (commit 3423b18, "chore: release v0.13.0 (#68)"); v0.13.1 hotfix shipped 2026-07-08 (04640d5). No tag pending; workstream B (v0.13.2) queued behind workstream C per OD-4.
    blocks_tag: false  # v0.13.0 tag SHIPPED 2026-07-07 (3423b18) — nothing tag-pending here
  workstream_b:
    milestone: v0.13.2
    milestone_name: Cross-link fidelity
    status: queued  # RESEQUENCED per OD-4 item 3 (2026-07-04): queued BEHIND v0.14.0 wave-2 hardening (established 2026-07-11) AND the launch-readiness milestone; phase numbers (originally P98-P107) shift again to accommodate v0.14.0's P102-P112 claim when eventually replanned
    phases_total: 10  # P98-P107 (placeholder range, pending renumber-on-insertion at replan time)
    phases_completed: 0
    next_phase: P98
    blocks_tag: false  # v0.13.2 tag ships after v0.13.0 tag AND v0.14.0 wave-2 hardening AND the launch-readiness milestone per OD-4; tag push delegated to orchestrator
  workstream_c:
    milestone: v0.14.0
    milestone_name: Wave-2 hardening
    status: shipped-green  # v0.14.0 SHIPPED + Latest 2026-07-14 (crates.io 0.14.0, GitHub release 'Latest'); 11/11 phases GREEN (P102-P112 + out-of-band P113); b773c04 RED-main CLOSED @ 8e2aae5. Nothing tag-blocked.
    phases_total: 11  # P102-P112 (P102 D2 hard gate; P103-P109 carried HIGHs + cheap wins; P110-P111 OP-8 +2 reservation; P112 OD-4 stub)
    phases_completed: 11  # P102-P112 ALL GREEN (P111 milestone-close grade c259718; P112 OD-4 launch-readiness scope stub landed) + out-of-band P113 GREEN
    next_phase: none  # v0.14.0 SHIPPED; post-tag queue items 0-5 in progress. Wholesale re-anchor pending at post-tag /gsd-new-milestone (P112 OD-4 launch-readiness stub).
    blocks_tag: false  # the v0.14.0 tag is owner-cut; orchestrator does not push
---

# Project State

## Current Position

**Mode:** serial-workstreams per OD-3 (workstream A → then B; the parallel-worktree model is RETIRED).

**OD-3 mandate (2026-07-03)** — see `.planning/milestones/v0.13.0-phases/89-framework-fixes-cadence-shell-kind/89-OWNER-DECISIONS.md` § "DECISION OD-3": drive to v1.0. Complete v0.13.0-ext (P89–P97, tag v0.13.0), then v0.13.2 (P98–P107, tag v0.13.2) STRICTLY SERIALLY; after both tags, formalize the research-only ladder (v0.14.0 observability/multi-repo → plugin ecosystem/launch readiness → v1.0.0 + ADR-009 semver activation) as real GSD milestones via `/gsd-new-milestone`. `main` is the working branch (workstream/v0.13.0-ext fast-forwarded into main 2026-07-03 and retired; per-phase push cadence targets origin/main). Full autonomy incl. former hard gates: OD-1's owner sign-off delegated to orchestrator (owner notified, not blocking), tag pushes at P97/P107 delegated contingent on GREEN verdicts, ~$50 pre-authorized for P106 L3 dogfood. OD-2 + litmus REOPEN gates remain in force UNCHANGED — on RED the orchestrator loops back, never waives.

> Full Workstream A (v0.13.0, CLOSED GREEN) pre-tag checklist / release runbook /
> queued post-tag items, and the superseded Workstream B (v0.13.2, QUEUED) narrative,
> live in `.planning/STATE-history.md`.

### Workstream C — v0.14.0 wave-2 hardening — 11/11 phases GREEN — SHIPPED + Latest

> **SHIPPED + Latest (2026-07-14).** v0.14.0 is tagged, released, and marked "Latest" on
> GitHub (crates.io 0.14.0); the b773c04 RED-main incident is CLOSED (fix @ `8e2aae5`).
> Nothing is tag-blocked. The `make_latest` back-tag hazard for FUTURE releases is handled
> in post-tag queue item 1 (`release.yml` `--latest` hardening,
> `.planning/quick/260713-mlh-make-latest-hardening/`). The B1–B5 tag-remediation cursor
> below is superseded by the ship + b773c04 closure — historical record only.

Phase: **P112** (OD-4 launch-readiness SCOPE-BUT-DO-NOT-START stub) — LANDED. **11/11
phases complete** as of 2026-07-12. **P102** (D2 self-safe dark-factory hardening
+ emergent Phase-0 re-seal), **P103, P104, P105** (RBF-LR-03 rebase recovery), **P106**
(waived tutorials/examples — 5 `docs-repro` rows PASS), **P107** (RUSTSEC memmap2/quinn-proto
cleared), **P108** (prune-completeness gate), **P109** (RBF-FW-11 grandfather rule), **P110**
(OP-8 Slot 1 surprises drain — 17 terminal entries), **P111** (OP-8 Slot 2 good-to-haves +
OP-9 milestone-close, graded GREEN at commit `c259718`: CHANGELOG `[v0.14.0]` + `RETROSPECTIVE.md`
v0.14.0 OP-9 distillation + intake prune; **GTH-09** ADR-010 slug→id DEFERRED-TO-v0.15.0 by an
owner scope call), and the out-of-band **P113** (lost-update shared-cursor guard) ALL shipped
GREEN. **P112** now landed: a scope stub at
`.planning/milestones/v0.14.0-phases/112-od-4-launch-readiness-scope-stub/PLAN.md` naming the four
OD-4 launch-readiness pillars (asciinema hero demo, CI-verified honest headline numbers,
install-path excellence, Show-HN positioning kit), one line each, marked **DO-NOT-START** and
deferred to a post-tag `/gsd-new-milestone` session — zero implementation, no verifier dispatch
(lightweight owner ack suffices per ROADMAP P112).
The aggregate `v0.14.0` tag was subsequently CUT — **v0.14.0 SHIPPED + Latest 2026-07-14**
(crates.io 0.14.0, GitHub release "Latest"); the b773c04 RED-main incident is CLOSED (fix
@ `8e2aae5`). No tag pending.

> **Superseded (2026-07-14) — historical record.** The B1–B5 tag-remediation cursor that
> formerly sat here (B1 mirror-refresh + B2 p93 CREATE-recovery "awaiting owner decision";
> B3/B4/B5 status; 2 owed orphan `p93 smoke A` TokenWorld page sweeps) is moot for the tag
> now that v0.14.0 shipped. Any residual product gaps + the owed teardown are tracked in the
> v0.15.0 intake (`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` /
> `GOOD-TO-HAVES.md`); prior evidence + diagnosis pointers live in
> `.planning/SESSION-HANDOVER.md` + git history.

## Current Focus

**Active milestones (SERIAL per OD-3 — A then C then B, per OD-4 resequencing):**

- **Workstream A — v0.13.0 extended.** **CLOSED GREEN 2026-07-05 (P78–P97, 20/20 phases).** Shipped P78–P88 2026-05-01; extended 2026-05-08 with P89–P97 (real-backend frictions); milestone-close verdict at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`. Tag v0.13.0 landed; v0.13.1 onboarding hotfix (P98–P101) additionally SHIPPED 2026-07-07 (tag `04640d5`). ROADMAP at `.planning/milestones/v0.13.0-phases/ROADMAP.md`.
- **Workstream C — v0.14.0 wave-2 hardening.** **SHIPPED + Latest 2026-07-14** — **11/11 phases GREEN** (P102–P112 + out-of-band P113; see § Workstream C above). v0.14.0 tagged/released (crates.io 0.14.0, GitHub release "Latest"); b773c04 RED-main incident CLOSED (@ `8e2aae5`). P112 OD-4 launch-readiness scope stub LANDED (DO-NOT-START; wholesale re-anchor deferred to a post-tag `/gsd-new-milestone` session). ROADMAP at `.planning/milestones/v0.14.0-phases/ROADMAP.md`.
- **Workstream B — v0.13.2 cross-link-fidelity.** QUEUED behind workstream C (this OD-4 resequencing) AND the not-yet-scoped launch-readiness milestone. Original placeholder range P98–P107 shifts again when eventually replanned (renumber-on-insertion convention). ROADMAP at `.planning/milestones/v0.13.2-phases/ROADMAP.md`.

**Last shipped milestone:** v0.12.1 (closed 2026-04-30). Verdict GREEN at `quality/reports/verdicts/milestone-v0.12.1/VERDICT.md` (commit 9ef348e).

**Cargo serialization rule (CLAUDE.md memory budget):** only ONE cargo invocation at a time. The separate-worktrees caveat is moot under OD-3 serial execution (single working branch: main); doc-only / planning-only subagents can still run truly concurrent with one cargo subagent.

> Superseded phase-decomposition + pre-kickoff scaffolding narrative →
> `.planning/STATE-history.md`.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260501-mgn | Polish 5 cold-reader nits in DVCS docs | 2026-05-01 | 2b9e9c9 | [260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b](./quick/260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b/) |
| 260706-rbf | RBF-LR-03 honest WAIVED known-limitation across ADR-010 §3 + troubleshooting + dvcs-topology | 2026-07-06 | dfc3a9b | [260706-rbf-rbf-lr-03-known-limitation](./quick/260706-rbf-rbf-lr-03-known-limitation/) |
| 260706-crf | DVCS cold-reader fixes — 7 findings across dvcs-topology + dvcs-mirror-setup + troubleshooting (findings 1 & 6 verified against code) | 2026-07-06 | (this commit) | [260706-crf-dvcs-cold-reader-fixes](./quick/260706-crf-dvcs-cold-reader-fixes/) |
| 260706-idp | v0.13.0 intake OP-8 disposition + bound-to-live-state sweep — carry-forward banners; 2 terminal SURPRISES + 4 completed RESOLVING-P97 rows deleted; 5 HIGHs confirmed live; 1 new MEDIUM filed (troubleshooting.md >20k) | 2026-07-06 | (this commit) | [260706-idp-v0.13.0-intake-disposition](./quick/260706-idp-v0.13.0-intake-disposition/) |
| 260712-bgv | Non-blocking timing-budget warning in pre-commit/pre-push hooks (SECONDS-based, stderr-only, never touches exit code) + CLAUDE.md/quality/CLAUDE.md cadence+scaling documentation | 2026-07-12 | b4e96d8 | [260712-bgv-add-non-blocking-timing-budget-warning-t](./quick/260712-bgv-add-non-blocking-timing-budget-warning-t/) |
| 260712-oa9 | 75% file-size early-warning tier in structure/file-size-limits.sh — non-blocking print-only WARN summary for the 75–99% band (top-12 by pct DESC + overflow), always emitted independent of --warn-only, never touches exit code; ≥100% block/waiver semantics unchanged. + catalog asserts + quality/CLAUDE.md § File-size limits + committed selftest | 2026-07-12 | (this commit) | [260712-oa9-file-size-75pct-warn](./quick/260712-oa9-file-size-75pct-warn/) |
| 260712-oke | Landed all 7 v0.14.0 DEFERRED surprises-intake entries + 1 hygiene row onto the v0.15.0 surface — new v0.15.0-phases/GOOD-TO-HAVES.md (GTH-V15-01..08, severity + fix-sketch each; shell-coverage cross-refs 999.5/999.6 not duplicated; release-plz code.json blocker noted cleared) + ROADMAP.md § Hardening candidates with 2 HIGH `Phase (candidate)` stubs (RBF-LR-03 modern-git verify, subprocess-bypass binary-side refusal) closing the roadmap-gap. UX Phase TBD stubs untouched; part-file back-pointers skipped (already >20k ceiling). | 2026-07-12 | (this commit) | [260712-oke-land-v0-14-deferred-onto-v0-15](./quick/260712-oke-land-v0-14-deferred-onto-v0-15/) |
| 260712-phc | Author the two missing pre-release-real-backend verifier scripts (B4 t4-conflict-rebase-ancestry-real-backend P0 + B5 github-front-door-real-backend P1) that blocked the v0.14.0 tag — ported sim-arm topology to confluence::TokenWorld / init github::reubenjohn/reposix, env-gate-first→exit75 NOT-VERIFIED, hermetic self-test (4/4), kcov harness (coverage 15.72%≥floor13), B5 catalog RFC3339 fix-twice. Both rows now grade instead of 'verifier not found'. | 2026-07-13 | fe8febb | [260712-phc-author-two-missing-pre-release-real-back](./quick/260712-phc-author-two-missing-pre-release-real-back/) |
| 260713-arc | Durably archive owner's 2026-07-12 reality-check audit (verbatim `cp`, 43492 bytes, byte-identical) to `.planning/milestones/audits/2026-07-12-reality-check.md` per "uncommitted = didn't happen" | 2026-07-13 | (this commit) | [260713-arc-archive-reality-check](./quick/260713-arc-archive-reality-check/) |
| 260713-q0e | Fix RED main (HONEST-REWORK, Manager Ruling #5 Option A) — post-release `quality-post-release` (run 29298424648, v0.14.0) went RED on 4 P1 docs-repro example gates (01/02/04/05); root cause = harness gap, NOT product: containers exit 0 but the generic `container-rehearse.sh` emitted one generic `asserts_passed` line, which F-K4b (`_audit_field.py::asserts_congruent`) rejects. The first fix (`0f2b7c5`, emit `expected.asserts` verbatim on exit 0) failed adversarial verification as a SYMPTOM-FIX — example-05's asserts overclaimed. Reworked: `git reset --soft d68fa8a` un-stacked the held commits; KEPT the emission (verified fail-loud for 01/02/04) and REWORDED example-05 asserts #2/#3 to the truth (pre-emptive `git sparse-checkout` pattern + `BLOB_LIMIT_EXCEEDED_FMT` source-constant presence — NOT a runtime-error observation; #3 scoped to the `ls issues/*.md` ≥1-file check). NO F-K4b weakening, NO waivers. Filed ONE v0.15.0 SURPRISES-INTAKE (MEDIUM): F-K4b container-tautology redesign + example-05 real-runtime-error deeper fix. post-release re-run: 6 PASS / 0 FAIL / exit 0. No push (orchestrator-gated). | 2026-07-13 | 03e7a6f (fix), 3775075 (intake) | [260713-q0e-fix-red-main-container-rehearse-sh-emits](./quick/260713-q0e-fix-red-main-container-rehearse-sh-emits/) |
| 260713-rug | Green RED-main `docs-repro/example-04-conflict-resolve` (FAILED at exactly 300.00s in `quality-post-release` run 29301412750, sha 05aa23c) via TIMEOUT-BUDGET fix (ruling b773c04). Diagnosis (opus repro): not a hang — the example runs ~0.5s and passes all 3 asserts; the 300s cap was eaten by per-container-row SETUP `apt-get install ... build-essential pkg-config libssl-dev ...`, compile-time deps NEVER exercised (examples run the host-mounted pre-built `target/debug/reposix`; no in-container cargo build). Two clean edits: (a) `container-rehearse.sh` SETUP drops `build-essential pkg-config libssl-dev`, keeps `curl ca-certificates python3 git sqlite3` + fix-it-twice comment; (b) `docs-reproducible.json` bumps `timeout_s` 300→600 symmetrically on all 4 `kind:container` rows (01/02/04/05), non-container rows untouched (tutorial-replay stays 300), JSON revalidated. Prove-before-fix: all 4 container rows rc=0 locally (01:16s, 02:15s, 04:16s, 05:19s), asserts_failed []. NO assert/waiver/example-proof touched (honesty guard CLEAN). No push (orchestrator-gated). | 2026-07-13 | (this commit) | [260713-rug-example04-timeout-budget](./quick/260713-rug-example04-timeout-budget/) |

## Session Continuity

Frontmatter (above) is the machine-readable cursor. Resume via `/gsd-resume-work`; current live cursor is "**Workstream C — v0.14.0 SHIPPED + Latest 2026-07-14 (crates.io 0.14.0, GitHub 'Latest'); 11/11 phases GREEN (P102–P112 + out-of-band P113); b773c04 RED-main CLOSED @ 8e2aae5. No tag pending. Post-tag queue items 0-5 in progress; wholesale re-anchor deferred to a post-tag /gsd-new-milestone session (P112 OD-4 launch-readiness stub)**" (see § Workstream C). Workstream A (v0.13.0-extension) is CLOSED GREEN historically (P78–P97, tag landed); the owner pre-tag checklist below is retained for record. Workstream B (v0.13.2) stays queued behind workstream C per OD-3/OD-4.

Top-level session handover: `.planning/SESSION-HANDOVER.md` (whole-session rotation handover for session 7e2a4cf2, 2026-07-04/05; distinct from per-phase relief handovers under `.planning/phases/`).

> Closed/historical cursor detail (Workstreams A & B, per-milestone cross-refs,
> Project Reference, resolved Blockers/Concerns) → `.planning/STATE-history.md`.
