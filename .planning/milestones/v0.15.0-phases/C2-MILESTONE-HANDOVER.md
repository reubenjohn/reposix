# C2-MILESTONE-HANDOVER.md — v0.15.0 "Floor" C2 continuation, P124/P125 boundary, 2026-07-18

Written by the outgoing C2 milestone coordinator-of-coordinators (seat #63-C2) at a
**P124-close / pre-P125 wave boundary**. P124 is CLOSED GREEN on green main; P125 has NOT
been dispatched yet — and a specific, ordered `docs/roadmap.md` quick + push cadence step
sits BEFORE opening P125 (see §6 step 1–2). Naming note: this file is **updated in place**
following the established convention (edited in place at both prior C2 boundaries —
`47c1f9d3` P122/P123, `d4ea76cb` P123/P124) — it is the milestone-scoped C2 continuity doc,
analogous to `.planning/SESSION-HANDOVER.md`, not a phase-scoped `<N>-HANDOVER.md`.

**Unconfirmed vs. the prior rotation:** the P123/P124 handover was written at a
*coordinated* clean relief where L0 rotated simultaneously. This time there is **no
evidence L0 is relieving at the same boundary** — `.planning/SESSION-HANDOVER.md`'s last
touch is still `57bf9376` ("#62→#63 relief", coincident with the P123/P124 boundary) with
no newer commit against that path. Do not assume a fresh L0 handover exists; re-check
`git log -- .planning/SESSION-HANDOVER.md` yourself before relying on any L0 seat claim.

**Successor's required first reads, in order:** `.planning/ORCHESTRATION.md` (full — you
are a C2, §3 governs your own relief and your C1-rotation-absorption duty), `.planning/
PROJECT.md` (Current Milestone: v0.15.0 Floor), `CLAUDE.md` (root — non-negotiables), then
this file in full before dispatching anything.

---

## 1. Ground truth (git)

- **origin/main == `790aa73c`.** Verified live via `gh run list --branch main`: newest
  runs on `790aa73c` are ALL `success` — `CI` (run `29657431393`), `CodeQL` (run
  `29657431196`), `release-plz` (run `29657431388`), `Docs` (run `29657621087`). Main is
  GREEN. The P0 `code/ci-green-on-main` probe passes on main's newest `ci.yml` run.
