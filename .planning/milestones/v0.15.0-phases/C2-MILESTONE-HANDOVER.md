# C2-MILESTONE-HANDOVER.md — v0.15.0 "Floor" C2 continuation, P123/P124 boundary, 2026-07-18

Written by the outgoing C2 milestone coordinator-of-coordinators at a **coordinated clean
relief**: both the current C2 and L0 rotate here simultaneously (seat #62→#63). P123 is
CLOSED GREEN on green main; P124 has NOT been dispatched yet. This is a phase boundary,
not a mid-phase pause — the successor C2 opens by running two EARLY quiet-point lanes
(§6 steps 1–2) and then dispatching a fresh C1 for P124.

L0 is relieving at this SAME boundary and writes its own `.planning/SESSION-HANDOVER.md`
in parallel; at the time this file was committed that L0 handover was NOT yet committed —
do NOT assume it exists or reference its SHA, just know L0 rotated alongside you.

**Successor's required first reads, in order:** `.planning/ORCHESTRATION.md` (full — you
are a C2, §3 governs your own relief and your C1-rotation-absorption duty), `.planning/
PROJECT.md` (Current Milestone: v0.15.0 Floor), `CLAUDE.md` (root — non-negotiables), then
this file in full before dispatching anything.

---

## 1. Ground truth (git)

- **HEAD == origin/main == `47283d75`.** `git status`: clean, nothing to commit, up to
  date with origin/main. **Nothing is currently unpushed** — P123's phase-close push
  carried everything, including the prior `eb4f02c0` relief handover. (Re-verify yourself
  — this is a snapshot at handover-write time.)
- **This handover commit will be the ONLY unpushed commit after you commit it.** Per the
  established pattern it rides to origin with the successor C2's first phase-close push —
  do NOT push it standalone.
- **Last 5 commits** (P123 close, newest first):
  - `47283d75` — docs(planning): #61→#62 relief — P122 CLOSED, v0.15.0 at 9/15 (60%)…
    *(NOTE: this is the STATE-advance / relief commit that closed P123 bookkeeping; the
    STATE.md now reflects P123 CLOSED GREEN, completed_phases=10. The message header dates
    from the relief that opened this rotation — read STATE.md for the authoritative
    cursor, not the commit subject.)*
  - `2f6d62ff` — gsd-verifier P123 verdict GREEN
    (**verdict file: `quality/reports/verdicts/p123/VERDICT.md`**)
  - `857e3c3a` — eager-fix: SC4/GTH-V15-03 graded-outcome (flags PASS + null-script,
    cadence-agnostic)
  - `95bc7c5f` — docs(v0.15.0): correct C2 handover liveness doctrine (L0 ruling)
  - `eb4f02c0` — docs(planning): #61→#62 relief handover (prior rotation)
  - *(Re-run `git log --oneline -12` yourself; the P123 execution/plan commits sit below
    these. The load-bearing four for close are `857e3c3a` eager-fix, `2f6d62ff` verdict,
    and the STATE advance `47283d75`.)*
- **CI, verified live via `gh run list --branch main`:** newest runs on `47283d75` are
  `success` — `ci.yml` (run `29648990069`) = success and `release-plz` (run
  `29648990053`) = success. Main is GREEN. The P0 `code/ci-green-on-main` probe passes on
  main's NEWEST `ci.yml` run.
- **One clean close, one confirmed anti-pattern.** The P123 close itself was a clean
  deterministic close (verifier→executor leaves, §5), but a `fork`-to-resume attempt
  mid-close CONFABULATED a no-op — read §5 before you drive any close.

## 2. Wave/cycle state

