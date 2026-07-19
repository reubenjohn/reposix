# SESSION-HANDOVER.md — v0.15.0 Floor: P125 CLOSED GREEN, 12/15
(80% — verifier-confirmed), next = P126 — 2026-07-19

**VERIFY LIVE BEFORE ACTING — every number below was LIVE-VERIFIED by workhorse seat
#65 (this writer) immediately before this write, but concurrent pushes drift state.
Re-run the ground-truth block in §1 yourself before doing anything else.**

Written by **workhorse seat #65** (L0 ROUTER), relieving to successor **seat #66**
(fresh L0 ROUTER — `.planning/ORCHESTRATION.md` § "L0 is a ROUTER"). This file
**REPLACES** the prior `#64→#65` handover in place (last reachable at commit
`043447c3`) — that handover's runbook (watch P125-close CI, rotate a fresh C2 off
`RELIEF-HANDOVER-C2-wave-1.md`, drive verdict→close→P126) is fully executed and DONE;
do not re-run it. Milestone **v0.15.0 "Floor"**. Router ROUTES ONLY — delegate reads
through a reader-digester, own the CI-watch loop yourself (§1 liveness doctrine below),
cap subagent reports at ≤400 words.

**Read order:** this file → §1 ground truth (verify live) → §2 milestone/phase state →
§3 owner/manager status (this rotation's headline — the recurring CI flake + the manager
directive) → §4 infra findings (file for owner ruling, do NOT self-resolve) → §5
mid-execution decisions + noticed-not-filed → §6 RAISE-LIST + HOLDS → Runbook (start at
step 1).

## 1. Ground truth (git/CI) — verified live, re-verify before acting

**Re-verify block (run this yourself first):**
```
git fetch origin main
git rev-parse HEAD origin/main && git status --porcelain
gh run list --branch main --workflow ci.yml --limit 4 --json databaseId,status,conclusion,headSha
```

**Live-verified by #65 at 2026-07-19T05:21Z:**

- `origin/main` = `HEAD` = `d2d1786b` (`d2d1786b9c92c3ff510e7340d4049dc7a61625d4`), tree
  clean (`git status --porcelain` empty), 0 unpushed.
- `git log --oneline -8` (newest first): `d2d1786b` docs(planning): #65 C2 relief —
  wave-1→wave-2 @ P125 closed GREEN / P126-not-started boundary / `4cc9836a`
  docs(roadmap): scope capability-map "no phase numbers/dates" line to the mermaid map
  (P125 raise #1 eager-fix) / `21b89fe3` docs(125): close GREEN — verifier VERDICT
  confirmed (12/15, 80%), roadmap strip P125→Landed, +3 verifier-NOTICED intake rows /
  `55d66378` docs(125): verifier VERDICT — P125 GREEN (SC1/SC2/SC3 verified against
  committed artifacts) / `043447c3` docs(planning): #64→#65 relief — P125 CLOSING
  (verdict pending) / `6472f020` docs(v0.15.0): C2 relief handover @ P125 close-drive
  boundary / `cb4c2b3d` docs(roadmap): sequenced three-block view + fix-twice
  no-phase-numbers supersede / `1b660472` docs(125): close-bookkeeping — STATE/ROADMAP
  12/15 (80%), DRAIN-02+DRAIN-12 complete.
- **CI (`gh run list --branch main --workflow ci.yml --limit 4`, re-checked at
  05:21Z) — ALL FOUR most-recent pushes GREEN:**

  | run id | headSha | status | conclusion |
  |---|---|---|---|
  | `29674505740` | `d2d1786b` (current HEAD) | completed | **success** |
  | `29674209349` | `4cc9836a` | completed | **success** (after ONE `--failed` re-run of the known `test_freshness_synth.py` flake — see §3) |
  | `29673847760` | `21b89fe3` | completed | success |
  | `29672909419` | `043447c3` | completed | success |

  **No in-flight CI. No active children. Main is GREEN at HEAD.**
- **L0-owns-watch, and watch for the true conclusion, not just exit code:** background
  `gh run watch <id> --exit-status` AND separately verify via `gh run view <id> --json
  conclusion` — exit 0 from `gh run watch` can mask a `cancelled` run. Seat #65 held this
  watch across all four push→CI boundaries this rotation (`043447c3`, `21b89fe3`,
  `4cc9836a`, `d2d1786b`) — all four resolved GREEN, one (`4cc9836a`) via the manager's
  documented flake-rerun call (§3). The liveness loop (C2 stops at push→CI boundary, L0
  holds watch, L0→C2 SendMessage on green) worked end-to-end this rotation.

## 2. Milestone/phase state

- `STATE.md` frontmatter reads `completed_phases: 12` / `percent: 80` — **this is now
  verifier-confirmed, not optimistic.** `last_activity`: "P125 CLOSED GREEN ...
  gsd-verifier verdict GREEN on all 3 SCs (`quality/reports/verdicts/p125/VERDICT.md`,
  verdict commit `55d66378`) — the 12/15 count is now verifier-confirmed, not
  optimistic. 12/15 v0.15.0 'Floor' phases complete (P114–P125); next = P126."
- **P125 (Real-backend cadence & mirror-drift resilience, v0.15.0 DRAIN-02/DRAIN-12) is
  FULLY CLOSED.** Confirmed live: `quality/reports/verdicts/p125/VERDICT.md` exists
  (12551 bytes, committed `55d66378`), independent gsd-verifier graded GREEN on all 3
  SCs against committed artifacts (SC1 mirror-refresh pre-step doc/DRAIN-02, SC2 litmus
  self-heal for backend+mirror drift/DRAIN-12, SC3 helper teaching-string + v0.14.0
  blockquote remote-explicit reword/DRAIN-12). Close bookkeeping landed `21b89fe3`
  (STATE/ROADMAP advance to 12/15, roadmap-strip refresh, +3 verifier-NOTICED intake
  rows filed). **RAISE #1** from the verifier (a cold-reader tension in the roadmap's
  "no phase numbers/dates" capability-map line) was eager-fixed same-rotation at
  `4cc9836a` (`/gsd-quick`, <1h, no new dependency — correct per OP-8).
