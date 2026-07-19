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
§3 owner/manager status (this rotation's headline — the recurring/ESCALATED CI flake +
the manager directive) → §4 infra findings (file for owner ruling, do NOT self-resolve)
→ §5 mid-execution decisions + noticed-not-filed → §6 RAISE-LIST + HOLDS → Runbook
(start at step 1).

## 1. Ground truth (git/CI) — verified live, re-verify before acting

**Re-verify block (run this yourself first):**
```
git fetch origin main
git rev-parse HEAD origin/main && git status --porcelain
gh run list --branch main --workflow ci.yml --limit 4 --json databaseId,status,conclusion,headSha
```

**Live-verified by #65 at 2026-07-19T06:36Z (final, post-recovery):**

- `origin/main` = `HEAD` = `394ebc3a` (`394ebc3ab5e6528ddbddf80474b8b8b3cd2b3bf1`), tree
  clean (`git status --porcelain` empty), 0 unpushed.
- **This tip includes ONE commit NOT authored by this session:** `394ebc3a`
  (`docs(intake): GTH-V15-92 — milestone-close session-retrospective ceremony step (via
  /gsd-capture)`) was committed locally by the OWNER (author `Reuben John`, real commit,
  not a session artifact) DURING this rotation's push-retry window and rode along in the
  same successful `git push` as this handover's own commit (`19a5f9f4`). This is
  legitimate, expected, and not a corruption — just note it so #66 isn't confused about
  provenance if it inspects `git log`.