| Phase | Plans | State | Verdict / commits |
|---|---|---|---|
| P114 t4 oid-drift fix-first + reconcile audit | 2/2 | DONE | `114-VERIFICATION.md`, real-backend PASS 2026-07-15T17:56Z |
| P115 Live MCP benchmark re-measurement | 1/1 | DONE | top-level execution mode |
| P116 ADR-010 mirror-fanout packet + slug→id design | 3/3 | DONE | top-level execution mode |
| P117 Doc-truth launch-blocker purge | 6/7 | DONE (E1 sub-task owner-PENDING, not blocking) | see §5 |
| P118 Post-bench honesty corrections | 1/1 | DONE | `p118/VERDICT.md` |
| P119 Docs/planning simplification (P112 RAISE) | 4/4 | DONE (DP-4 pivot) | `p119/VERDICT.md` |
| P120 CLI + helper error hardening | 1/1 | DONE | `p120/VERDICT.md` |
| P121 RPX error-code namespace + `reposix explain` | 1/1 | DONE | `p121/VERDICT.md` (`80a37cea`) |
| P122 `reposix-remote` + `init` hardening | 4/4 | DONE — CLOSED GREEN | `p122/VERDICT.md` (`00ab1579`); close `a9e1f4c4` |
| **P123 Quality-runner & catalog integrity hardening** | **DONE — CLOSED GREEN** | **5/5 SC PASS** | `p123/VERDICT.md` (verdict `2f6d62ff`); close `47283d75` |
| **P124 Container-rehearse harness hardening** | **0/TBD** | **NOT STARTED — next** | — |
| P125 Real-backend cadence & mirror-drift resilience | 0/TBD | NOT STARTED | — |
| P126 Docs-alignment tooling polish | 0/TBD | NOT STARTED | — |
| P127 Surprises absorption (OP-8 Slot 1) | 0/TBD | NOT STARTED | — |
| P128 Good-to-haves polish + milestone close (OP-9 Slot 2) | 0/TBD | NOT STARTED | — |

**Cursor: 10/15 phases complete (P114–P123), 67%.** Next = P124.

**P123 close highlights (2026-07-18):** 5/5 SC PASS, independent gsd-verifier verdict
GREEN at `quality/reports/verdicts/p123/VERDICT.md`, STATE advance at `47283d75`.
- **SC4 / GTH-V15-03 graded-outcome SOUND** — the eager-fix `857e3c3a` makes the new
  `structure/verifier-script-exists` gate flag BOTH `flags: PASS` and a null
  `verifier.script` **cadence-agnostically**; an orthogonal all-catalog scan found **0**
  remaining instances. P123's NEW SC4 gate caught **37 real pre-existing catalog
  violations** on first run — a live proof it earns its keep.
- **SC1** — `run.py` self-sources `./.env` (present-only, non-clobbering, via
  `quality/runners/_env_load.py`), closing the false-green-preflight gap. Secret hygiene
  held to **KEY-NAMES-only** (no values logged).
- **SC2** downgrade-guard (`--persist` refuses to downgrade a committed-GREEN row without
  `--allow-downgrade`) + **SC3** flock single-lane were exercised end-to-end.
- **SC5** — `code/ci-green-on-main` now watches a required-workflow **LIST**, not
  hardcoded `ci.yml` only.

**Named-liveness post-mortem + confirmed anti-pattern to read before dispatching any C1
that will push with CI in-flight OR before driving any phase close:** §5 below. Read it;
those two operational lessons (L0-owns-CI-watch; `fork`-to-resume is an anti-pattern) are
the most important of this rotation.

### Phase-list ground truth: P124–P128 scope is ALREADY PINNED, not a blank slate

**The AUTHORITATIVE live roadmap is the top-level `.planning/ROADMAP.md`** (§ "v0.15.0
Floor"). It already has concrete Goal + Depends-on + Requirements + Success-Criteria for
EVERY remaining phase P124–P128 — what's NOT yet done is the `/gsd-plan-phase` wave/plan
breakdown. So the successor C2's job per phase is "run `/gsd-plan-phase 12X`" (which pins
the concrete plan/wave shape against the already-scoped Goal+SC), not "invent scope from
scratch."

