# C2-MILESTONE-HANDOVER.md — v0.15.0 "Floor" C2 continuation, P124/P125 boundary, 2026-07-18

Written by the outgoing C2 milestone coordinator-of-coordinators (seat #63-C2) at a
**clean P124-close / pre-P125 wave boundary — nothing in flight.** P124 is now CLOSED
GREEN **with an independent verifier `VERDICT.md` committed** (OP-7 remediation, this
rotation); the owner docs-transparency quick landed and pushed GREEN; P125 has **not**
been dispatched yet and is the successor's very next action. Naming note: this file is
**updated in place** following the established convention — third revision at this same
P124/P125-adjacent boundary (`47c1f9d3` P122/P123, `d4ea76cb` P123/P124,
`5690e50e` P124/P125-boundary-first-write, this commit = P124/P125-boundary-refresh) — it
is the milestone-scoped C2 continuity doc, analogous to `.planning/SESSION-HANDOVER.md`,
not a phase-scoped `<N>-HANDOVER.md`.

**Unconfirmed vs. prior rotations, STILL true this rotation:** `.planning/
SESSION-HANDOVER.md`'s last touch is still `57bf9376` ("#62→#63 relief") — no newer
commit against that path, confirmed by direct `git log` this rotation. There is still
**no evidence L0 has relieved since seat #63 opened**; this C2 has now written three
revisions of this file inside that same L0 seat without an L0-side rotation. Do not
assume a fresh L0 handover exists; re-check `git log -- .planning/SESSION-HANDOVER.md`
yourself before relying on any L0 seat claim.

**Successor's required first reads, in order:** `.planning/ORCHESTRATION.md` (full — you
are a C2, §3 governs your own relief and your C1-rotation-absorption duty), `.planning/
PROJECT.md` (Current Milestone: v0.15.0 Floor), `CLAUDE.md` (root — non-negotiables), then
this file in full before dispatching anything.

---

## 1. Ground truth (git)

- **`origin/main == local HEAD == c267f0e8`.** Zero divergence — `git rev-list
  --left-right --count origin/main...HEAD` = `0 0`. `git status --porcelain` clean.
  Verified live via `gh run list --branch main`: the newest runs on `c267f0e8` are ALL
  `success` — `CI` (run `29667972559`), `CodeQL` (run `29667972286`), `release-plz` (run
  `29667972532`), `Docs` (run `29668131154`). Main is GREEN. The P0 `code/ci-green-on-main`
  probe passes on main's newest `ci.yml` run.
- **Commit chain since the prior handover write (`5690e50e`), all landed + pushed this
  rotation:**
  - `60c39a57` — docs(roadmap): add "Progress right now" strip + document two refresh
    cadences (owner docs-transparency quick, step (a)+(b))
  - `78bb9e43` — docs(planning): encode per-phase-close roadmap-strip refresh into
    phase-close doctrine (`.planning/CLAUDE.md`, fix-twice step (c), dimension `structure`)
  - `d3d8052f` — docs(roadmap): reconcile P123 completion date + P124 progress row
    (quick steps (d)+(e))
  - `c267f0e8` — docs(124-verdict): independent phase-close grade — GREEN (**OP-7
    remediation** — supplies the `VERDICT.md` that was missing when P124's STATE-advance
    landed; see §2/§4 below for the finding)
- **P124 CLOSED GREEN — now WITH a committed independent verdict.** `quality/reports/
  verdicts/p124/VERDICT.md` (15,123 bytes, tracked — confirmed via `git ls-files`, NOT
  gitignored; the `.gitignore:110` pattern only excludes the dated bare-session roll-up
  `[0-9]*.md` siblings, not `VERDICT.md` by name) is committed at `c267f0e8`. Overall:
  **GREEN**, 4/4 SC PASS, all 8 aggregate P2 FAILs confirmed pre-existing-baseline (3 are
  stale persisted-FAILs that PASS on re-run), no P0/P1 FAIL. This closes the OP-7 gap the
  prior handover flagged — see §2 for the finding detail.