- **The follow-on-milestone roadmap arc is CONFIRMED COMPLETE** (was an open item in the
  #64→#65 handover; landed as part of `cb4c2b3d`, live-verified this rotation): `docs/
  roadmap.md`'s "Up next, in order" block now lists P126→P127→P128 followed by the full
  Arc D follow-on arc (v0.17 meta-milestone → v0.19 truth-purge/IA rebuild → v0.21
  benchmark-honesty → v0.23 journey slices → v0.25 launch kit), sourced correctly against
  `.planning/PROJECT.md` § Arc D. No further action needed here.
- **Next = P126** (Docs-alignment tooling polish, DRAIN-15..21). Remaining v0.15.0 arc:
  **P126 → P127 → P128 → milestone-close** (OP-8 +2-phase absorption: P127 drains
  `SURPRISES-INTAKE.md`, P128 drains `GOOD-TO-HAVES.md` + does OP-9 retrospective
  distillation + the 9th `pre-release-real-backend` probe before archive).

## 3. Owner/manager status (this rotation's headline)

- **The MANAGER is actively co-watching CI this rotation** — not a passive audience.
  When `4cc9836a`'s CI run (`29674209349`) went RED, the manager judged it against the
  known-flake pattern (identical CI GREEN on `21b89fe3` only ~15 minutes earlier with the
  only diff being roadmap wording — not a plausible real regression), issued `gh run
  rerun 29674209349 --failed`, and the rerun resolved GREEN. **Do NOT double-rerun
  `29674209349` — it is already resolved GREEN, this is historical record only.**
- **Recurring flake, now on its 3rd recurrence this milestone:**
  `test_freshness_synth.py`'s stale-P2 HERMETIC flake (PR#77 family — the test makes live
  `crates.io` network probes and leaks a "verifier not found at None" state on network
  variance). Manager DIRECTIVE (authoritative, verbatim intent, recurrence #3):
  **promote the hermetic-test debt into a near-term ACTIVE fix lane** — this has now
  cost three separate CI reruns across the milestone and is a debt item, not
  noise-to-tolerate-forever.
  - **Standing rule for #66 (until the fix lands):** on a fresh main push, if CI goes RED
    specifically matching this known flake signature (freshness-synth-only failure,
    everything else green, no code diff that plausibly explains it) → `gh run rerun
    <id> --failed` ONCE and re-watch. If the RERUN is ALSO red → treat as a REAL
    regression, stop, investigate — do not keep re-running past one retry.
  - Seat #65 could NOT find a canonical filed tracking row for this specific hermeticity
    debt during this rotation's search (checked `GOOD-TO-HAVES.md` +
    `good-to-haves/part-*.md` + `SURPRISES-INTAKE.md`); the closest adjacent rows are
    `GTH-V15-55` (a different cdn-smell item) and `good-to-haves/part-06.md` (an adjacent
    None-leakage symptom, not this exact test). **#66's C2 charter (Runbook step 2) MUST
    locate-or-file this as a first-class near-term lane** per the manager's directive —
    do not let it recur a 4th time unaddressed.

## 4. Infra findings (two, file for owner ruling — do NOT self-resolve)

1. **SendMessage is DISABLED at the C2 (phase-coordinator) tier and below** — a C2
   cannot SendMessage/halt/resume its own background children, and a child cannot
   resume-by-id back to its parent C2 either. **L0→C2 SendMessage DOES work** — confirmed
   directly this rotation: seat #65 (L0) resumed the wave-1 C2 by session id twice,
   successfully, across this rotation's push→CI boundaries. The failure is specifically
   C2→child and child→C2, not L0→C2.
   - **This is now the 2nd independent re-discovery** of the same finding (first surfaced
     in the #64→#65 handover's §4, re-confirmed live by #65 this rotation). It remains
     **UNFORMALIZED in durable doctrine** — grep-empty in `.planning/STATE.md` and
     `.planning/CONSULT-DECISIONS.md`; `.planning/ORCHESTRATION.md` mentions
     `SendMessage` generically (background-watch-and-resume pattern, §3/§11) but carries
     NO tier-limitation caveat.
   - **Open owner question, unresolved:** is this a PERMANENT tier limitation on
     SendMessage (in which case `ORCHESTRATION.md` §3/§11 need a permanent doctrine
     caveat) or a session/config gap specific to this rotation's tooling context? **Seat
     #65 surfaced this question to the manager this rotation; a ruling is still
     PENDING as of this handover.** #66 should chase the ruling at the next natural
     check-in, and — until ruled — file a `.planning/CONSULT-DECISIONS.md` ledger entry
     recording the finding + pending-ruling status, so a 3rd independent re-discovery
     doesn't happen on a future rotation.
2. **Consequence (working mitigation, apply until ruled):** coordinators at the C2 tier
   and below MUST serialize strictly and drive every phase close via **FRESH
   verifier→executor LEAVES** (the P122-blessed deterministic pattern at
   `.planning/ORCHESTRATION.md` §11 — "dispatch the verifier→executor LEAVES directly...
   NEVER `fork` a coordinator to resume/close it") — never fork-to-resume, never
   background-and-resume a child at the C2 tier. This caveat must be embedded VERBATIM in
   every C2/C1 charter #66 dispatches until the owner rules.

## 5. Mid-execution decisions + noticed-not-filed

- The wave-1 C2 (from the #64→#65 handover) relieved itself cleanly at ~100k own-context
  after judging the remaining 4-phase arc (P126→P127→P128→close) would breach the 150k
  hard stop mid-arc — correct, proactive relief per doctrine, not a forced/incident
  relief. Its handover is `.planning/milestones/v0.15.0-phases/
  RELIEF-HANDOVER-C2-wave-2.md`, committed `d2d1786b`, live-confirmed this rotation
  (21514 bytes, exists). **This is the authoritative C2-lane reading for the fresh C2 —
  route it through a reader-digester, do not raw-dump into the C2's context.**
- Seat #65 (L0) owned the durable CI watch for all four push→CI boundaries this rotation
  (`043447c3`, `21b89fe3`, `4cc9836a`, `d2d1786b`) — all four resolved GREEN (`4cc9836a`
  required the manager's flake-rerun call, documented in §3). The liveness doctrine
  (C2 stops at the push→CI boundary rather than self-watching; L0 holds the watch;
  L0→C2 SendMessage relays the green signal to resume) worked end-to-end this rotation
  with zero L0-side misses.
- P125's RAISE #1 (verifier-noticed cold-reader tension) was eager-fixed same-rotation
  at `4cc9836a` — a clean example of OP-8's "<1h + no new dependency → fix in place"
  rule being followed correctly, not deferred or silently skipped.

## 6. RAISE-LIST + HOLDS (carry forward from the wave-2 C2 report — route, don't drop)

- **P126:** `verdict.py --phase` bare-session false-RED fix-twice (confirmed present in
  `SURPRISES-INTAKE.md` per prior rotation's live check — carry forward, not
  re-independently-verified this rotation); `docs.yml` deploy-gap item — **UNVERIFIED
  this rotation, no committed artifact found on a light search; #66 should verify-or-drop
  rather than treat it as settled fact**; stale `docs/development/roadmap.md` LYING
  duplicate — **VERIFIED historically** (5 doc-alignment bindings, stale since
  2026-07-07, claims v0.11.0 active while nav-excluded but still deployed/reachable) —
  delete it and redirect to `docs/roadmap.md`.
- **P127 CLOSE-INTEGRITY (Slot 1, drains `SURPRISES-INTAKE.md`):** `code/shell-coverage`
  34-vs-27 counter-drift (tracked at `good-to-haves/part-07.md:44` per the wave-2 C2
  report — spot-check the exact line on pickup, content drifts as the file grows); split
  `good-to-haves/part-07.md` (referenced as GTH-V15-90 in the incoming facts — confirm
  the id on pickup, it was not independently re-derived this rotation) + the file-size
  waiver clock expiring **2026-08-08**; dead `PROTECTED_IDS` cleanup.
- **P128 (Slot 2, drains `GOOD-TO-HAVES.md` + OP-9 retrospective + milestone close):**
  `DRAIN-13/14/22/23/24` are **CONFIRMED still unmarked** in `.planning/REQUIREMENTS.md`
  (live-verified this rotation: lines 179/191/201/247/252 show `- [ ]` unchecked, and the
  tracking table at lines 333/334/342/343/344 shows all five as `Pending` despite P124
  having delivered them) — this is real bookkeeping debt, not carried-forward hearsay.
- **HOLDS (never self-authorize, only route/surface):**
  - **E1** launch-animation publish (`GTH-V15-37`) — owner-PENDING, do not publish.
  - Any release action (tag `v*`, crates.io publish) — outward-facing, owner-gated,
    never self-authorize.
  - Milestone archive is gated on BOTH the OP-9 retrospective distillation AND the 9th
    `pre-release-real-backend` probe passing — do not archive v0.15.0 without both.
  - `L1198` `.env` credential sign-off → routed to P128.
  - File-size waiver umbrella expires **2026-08-08**.
  - Hero-number doc-alignment waivers expire **2026-08-15**.

## Runbook (seat #66 — numbered, start at step 1)

1. **Ground-truth re-verify** using the exact block in §1. Confirm `origin/main` =
   `d2d1786b` (or a fast-forward ahead of it) and that main's newest `ci.yml` run is
   GREEN (`success`, not merely `completed`). As of this write there is no in-flight CI
   and no active children — a clean rotation boundary.
2. **Dispatch ONE fresh `opus` milestone `phase-coordinator` C2**, pointed at
   `.planning/milestones/v0.15.0-phases/RELIEF-HANDOVER-C2-wave-2.md` (route the ~21.5KB
   read through a reader-digester, not a raw context dump). Charter = P126 → P127 → P128
   → milestone-close. The charter MUST inject VERBATIM (none of these are already inside
   the wave-2 handover — they post-date it):
   (a) the standard ownership charter block (root `CLAUDE.md` § "Ownership charter for
       dispatched subagents");
   (b) §4's SendMessage-disabled-at-C2-tier caveat, IN FULL, plus §4.2's working
       mitigation (serialize strictly; fresh verifier→executor LEAVES only; never
       fork-to-resume);
   (c) the L0-owns-watch liveness doctrine (`.planning/ORCHESTRATION.md` §3) — the C2
       stops at every push→CI boundary and returns/SendMessages up rather than
       self-watching;
   (d) the **hermetic-fix directive** from §3: locate-or-file the
       `test_freshness_synth.py` hermeticity debt as a first-class near-term ACTIVE lane
       (P126 first-class scope, OR a `/gsd-quick` landed before P126 opens — coordinator's
       call). Acceptance floor: the test passes DETERMINISTICALLY OFFLINE (mock the
       crates.io probes, resolve the stale-P2 assertion, eliminate the None-verifier
       leakage), verified with network denied. Fix-twice: tag the hermeticity dimension +
       update the relevant `CLAUDE.md`;
   (e) the **SendMessage-formalization task**: file the `.planning/CONSULT-DECISIONS.md`
       ledger entry per §4.1 (2nd re-discovery this milestone, owner ruling still
       pending as of this handover).
   First C1 under this C2 = open P126.
3. **Own the CI-watch loop** for every push→CI boundary the C2 returns at — background
   `gh run watch <id> --exit-status`, separately confirm via `gh run view <id> --json
   conclusion` (never trust exit code alone — a cancelled run can exit 0), watching
   main's NEWEST `ci.yml` run specifically. Apply §3's hermetic-flake rerun rule: ONE
   `--failed` rerun on a matching-signature RED, escalate if the rerun is also RED.
4. **Surface §4's infra findings and §6's HOLDS** to the owner/manager at natural
   check-ins; never self-resolve an E-class item. §4.1's owner question is already
   surfaced by seat #65 — chase a ruling, do not re-surface it as if new.
5. **REPLACE this handover file in place** (do not append) at your own relief or pause,
   following the same `.planning/ORCHESTRATION.md` §3 / `.planning/ORCHESTRATION-
   REFERENCE.md` § "Handover file template (§3 detail)" shape; re-verify every claim
   live before writing it down, the same way this handover's §1 was re-verified against
   `gh run list`/`git rev-parse` moments before commit.
