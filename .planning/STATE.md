---
gsd_state_version: 1.0
mode: serial-workstreams
status: v0.13.0-shipped-v0.13.1-onboarding-hotfix-queued
last_updated: "2026-07-07"
last_activity: "2026-07-07 — v0.13.0 verified SOUND post-release: both release-blocking scares confirmed FALSE ALARMS (cargo-binstall-resolves was a stale-literal brittle gate, fixed in PR #70; the crlf CI failure is a wiremock test-harness artifact under CI CPU starvation, not byte corruption, root cause S-260707-rbf-01 OPEN as monitor). Zero-shot human-simulation testing (3 independent fresh-agent reproductions) found reposix-sim onboarding 100% broken on the shipped binary (sim ships in no prebuilt distribution; reposix init masks the failure behind exit 0) -> D1 decided: v0.13.1 onboarding hotfix sequenced BEFORE the v0.14.0 pivot. D2: worktree-isolation + reject-t@t-identity hook are P0 for v0.14.0 hardening (a dispatched sim/seed leaf corrupted the local shared repo TWICE this session via shared .git/config; both repaired, origin never affected). D3: zero-shot human-simulation testing becomes a standing milestone-close gate. PRs #70 and #71 merged. HEAD == origin/main == 5fd4731. NEXT: enter v0.13.1 via GSD (discuss/plan/execute the D1 acceptance items), then resume v0.14.0 pivot scoping."
workstreams:
  workstream_a:
    milestone: v0.13.0
    milestone_name: DVCS over REST (extended)
    status: closed-green  # v0.13.0-extension CLOSED GREEN with owner-gated caveats 2026-07-05 (milestone verdict quality/reports/verdicts/milestone-v0.13.0/VERDICT.md, verdict commit 390ce31, graded HEAD 3c6d72f)
    phases_total: 20  # P78-P97 (P78-P88 shipped + P89-P97 extension)
    phases_completed: 20  # P78-P97 (workstream A / v0.13.0-extension COMPLETE)
    next_phase: P98  # v0.13.0 milestone CLOSED GREEN; awaiting owner pre-tag actions + L0 tag push; then workstream B (v0.13.2, P98+)
    blocks_tag: true  # P97 GREEN so the tag is phase-ready, BUT the v0.13.0 tag push is L0/owner's (NOT the coordinator's) and gated on the OWNER PRE-TAG ACTIONS in § Workstream A
  workstream_b:
    milestone: v0.13.2
    milestone_name: Cross-link fidelity
    status: queued  # RESEQUENCED per OD-4 item 3 (2026-07-04): queued BEHIND the new launch-readiness milestone, which itself follows P97/v0.13.0 tag
    phases_total: 10  # P98-P107
    phases_completed: 0
    next_phase: P98
    blocks_tag: false  # v0.13.2 tag ships at P107, sequenced after v0.13.0 tag AND the launch-readiness milestone per OD-4; tag push delegated to orchestrator
---

# Project State

## Current Position

**Mode:** serial-workstreams per OD-3 (workstream A → then B; the parallel-worktree model is RETIRED).

**OD-3 mandate (2026-07-03)** — see `.planning/phases/89-framework-fixes-cadence-shell-kind/89-OWNER-DECISIONS.md` § "DECISION OD-3": drive to v1.0. Complete v0.13.0-ext (P89–P97, tag v0.13.0), then v0.13.2 (P98–P107, tag v0.13.2) STRICTLY SERIALLY; after both tags, formalize the research-only ladder (v0.14.0 observability/multi-repo → plugin ecosystem/launch readiness → v1.0.0 + ADR-009 semver activation) as real GSD milestones via `/gsd-new-milestone`. `main` is the working branch (workstream/v0.13.0-ext fast-forwarded into main 2026-07-03 and retired; per-phase push cadence targets origin/main). Full autonomy incl. former hard gates: OD-1's owner sign-off delegated to orchestrator (owner notified, not blocking), tag pushes at P97/P107 delegated contingent on GREEN verdicts, ~$50 pre-authorized for P106 L3 dogfood. OD-2 + litmus REOPEN gates remain in force UNCHANGED — on RED the orchestrator loops back, never waives.