**The ROADMAP § Progress table is now correct** — it shows P121 / P122 / P123 all
"Complete." Do NOT carry any "progress-table fix owed" lane; that lane is DONE.

The milestone-scoped `.planning/milestones/v0.15.0-phases/ROADMAP.md` is STILL a stale
"PLANNING / Phase TBD" stub (superseded by the live top-level file) — do NOT plan phases
from it. Tracked as **GTH-V15-27** (LOW, OPEN), good <1h eager-fix candidate for whoever
next touches `v0.15.0-phases/`. Ground truth of the still-pinned remaining scope (verified
against the live file at the P122/P123 boundary; unchanged this rotation):

- **P124 — Container-rehearse harness hardening.** Reqs DRAIN-13/14/22/23/24. Earned
  per-step `ASSERT-PASS:` congruence (not verbatim-emitted); SIGKILL-proof sim teardown;
  `target/debug/reposix` provenance on the CI runner; exit code derived strictly from
  persisted `exit_code`; `.sim-*.log` gitignored.
- **P125 — Real-backend cadence & mirror-drift resilience.** Reqs DRAIN-02/12. Mandatory
  mirror-refresh pre-step (or self-reconciling litmus) for `pre-release-real-backend`;
  milestone-close vision-litmus self-heals for BOTH backend AND GitHub-mirror drift.
- **P126 — Docs-alignment tooling polish.** Reqs DRAIN-15..21. `doc-clarity-review`
  hard-fails on a canary probe; README expands "MCP" on first use; grader binds only when
  a cited-number drift would fail the test; `plan-refresh` warns when invoked cold;
  `status` surfaces `waived_active`; the 16 stale "cites out-of-eligible-file" warnings
  resolved.
- **P127 — Surprises absorption, OP-8 Slot 1.** Every SURPRISES-INTAKE row added DURING
  P114–P126's own execution gets a terminal STATUS. Explicitly NOT the already-routed
  DRAIN-01..25 work (that's P123–P126's job) — only NEW intake surfaced while those run.
- **P128 — Good-to-haves polish + milestone close, OP-9 Slot 2.** GOOD-TO-HAVES drained
  to terminal STATUS; `.planning/RETROSPECTIVE.md` v0.15.0 section distilled BEFORE
  archive (verifier grades RED if missing); `CHANGELOG.md` `[v0.15.0]` finalized;
  milestone-close verdict dispatched GREEN including the non-skippable 9th probe;
  `tag-v0.15.0.sh` authored (owner gate — owner runs it, you do not).

## 3. Binding constraints (unchanged — embed in EVERY C1 charter verbatim)

- **ONE cargo invocation machine-wide.** Cargo is **FOREGROUND-only** — never
  `run_in_background`/detached (orphans the build, evades `cargo-mutex.sh`, OOM risk —
  this VM has OOM-crashed three times from parallel cargo builds). Prefer `-p <crate>`,
  `jobs=2`.
- **Leaf test setup (`reposix init` / sim-seed / `git config` / `git commit` for test
  fixtures) runs in a `/tmp` clone, `cd`-ing into it in the SAME Bash invocation — NEVER
  the shared repo.** Mechanically enforced (PreToolUse `leaf-isolation-guard.sh`, exit 2)
  + pre-commit backstop, but the Bash-tool-only coverage boundary means a subprocess/
  script write can still bypass it (see the P116-close incident, §5 SURPRISES-INTAKE
  line ~545, still OPEN as a hardening gap — HIGH).
- **Uncommitted = didn't happen.** Commit before ending any turn. Mid-phase commits stay
  local until the phase-close push.
- **No `--no-verify`, ever.**
- **One tree-writer at a time.** Tree-mutating work is serial; read-only inspection may
  fan out in parallel.
