# RELIEF-HANDOVER-C2-wave-1.md — v0.15.0 "Floor" C2 relief @ P125 close-drive boundary, 2026-07-19

Written by the outgoing v0.15.0 C2 milestone coordinator-of-coordinators, relieving at a
clean **push→CI-in-flight boundary** (not mid-phase; nothing in a tree-writer's hands).
Follows the established `RELIEF-HANDOVER-C2-wave-N.md` naming convention (precedent:
`.planning/milestones/v0.13.1-phases/RELIEF-HANDOVER-C2-wave-{c,f}.md`,
`.planning/milestones/v0.14.0-phases/RELIEF-HANDOVER-C2-wave-{2,2b}.md`) rather than a
further in-place edit of `C2-MILESTONE-HANDOVER.md`: that file is **already ~31KB**
against its own prior-rotation-flagged ~35-40KB ceiling (see its own §5 "Handover-doc
growth NOTICED"), and this rotation's content (a critical infra finding + a full P125
close-drive state) would push it over. `C2-MILESTONE-HANDOVER.md` gets a **one-line
pointer** to this file in the same commit; it is NOT otherwise touched — its content is
stale as of `c267f0e8` (P124/P125 boundary) and superseded by this file for anything P125+.

**Successor's required first reads, in order:** this file in full, then
`.planning/ORCHESTRATION.md` §3 (relief/liveness doctrine — read the Liveness doctrine
paragraph twice given the SendMessage finding below), `.planning/STATE.md` (authoritative
cursor), `C2-MILESTONE-HANDOVER.md` (background/doctrine only — NOT current state), the
original C2 charter (P125→P128 + milestone close).

**Do-not-touch guardrails:** do NOT push over the in-flight/unconfirmed CI run named
below; do NOT self-watch it (L0's job); do NOT reorder the pending-work list past the
CI-green gate; do NOT self-authorize anything under §6's HELD/ESCALATE list.

---

## 1. Ground truth (git)

Verified live this rotation, not carried from a stale snapshot:

- **Local HEAD = `cb4c2b3d`.** `git status --porcelain` — **clean**. `git rev-list
  --left-right --count origin/main...HEAD` = **`0 1`** (0 behind, 1 ahead) — local is
  exactly one commit ahead of `origin/main`, that commit **UNPUSHED**.
- **`origin/main` = `1b660472`, PUSHED.** Confirmed via `git log origin/main -3`.
- **CI run `29671820890`** (workflow "CI", event push, headSha `1b660472`) — checked
  read-only this rotation (single status read, NOT a background watch): **14/15 jobs
  green**, the `quality gates (pre-pr)` job was still running (triggered ~18 min prior at
  check time). **This run's final verdict is UNKNOWN to this handover** — **L0 holds the
  durable watch**; the successor's first move on waking is to re-check it fresh, not trust
  this snapshot.
- **Commit chain since the last handover boundary (`c267f0e8`, where
  `C2-MILESTONE-HANDOVER.md` was last written), all LOCAL-then-pushed except the tip:**
  - `ea061553`…`e9014566` (7 commits) — P125 research + validation-strategy + plan
    authoring (3 plans, 2 waves: mirror-drift resilience).
  - `2ccdf463`/`77018098`/`dda95b5c`/`88b5e797` — **plan 125-01** (SC3/DRAIN-12): fixed +
    tested the helper's `git pull --rebase` mirror-lag hint, remote-explicit Pattern-C
    rebase in troubleshooting docs, SUMMARY.
  - `3864e1a6`/`35c41ac1`/`d219ef51` — **plan 125-02** (SC2/DRAIN-12): litmus self-heals
    BOTH backend drift AND GitHub-mirror drift before the marker edit; run-twice +
    backend-drift manual verification doc; SUMMARY.
  - `28a8646b` — an **L0 session-handover rotation landed mid-span** (`#13→#14`,
    `.planning/SESSION-HANDOVER.md`) — confirms L0's own seat rotated during this C2's
    watch; not this C2's file, no action owed, noted for ground-truth completeness.
  - `cdaee30a`/`0808d48f`/`8604f087` — **plan 125-03** (SC1/DRAIN-02 + PART2 SC3/DRAIN-12):
    documented `scripts/refresh-tokenworld-mirror.sh` mandatory pre-step; made the v0.14.0
    attach-tree recovery blockquote remote-explicit.
  - **`1b660472`** — P125 **close-bookkeeping**: STATE.md 11→12 (73%→80%), ROADMAP.md
    phase-125 checkbox + 3/3 row, REQUIREMENTS.md DRAIN-02/DRAIN-12 flipped `[x]`
    Complete, docs/roadmap.md strip refresh, 2 new SURPRISES-INTAKE rows (part-07). **This
    is the commit that got pushed** (carries the 2 already-held relief commits `3bb3d39a` +
    `c487bb91` too, per the harness note — confirmed those are ancestors of `1b660472` via
    `origin/main` log).
  - **`cb4c2b3d`** (HEAD, **UNPUSHED**) — owner-directed `/gsd-quick`: reshaped
    `docs/roadmap.md`'s "Progress right now" strip into the sequenced three-block view
    (`Landed recently` / `In flight now` / `Up next, in order`), reversed the prior
    "no phase numbers" HARD CONSTRAINT in `.planning/CLAUDE.md` (phase numbers now allowed
    as sequence markers), recorded a `[SELF]` DP-4 entry in `.planning/CONSULT-DECISIONS.md`.
- **P124 CLOSED GREEN**, independent `quality/reports/verdicts/p124/VERDICT.md` committed
  (`c267f0e8`) — unchanged this rotation, re-confirmed present via `ls`.
- **P125 is EXECUTED + close-bookkeeping PUSHED but NOT verdict-graded.** `quality/reports/
  verdicts/p125/` directory **does not exist yet** (confirmed via `ls` — `No such file or
  directory`). `.planning/STATE.md`'s own Current-Position prose says it plainly: *"P125…
  CLOSING 2026-07-19 (phase-close bookkeeping + push lane; verifier not yet dispatched)"* —
  the frontmatter (`completed_phases: 12`, `percent: 80`) is **optimistic**, advanced
  ahead of verdict per this milestone's established push-before-verify convention (same
  pattern P124 briefly had, closed by OP-7 remediation). **Do not treat 12/15 as verdict-
  confirmed** until the gsd-verifier dispatch below completes GREEN.
- **Milestone: 11/15 phases verdict-closed GREEN (P114–P124); P125 in-flight (executed,
  pushed, ungraded); 12/15 optimistic-STATE.**

## 2. Wave/cycle state

| Phase | Plans | State | Commits / verdict |
|---|---|---|---|
| P114–P121 | — | DONE, CLOSED GREEN | unchanged, see `C2-MILESTONE-HANDOVER.md` history for detail |
| P122 `reposix-remote` + `init` hardening | 4/4 | DONE, CLOSED GREEN | `p122/VERDICT.md` (`00ab1579`) |
| P123 Quality-runner & catalog integrity | 5/5 SC | DONE, CLOSED GREEN | `p123/VERDICT.md` (`2f6d62ff`) |
| P124 Container-rehearse harness hardening | 4/4 SC | DONE, CLOSED GREEN | `p124/VERDICT.md` (`c267f0e8`, OP-7 remediation) |
| **P125 Real-backend cadence & mirror-drift resilience** | **3/3 plans EXECUTED** | **CLOSING — close-bookkeeping pushed (`1b660472`), verifier NOT yet dispatched** | Wave1: 125-01 (`2ccdf463`…`88b5e797`), 125-02 (`3864e1a6`…`d219ef51`); Wave2: 125-03 (`cdaee30a`…`8604f087`); close-bookkeeping `1b660472` |
| P126 Docs-alignment tooling polish | 0/TBD | NOT STARTED | — |
| P127 Surprises absorption (OP-8 Slot 1) | 0/TBD | NOT STARTED | — |
| P128 Good-to-haves polish + milestone close (OP-9 Slot 2) | 0/TBD | NOT STARTED | — |

**Named incident to read before dispatching anything — SendMessage disabled at C2 tier.**
This rotation discovered `SendMessage` returns **"No such tool available… not enabled in
this context"** when invoked from the C2 seat. The charter's/doctrine's standing
"resume a warm agent via SendMessage-to-its-id" liveness pattern (`ORCHESTRATION.md` §11)
**does not work at this tier this session**. Practical consequence already applied this
rotation: **never dispatch a new tree-writer while another child might still be mid-write
— you cannot recall or resume it.** Serialize strictly; before every writer dispatch,
verify a clean, no-live-leaf tree via `git status` AND a check that no child's expected
output file has a very-recent unexplained mtime. Phase CLOSE itself is unaffected — it is
driven by **fresh** `gsd-verifier`→`gsd-executor` LEAVES (a new dispatch each time), which
needs no SendMessage. At a push→CI-in-flight boundary, stop and return the run id to L0
(which DOES appear to retain SendMessage capability per the Liveness doctrine's own
wording — untested by this C2 but not contradicted). See §5 for the "noticed, not yet
filed" disposition of this finding.

## 3. Binding constraints (unchanged — embed verbatim in every dispatch)

- **ONE cargo invocation machine-wide, FOREGROUND-only** (never `run_in_background`;
  orphans the build, evades `cargo-mutex.sh`, OOM risk). Prefer `-p <crate>`.
- **Leaf test setup runs in a `/tmp` clone, `cd`-ing in the SAME Bash invocation — NEVER
  the shared repo.** Mechanically enforced (PreToolUse `leaf-isolation-guard.sh` exit 2 +
  pre-commit backstop + binary-side `reposix init` refusal RPX-0406).
- **Uncommitted = didn't happen.** Commit before ending any turn.
- **No `--no-verify`, ever.**
- **One tree-writer at a time** — now with NO SendMessage-based recall/resume available
  at this tier (§2 finding); this makes strict serialization non-negotiable, not just
  best practice.
- **Push cadence:** `git fetch origin && git rebase origin/main`, then `git push origin
  main` BEFORE the verifier-subagent dispatch, THEN `python3 quality/runners/run.py
  --cadence post-push --persist` — the P0 `code/ci-green-on-main` probe must show main's
  NEWEST `ci.yml` run = success. Never open the next phase over a red main. **STOP at the
  push→CI-in-flight boundary and RETURN to L0** — do not self-watch (P122 incident,
  `ORCHESTRATION.md` §3 Liveness doctrine).
- **Tainted-by-default / `REPOSIX_ALLOWED_ORIGINS` egress allowlist.** Sim is the default
  backend everywhere.
- **Model tiering:** every C1 gets an EXPLICIT `model` override — opus
  security/genuinely-complex, sonnet default, haiku mechanical. Never fable at a leaf.
- **Commit-trailer format:** `Co-Authored-By: Claude Opus 4.8 (1M context)
  <noreply@anthropic.com>` on every non-trivial commit (see this handover's own commit).
- **OD-3 ownership charter (5 points) embedded in every dispatch:** acceptance criteria
  are the floor not the ceiling; noticing is a deliverable; eager-fix (<1h, no new dep) or
  file to `SURPRISES-INTAKE`/`GOOD-TO-HAVES` (OP-8), never silently skip; verify against
  reality; Rust-compiler-grade UX as the standing north star.

## 4. Litmus / gate / REOPEN state

- **CI run `29671820890` on `1b660472`** — IN-FLIGHT at last read (14/15 green, `quality
  gates (pre-pr)` still running). **L0 holds this watch.** No REOPEN state — nothing has
  failed; this is a pending-verdict, not a red.
- **P124 verdict:** GREEN, committed, unchanged — `quality/reports/verdicts/p124/
  VERDICT.md` (`c267f0e8`).
- **P125 verdict: NOT YET MINTED.** `quality/reports/verdicts/p125/` does not exist. This
  is the successor's first substantive task once CI confirms green (§6 step 2).
- **Ground-truth-verified doc-integrity gaps found this rotation (routed, not fixed —
  scope stayed docs-only close-bookkeeping):**
  - **`.planning/REQUIREMENTS.md` still shows DRAIN-13/14/22/23/24 as `[ ]` unchecked /
    "Pending"** (verified via direct `grep` this rotation: lines 247, 252, 179, 191, 201
    for the checkbox text; lines 333-334, 342-344 for the traceability-table rows) —
    **despite P124 being CLOSED GREEN** with its own `VERDICT.md` attesting all 4 SCs
    (which map to these 5 DRAIN items) were delivered against reality. P124's own
    close-bookkeeping commit (`b01afabc`) never touched `REQUIREMENTS.md`. Milestone
    bookkeeping is currently **lying stale** on this file ahead of milestone-close
    ratification — see §5/§6 raise-list routing to P128.
  - By contrast, **DRAIN-02 and DRAIN-12 (P125's own reqs) ARE already flipped `[x]`
    Complete** in `REQUIREMENTS.md`, and `ROADMAP.md` line ~314 already shows `125. …
    | 3/3 | Complete | 2026-07-19` — **ahead of verifier-graded truth** (P125 has no
    verdict yet). This is expected under the push-before-verify convention but must be
    watched: if the P125 verdict comes back RED, these rows (and STATE.md's 12/15) need
    to be walked back, not left lying.
  - **`docs/roadmap.md`'s "Landed recently" block currently lists P120–P124 (the last 5)**
    — verified via direct read of `docs/roadmap.md` lines 17-30 post-`cb4c2b3d`. This is
    an **OPEN CONTENT QUESTION**, not a confirmed bug: the executed quick's own commit
    message (`cb4c2b3d`) explicitly frames this as "P120-P124" by design, but the outgoing
    C2 flags that L0's turn-7 RACE-RESOLVED spec (as relayed into this rotation's charter)
    said **"P114–P124 = Landed recently"** (i.e., all 11 closed phases, not just the last
    5). **Successor must clarify with L0/owner which reading is correct before the next
    push** (§6 step 3) — do not unilaterally pick a side.
- **Open-waiver expiry clocks (unchanged this rotation, re-carry):**
  - `structure/file-size-limits` OVER-BUDGET `--warn-only` waiver — **expires
    2026-08-08T00:00:00Z**, covers 91 files including `C2-MILESTONE-HANDOVER.md`. **This
    new file (`RELIEF-HANDOVER-C2-wave-1.md`) is a FRESH file, not yet added to that
    count** — check its own size against the plain `structure/file-size-limits` gate (not
    assume waiver coverage) before it grows further; keep future rotations lean by
    starting `RELIEF-HANDOVER-C2-wave-2.md` rather than re-editing this one past ~20-25KB.
  - Hero-number doc-alignment waivers (8 rows) — **expire 2026-08-15**, still not
    re-verified fresh this rotation.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **DP-4 `[SELF]` decision, already formalized and committed** (`.planning/
  CONSULT-DECISIONS.md`, 2026-07-18 entry "roadmap three-block reshape interleaved at the
  P125 wave boundary") — the roadmap-reshape quick was executed under standing DP-4
  executive-resequencing authority, not an ad-hoc edit. Nothing further owed here; cited
  for completeness since it's the reasoning behind `cb4c2b3d`.
- **NOTICED, NOT YET FILED — the SendMessage-disabled-at-C2-tier finding (§2).** This is
  operationally significant enough that it should be surfaced to L0 explicitly (not just
  buried in this handover) so a decision can be made: (a) is this expected/permanent for
  the C2 tier and `ORCHESTRATION.md` §3/§11's "SendMessage-to-id resume" doctrine needs a
  tier caveat added (fix-twice meta-rule), or (b) this was a session-specific/transient
  tool-availability gap not representative of the tier in general. **Recommend the
  successor raise this to L0 at the next natural report-up point** rather than silently
  re-discovering it fresh next rotation. Not filed to SURPRISES-INTAKE by this outgoing C2
  because it is an orchestration-infra finding, not a v0.15.0 product/docs surprise — the
  successor may judge differently.
- **NOTICED, NOT YET FILED — `REQUIREMENTS.md` DRAIN-13/14/22/23/24 staleness (§4).**
  Already routed to P128 in the raise-list below; flagging here too since it is exactly
  the kind of "milestone bookkeeping lying ahead of ratification" issue OP-9's
  retrospective-distillation gate exists to catch — worth a one-line mention in the P128
  OP-9 RETROSPECTIVE section when that lands, not just a REQUIREMENTS.md checkbox fix.
- **RAISE-LIST for the successor (each with a destination — do not re-diagnose):**
  1. **P124 left DRAIN-13/14/22/23/24 unmarked in `REQUIREMENTS.md`** (P124 CLOSED GREEN,
     but its close-bookkeeping commit never touched this file) → **P128**: verify P124
     actually delivered them (cross-check against `p124/VERDICT.md`'s SC1-SC4), then flip
     the checkboxes + traceability rows. MEDIUM.
  2. **`.planning/ROADMAP.md` line ~314 marks P125 "Complete" 3/3 ahead of verifier
     truth** → reconcile at P125 verdict time (§6 step 2) — GREEN confirms it as-is; RED
     requires walking it back along with STATE.md's 12/15.
  3. **Tokenworld-mirror doc-truth (MEDIUM, filed `surprises-intake/part-07.md` line
     328)** — no `reposix-tokenworld-mirror` GitHub Action workflow exists in this repo
     (`gh workflow list` shows none; zero mirror/tokenworld runs in the last 30); root
     `CLAUDE.md`'s "authoritative 30-minute cron converger" claim is unverifiable from
     this repo (presumably owner-configured in the EXTERNAL mirror repo, or stale). P125's
     litmus self-heal (SC2/DRAIN-12) + the manual `refresh-tokenworld-mirror.sh` pre-step
     (SC1/DRAIN-02) may be the actual live compensating controls today, not the
     "authoritative" cron the docs lean on. **Owner confirmation needed** before the
     milestone-close real-backend vision-litmus leg is graded on the strength of that doc
     claim. → escalate to owner + **P128**.
  4. **gsd-sdk STATE.md all-time-scan corruption (filed `surprises-intake/part-07.md` line
     297, EXECUTED evidence this session)** — `gsd-sdk query state.planned-phase` (and
     sibling progress verbs) recomputed milestone progress as 21 phases/4 complete/19% by
     scanning ALL-TIME `.planning/phases/` dirs instead of this milestone's scoped 15, and
     dropped `last_activity` in the same pass. Reverted + hand-rewritten this rotation.
     **Standing mitigation: use the READ path only** (`gsd-sdk query state.load`) +
     hand-Edit for any STATE.md bump — NEVER `state.update-progress`/`state.advance-plan`.
     This is why every P114+ close hand-edits STATE.md. Cross-cutting tooling fix is a
     P126-or-later candidate (docs-tooling-polish or an OP-8 slot); held upstream, not an
     in-repo quick fix.
  5. **Pre-push hook now measured ~194s** (vs. root `CLAUDE.md`'s documented ~122s) —
     budget doc outgrowing corpus growth further since the P124-rotation's own
     re-measurement (109-113s → 122s → now reported 194s this rotation). Folds into the
     pre-pr-runtime-creep cluster already routed to **P126/P127**; re-measure fresh at
     milestone-close rather than chasing it mid-milestone.
  6. **Carried from the original C2 charter, unchanged:** 3 stale persisted-FAIL rows +
     `code/shell-coverage` P2 counter-drift → **P127 Slot-1** (this is the systemic
     mechanism behind P124's OP-7 near-miss — an owner decision among fix-drift /
     accept-FAIL / decouple-via-WARN is still owed); `verdict.py --phase` bare-session
     false-RED trap → **P126 fix-twice** (note owed to `quality/PROTOCOL.md`);
     **GTH-V15-89** (roadmap-strip machine gate, now ALSO must cross-check block-3
     ordering against STATE.md's phase count under the three-block design) → **P128**;
     LOW: CI-runner-variance note, STATE.md `total_plans`/`completed_plans` staleness —
     fold when convenient.

## 6. Precise next steps (successor runbook)

1. **Re-verify ground truth yourself first** — do not trust this snapshot past your own
   check: `git log --oneline -5`, `git status --porcelain`, `git rev-list --left-right
   --count origin/main...HEAD` (expect `0 1`), `gh run view 29671820890` (or the newest
   run on `1b660472` if a newer one exists). Read `.planning/STATE.md`'s Current-Position
   prose fresh (do not trust its frontmatter percent alone — it is optimistic, §1).
2. **AWAIT L0 GREEN on `1b660472` / run `29671820890`.** Do not push, do not open any new
   tree-writer dispatch, over an in-flight or red main. If L0 has not yet relayed a
   green/red verdict, this successor's correct posture is to STOP and wait for L0 to
   SendMessage a resume (per the Liveness doctrine — noting the §2/§5 caveat that this may
   not function reliably at the C2 tier; if a resume doesn't arrive, escalate to L0
   directly rather than self-watching).
3. **On green — P125 close, in this order:**
   a. Run `python3 quality/runners/run.py --cadence post-push --persist` — confirm the P0
      `code/ci-green-on-main` probe passes against main's NEWEST run (`1b660472`'s), while
      `cb4c2b3d` stays unpushed (it does not touch code, so it cannot regress this probe,
      but do not push it yet — see step 5).
   b. Dispatch an **independent, fresh** `gsd-verifier` (no session context beyond its
      charter) to grade P125's 3 success criteria against reality, minting
      `quality/reports/verdicts/p125/VERDICT.md`. **Verifier brief (from the P125
      executor, embed verbatim):**
      - **SC1/DRAIN-02** — documented mandatory mirror-refresh pre-step
        `scripts/refresh-tokenworld-mirror.sh` for the `pre-release-real-backend` cadence.
      - **SC2/DRAIN-12** — the milestone-close vision-litmus fixture self-heals BOTH
        backend drift AND GitHub-mirror drift by reconciling through the reposix bus
        remote before the marker push.
      - **SC3/DRAIN-12** — the git-remote-reposix helper's `git pull --rebase` teaching
        string is corrected for the mirror-drift case, AND the v0.14.0 attach-tree
        recovery blockquote in `docs/guides/troubleshooting.md` is made remote-explicit.
      - **Caveat — do not just wave the standing `code/shell-coverage` P2 FAIL through as
        pre-existing drift (D-124-W1a-1).** P125 plans 125-02/125-03 ADDED new shell
        scripts (`quality/gates/agent-ux/lib/litmus-self-heal.sh`, `litmus-flow.sh`) — the
        verifier must INDEPENDENTLY re-run shell-coverage and confirm it is not a NEW gap
        introduced by these additions, not just cite the pre-existing-baseline finding.
      - **Caveat — plan 125-02 AC#3's git-rm assertion.** Grade it using `grep -qF` (fixed
        string), not a bare `$`-anchored pattern — this environment's `ugrep` treats `$` as
        BRE-fragile in that context and can false-negative a genuinely-passing assertion.
      - **RED verdict → reopen P125**, loop back before advancing to P126. Do not proceed
        on a self-graded PASS — the verifier must be a fresh dispatch, per HCI (§11/§12
        `ORCHESTRATION.md`).
4. **Roadmap correction — resolve the open content question BEFORE the next push.**
   Confirm with L0/owner: should `docs/roadmap.md`'s "Landed recently" block list all 11
   closed phases (P114–P124, per L0's turn-7 spec as relayed into this charter) or the
   recent ~5 (P120–P124, the content `cb4c2b3d` actually shipped)? Correct in a
   sole-writer window if the answer is "all 11." Everything else in `cb4c2b3d` is already
   confirmed correct and does not need rework: phase numbers as sequence markers are now
   allowed (the HARD CONSTRAINT reversal landed in `.planning/CLAUDE.md`); dates only on
   the Landed side; P125 correctly stays "In flight now" (not yet verifier-GREEN) even
   though STATE.md's counter already reads 12/15; the block-3 content is binding-free
   (verified — no `doc-alignment.json` row cites its moving lines); the DP-4 `[SELF]`
   CONSULT-DECISIONS entry is landed.
5. **Push at the next CLEAN, CI-SAFE boundary** (after step 3's green verdict AND step 4's
   roadmap correction, if any, land): `git fetch origin main && git rebase origin/main &&
   git push origin main`, carrying {`cb4c2b3d` + any roadmap correction + the new
   `p125/VERDICT.md` commit + this handover's commit}. **Return the NEW CI run id to L0
   and STOP — do not self-watch** (same Liveness-doctrine discipline as step 2).
6. **Continue P126 → P127 (OP-8 Slot 1) → P128 (OP-9 Slot 2 + milestone close)** in order,
   one fresh C1 `phase-coordinator` per phase with an explicit model override, full GSD
   arc each (plan → execute → code-review → phase-close push → post-push cadence →
   independent gsd-verifier → committed `p12X/VERDICT.md` in the SAME close, not deferred
   — the OP-7 lesson). Hand each C1 the §5 raise-list items that match its phase (items
   1/3/4/GTH-V15-89 → P128; item 5 → P126/P127; item 6's fix-twice → P126; item 6's
   Slot-1 → P127) so it opens already primed.
7. **HELD / ESCALATE-FIRST — never self-authorize, carry forward:** E1 launch-animation
   publish (GTH-V15-37, owner approval still PENDING); the 91-file file-size waiver
   umbrella (expires 2026-08-08, owner decision among fix-drift/accept/shard still owed);
   L1198 `.env` credential-hydration security sign-off (deferred to P128 milestone-close);
   hero-number doc-alignment waivers (expire 2026-08-15); any release/tag
   (`v*`/crates.io); any real-backend mutation beyond the 3 sanctioned targets
   (Confluence TokenWorld / GitHub `reubenjohn/reposix` issues / JIRA `TEST`); the
   milestone ARCHIVE itself (gated on OP-9 RETROSPECTIVE distillation + the non-skippable
   9th `pre-release-real-backend` probe + report-to-L0-before-archive).
8. **Relieve yourself (the C2) past ~100k tokens of your OWN context** (hard stop ~150k,
   absolute not %) at the next PHASE boundary: dispatch `relief-handover-writer` to write
   `RELIEF-HANDOVER-C2-wave-2.md` (same convention, fresh file — do not re-edit this one
   past ~20-25KB), report the SHA to L0, stop. Report to L0 ONLY at: your own relief, an
   owner-decision escalation, milestone-close-ready, a 2-3-phase checkpoint, and each
   push→CI-in-flight handoff.

---

**Pointer note added to `C2-MILESTONE-HANDOVER.md` in this same commit:** that file's
content is current only through `c267f0e8` (P124/P125 boundary); read this file for
everything P125-onward.