- **STATE.md frontmatter** (`last_updated: 2026-07-18T20:30:00.000Z`, unchanged by this
  rotation's work — the quick + VERDICT.md did not advance the phase counter):
  `completed_phases: 11`, `total_phases: 15`, `percent: 73`. **Cursor: 11/15 phases
  complete (P114–P124), 73%. Next = P125.**
- **Stale-but-carry-only:** STATE.md frontmatter `total_plans: 3 / completed_plans: 2` is
  still stale (unchanged, re-confirmed this rotation) — reads under 11 closed phases;
  appears to only track the currently-active phase's plan sequence, not a cumulative
  count. Not a blocker; fold into the next close-bookkeeping touch (see §5).

## 2. Wave/cycle state

| Phase | Plans | State | Verdict / commits |
|---|---|---|---|
| P114–P121 | — | DONE | unchanged since the P123/P124 handover — see prior revision in git history (`d4ea76cb`) for detail |
| P122 `reposix-remote` + `init` hardening | 4/4 | DONE — CLOSED GREEN | `p122/VERDICT.md` (`00ab1579`); close `a9e1f4c4` |
| P123 Quality-runner & catalog integrity hardening | 5/5 SC PASS | DONE — CLOSED GREEN | `p123/VERDICT.md` (`2f6d62ff`); close `47283d75` |
| **P124 Container-rehearse harness hardening** | **4/4 SC PASS** | **DONE — CLOSED GREEN, independent VERDICT.md now committed** | `p124/VERDICT.md` (`c267f0e8`, OP-7 remediation); SUMMARY `3b1a61ce`; catalog mint `8d9a269a`; STATE advance `b01afabc` — STATE-advance CONFIRMED CORRECT by the independent re-grade, not just re-stamped |
| **P125 Real-backend cadence & mirror-drift resilience** | **0/TBD** | **NOT STARTED — successor's next action** | — |
| P126 Docs-alignment tooling polish | 0/TBD | NOT STARTED | — |
| P127 Surprises absorption (OP-8 Slot 1) | 0/TBD | NOT STARTED | — |
| P128 Good-to-haves polish + milestone close (OP-9 Slot 2) | 0/TBD | NOT STARTED | — |

**Cursor: 11/15 phases complete (P114–P124), 73%.** Next = P125 — the owner
docs-transparency quick and the phase-close push are BOTH DONE (unlike the prior
handover write, there is no interstitial quick blocking P125 anymore). The successor's
FIRST action is `/gsd-plan-phase 125` (§6).

**OP-7 finding this rotation (P124 independent verdict):** the prior handover flagged
that P124's GREEN close + STATE 10→11 advance had **no independent verifier
`VERDICT.md`** (unlike p120–p123, each of which has one) — a process gap named OP-7. This
rotation dispatched an independent re-grade against reality (zero session context beyond
the charter): re-ran all 4 SC gates + selftests, re-ran all 8 aggregate-FAIL gates, ran
the pre-push cadence, hit the live GitHub API for CI status. **Finding: the original
GREEN close was substantively correct** — every SC is genuinely delivered (SC1 congruence
harvesting + example-05 real `[RPX-0503]` refusal in an `ubuntu:24.04` container; SC2
SIGKILL-proof teardown proven via a live mid-run SIGKILL; SC3 explicit `cargo build -p
reposix-cli` provenance step; SC4 exit strictly from persisted `exit_code`), and all 8
aggregate P2 FAILs are pre-existing baseline (3 are stale persisted-FAILs that PASS on
re-run; none is P124-introduced). **The OP-7 defect was the missing artifact, not a wrong
grade.** The verdict also surfaced one now-resolved coordinator-gated caveat: at
verdict-authoring time HEAD `d3d8052f`'s `ci.yml` had CANCELLED via a transient "quality
gates (pre-pr)" 15m-cap timeout (all other jobs green; the identical gate set had passed
on P124's own tip `790aa73c`) — the coordinator held the push until a fresh run went
green, which it did on the verdict's own push (`c267f0e8`, `ci.yml` run `29667972559`,
success, "quality gates (pre-pr)" job = 247s wall-clock, well inside the 900s/15m cap —
see §5 raise-list item 1 for why this reads as transient-variance, not a budget
regression).

### Phase-list ground truth: P125–P128 scope is ALREADY PINNED, not a blank slate