- `git log --oneline -8` (newest first): `394ebc3a` docs(intake): GTH-V15-92 —
  milestone-close session-retrospective ceremony step (owner, via `/gsd-capture`) /
  `19a5f9f4` docs(planning): #65→#66 relief handover (this commit) / `d2d1786b`
  docs(planning): #65 C2 relief — wave-1→wave-2 @ P125 closed GREEN / `4cc9836a`
  docs(roadmap): scope capability-map line (P125 raise #1 eager-fix) / `21b89fe3`
  docs(125): close GREEN — verifier VERDICT confirmed (12/15, 80%) / `55d66378`
  docs(125): verifier VERDICT — P125 GREEN / `043447c3` docs(planning): #64→#65 relief /
  `6472f020` docs(v0.15.0): C2 relief handover.
- **CI on the current HEAD is GREEN, but getting there took THREE reruns — this is a
  NEW, WORSE escalation of the known pre-pr timeout pattern, not a simple single-rerun
  flake. Full history for run `29675427742` (headSha `394ebc3a`):**

  | attempt | started | completed | conclusion | duration |
  |---|---|---|---|---|
  | 1 (original push) | 05:49:04Z | 06:09:04Z | **cancelled** (`quality gates (pre-pr)` job hit its internal budget) | ~20min |
  | 2 (`gh run rerun --failed`) | 06:09:59Z | 06:29:59Z | **cancelled again** (same job, same ~20min pattern) | ~20min |
  | 3 (`gh run rerun --failed`) | 06:31:39Z | 06:35:13Z | **success** | 3.5min |

  Note the successful attempt 3 took only 3.5 minutes — well inside the job's documented
  15-min budget (`.github/workflows/ci.yml:97`). This strongly suggests attempts 1–2 were
  **runner-queue contention**, not a genuine regression: at the time of attempts 1–2,
  `gh run list --limit 15` showed OTHER concurrent CI activity on the repo (branches
  `d7b09172`/`1aac21f6`, unrelated PRs/pushes, `action_required`/`success` conclusions
  around 05:18Z–05:57Z) that had fully drained by the time attempt 3 ran. **The diff in
  the pushed commit is docs-only markdown (`SESSION-HANDOVER.md` + the owner's
  `GOOD-TO-HAVES.md`/`part-10.md` intake row) — there is no plausible content-driven
  cause for a quality-gate hang.**
  - `gh run view <id> --json status,conclusion` reported `"conclusion":"cancelled"` both
    times (NOT `"failure"`) — confirms the documented gotcha that a GH Actions job
    exceeding its `timeout-minutes` reports as `cancelled`, and that `gh run watch
    --exit-status` can exit 0 even when the true conclusion is `cancelled`, not
    `success`. Both cancelled attempts were caught ONLY by reading the actual
    `conclusion` field, exactly as the liveness doctrine prescribes.
  - **This now graduates past "known flake, one rerun clears it."** The original
    #64→#65 handover documented TWO single-rerun-recovered pre-pr timeouts earlier
    today and an explicit rule: "if it times out a THIRD time today, it graduates from
    flake to confirmed CI-budget blocker → escalate to owner/fable." This rotation's
    two consecutive cancellations on the SAME commit are that 3rd+4th occurrence. **See
    §3 for the escalation this triggers — do not treat this as routine going forward.**
- **CI table, most recent 3 `ci.yml` runs on main (re-checked 06:36Z) — all GREEN at
  HEAD:**

  | run id | headSha | conclusion |
  |---|---|---|
  | `29675427742` (3rd attempt) | `394ebc3a` (current HEAD) | **success** |
  | `29674505740` | `d2d1786b` | success |
  | `29674209349` | `4cc9836a` | success (after the manager's separate freshness-flake rerun — §3) |

  **No in-flight CI as of this write. Main is GREEN at HEAD, but only after the recovery
  above — #66 should read that recovery, not just the final green state.**
- **L0-owns-watch, and watch for the true conclusion, not just exit code:** background
  `gh run watch <id> --exit-status` AND separately verify via `gh run view <id> --json
  conclusion` — exit 0 from `gh run watch` can mask a `cancelled` run (empirically
  reconfirmed twice this rotation, see the table above). Seat #65 held this watch across
  all push→CI boundaries this rotation; every boundary eventually resolved GREEN, two of
  them (`4cc9836a`'s freshness flake, `394ebc3a`'s double pre-pr timeout) only after
  manual intervention.

## 2. Milestone/phase state

- `STATE.md` frontmatter reads `completed_phases: 12` / `percent: 80` — this is
  verifier-confirmed, not optimistic. `last_activity`: "P125 CLOSED GREEN ...
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

## 3. Owner/manager status (this rotation's headline — TWO co-equal escalations)

**(A) NEW — pre-pr CI timeout graduated to a confirmed CI-budget blocker this
rotation (see §1 for the full recovery trace).** The `quality gates (pre-pr)` job
(`.github/workflows/ci.yml:97`, `timeout-minutes: 15`) cancelled TWICE in a row on the
SAME commit (`394ebc3a`) before a 3rd rerun cleared it in 3.5 minutes — well inside
budget. The evidence points to transient GitHub Actions runner-queue contention from
OTHER concurrent repo CI activity (unrelated branches/PRs mid-flight at the same time),
not a regression caused by this rotation's docs-only diff. This is the 3rd+4th pre-pr
timeout TODAY (the #64→#65 handover already recorded two single-rerun-recovered
instances earlier), crossing the explicit doctrine threshold ("times out a THIRD time
today → escalate to owner/fable, don't keep silently re-running"). **#66 must surface
this to the owner/manager as a live escalation, not just carry it forward as a RAISE-LIST
line** — recommend investigating whether the `quality gates (pre-pr)` 15-min budget needs
headroom, or whether GitHub Actions concurrency/queueing needs a dedicated runner
group/priority for `ci.yml`.

**(B) The MANAGER is actively co-watching CI this rotation** — not a passive audience.
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
  noise-to-tolerate-forever. **This is a DIFFERENT root cause from (A) above** — do not
  conflate the two; (A) is a CI-infra/timeout-budget issue, (B) is a hermeticity/test-
  isolation issue in one specific Python test.
  - **Standing rule for #66 (until the fix lands):** on a fresh main push, if CI goes RED
    specifically matching this known flake signature (freshness-synth-only failure,
    everything else green, no code diff that plausibly explains it) → `gh run rerun
    <id> --failed` ONCE and re-watch. If the RERUN is ALSO red → treat as a REAL
    regression, stop, investigate — do not keep re-running past one retry. (§1's (A)
    finding shows this "one retry" heuristic can legitimately need a 2nd retry for
    UNRELATED runner-contention reasons — use judgment: if the SAME test/job keeps
    failing with the SAME signature, it's real; if a cancelled/timeout conclusion clears
    on a later attempt with no code change, it was contention.)
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
- Seat #65 (L0) owned the durable CI watch for every push→CI boundary this rotation. All
  resolved GREEN eventually; two required manual intervention beyond a passive watch
  (§3(A)'s pre-pr double-timeout, needing 2 reruns; §3(B)'s freshness-synth flake,
  needing 1 rerun by the manager). The liveness doctrine (C2 stops at the push→CI
  boundary; L0 holds the watch; L0→C2 SendMessage relays the green signal) worked
  end-to-end, but this rotation is evidence the "L0 watches, rerun-once-if-known-flake"
  heuristic needs judgment, not blind automation — §3(A)'s finding documents why.
- P125's RAISE #1 (verifier-noticed cold-reader tension) was eager-fixed same-rotation
  at `4cc9836a` — a clean example of OP-8's "<1h + no new dependency → fix in place"
  rule being followed correctly, not deferred or silently skipped.
- **Noticed, not independently pursued further:** while investigating §3(A), seat #65
  observed OTHER concurrent CI runs on unrelated branches this rotation
  (`d7b09172`/`1aac21f6`) surfacing `"conclusion":"action_required"` on both `CI` and
  `Security audit` workflows — this looks like a first-time-contributor/fork-PR approval
  gate, unrelated to main and out of scope for this handover, but flagging it in case
  it's an unexpected surface (e.g., a fork PR nobody has triaged) rather than expected
  repo policy. Not filed anywhere — #66 or the owner should eyeball it if it recurs.

## 6. RAISE-LIST + HOLDS (carry forward from the wave-2 C2 report — route, don't drop)

- **P126:** `verdict.py --phase` bare-session false-RED fix-twice (confirmed present in
  `SURPRISES-INTAKE.md` per prior rotation's live check — carry forward, not
  re-independently-verified this rotation); `docs.yml` deploy-gap item — **UNVERIFIED
  this rotation, no committed artifact found on a light search; #66 should verify-or-drop
  rather than treat it as settled fact**; stale `docs/development/roadmap.md` LYING
  duplicate — **VERIFIED historically** (5 doc-alignment bindings, stale since
  2026-07-07, claims v0.11.0 active while nav-excluded but still deployed/reachable) —
  delete it and redirect to `docs/roadmap.md`.
- **P126 (NEW, this rotation):** §3(A)'s pre-pr CI-timeout escalation — investigate
  whether `.github/workflows/ci.yml:97`'s 15-min budget needs headroom or whether runner
  concurrency needs dedicated capacity; this is now empirically a repeat CI-budget
  blocker, not routine noise.
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
   `394ebc3a` (or a fast-forward ahead of it) and that main's newest `ci.yml` run is
   GREEN (`success`, not merely `completed` — a `cancelled` run also reports
   `completed`, see §1's trace). As of this write there is no in-flight CI — a clean
   rotation boundary, reached only after the recovery documented in §1/§3(A).
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
       self-watching; INCLUDE §1's amended judgment note (a `cancelled` conclusion that
       clears on a later rerun with no code change is contention, not a real fail — but
       don't blindly rerun past ~2-3 attempts without escalating);
   (d) the **hermetic-fix directive** from §3(B): locate-or-file the
       `test_freshness_synth.py` hermeticity debt as a first-class near-term ACTIVE lane
       (P126 first-class scope, OR a `/gsd-quick` landed before P126 opens — coordinator's
       call). Acceptance floor: the test passes DETERMINISTICALLY OFFLINE (mock the
       crates.io probes, resolve the stale-P2 assertion, eliminate the None-verifier
       leakage), verified with network denied. Fix-twice: tag the hermeticity dimension +
       update the relevant `CLAUDE.md`;
   (e) the **pre-pr CI-timeout escalation** from §3(A)/§6 — route to P126 or surface to
       the owner/manager directly, per their preference;
   (f) the **SendMessage-formalization task**: file the `.planning/CONSULT-DECISIONS.md`
       ledger entry per §4.1 (2nd re-discovery this milestone, owner ruling still
       pending as of this handover).
   First C1 under this C2 = open P126.
3. **Own the CI-watch loop** for every push→CI boundary the C2 returns at — background
   `gh run watch <id> --exit-status`, separately confirm via `gh run view <id> --json
   conclusion` (never trust exit code alone — a cancelled run can exit 0, reconfirmed
   twice this rotation), watching main's NEWEST `ci.yml` run specifically. Apply §3(B)'s
   hermetic-flake rerun rule for that specific test's signature; apply §1/§3(A)'s
   contention judgment for pre-pr job cancellations — don't rerun indefinitely, but a
   2nd rerun clearing cleanly with no code change is legitimate evidence of contention,
   not something to treat as a regression.
4. **Surface §4's infra findings, §3(A)'s NEW pre-pr timeout escalation, and §6's HOLDS**
   to the owner/manager at natural check-ins; never self-resolve an E-class item. §4.1's
   owner question is already surfaced by seat #65 — chase a ruling, do not re-surface it
   as if new.
5. **REPLACE this handover file in place** (do not append) at your own relief or pause,
   following the same `.planning/ORCHESTRATION.md` §3 / `.planning/ORCHESTRATION-
   REFERENCE.md` § "Handover file template (§3 detail)" shape; re-verify every claim
   live before writing it down, the same way this handover's §1 was re-verified against
   `gh run list`/`git rev-parse` moments before commit — including watching any push
   through to a DEFINITIVE (not just "completed") conclusion before declaring done.