- **Push cadence:** `git push origin main` BEFORE the verifier subagent dispatch, THEN
  `python3 quality/runners/run.py --cadence post-push --persist` — the P0
  `code/ci-green-on-main` probe must show main's NEWEST `ci.yml` run = success. Never
  open the next phase over a red main.
- **Tainted-by-default / `REPOSIX_ALLOWED_ORIGINS` egress allowlist.** Sim is the default
  backend for every demo/unit-test/autonomous loop.
- **Model tiering:** every C1 gets an EXPLICIT `model` override — opus for
  security/genuinely-complex (P124's SIGKILL-proofing / harness-hardening is a strong
  opus candidate), sonnet default, haiku mechanical. Never `fable` at a leaf.

## 4. Litmus / gate / REOPEN state

- **P123 verdict:** GREEN, `quality/reports/verdicts/p123/VERDICT.md`, verdict commit
  `2f6d62ff`. 5/5 SC PASS. **No open REOPEN state.** P123 is CLOSED GREEN with no
  outstanding gate failures.
- **Non-gating known-FAIL (NOT a blocker):** P2 `code/shell-coverage` counter-drift —
  `transcript.sh` self-reports 34 covered lines vs kcov's 27 (25.9% drift > 15%
  threshold), but the aggregate coverage floor PASSED and the cadence exits 0. Filed as
  **L1166** (LOW-MED), routed for P124 investigation. Do NOT treat it as a close blocker.
- **Open-waiver expiry clocks:**
  - `structure/file-size-limits` OVER-BUDGET-tier `--warn-only` waiver on
    `GOOD-TO-HAVES.md` / `SURPRISES-INTAKE.md` / etc — **expires 2026-08-08T00:00:00Z**
    (`quality/catalogs/freshness-invariants.json`). **Both files GREW this rotation:
    `GOOD-TO-HAVES.md` is now 143,905 B and `SURPRISES-INTAKE.md` 119,234 B** — the split
    is increasingly urgent. See §5 OWED lane + §6 step 2; this is the single most
    time-boxed item you own.
  - Hero-number doc-alignment waivers (8 rows, BENCH-01-fed) — **expire 2026-08-15**
    (already re-measured by P115; lift them using P115's figures — check
    `115-UNWAIVE-PATH.md`; **not verified fresh** by this handover, flag as a re-ground
    item for the successor).

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

### CRITICAL OPERATIONAL LESSON #1 — L0 owns the CI watch (liveness)

A subagent's OWN background `gh run watch` does NOT reliably re-invoke that subagent:
when the watch concludes the completion bubbles up to L0 as "no live background children"
and the subagent is NOT self-re-woken. **Only the TOP-LEVEL (L0) gets reliable
background-task re-invocation.** Reliable pattern: **L0 owns CI-watching and pokes the
coordinator (via SendMessage) on green.** When a C1 pushes and CI is in-flight, the
coordinator STOPS-and-returns-to-C2 at the push→CI-in-flight boundary, REPORTS the run id
up to L0 (which holds the durable watch), and does NOT end its turn assuming a self-owned
watcher — nor a grandchild's report bubbling up — will re-wake it. Direct child-AGENT
completion notifications DO re-invoke the parent (that path works); bare background-bash
watchers do not.