Unchanged from the prior handover — **the AUTHORITATIVE live roadmap is the top-level
`.planning/ROADMAP.md`** (§ "v0.15.0 Floor"). Re-verified fresh this rotation (lines
unchanged from the prior handover's citation):

- **P125 — Real-backend cadence & mirror-drift resilience** (`.planning/ROADMAP.md:254`).
  Reqs **DRAIN-02, DRAIN-12**. Goal: the `pre-release-real-backend` cadence and the
  milestone-close vision-litmus survive GitHub-mirror drift instead of false-negatives.
  3 SCs: (1) a documented mandatory mirror-refresh pre-step
  (`scripts/refresh-tokenworld-mirror.sh`) or a self-reconciling litmus prevents a
  second-run false-negative caused by the litmus's own prior push re-staling the GitHub
  mirror; (2) the milestone-close vision-litmus fixture self-heals for BOTH backend drift
  (trashed protected pages) AND GitHub mirror drift, reconciling the mirror to
  backend-current through the reposix bus remote before the marker push; (3) the helper's
  `git pull --rebase` teaching string is corrected for the mirror-drift case specifically.
  Plans: TBD (first act: `/gsd-plan-phase 125`).
- P126–P128 scope unchanged from the prior handover revision (`d4ea76cb`) — re-read
  `.planning/ROADMAP.md` lines ~264–onward yourself before dispatching either.

## 3. Binding constraints (unchanged — embed in EVERY C1 charter verbatim)

- **ONE cargo invocation machine-wide.** Cargo is **FOREGROUND-only** — never
  `run_in_background`/detached (orphans the build, evades `cargo-mutex.sh`, OOM risk —
  this VM has OOM-crashed three times from parallel cargo builds). Prefer `-p <crate>`,
  `jobs=2`.
- **Leaf test setup (`reposix init` / sim-seed / `git config` / `git commit` for test
  fixtures) runs in a `/tmp` clone, `cd`-ing into it in the SAME Bash invocation — NEVER
  the shared repo.** Mechanically enforced (PreToolUse `leaf-isolation-guard.sh`, exit 2)
  + pre-commit backstop + the binary-side `reposix init` refusal (RPX-0406).
- **Uncommitted = didn't happen.** Commit before ending any turn. Mid-phase commits stay
  local until the phase-close push.
- **No `--no-verify`, ever.**
- **One tree-writer at a time.** Tree-mutating work is serial; read-only inspection may
  fan out in parallel.
- **Push cadence:** `git fetch origin && git rebase origin/main` (other sessions push
  concurrently — verified clean this rotation, 0 behind / 0 ahead at handover time), then
  `git push origin main` BEFORE the verifier subagent dispatch, THEN `python3
  quality/runners/run.py --cadence post-push --persist` — the P0 `code/ci-green-on-main`
  probe must show main's NEWEST `ci.yml` run = success. Never open the next phase over a
  red main. **STOP at the push→CI-in-flight boundary and RETURN to L0** — L0 holds the
  durable CI watch; do not self-watch (P122 incident, §5 doctrine below).
- **Tainted-by-default / `REPOSIX_ALLOWED_ORIGINS` egress allowlist.** Sim is the default
  backend for every demo/unit-test/autonomous loop.
- **Model tiering:** every C1 gets an EXPLICIT `model` override — opus for
  security/genuinely-complex, sonnet default, haiku mechanical. Never `fable` at a leaf.
  P125's mirror-drift/real-backend-cadence work is a plausible opus candidate (real
  backend + security-adjacent reconciliation logic) — the dispatching C2 decides at
  `/gsd-plan-phase 125` time.

## 4. Litmus / gate / REOPEN state

- **P124 verdict:** GREEN — **now backed by a committed independent `VERDICT.md`**
  (`quality/reports/verdicts/p124/VERDICT.md`, `c267f0e8`; see §2 for the finding). No
  open REOPEN state.
- **Non-gating known-FAIL / process-friction, carried (NOT a close blocker):**
  - `code/shell-coverage` P2 counter-drift (L1166, pre-existing, still open) — forces a
    `--persist` downgrade-REFUSAL on every phase-close mint. Still a P127 Slot-1
    candidate needing an owner decision among 3 options (fix-drift / accept-FAIL /
    decouple-via-WARN); see §5 raise-list item 2 — the independent P124 verdict
    additionally confirmed 3 of the 8 standing P2 FAILs are stale persisted rows that
    PASS on re-run, sharpening the case for a `--persist` hygiene refresh alongside the
    root-cause fix.
  - `verdict.py --phase N` bare-session collation reads a **false RED** (411 un-run
    NOT-VERIFIED rows + pre-existing standing reds from older milestones) — a naive
    verifier could mis-grade a clean phase RED off this artifact alone. This is exactly
    the trap the independent P124 verdict had to reason past explicitly (§2). Filed to
    SURPRISES-INTAKE part-07; fix-twice note owed to `quality/PROTOCOL.md`; P126
    candidate.