### Workstream A — v0.13.0 (extended) — CLOSED GREEN

Phase: **v0.13.0-extension (P78–P97) CLOSED GREEN with owner-gated caveats, 2026-07-05.** Milestone verdict `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md` (verdict commit `390ce31`, graded HEAD `3c6d72f`, on origin/main). Disk tally: **117 PASS / 0 FAIL / 15 WAIVED / 13 ratified-honest NOT-VERIFIED**. P97 (the OP-8 Slot 2 + milestone-close phase) is **closed via this milestone verdict** — there is no separate `p97/VERDICT.md`; the milestone-close verdict IS P97's close. OP-9 RETROSPECTIVE distilled (ratifier exit 0); the non-skippable **9th probe** graded honestly **NOT-VERIFIED** (real-backend env-gate unset — never skip-as-pass); the release-plz git-state blocker (5 tracked+ignored P93 evidence JSONs) was fixed at `3c6d72f`.
Status: **20/20 phases complete (P78–P97)** — workstream A / v0.13.0-extension COMPLETE. The v0.13.0 tag is phase-ready; the **tag push itself is L0/owner's, NOT the coordinator's** (`.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh.disabled` stays disabled — do NOT run it) and is gated on the owner pre-tag actions below.

**PRE-TAG CHECKLIST — doc/planning items CLEARED by L0 2026-07-06 (commits cd93dbc→56307be, verified HEAD==origin/main, tree clean, no tag cut):**
- ✅ **9th probe VERIFIED** (owner decision, SESSION-HANDOVER §4): committed `last_real_grade=PASS` + fresh 2026-07-06 green Confluence round-trip. Mechanical `status: NOT-VERIFIED` is honest-by-design env-gate; no new real call needed to tag.
- ✅ **RBF-LR-03 documented** as honest WAIVED-for-v0.13.0 known-limitation (ADR-010 §3 additive marker + troubleshooting.md recovery subsection + dvcs-topology.md out-of-scope cross-ref → v0.14.0 pivot). `dfc3a9b`/`b03266d`.
- ✅ **DVCS cold-reader review** via `/doc-clarity-review` — 7 findings incl. a real `refs/mirrors/*` doc↔code BLOCKER, ALL fixed, 0 filed. `b8de57c`. NOTE: this satisfied the cold-reader **doc** review (SESSION-HANDOVER §3 item 2 reframing); the separate subjective-rubric row `dvcs-cold-reader` in `subjective-rubrics.json` may still show TTL-expired — non-blocking, carry-forward.
- ✅ **OP-8 SURPRISES/GOOD-TO-HAVES disposition** + bound-to-live-state sweep: 6 terminal entries deleted (git-archived), open carried forward, 1 MEDIUM filed. `56307be`. No un-addressed HIGH/BLOCKER left dangling.

**RELEASE DECISION DELEGATED TO L0 (owner, 2026-07-06):** the owner explicitly handed the
**release decision** to the orchestrator — PR #61 merge, the **crates.io publish (irreversible,
E3 spend)**, and cutting the **v0.13.0 tag (E1)**. This extends the OD-3 delegation ("tag pushes
delegated contingent on GREEN verdicts") to the publish spend. L0 owns it, gated on: regenerated
PR #61 verified clean (only expected version bumps + changelog) AND green CI. Owner chose "steward
regen + review PR #61 now."

**RELEASE RUNBOOK (L0-owned tail) — LIVE STATUS 2026-07-07 — VERDICT: GO:**
- Superseding regen: **PR #68** (branch `release-plz-2026-07-07T02-37-20Z`, head
  `14bb5e43d7ff9552245dae6f3b47caeaece4ea1f`) — the release-plz branch was regenerated again after
  the PR #61 NO-GO above; PR #68 is the current live release PR.