**This doctrine is ALREADY FULLY LANDED** at `b2eca628` — it lives in
`.planning/ORCHESTRATION.md` §3 and §11, the `coordinator-dispatch` skill §6a, AND root
`CLAUDE.md`. **The successor VERIFIES it is present, no rewrite owed.** (See "Doctrine
updates owed" below — this is item (a), already-landed.)

### CRITICAL OPERATIONAL LESSON #2 — `fork`-to-resume is a CONFIRMED ANTI-PATTERN

Mid-P123-close, this C2 tried to resume a warm C1 by dispatching with
`subagent_type: fork`. **The fork inherited the C2's coordinator context, behaved as a
router, and returned a CONFABULATED no-op** — 0 tool uses, claimed "close executing"
while nothing actually happened (STATE unchanged, no verdict dir written). It was caught
only by verifying against reality (checking STATE.md + the verdict directory), not by
trusting the fork's self-report.

**Lesson (three parts):**
1. To resume a warm agent, use **SendMessage to its agent id** — never `fork`.
2. To drive a phase close, dispatch **verifier → executor LEAVES directly** (the
   P122-blessed deterministic-control pattern) — do NOT route the close through a
   resumed/forked coordinator.
3. NEVER `fork`-to-resume — a fork is a fresh sibling that inherits your context and
   confabulates continuity it never had.

This anti-pattern is **genuinely absent from all doctrine** and IS the real owed early
`/gsd-quick` — see "Doctrine updates owed" item (b).

### The P123 close itself (the pattern that WORKED)

Driven by **DETERMINISTIC CONTROL** (the P122-blessed pattern): C2 dispatched an
independent **gsd-verifier** directly (→ verdict `2f6d62ff`), then a **gsd-executor**
directly (→ STATE advance `47283d75`). No passive relay, no self-watch dependency, no
fork. This is the pattern the successor should reuse for every close.

### OWED / TRACKED lanes (carry forward)

- **GOOD-TO-HAVES / SURPRISES split-archive lane — run it EARLY, at THIS quiet point.**
  `GOOD-TO-HAVES.md` = **143,905 B** (~7× the 20k `.md` ceiling), `SURPRISES-INTAKE.md` =
  **119,234 B**; both waived under the corpus-growth umbrella **until 2026-08-08**, both
  still growing. They MUST be archived/split (resolved-entry archive + index) BEFORE that
  waiver lapses or pushes start blocking repo-wide. Every C1 writes to these files at its
  own phase close (fix-twice / noticing-disposition), so the split MUST run at a
  **no-C1-intake-writing quiet point** — a between-phase window YOU control. **This
  P123→P124 boundary IS that quiet point** (no C1 currently writing intake). **Do NOT
  gamble it lands naturally by P128** — the milestone may not close before 2026-08-08, and
  P128 (OP-9 Slot 2, milestone close) is the wrong place to discover the waiver has lapsed
  and is blocking the close push itself. Run it as §6 step 2.
- **P117 anomaly (live, not a P124 blocker).** P117's W5 coordinator close is incomplete:
  the launch-animation E1 mp4 asset publish (`gh release upload` to the `docs-assets`
  release) + the post-upload `animation-renders` playwright verify are **owner-PENDING**
  (manager-deferred 2026-07-17 under standing "outward publishing = owner-only" doctrine —
  `.planning/CONSULT-DECISIONS.md` 2026-07-17: "OWNER DECISION STILL PENDING"). Tracked as
  **GTH-V15-37**. E1-class — see §"ESCALATE-to-L0" below. Do not self-authorize; do not
  let it rot — carry forward in every handover until an owner ruling lands.
- **Milestone-scoped ROADMAP staleness (GTH-V15-27, LOW, OPEN)** — populate with a pointer
  to the live top-level ROADMAP.md or delete the stub; <1h eager-fix for whichever C1 next
  touches `.planning/milestones/v0.15.0-phases/`.
- **Pre-ADF list-path storage-fallback residual (GOOD-TO-HAVE).** P114 closed the
  Confluence oid-drift defect for **ADF-native pages only**; a pre-ADF (storage-only) page
  would still trip `OidDrift` (P114 verifier OQ1 residual; did NOT manifest on the current
  all-ADF TokenWorld substrate). Documented residual, not an open fix-first blocker.

### Intake filed at P123 close (track, don't re-file)

- **L1129** (MEDIUM) — pre-push hook now runs ~109–121s vs the documented ~55s (roughly
  2×). Routed for **P124 investigation**.
- **L1166** (LOW-MED) — `code/shell-coverage` counter-drift (§4). Folded note: the
  script-exit-0-vs-catalog-FAIL two-layer honesty mechanism wants a one-line
  `quality/CLAUDE.md` clarification. **P124.**
- **L1198** (LOW/INFO) — SC1 `.env` self-sourcing puts real creds into every `run.py`
  process; routed to **milestone-close security sign-off (P128)**.
- Earlier P123 dispositions (already RESOLVED, do NOT re-open): the 2 HIGH SURPRISES rows
  (`.env` self-sourcing false-green + `--persist` silent downgrade) → RESOLVED into
  SC1/SC2; the stale Confluence oid-drift SURPRISES row → RESOLVED (cross-ref
  `114-VERIFICATION.md`); a gh-auth `env -u` 2-gate audit → filed as a **P127 candidate**
  (SURPRISES ~L1059).

### Doctrine updates owed (GSD-gated) — CORRECTED SCOPE

This is a **SMALL, single-item** early `/gsd-quick`, not two items:
- **(a) L0-owns-CI-watch liveness** — **ALREADY FULLY LANDED** at `b2eca628`
  (ORCHESTRATION §3 + §11, coordinator-dispatch skill §6a, root CLAUDE.md). Successor
  **VERIFIES only**, no rewrite owed.
- **(b) `fork`-to-resume anti-pattern** (Lesson #2 above) — **genuinely absent
  everywhere**. This IS the real owed doctrine add: add it to `.planning/ORCHESTRATION.md`
  §11 + the `coordinator-dispatch` skill. Must go through a GSD command (`/gsd-quick`),
  never an out-of-band edit (fix-twice meta-rule).

## 6. Precise next steps (successor runbook)

1. **Re-verify ground truth yourself** before dispatching anything: `git log --oneline
   -10`, `git status`, `gh run list --branch main --limit 5` (confirm still green — do not
   trust this handover's snapshot past your own check). Read `.planning/STATE.md` for the
   authoritative cursor (P123 CLOSED, completed_phases=10, next=P124).
2. **EARLY, at THIS P123→P124 quiet point (no C1 writing intake), run two lanes BEFORE
   opening P124** so they don't collide with a C1's tree writes:
   - **The GOOD-TO-HAVES / SURPRISES split-archive lane** (143,905 B / 119,234 B, waiver
     lapses **2026-08-08** — do NOT gamble it lands by P128): resolved-entry archive +
     index. This is time-boxed; treat it as the higher-priority of the two.
   - **The small early doctrine `/gsd-quick`** — single item (b) the `fork`-to-resume
     anti-pattern into ORCHESTRATION §11 + coordinator-dispatch skill; **verify** item (a)
     (L0-owns-CI-watch) is already present (it is, at `b2eca628`) and do NOT rewrite it.
   Sequence these before/alongside opening P124.
3. **Dispatch P124's C1** (`phase-coordinator`, **explicit `model: opus`** for
   harness-hardening / SIGKILL-proofing complexity). Full GSD arc. **CHARGE it to
   investigate the health-decay items `L1129` (pre-push ~2× slowdown) + `L1166`
   (shell-coverage counter-drift + the honesty-mechanism one-line clarification) as owned
   first-grounding actions.** Embed verbatim: §3 binding constraints, the OD-3 ownership
   charter (below), and the liveness protocol — STOP-and-return-to-C2 at the push→CI-in-
   flight boundary; **L0 owns the durable watch**; do not self-watch to re-wake. Full arc:
   `/gsd-plan-phase 124` (gsd-planner + gsd-plan-checker) → execute (gsd-executor waves,
   catalog-first for any new gate) → gsd-code-reviewer → phase-close push (fetch-rebase →
   `git push origin main` → `run.py --cadence post-push --persist`, confirm
   `code/ci-green-on-main` P0 shows main's NEWEST ci.yml run = success) → **gsd-verifier
   directly → verdict at `quality/reports/verdicts/p124/VERDICT.md`** → on GREEN,
   STATE-advance (completed_phases 10→11, bump percent, cursor next=P125) → commit + push +
   re-confirm CI green. Drive the close with **verifier→executor LEAVES directly** (§5) —
   NEVER a `fork`-to-resume.
4. **Continue P125 → P126 → P127 (OP-8) → P128 (OP-9 + milestone close)** in order, one C1
   per phase, each gated CLOSED GREEN on green main before the next dispatch. Absorb C1
   rotations below yourself (re-dispatch a fresh successor C1 pointed at its handover — do
   NOT bubble C1 rotations to L0). Report to L0 ONLY at stop points: your own relief, an
   owner-decision escalation, milestone-close-ready, a 2–3-phase checkpoint, and each
   push→CI-in-flight handoff (L0 holds the watch).
5. **Relieve yourself (the C2) past ~100k tokens of your OWN context** (hard stop ~150k,
   absolute not %) at a PHASE boundary — dispatch `relief-handover-writer` (update this
   file in place), report the SHA to L0, stop.
6. **At P128 / milestone-close:** do NOT self-authorize any tag/release. **Report
   milestone-close-ready to L0 and WAIT.** Confirm BEFORE archive: the OP-9
   `.planning/RETROSPECTIVE.md` v0.15.0 section is distilled (verifier grades RED if
   missing), and the non-skippable 9th probe
   `python3 quality/runners/run.py --cadence pre-release-real-backend` exits 0 with
   catalog row `agent-ux/milestone-close-vision-litmus-real-backend` PASS (P0, never
   waived).

---

## ESCALATE-to-L0 list (report and WAIT — never self-authorize)

- **Global `gsd-sdk state.advance-plan` STATE.md-corruption bug** — held escalated
  upstream with L0; mitigation = **hand-advance STATE.md, NEVER that tool**. Do not use
  `state.advance-plan` at any close.
- **E1 launch-animation mp4/playwright publish** (GTH-V15-37, owner-PENDING per
  `.planning/CONSULT-DECISIONS.md` 2026-07-17) — never self-authorize, never tag
  `[OWNER]` without genuine owner input.
- **Any outward release** — a git tag matching `v*` or a crates.io publish triggers the
  release pipeline. Do NOT self-cut a release/tag at milestone close; report
  milestone-close-ready to L0 and WAIT for owner routing.
- **Milestone ARCHIVE** — before `/gsd-complete-milestone` archives v0.15.0, the OP-9
  distillation into `.planning/RETROSPECTIVE.md` MUST land, AND the non-skippable 9th
  `pre-release-real-backend` probe must exit 0. Report milestone-close-ready to L0 BEFORE
  final archive.
- **`.env` credential sign-off** (L1198) — the real-creds-in-every-run.py-process posture
  gets an explicit milestone-close security sign-off at P128; do not wave it through.
- **Any user-visible breaking change**; any real-backend MUTATION beyond the sanctioned
  targets (Confluence TokenWorld / GitHub `reubenjohn/reposix` issues / JIRA `TEST`); a
  roadmap item that no longer seems right given new info.

## NON-NEGOTIABLES (embed in every C1 charter, verbatim)

ONE cargo invocation machine-wide; cargo FOREGROUND-only (never `run_in_background` —
orphans the build + evades `cargo-mutex.sh` + OOM risk); prefer `-p <crate>`, jobs=2 (VM
OOM history). Leaf test setup (`reposix init` / sim-seed / `git config` / `git commit`)
runs in a `/tmp` clone with `cd /tmp/...` in the SAME Bash invocation — NEVER the shared
repo (PreToolUse `leaf-isolation-guard.sh` enforces exit 2; pre-commit backstops).
Uncommitted = didn't happen; commit before ending any turn; mid-phase commits stay local
until phase-close push. Tainted-by-default / `REPOSIX_ALLOWED_ORIGINS` egress allowlist;
sim is the default backend. Catalog-first for any new gate (GREEN-contract row committed
BEFORE the implementation the verifier reads). No `--no-verify`.

## RECURRING PROCESS LESSONS (reinforce in every C1 charter)

- Never hand-pick gates before a push — run `python3 quality/runners/run.py --cadence
  pre-push` (FULL cadence). P117 and P121 both shipped avoidable 2-push cascades from
  hand-picked gate sweeps.
- Doc-alignment rebind-in-same-commit: any edit to a line carrying a `doc-alignment`
  binding (`quality/catalogs/doc-alignment.json`) MUST rebind cited rows in the SAME
  commit, or the next push BLOCKs with `STALE_DOCS_DRIFT`.
- Docs edits pass BOTH `docs-build/*` AND `structure/banned-words` locally before a push
  (fix-twice, P117).
- Fix-twice meta-rule: any new file/convention/gate revises the relevant CLAUDE.md (root
  or scoped) / ORCHESTRATION.md in the SAME PR, tagged with the quality dimension.

## OD-3 OWNERSHIP CHARTER (embed in every C1 + subagent charter, verbatim)

1. Acceptance criteria are the floor, not the ceiling: "done" = "I'd defend this in
   review as excellent," not "plan executed."
2. Noticing is a deliverable — every code-touching report names what it noticed nearby
   (lying doc claims, tests that don't assert their namesake, teaching-free error
   messages, dead code, stale comments, missing edge cases). An empty noticing section
   from code-touching work is a red flag.
3. Eager-fix or file — never silently skip: <1h + no new dependency → fix in place;
   else → `.planning/milestones/v0.15.0-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` with
   severity + sketch.
4. Verify against reality — run the binary, hit the path, render the page; a claim
   without a committed artifact isn't done.
5. North star — Rust-compiler-grade UX: every user-facing error teaches the fix,
   suggests the alternative, gives a copy-paste recovery command (exemplar
   `crates/reposix-cli/src/init.rs::refuse_existing_repo_root`).

## C2 OPERATING DOCTRINE (your own, verbatim)

- Dispatch ONE C1 `phase-coordinator` per phase with an EXPLICIT `model` override (opus
  for security/genuinely-complex phases, sonnet default, haiku mechanical; NEVER fable
  at a leaf). Charge each C1 with the full GSD arc: plan (gsd-planner + gsd-plan-checker)
  → execute (gsd-executor waves) → code-review (gsd-code-reviewer) → phase-close push
  (fetch-rebase → `git push origin main` → `run.py --cadence post-push --persist`,
  `code/ci-green-on-main` P0 must show main's NEWEST ci.yml run = success) → verify+close
  (gsd-verifier → `quality/reports/verdicts/p12X/VERDICT.md`) → on GREEN advance STATE
  (completed_phases+1, bump percent, cursor next=P12Y) → commit+push+re-confirm CI. Only
  advance to the next phase after the current is CLOSED GREEN on green main.
- Drive every close with **verifier→executor LEAVES dispatched directly**; NEVER
  `fork`-to-resume a warm coordinator (confabulates a no-op — §5 Lesson #2). Resume a warm
  agent via SendMessage-to-its-id.
- Absorb C1 rotations below the top: when a C1 relieves (writes a handover under
  `.planning/phases/12X-*/`), the C2 re-dispatches a FRESH successor C1 pointed at that
  handover — do NOT bubble C1 rotations to L0.
- Relieve YOURSELF past ~100k tokens of OWN context (hard stop ~150k; absolute, not %)
  at a PHASE boundary (never mid-phase): dispatch relief-handover-writer → report SHA to
  L0 → stop.
- Report to L0 only: (a) own relief, (b) owner-decision escalation, (c)
  milestone-close-ready, (d) a 2–3-phase checkpoint, (e) each push→CI-in-flight handoff
  (L0 holds the durable watch). Otherwise route and integrate.