- **Open-waiver expiry clocks (unchanged this rotation, re-confirmed):**
  - `structure/file-size-limits` OVER-BUDGET-tier `--warn-only` waiver — **expires
    2026-08-08T00:00:00Z**. Covers **91 files** (re-counted 2026-07-18 via the gate — see
    catalog comment in `quality/catalogs/freshness-invariants.json:667`). This is **HELD,
    escalate-first — do NOT self-resolve** (filed `bc4decf3`; L0 is holding for an owner
    decision). **This handover file itself is one of the 91** — see the growth note in §5.
  - Hero-number doc-alignment waivers (8 rows, BENCH-01-fed) — **expire 2026-08-15**;
    unchanged, still **not re-verified fresh** by this handover — flag as a re-ground item
    for whoever plans P126+.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

### This rotation's accomplishments (for continuity)

- **PENDING #1 (owner docs-transparency quick) — DONE.** `/gsd-quick` landed the
  `docs/roadmap.md` "Progress right now" strip (Floor, 11/15, 73%, one capability-language
  line, no phase numbers, dated `2026-07-18`) — verified binding-free (no
  `doc-alignment.json` row cites its moving lines, sidestepping the P117 W3 rebind trap).
  Fix-twice landed into `.planning/CLAUDE.md` phase-close/push-cadence section (mandatory
  per-phase-close strip refresh, dimension tagged `structure`); the "How to read this"
  section + `<!-- SYNC: -->` comment now document TWO distinct cadences (strip refreshes
  every phase-close; mermaid arcs re-color only at milestone-close). P123's missing
  completion date and P124's stale `0/TBD` progress-table row were both reconciled in
  `.planning/ROADMAP.md`. Commits `60c39a57` / `78bb9e43` / `d3d8052f`.
- **PENDING #2 (push + confirm CI green) — DONE.** The quick's commits plus the
  already-local P124-close commits landed on `origin/main` this rotation; `ci.yml` /
  `CodeQL` / `release-plz` / `Docs` all `success` on the pushed tip. One transient
  cancellation (a "quality gates (pre-pr)" 15m-cap timeout on the intermediate tip
  `d3d8052f`) resolved on re-run — see §2's OP-7 paragraph and raise-list item 1 below.
- **OP-7 remediation — DONE.** Independent `quality/reports/verdicts/p124/VERDICT.md`
  authored and committed (`c267f0e8`), closing the process gap the prior handover
  flagged. Finding: GREEN, substantively correct close; see §2 for detail. This was NOT
  in the prior handover's PENDING list as a named item — the outgoing coordinator
  proactively closed the OP-7 gap it had itself flagged, judged in-scope given the
  boundary was otherwise clean and the fix was a bounded, well-charterable dispatch.

### Doctrine landed in prior rotations (verify-present, no rewrite owed — compressed)

Both doctrine items the P123/P124 handover once flagged as "owed" remain **CONFIRMED
LANDED** (L0-owns-CI-watch liveness at `b2eca628`; `fork`-to-resume anti-pattern doctrine
at `88168478`) — nothing further owed on either. Full prose detail is in the git history
of this file (`5690e50e` revision) if a future reader needs the citation; not re-derived
here to keep this section current-rotation-focused.

### RAISE-LIST for the successor (each with a destination — do not re-diagnose)