- **All required checks GREEN** — `gh pr checks 68` shows the full 22-check matrix PASS, incl.
  `quality gates (pre-pr)` (CI run `28838198234`, job `85526336500`: 70 PASS / 1 unrelated
  pre-existing FAIL (`docs-build/p94-badges-real-vs-transient`, already tracked in
  `GOOD-TO-HAVES.md`, non-blocking) / 1 WAIVED cadence, exit=0), `test`, `clippy`, `rustfmt`,
  `shell-coverage`, `cargo-audit`, `gitleaks`, `coverage`, `bench-latency-v09`, `CodeQL`, and all
  real-backend integration jobs (confluence/github/jira, incl. `-v09` arms).
- **Diff verified release-churn-only**: `gh pr diff 68 --name-only` → only `Cargo.lock`,
  `Cargo.toml`, and per-crate `Cargo.toml`/`CHANGELOG.md` files. No stray source/logic.
- **`crlf_blob_body_round_trips_byte_for_byte` flake (S-260707-rbf-01) did NOT recur this run** —
  the required `quality gates (pre-pr)` check passed clean. This is release-unblocking evidence for
  THIS run only — root cause remains unproven (hypotheses A/B both still open, local repro 0/7+
  across two sessions). Tracked as a non-blocking **monitor** item in `SURPRISES-INTAKE.md`
  (STATUS kept OPEN/HIGH, reframed "monitor — not release-blocking on a green run, revisit if it
  recurs") — NOT closed, NOT root-caused.
1. **PR #68** — verdict **GO**: all required checks green, diff churn-only, crlf flake absent this
   run (non-blocking per above) → **merge + crates.io publish** (L0 owns; IRREVERSIBLE — verify
   publish succeeded per-crate before proceeding).
2. **Cut the v0.13.0 tag** — `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh.disabled` stays
   disabled; canonical release is `.github/workflows/release.yml` (tag `v*`). Push tag → watch the
   release workflow to green (`gh run watch`).
3. **6 env-gated real-backend** transport/attach/conflict rows — accept via creds or leave honestly
   NOT-VERIFIED (env-gate, not a gap; not release-blocking per the settled 9th-probe verification).
4. **Renew `structure/file-size-limits` waiver before 2026-08-08** (active-corpus WAIVED) — future.
5. **Monitor S-260707-rbf-01** on future CI runs (this PR's regens or the next release) — if the
   crlf flake recurs, pull the job log immediately (full diagnostics now in place via `fbe5bee`)
   before any further truncation regression can hide it again.

**Queued post-tag `/gsd-quick` meta-infra items (run AFTER the tag lands, each via GSD):**
- The doctrine **4-edit** `/gsd-quick` — NOTE its "create DP-5" step is **stale** (DP-5 already exists = tangent-classification); adjust the edit, do NOT apply verbatim.
- The `dispatch-doctrine.sh` **session-guard** addition.
- The **verifications-layout** evidence/transient subtree refactor.
- The `cli.md` **exit-code v0.14 doc fix**.

Next agent action: **none in workstream A** — it is COMPLETE. Clear the OWNER PRE-TAG ACTIONS above (owner/L0), push the v0.13.0 tag (L0), then proceed to the launch-readiness milestone scoping (`/gsd-new-milestone`) and workstream B (v0.13.2, P98+) per OD-3/OD-4.

### Workstream B — v0.13.2 — QUEUED (RESEQUENCED per OD-4)

Phase: P98 (entry-point) — crate skeleton + shared-compute lift + edge model + walker + catalog + tracker schemas. Sourced from `.planning/research/v0.13.2-cross-link-fidelity/`.
Plan: TBD — P98 plan-overview not yet authored.
Status: RESEQUENCED per OD-4 item 3 (2026-07-04, `89-OWNER-DECISIONS.md` § "DECISION OD-4"): a new **launch-readiness milestone** (asciinema hero demo, CI-verified headline numbers, install-path excellence, positioning/Show-HN kit) is scoped and executed AFTER the P97/v0.13.0 tag and BEFORE P98. P98's "Depends on: v0.13.0 milestone GREEN" still holds; it additionally now depends on launch-readiness GREEN. 0/10 phases complete; ROADMAP scaffolded; REQUIREMENTS scaffolded; intakes scaffolded with 2 Q6 deferrals seeded in GOOD-TO-HAVES.
Next agent action: **P97 is now GREEN (v0.13.0 milestone CLOSED).** Still gated behind the OWNER PRE-TAG ACTIONS + L0 v0.13.0 tag push (see § Workstream A), then the launch-readiness milestone scoped via `/gsd-new-milestone`; then `/gsd-discuss-phase 98`.

Last activity: 2026-07-05 — v0.13.0 milestone CLOSED GREEN with owner-gated caveats (milestone verdict `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`, verdict commit `390ce31`, graded HEAD `3c6d72f`); STATE advanced to 20/20 phases, workstream A COMPLETE, owner pre-tag actions + post-tag `/gsd-quick` queue recorded durably in § Workstream A.

## Current Focus

**Active milestones (SERIAL per OD-3 — A then B):**

- **Workstream A — v0.13.0 extended.** **CLOSED GREEN 2026-07-05 (P78–P97, 20/20 phases).** Shipped P78–P88 2026-05-01; extended 2026-05-08 with P89–P97 (real-backend frictions); milestone-close verdict at `quality/reports/verdicts/milestone-v0.13.0/VERDICT.md`. The v0.13.0 tag is phase-ready; its push is **L0/owner's**, gated on the OWNER PRE-TAG ACTIONS in § Workstream A. ROADMAP at `.planning/milestones/v0.13.0-phases/ROADMAP.md`.
- **Workstream B — v0.13.2 cross-link-fidelity.** QUEUED behind workstream A per OD-3 (serial). Scoped 2026-05-08; P98–P107. ROADMAP at `.planning/milestones/v0.13.2-phases/ROADMAP.md`. **P97 is now GREEN, but P98 still does not start** until the OWNER PRE-TAG ACTIONS clear, the L0 v0.13.0 tag lands, and the launch-readiness milestone is scoped (OD-4).

**Last shipped milestone:** v0.12.1 (closed 2026-04-30). Verdict GREEN at `quality/reports/verdicts/milestone-v0.12.1/VERDICT.md` (commit 9ef348e).

**Cargo serialization rule (CLAUDE.md memory budget):** only ONE cargo invocation at a time. The separate-worktrees caveat is moot under OD-3 serial execution (single working branch: main); doc-only / planning-only subagents can still run truly concurrent with one cargo subagent.

**Phase decomposition + pre-kickoff scaffolding (superseded — see live cursor above):** the milestone-start scaffolding (pre-kickoff checklist, the original P78–P88 decomposition, the 36/36 REQ-ID coverage map, the CARRY-FORWARD bundle, and the long-dead "execute P89 waves 1–4" cursor) is historical. The live cursor is the frontmatter + § "Workstream A — v0.13.0 (extended)" above (**P96 CLOSED GREEN 2026-07-05, next P97**). Archived detail: `.planning/milestones/v0.13.0-phases/ROADMAP.md` (full P78–P97 decomposition) + `.../CARRY-FORWARD.md`. **Still live:** P97 is now GREEN (milestone CLOSED), so the tag is phase-ready — but the tag-script `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh.disabled` stays disabled (**do NOT run it**); the **v0.13.0 tag push is L0/owner's, NOT the coordinator's**, and is gated on the OWNER PRE-TAG ACTIONS in § Workstream A.

## Per-milestone history (cross-references)

Historical phase-by-phase contribution narrative lives in per-milestone ARCHIVE files:

- `.planning/milestones/v0.12.1-phases/ARCHIVE.md` — most recently shipped (P72–P77 + owner-TTY close-out).
- `.planning/milestones/v0.12.0-phases/archive/` — Quality Gates framework + 8 dimensions (P56–P65).
- `.planning/milestones/v0.11.0-phases/`, `v0.10.0-phases/`, `v0.9.0-phases/`, `v0.8.0-phases/`, etc. — earlier milestones.

## Project Reference

- `.planning/PROJECT.md` — scope and decisions table (Current Milestone now v0.13.0).
- `.planning/ROADMAP.md` — milestone-level roadmap (P78–P88 drafted 2026-04-30 by gsd-roadmapper).
- `.planning/REQUIREMENTS.md` — milestone requirements (36 v0.13.0 REQ-IDs; traceability table mapped to phases).
- `.planning/research/v0.13.0-dvcs/` — full research bundle (vision, architecture, kickoff, decisions, CARRY-FORWARD).
- `CLAUDE.md` — operating principles + per-phase push cadence + Quality Gates protocol.
- `quality/PROTOCOL.md` — Quality Gates runtime contract.
- `quality/SURPRISES.md` — append-only pivot journal.

## Blockers / Concerns

- POC-FINDINGS.md re-engagement checkpoint pending (orchestrator decision; F01 + F04 are REVISE-tagged but small).
- Tag scripts `tag-v0.9.0.sh` / `tag-v0.10.0.sh` RELOCATED to `.planning/milestones/v0.9.0-phases/` + `.planning/milestones/v0.10.0-phases/` (no longer at `scripts/`); the v0.9.0 + v0.10.0 tags themselves remain unpushed — absent both locally and on origin (verified 2026-07-03). Owner gate, pre-existing.
- 3 WAIVED structure rows expired 2026-05-15 — RESOLVED in P78.
- ROADMAP.md top-level v0.12.0 entries — RESOLVED (top-level `.planning/ROADMAP.md` is now ~156 lines; v0.12.0 entries relocated per CLAUDE.md §0.5).

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260501-mgn | Polish 5 cold-reader nits in DVCS docs | 2026-05-01 | 2b9e9c9 | [260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b](./quick/260501-mgn-polish-5-cold-reader-nits-in-dvcs-docs-b/) |
| 260706-rbf | RBF-LR-03 honest WAIVED known-limitation across ADR-010 §3 + troubleshooting + dvcs-topology | 2026-07-06 | dfc3a9b | [260706-rbf-rbf-lr-03-known-limitation](./quick/260706-rbf-rbf-lr-03-known-limitation/) |
| 260706-crf | DVCS cold-reader fixes — 7 findings across dvcs-topology + dvcs-mirror-setup + troubleshooting (findings 1 & 6 verified against code) | 2026-07-06 | (this commit) | [260706-crf-dvcs-cold-reader-fixes](./quick/260706-crf-dvcs-cold-reader-fixes/) |
| 260706-idp | v0.13.0 intake OP-8 disposition + bound-to-live-state sweep — carry-forward banners; 2 terminal SURPRISES + 4 completed RESOLVING-P97 rows deleted; 5 HIGHs confirmed live; 1 new MEDIUM filed (troubleshooting.md >20k) | 2026-07-06 | (this commit) | [260706-idp-v0.13.0-intake-disposition](./quick/260706-idp-v0.13.0-intake-disposition/) |

## Session Continuity

Frontmatter (above) is the machine-readable cursor. Resume via `/gsd-resume-work`; current cursor is "**v0.13.0 milestone CLOSED GREEN (P78–P97, 20/20) — awaiting OWNER PRE-TAG ACTIONS + L0 v0.13.0 tag push, then workstream B (v0.13.2, P98+)**" (milestone verdict commit `390ce31`, graded HEAD `3c6d72f`). The durable owner pre-tag checklist + queued post-tag `/gsd-quick` meta-infra items live in § Workstream A. Workstream B stays queued behind A per OD-3/OD-4.

Top-level session handover: `.planning/SESSION-HANDOVER.md` (whole-session rotation handover for session 7e2a4cf2, 2026-07-04/05; distinct from per-phase relief handovers under `.planning/phases/`).