- **Local HEAD == `21dcfd7a`, 4 commits AHEAD of origin/main.** `git status`: clean,
  nothing to commit. These 4 commits do **NOT** ride a standalone push — they ride
  **P125's** phase-close push (confirmed: `git rev-list --left-right --count
  origin/main...HEAD` = `0 4`, i.e. 0 behind / 4 ahead, no divergence to rebase).
  - `21dcfd7a` — docs(intake): file P124-close noticings (shell-coverage/verdict-trap/
    coverage/zsh/sc2-flake) — 2 SURPRISES-INTAKE (part-07, 5→7 entries) + 3 GOOD-TO-HAVES
    (new part-10, GTH-V15-86..88)
  - `b01afabc` — docs(124-close): STATE 10→11 P124 CLOSED GREEN (hand-advanced via the
    read path — `gsd-sdk query state.load` + manual Edit, **NOT** `state.advance-plan`,
    per the global corruption-bug workaround); ROADMAP phase-index Phase 124 → `[x]`
    complete (2026-07-18)
  - `8d9a269a` — chore(124-close): mint P124 docs-repro/structure catalog rows to PASS
    (4 mechanical rows NOT-VERIFIED → PASS: `container-congruence-earned`,
    `container-rehearse-sigkill-safe`, `container-rehearse-exit-from-artifact`,
    `container-rehearse-binary-provenance`)
  - `3b1a61ce` — docs(124-close): P124 SUMMARY + ROADMAP DRAIN-annotation fix (adds
    `124-SUMMARY.md`; corrects SC→DRAIN mapping: SC1=DRAIN-22, SC2=DRAIN-23,
    SC3=DRAIN-24, SC4=DRAIN-13+14)
- **P124 CLOSED GREEN.** `.planning/phases/124-container-rehearse-harness-hardening/`
  contains `124-PLAN.md` + `124-SUMMARY.md` (+ a stale phase-local `deferred-items.md`,
  now migrated into the two intake ledgers per the 21dcfd7a commit). **There is no
  committed `124-VERIFICATION.md`** — the gsd-verifier's GREEN verdict was returned
  directly as the signal (per the P124 close-bookkeeping lane's harness directive), not
  written to a committed file. The gitignored bare-session roll-up at
  `quality/reports/verdicts/p124/2026-07-18T20-05-49Z.md` exists but is **excluded** by
  `.gitignore:110` (`quality/reports/verdicts/*/[0-9]*.md`) — confirmed via
  `git check-ignore -v`. STATE.md documents that this roll-up reads a misleading RED (411
  un-run NOT-VERIFIED rows + 8 pre-existing standing P2 FAILs from older milestones, NOT a
  P124 regression) — this is filed as a SURPRISES-INTAKE row, see §5 "verdict-trap".
- **STATE.md frontmatter** (`last_updated: 2026-07-18T20:30:00.000Z`):
  `completed_phases: 11`, `total_phases: 15`, `percent: 73`. **Cursor: 11/15 phases
  complete (P114–P124), 73%. Next = P125.**
- **Stale-but-carry-only:** STATE.md frontmatter `total_plans: 3 / completed_plans: 2` is
  stale (P124 shipped as a monolithic close, not a tracked multi-plan sequence) — noted,
  not a blocker, not yet fixed. Do not treat it as authoritative plan-count.

## 2. Wave/cycle state

| Phase | Plans | State | Verdict / commits |
|---|---|---|---|
| P114–P121 | — | DONE | unchanged since the P123/P124 handover — see prior revision in git history (`d4ea76cb`) for detail |
| P122 `reposix-remote` + `init` hardening | 4/4 | DONE — CLOSED GREEN | `p122/VERDICT.md` (`00ab1579`); close `a9e1f4c4` |
| P123 Quality-runner & catalog integrity hardening | 5/5 SC PASS | DONE — CLOSED GREEN | `p123/VERDICT.md` (`2f6d62ff`); close `47283d75` |
| **P124 Container-rehearse harness hardening** | **4/4 SC PASS** | **DONE — CLOSED GREEN** | verdict returned directly (no committed VERIFICATION.md); SUMMARY `3b1a61ce`; catalog mint `8d9a269a`; STATE advance `b01afabc`; intake `21dcfd7a` — **all 4 LOCAL, unpushed, ride P125's push** |
| **P125 Real-backend cadence & mirror-drift resilience** | **0/TBD** | **NOT STARTED — next, but a `docs/roadmap.md` quick + push run FIRST (§6 step 1–2)** | — |
| P126 Docs-alignment tooling polish | 0/TBD | NOT STARTED | — |
| P127 Surprises absorption (OP-8 Slot 1) | 0/TBD | NOT STARTED | — |
| P128 Good-to-haves polish + milestone close (OP-9 Slot 2) | 0/TBD | NOT STARTED | — |

**Cursor: 11/15 phases complete (P114–P124), 73%.** Next = P125 — but the successor's
FIRST action is a `/gsd-quick` on `docs/roadmap.md`, then the push, THEN opening P125
(§6, in order — do not skip ahead to `/gsd-plan-phase 125` first).

**P124 close highlights (2026-07-18):** 4/4 SC PASS, verdict returned directly (harness
directive, no committed file). SC1 (DRAIN-22) — container-row congruence is now EARNED
via per-step `ASSERT-PASS:` stdout harvesting instead of copied-verbatim
`expected.asserts`; a non-empty-harvest guard closes the F-K4b zero-line tautology;
example-05 now drives the REAL runtime `BLOB_LIMIT_EXCEEDED_FMT` refusal + `git
sparse-checkout` recovery cycle (was previously a pre-emptive source-constant stand-in).
SC2 (DRAIN-23) — SIGKILL-proof ephemeral-sim teardown via `setsid` process-group kill +
an internal `timeout` shorter than the row's `timeout_s`, plus a fail-loud pre-run
stale-orphan gate on port 7878. SC3 (DRAIN-24) — explicit `cargo build -p reposix-cli`
provenance step precedes post-release gates in `quality-post-release.yml`. SC4
(DRAIN-13+14) — harness exit derived strictly from the persisted artifact `exit_code`
(docker rc=0 can no longer mask exit_code=1) + `.sim-*.log` gitignored. Catalog-first
held: 5 rows minted NOT-VERIFIED at W0 strictly before impl; 4 mechanical rows graded
PASS at close; `example-05-blob-limit-recovery` (post-release cadence) re-grades in the
post-release container job — NOT-VERIFIED by-design at pre-push (mirrors P123's SC5b
pattern). A post-review fix wave (`790aa73c`) landed 4 review findings (M1/M2/L1/L2)
before the push.

### Phase-list ground truth: P125–P128 scope is ALREADY PINNED, not a blank slate

Unchanged from the prior handover — **the AUTHORITATIVE live roadmap is the top-level
`.planning/ROADMAP.md`** (§ "v0.15.0 Floor"). Verified fresh this rotation:

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
  concurrently — verified clean this rotation, 0 behind), then `git push origin main`
  BEFORE the verifier subagent dispatch, THEN `python3 quality/runners/run.py --cadence
  post-push --persist` — the P0 `code/ci-green-on-main` probe must show main's NEWEST
  `ci.yml` run = success. Never open the next phase over a red main. **STOP at the
  push→CI-in-flight boundary and RETURN to L0** — L0 holds the durable CI watch; do not
  self-watch (P122 incident, §5 doctrine below).
- **Tainted-by-default / `REPOSIX_ALLOWED_ORIGINS` egress allowlist.** Sim is the default
  backend for every demo/unit-test/autonomous loop.
- **Model tiering:** every C1 gets an EXPLICIT `model` override — opus for
  security/genuinely-complex, sonnet default, haiku mechanical. Never `fable` at a leaf.
  P125's mirror-drift/real-backend-cadence work is a plausible opus candidate (real
  backend + security-adjacent reconciliation logic) — the dispatching C2 decides at
  `/gsd-plan-phase 125` time.

## 4. Litmus / gate / REOPEN state

- **P124 verdict:** GREEN, returned directly (no committed VERIFICATION.md — see §1). No
  open REOPEN state.
- **Non-gating known-FAIL / process-friction, carried (NOT a close blocker):**
  - `code/shell-coverage` P2 counter-drift (L1166, pre-existing, still open) — now
    ADDITIONALLY forces a `--persist` downgrade-REFUSAL on every phase-close mint (new
    angle surfaced at P124 close, filed to SURPRISES-INTAKE part-07). See §5 HELD list —
    this is a P127 Slot-1 candidate needing an owner decision among 3 options (fix-drift
    / accept-FAIL / decouple-via-WARN).
  - `verdict.py --phase N` bare-session collation reads a **false RED** (411 un-run
    NOT-VERIFIED rows + pre-existing standing reds from older milestones) — a naive
    verifier could mis-grade a clean phase RED off this artifact alone. Filed to
    SURPRISES-INTAKE part-07; fix-twice note owed to `quality/PROTOCOL.md`; P126
    candidate.
- **Open-waiver expiry clocks:**
  - `structure/file-size-limits` OVER-BUDGET-tier `--warn-only` waiver — **expires
    2026-08-08T00:00:00Z**. **STATUS CHANGED since the prior handover**: the two v0.15.0
    ledgers (`GOOD-TO-HAVES.md`, `SURPRISES-INTAKE.md`) were split THIS ROTATION via
    `scripts/split_ledger.py` (`f654cfc3`) into `good-to-haves/` (9 parts) +
    `surprises-intake/` (7 parts), each part <20k, byte-exact round-trip proven, INDEX
    files rewritten in place. Those two specific files are **NO LONGER waived**. The
    umbrella waiver **still covers 91 other files total** (re-counted 2026-07-18 via the
    gate — see catalog comment in `quality/catalogs/freshness-invariants.json:667`,
    `structure/file-size-limits` row). **This 91-file residual is HELD, escalate-first —
    do NOT self-resolve** (filed `bc4decf3`, expires with the same 2026-08-08 clock; L0 is
    holding for an owner decision). §5/§"HELD" carries this forward.
  - Hero-number doc-alignment waivers (8 rows, BENCH-01-fed) — **expire 2026-08-15**;
    unchanged, still **not re-verified fresh** by this handover — flag as a re-ground item
    for whoever plans P126+.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

### Doctrine landed THIS rotation (verify-present, no rewrite owed)

Both doctrine items flagged as "owed" in the P123/P124 handover are now **CONFIRMED
LANDED**, verified by direct commit lookup this rotation:
- **(a) L0-owns-CI-watch liveness** — landed at `b2eca628` (prior rotation), still present
  in `.planning/ORCHESTRATION.md` §3/§11. Re-verified present.
- **(b) `fork`-to-resume anti-pattern** — landed THIS rotation at `88168478`
  (`docs(quick-260718-fork): fork-to-resume anti-pattern doctrine (ORCHESTRATION §11 +
  coordinator-dispatch §6a)`). Both items are now fully formalized; **nothing further
  owed on either.**

### Rotation accomplishments (this seat, for continuity)

- **Quiet-point lanes CLOSED** at the P123→P124 boundary (the prior handover's §6 step 2
  directive): split-archive of the two over-budget v0.15.0 intake ledgers via
  `scripts/split_ledger.py` (`f654cfc3`, reusing the P103 precedent mechanism); the
  `fork`-to-resume anti-pattern doctrine landed into ORCHESTRATION §11 +
  `coordinator-dispatch` skill (`88168478`); the 91-file waiver-residual surprise filed
  (`bc4decf3`) so the umbrella's true scope is visible and tracked, not silently
  shrinking.
- **P124 driven end-to-end** via a C1 phase-coordinator (full GSD arc: plan → execute →
  code-review → push → close), then a **direct-leaf close** (P122-blessed deterministic
  pattern — verifier→executor leaves dispatched directly, NOT a fork-to-resume).
  **L1129** (pre-push budget measured ~109–121s vs. the documented ~55s, filed at P123
  close) and **L1166** (shell-coverage two-honesty-layers ambiguity) were BOTH the
  charged first-grounding investigation items for P124's C1 and are now **RESOLVED
  fix-twice**: `CLAUDE.md`/`quality/CLAUDE.md` re-measured and republished the pre-push
  budget as ~90-120s (122s re-measured 2026-07-18/P124, dominated by kcov shell-coverage
  + full-workspace clippy/mkdocs, NOT diff-size), and the shell-coverage
  self-report-vs-kcov two-layer honesty mechanism got the one-line `quality/CLAUDE.md`
  clarification called for. Confirmed present via direct grep this rotation.

### Verdict-trap NOTICED (filed, tracked — see §4)

The gitignored bare-session `verdict.py` roll-up at
`quality/reports/verdicts/p124/2026-07-18T20-05-49Z.md` reads RED on a naive read (411
un-run NOT-VERIFIED + 8 pre-existing standing P2 FAILs from v0.13.0/v0.14.0
drain/tag/changelog/hygiene rows) — this is NOT a P124 regression; phase truth is the
per-phase gates + `code/ci-green-on-main`. A future verifier or successor coordinator
reading that file cold could mis-grade a clean phase RED; the fix-twice note to
`quality/PROTOCOL.md` clarifying "per-phase gates are the truth, not a bare-session
full-catalog collation" is OWED, not yet written — P126 candidate (tracked in
SURPRISES-INTAKE part-07, do not re-file).

### Carry-only NOTICED (not yet filed, low priority — track, decide later)

- STATE.md frontmatter `total_plans: 3 / completed_plans: 2` is stale (P124 shipped as a
  monolithic close, not a tracked multi-plan sequence). Cosmetic; not yet worth a
  dedicated GTH row, but flag it if another close makes the drift compound.
- The two-ledger INDEX "N entries" unit is loose/undocumented — GOOD-TO-HAVES INDEX
  counts *sections* (part files), SURPRISES-INTAKE INDEX counts *dated entries* within
  sections. Not a bug, just an inconsistent unit across the two files' own headers; a
  future split-ledger touch-up could standardize it.

## HELD / ESCALATE-FIRST (NEVER self-authorize — carry forward to L0/owner)

- **91-file file-size global-waiver umbrella** (`structure/file-size-limits` single row,
  expires **2026-08-08**, real count 91 not the historical 56; STATE.md/ORCHESTRATION.md
  among the un-splittable live planning ledgers) — L0 is HOLDING for owner. Do NOT
  self-resolve. Options sketched in the `bc4decf3` surprise entry (accept as permanent
  waived category vs. shard convention vs. selective drain). **GTH-V15-84**
  (`container-rehearse.sh`/`sigkill-safe.sh` now over 10k after P124's additions) +
  **GTH-V15-78** share this umbrella.
- **E1 launch-animation publish (GTH-V15-37)** — owner approval still PENDING per
  `.planning/CONSULT-DECISIONS.md` (2026-07-17 entries). `docs-build/animation-renders`
  staying NOT-VERIFIED is a pending gate, not an accepted deferral. Never self-authorize a
  `gh release upload` for this asset.
- **Global `gsd-sdk state.advance-plan` STATE.md-corruption bug** — held upstream with L0.
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
   authoritative cursor (P124 CLOSED, completed_phases=11, next=P125).
2. **Run L0's `docs/roadmap.md` quick FIRST** (a `/gsd-quick`, `gsd-executor`, sonnet is
   sufficient — mechanical prose + a mermaid-adjacent doc, not security/architecture
   work). Requirements, verified against the live file this rotation (`docs/roadmap.md`
   currently has NO "Progress right now" strip — confirmed by direct read):
   - (a) Add a **"Progress right now"** strip to `docs/roadmap.md`: milestone name
     "Floor", phases-closed fraction+percent (**11/15, 73%**), ONE capability-language
     line in plain user terms (**NO phase numbers**), last-updated date. Keep the moving
     lines **BINDING-FREE** — verify no `quality/catalogs/doc-alignment.json` row cites
     them (P117 W3 lesson: a later, separate reword re-drifts a binding even after a
     prior correct rebind).
   - (b) Update the `<!-- SYNC: -->` pairing comment at `docs/roadmap.md:7` + the "How to
     read this" section (currently starts ~line 39 — confirmed by direct read) to document
     that the strip refreshes EVERY phase close, while the mermaid arcs re-color only at
     milestone close (two distinct cadences, currently undocumented as two cadences).
   - (c) **FIX-TWICE**: encode the per-phase-close strip refresh into phase-close doctrine
     (`.planning/CLAUDE.md` push-cadence/phase-close section + the GSD phase-close
     checklist the verifier grades against); tag the dimension (`docs-alignment` or
     `structure`, whichever the new invariant actually belongs to).
   - (d) Add P123's missing completion date in `.planning/ROADMAP.md` phase index — line
     76 currently reads `**Phase 123: Quality-runner & catalog integrity hardening** -
     ... resist false-greens, silent corruption, and misleading errors.` with **no**
     `(completed ...)` suffix (confirmed by direct read; every sibling phase 114–122 and
     124 HAS one). Match the sibling date format: `(completed 2026-07-18)`.
   - (e) **[C2-added, from the P124-close executor's NOTICED]** ALSO reconcile the
     `.planning/ROADMAP.md` "## Progress" TABLE (around line 313 in the current file —
     confirmed the P124 row still reads `0/TBD | Not started | -` while the phase-index
     already shows it complete). Fix the table row to `124. Container-rehearse harness
     hardening | 4/4 | Complete | 2026-07-18` for internal consistency. Consider whether
     the strip-refresh doctrine from (c) should also cover this table to prevent
     recurrence at future phase closes.
   Local gates before push: `python3 quality/runners/run.py --cadence pre-push` (runs
   mkdocs-strict + mermaid-renders + banned-words — run the FULL cadence, not
   hand-picked gates; P117 W3 lesson: docs-build-only sweeps let a banned-words violation
   through). Optional `/doc-clarity-review` cold-reader pass on the new strip (north-star
   #5, OD-3). Report the quick's outcome + SHA in your next return.
3. **Push:** `git fetch origin && git rebase origin/main` (re-check for concurrent
   pushes — was clean at handover-write time, 0 behind), then `git push origin main` —
   carries the 4 already-local P124-close commits + the quick's new commit(s) + this
   handover's commit. **STOP at the push→CI-in-flight boundary and RETURN to L0** with
   the pushed SHA + in-flight `ci.yml` run id (L0 holds the durable CI watch; do NOT
   self-watch — this is the P122 close-liveness incident's exact lesson). On L0's green
   signal, run `python3 quality/runners/run.py --cadence post-push --persist` and confirm
   the P0 `code/ci-green-on-main` probe shows main's newest run = success.
4. **Open P125 only after that push is confirmed CI-green.** P125's scope (real-backend
   cadence & mirror-drift resilience, Reqs DRAIN-02/DRAIN-12) is ALREADY PINNED in the
   top-level `.planning/ROADMAP.md` (§2 above) — first act is `/gsd-plan-phase 125`
   (gsd-planner + gsd-plan-checker), then dispatch a C1 `phase-coordinator` with an
   EXPLICIT model override (see §3 note — opus is a plausible fit for real-backend +
   mirror-reconciliation security-adjacent work; the dispatching C2 decides at plan time)
   per the same pattern used for P124: full GSD arc (plan → execute → code-review →
   phase-close push → `run.py --cadence post-push --persist` confirming
   `code/ci-green-on-main` → **gsd-verifier directly → verdict** → on GREEN, STATE-advance
   completed_phases 11→12, bump percent, cursor next=P126 → commit+push+re-confirm CI).
   Drive the close with **verifier→executor LEAVES dispatched directly** — NEVER a
   `fork`-to-resume (confirmed anti-pattern, now fully landed in doctrine).
5. **Continue P126 → P127 (OP-8) → P128 (OP-9 + milestone close)** in order, one C1 per
   phase, each gated CLOSED GREEN on green main before the next dispatch. Absorb C1
   rotations below yourself — re-dispatch a fresh successor C1 pointed at its own
   handover; do NOT bubble C1 rotations to L0. Report to L0 ONLY at stop points: your own
   relief, an owner-decision escalation, milestone-close-ready, a 2–3-phase checkpoint,
   and each push→CI-in-flight handoff.
6. **Relieve yourself (the C2) past ~100k tokens of your OWN context** (hard stop ~150k,
   absolute not %) at a PHASE boundary — dispatch `relief-handover-writer` (update THIS
   file in place, per the established convention), report the SHA to L0, stop.
7. **At P128 / milestone-close:** do NOT self-authorize any tag/release. Report
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
user-visible breaking change or real-backend mutation beyond the sanctioned targets. **NEW
this rotation:** the 91-file file-size waiver umbrella residual (`bc4decf3`) is now
explicitly HELD for an owner decision among 3 sketched options — do not self-resolve.

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
  `docs/roadmap.md`'s moving "Progress right now" strip too if it's ever bound; §6 step 2
  explicitly calls out keeping it binding-free to sidestep this.
- Docs edits pass BOTH `docs-build/*` AND `structure/banned-words` locally before a push
  (fix-twice, P117).
- Fix-twice meta-rule: any new file/convention/gate revises the relevant CLAUDE.md (root
  or scoped) / ORCHESTRATION.md in the SAME PR, tagged with the quality dimension.
- L0-owns-CI-watch + fork-to-resume-is-an-anti-pattern are BOTH now fully landed in
  doctrine (§5 above) — a C1 charter can cite ORCHESTRATION §3/§11 directly instead of
  re-deriving the lesson prose each time.

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
  (gsd-verifier → committed verdict where the harness calls for one) → on GREEN advance
  STATE (completed_phases+1, bump percent, cursor next=P12Y) → commit+push+re-confirm CI.
  Only advance to the next phase after the current is CLOSED GREEN on green main.
- Drive every close with **verifier→executor LEAVES dispatched directly**; NEVER
  `fork`-to-resume a warm coordinator (confabulates a no-op — confirmed anti-pattern,
  fully landed in doctrine at `88168478`). Resume a warm agent via SendMessage-to-its-id.
- Absorb C1 rotations below the top: when a C1 relieves (writes a handover under
  `.planning/phases/12X-*/`), the C2 re-dispatches a FRESH successor C1 pointed at that
  handover — do NOT bubble C1 rotations to L0.
- Relieve YOURSELF past ~100k tokens of OWN context (hard stop ~150k; absolute, not %)
  at a PHASE boundary (never mid-phase): dispatch relief-handover-writer → report SHA to
  L0 → stop.
- Report to L0 only: (a) own relief, (b) owner-decision escalation, (c)
  milestone-close-ready, (d) a 2–3-phase checkpoint, (e) each push→CI-in-flight handoff
  (L0 holds the durable watch). Otherwise route and integrate.