1. **Pre-pr CI runtime-creep — MEDIUM, filed `surprises-intake/part-07.md` (the L1129
   entry).** Investigated this rotation: did **NOT** reproduce on the VERDICT.md-push CI
   run (`29667972559` — the "quality gates (pre-pr)" job measured **247s wall-clock**
   against the workflow's 900s/15m cap, a healthy margin) — the docs-quick's earlier
   `d3d8052f` cancellation was transient slow-runner variance, **not** a budget
   regression. The ACTIONABLE signal remains the *local* pre-push hook budget drift
   (~109–113s measured vs. the documented ~55–60s, L1129) → route to **P126/P127**, NOT a
   CI `timeout-minutes` change.
2. **3 stale persisted-FAIL rows + `code/shell-coverage` P2 counter-drift → P127 Slot-1.**
   The independent P124 verdict confirmed 3 of the 8 standing aggregate P2 FAILs are
   stale persisted rows that PASS on re-run — bundle a `--persist` hygiene refresh with
   the root-cause fix for the runner over-counting P2 FAILs by 3 (this over-count is what
   colored the P124 bare-session roll-up RED and contributed to the original OP-7 miss —
   a cleaner signal there would have made the missing-VERDICT.md gap more visible sooner).
3. **`verdict.py --phase` bare-session false-RED trap → P126 fix-twice** (note owed to
   `quality/PROTOCOL.md`; unchanged from prior handover, re-confirmed live this rotation
   — see §2/§4).
4. **LOW CI-runner-variance note** (docs-quick first-attempt cancel/re-run, `d3d8052f`) —
   fold to intake when a doc-touching lane is convenient; not urgent on its own now that
   item 1 above has ruled out a budget regression.
5. **GTH-V15-89 (roadmap-strip machine gate, MEDIUM)** filed at `good-to-haves/part-10.md`
   this rotation (by the docs-quick lane) — a `structure`-dimension freshness invariant
   that would mechanically catch a stale "Progress right now" strip instead of relying on
   prose doctrine alone. Consider for P128 drain.
6. **STATE counters `total_plans: 3 / completed_plans: 2` read stale** vs. 11 closed
   phases (appears to only track the active phase's plan sequence, not a cumulative
   count) — cosmetic NOTICED, unchanged this rotation. Fold at next close-bookkeeping.

### Handover-doc growth NOTICED (self-referential, for the next relief writer)

This file is ~30.6KB pre-this-edit and is itself one of the 91 files under the
`structure/file-size-limits` waiver umbrella (§4). Each rotation adds new-rotation detail;
this edit compressed the now-fully-absorbed "doctrine landed" prior-rotation prose (see
above) rather than pure-appending, to hold growth down. **Recommend the next relief
writer do the same** — once a PENDING item is DONE and its full reasoning is captured in
git history via a real commit, a one-line "confirmed landed, see `<sha>`" pointer is
sufficient here; the full prose does not need to survive every rotation. If this file
keeps growing past ~35–40KB, consider whether it has crossed from "continuity doc" into
"needs its own split-ledger treatment" (the `f654cfc3` precedent) — not yet warranted,
but worth a look at the next relief if growth continues unabated.

## HELD / ESCALATE-FIRST (NEVER self-authorize — carry forward to L0/owner)

- **91-file file-size global-waiver umbrella** (`structure/file-size-limits` single row,
  expires **2026-08-08**, real count 91 not the historical 56; STATE.md/ORCHESTRATION.md
  among the un-splittable live planning ledgers, this handover doc among them too) — L0
  is HOLDING for owner. Do NOT self-resolve. Options sketched in the `bc4decf3` surprise
  entry (accept as permanent waived category vs. shard convention vs. selective drain).
  **GTH-V15-84** (`container-rehearse.sh`/`sigkill-safe.sh` now over 10k after P124's
  additions) + **GTH-V15-78** share this umbrella.
- **E1 launch-animation publish (GTH-V15-37)** — owner approval still PENDING per
  `.planning/CONSULT-DECISIONS.md` (2026-07-17 entries). `docs-build/animation-renders`
  staying NOT-VERIFIED is a pending gate, not an accepted deferral (no verifier exists yet
  at `quality/gates/docs-build/animation-renders.sh`). Never self-authorize a `gh release
  upload` for this asset.
- **Global `gsd-sdk state.advance-plan` STATE.md-corruption bug** — held upstream with L0;
  silently corrupts STATE.md on parse error, hits ALL sessions. Do NOT in-repo fix.
  Mitigation used THIS rotation (and every prior P12X close): hand-advance STATE.md via
  the read path (`gsd-sdk query state.load` + manual Edit), **NEVER** the write tool.
- **L1198** — `.env` credential-hydration security sign-off (`run.py` self-sources real
  creds into every process since P123's SC1), deferred to the P128 milestone-close
  security sign-off. Do not wave it through early.
- **Any release/tag; milestone archive** — gated on the OP-9 RETROSPECTIVE distillation +
  the non-skippable 9th `pre-release-real-backend` probe + report-to-L0-before-archive.
- **Any real-backend mutation beyond the sanctioned targets** (Confluence TokenWorld /
  GitHub `reubenjohn/reposix` issues / JIRA `TEST`) — escalate-first.
- **A roadmap item that no longer seems right given new info; any user-visible breaking
  change** — escalate-first per standing doctrine.

## 6. Precise next steps (successor runbook)

1. **Re-verify ground truth yourself** before touching anything: `git log --oneline -10`,
   `git status`, `gh run list --branch main --limit 5` (confirm still green — do not
   trust this handover's snapshot past your own check). Read `.planning/STATE.md` for the
   authoritative cursor (P124 CLOSED, completed_phases=11, next=P125). Confirm `quality/
   reports/verdicts/p124/VERDICT.md` is present and GREEN (OP-7 remediation, §2/§4).
2. **Open P125 — this is now the successor's FIRST action** (the owner docs quick and the
   phase-close push are BOTH already done this rotation — do not repeat them). P125's
   scope (real-backend cadence & mirror-drift resilience, Reqs DRAIN-02/DRAIN-12) is
   ALREADY PINNED in the top-level `.planning/ROADMAP.md` (§2 above). First act:
   `/gsd-plan-phase 125` (gsd-planner + gsd-plan-checker), then dispatch a C1
   `phase-coordinator` with an EXPLICIT model override (see §3 — opus is a plausible fit
   for real-backend + mirror-reconciliation security-adjacent work; the dispatching C2
   decides at plan time) per the same pattern used for P124: full GSD arc (plan → execute
   → code-review → phase-close push → `run.py --cadence post-push --persist` confirming
   `code/ci-green-on-main` → **gsd-verifier directly → committed verdict** (do not repeat
   the OP-7 gap — mint the `p125/VERDICT.md` as part of the close, not as a later
   remediation) → on GREEN, STATE-advance completed_phases 11→12, bump percent, cursor
   next=P126 → commit+push+re-confirm CI). Drive the close with **verifier→executor
   LEAVES dispatched directly** — NEVER a `fork`-to-resume (confirmed anti-pattern, fully
   landed in doctrine).
3. **Continue P126 → P127 (OP-8) → P128 (OP-9 + milestone close)** in order, one C1 per
   phase, each gated CLOSED GREEN on green main (WITH a committed independent VERDICT.md
   at close, per the OP-7 lesson) before the next dispatch. Absorb C1 rotations below
   yourself — re-dispatch a fresh successor C1 pointed at its own handover; do NOT bubble
   C1 rotations to L0. Report to L0 ONLY at stop points: your own relief, an
   owner-decision escalation, milestone-close-ready, a 2–3-phase checkpoint, and each
   push→CI-in-flight handoff.
4. **When dispatching each C1, hand it the RAISE-LIST items above that match its phase**
   (item 1+2 → P126/P127; item 3 → P126; item 5 → P128) so the destination phases open
   already primed rather than rediscovering these findings cold.
5. **Relieve yourself (the C2) past ~100k tokens of your OWN context** (hard stop ~150k,
   absolute not %) at a PHASE boundary — dispatch `relief-handover-writer` (update THIS
   file in place, per the established convention; compress now-superseded prior-rotation
   detail rather than pure-appending — see the growth note in §5), report the SHA to L0,
   stop.
6. **At P128 / milestone-close:** do NOT self-authorize any tag/release. Report
   milestone-close-ready to L0 and WAIT. Confirm BEFORE archive: the OP-9
   `.planning/RETROSPECTIVE.md` v0.15.0 section is distilled (verifier grades RED if
   missing), and the non-skippable 9th probe `python3 quality/runners/run.py --cadence
   pre-release-real-backend` exits 0 with catalog row
   `agent-ux/milestone-close-vision-litmus-real-backend` PASS (P0, never waived).

---

## ESCALATE-to-L0 list (report and WAIT — never self-authorize)

See §"HELD / ESCALATE-FIRST" above (§5) for the full list with fresh 2026-07-18 detail.
Unchanged headline items: the global `gsd-sdk state.advance-plan` corruption bug
(hand-advance STATE.md only); the E1 launch-animation mp4/playwright publish
(GTH-V15-37, still owner-PENDING); any outward release/tag; the milestone ARCHIVE gate
(OP-9 distillation + 9th probe); the `.env` credential sign-off (L1198, P128); any
user-visible breaking change or real-backend mutation beyond the sanctioned targets. The
91-file file-size waiver umbrella residual (`bc4decf3`) remains explicitly HELD for an
owner decision among 3 sketched options — do not self-resolve. **Nothing new escalated
this rotation** — the two PENDING items from the prior handover (owner docs quick; push +
CI green) both resolved cleanly without needing an owner decision, and the OP-7
remediation was a bounded self-contained fix, not an escalation.

## NON-NEGOTIABLES (embed in every C1 charter, verbatim)

ONE cargo invocation machine-wide; cargo FOREGROUND-only (never `run_in_background` —
orphans the build + evades `cargo-mutex.sh` + OOM risk); prefer `-p <crate>`, jobs=2 (VM
OOM history). Leaf test setup (`reposix init` / sim-seed / `git config` / `git commit`)
runs in a `/tmp` clone with `cd /tmp/...` in the SAME Bash invocation — NEVER the shared
repo (PreToolUse `leaf-isolation-guard.sh` enforces exit 2; pre-commit backstops; the
binary-side `reposix init` refusal, RPX-0406, closes the subprocess-bypass residual).
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
  commit, or the next push BLOCKs with `STALE_DOCS_DRIFT`. A LATER, SEPARATE reword of an
  already-correctly-rebound line re-drifts it (P117 W3, 2-push cascade) — this bites
  `docs/roadmap.md`'s moving "Progress right now" strip too; it was deliberately kept
  binding-free this rotation to sidestep this (verified — no bound row cites it).
- Docs edits pass BOTH `docs-build/*` AND `structure/banned-words` locally before a push
  (fix-twice, P117).
- Fix-twice meta-rule: any new file/convention/gate revises the relevant CLAUDE.md (root
  or scoped) / ORCHESTRATION.md in the SAME PR, tagged with the quality dimension.
- L0-owns-CI-watch + fork-to-resume-is-an-anti-pattern are BOTH now fully landed in
  doctrine (§5 above) — a C1 charter can cite ORCHESTRATION §3/§11 directly instead of
  re-deriving the lesson prose each time.
- **NEW this rotation — close with a committed VERDICT.md, not just a returned grade.**
  P124's original close returned its GREEN verdict directly (harness directive) without
  writing it to a committed file — a gap named OP-7, remediated this rotation via a
  standalone independent re-grade. The finding was substantively correct, but the
  artifact was missing, costing a full extra dispatch to fix after the fact. Every future
  phase close should mint `p12X/VERDICT.md` as part of the SAME close, not defer it.

## OD-3 OWNERSHIP CHARTER (embed in every C1 + subagent charter, verbatim)

1. Acceptance criteria are the floor, not the ceiling: "done" = "I'd defend this in
   review as excellent," not "plan executed."
2. Noticing is a deliverable — every code-touching report names what it noticed nearby
   (lying doc claims, tests that don't assert their namesake, teaching-free error
   messages, dead code, stale comments, missing edge cases). An empty noticing section
   from code-touching work is a red flag.
3. Eager-fix or file — never silently skip: <1h + no new dependency → fix in place;
   else → `.planning/milestones/v0.15.0-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` (now
   split into `surprises-intake/part-NN.md` / `good-to-haves/part-NN.md` per the
   `f654cfc3` split — append to the newest part under 20k, or start a new part) with
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
  (gsd-verifier → **committed `p12X/VERDICT.md` in the SAME close**, not deferred — the
  OP-7 lesson) → on GREEN advance STATE (completed_phases+1, bump percent, cursor
  next=P12Y) → commit+push+re-confirm CI). Only advance to the next phase after the
  current is CLOSED GREEN on green main.
- Drive every close with **verifier→executor LEAVES dispatched directly**; NEVER
  `fork`-to-resume a warm coordinator (confabulates a no-op — confirmed anti-pattern,
  fully landed in doctrine at `88168478`). Resume a warm agent via SendMessage-to-its-id.
- Absorb C1 rotations below the top: when a C1 relieves (writes a handover under
  `.planning/phases/12X-*/`), the C2 re-dispatches a FRESH successor C1 pointed at that
  handover — do NOT bubble C1 rotations to L0.
- Relieve YOURSELF past ~100k tokens of OWN context (hard stop ~150k; absolute, not %)
  at a PHASE boundary (never mid-phase): dispatch relief-handover-writer → report SHA to
  L0 → stop. Compress now-superseded prior-rotation prose in this file rather than
  pure-appending (see §5 growth note) — this file is on the file-size waiver umbrella and
  should not grow unboundedly just because it is currently covered.
- Report to L0 only: (a) own relief, (b) owner-decision escalation, (c)
  milestone-close-ready, (d) a 2–3-phase checkpoint, (e) each push→CI-in-flight handoff
  (L0 holds the durable watch). Otherwise route and integrate.
