# SESSION-HANDOVER.md — v0.15.0 Floor: P125 CLOSING (verdict pending), 12/15
(80% — STATE counter optimistic, NOT verifier-confirmed), next close-out = P125 verdict
then P126 — 2026-07-19

**VERIFY LIVE BEFORE ACTING — every number below was LIVE-VERIFIED by workhorse seat
#64 (this writer) immediately before this write, but concurrent pushes drift state.
Re-run the ground-truth block in §1 yourself before doing anything else.**

Written by **workhorse seat #64** (L0 ROUTER), relieving to successor **seat #65**
(fresh L0 ROUTER — `.planning/ORCHESTRATION.md` § "L0 is a ROUTER"). This file
**REPLACES** the prior `#63→#64` handover in place (last reachable at commit
`3bb3d39a`) — that handover's runbook (dispatch C2 for P124 close, watch P124, hold the
E1 owner-gate) is fully executed and DONE; do not re-run it. Milestone **v0.15.0
"Floor"**. Router ROUTES ONLY — delegate reads through a reader-digester, own the
CI-watch loop yourself (§1 liveness doctrine below), cap subagent reports at ≤400 words.

**Read order:** this file → §1 ground truth (verify live) → §2 milestone/phase state →
§3 owner directive status (this rotation's headline) → §4 critical infra finding (file
for owner ruling) → §5 mid-execution decisions + noticed-not-filed → §6 RAISE-LIST +
HOLDS → Runbook (start at step 1).

## 1. Ground truth (git/CI) — verified live, re-verify before acting

**Re-verify block (run this yourself first):**
```
git fetch origin main
git rev-parse HEAD origin/main && git status --porcelain
gh run list --branch main --workflow ci.yml --limit 3 --json databaseId,status,conclusion,headSha
```

**Live-verified by #64 at 2026-07-19T04:02Z:**

- `origin/main` = `HEAD` = `6472f020` (`6472f0204da6fb3c8a30a099cd2213984bfeb8ad`), tree
  clean (`git status --porcelain` empty), 0 unpushed. Pushed this rotation: `cb4c2b3d`
  (roadmap three-block view + fix-twice doctrine supersede, owner quick) then `6472f020`
  (prior-C2 relief handover, `RELIEF-HANDOVER-C2-wave-1.md`).
- `git log --oneline -10` (newest first): `6472f020` docs(v0.15.0): C2 relief handover @
  P125 close-drive boundary / `cb4c2b3d` docs(roadmap): sequenced three-block view
  (Landed/In-flight/Up-next) + fix-twice no-phase-numbers supersede [owner quick] /
  `1b660472` docs(125): close-bookkeeping — STATE/ROADMAP 12/15 (80%), DRAIN-02+DRAIN-12
  complete, roadmap-strip refresh + 2 SURPRISES-INTAKE rows / `8604f087` docs(125-03):
  complete mirror-refresh pre-step plan (SC1/DRAIN-02 + PART 2 SC3/DRAIN-12) /
  `0808d48f` docs(125): make attach-tree recovery remote-explicit in v0.14.0 blockquote
  (SC3/DRAIN-12) / `cdaee30a` docs(125-03): document refresh-tokenworld-mirror pre-step
  / `28a8646b` docs(planning): refresh manager handover — rotation #13→#14 / `d219ef51`
  docs(125-02): SUMMARY for litmus self-heal / `35c41ac1` docs(125-02): document
  DRAIN-02 run-twice + backend-drift manual verification / `3864e1a6` feat(125-02):
  litmus self-heals backend drift + mirror drift before marker edit.
- **CI (`gh run list --branch main --workflow ci.yml --limit 5`, re-checked at
  04:02Z):**

  | run id | headSha | status | conclusion |
  |---|---|---|---|
  | `29672608656` | `6472f020` (current HEAD) | **`in_progress`** (NOT queued anymore — advanced since handoff instructions were drafted) | — |
  | `29671820890` | `1b66047` | completed | **success** (after ONE re-run of the pre-pr timeout flake) |
  | `29671165231` | `28a8646b` | completed | cancelled (superseded by the next push, expected/benign) |
  | `29667972559` | `c267f0e8` | completed | success |
  | `29667079687` | `d3d8052f` | completed | success (this is the run that flaked once and was rerun) |

  **IN-FLIGHT CI #65 MUST WATCH FIRST: run `29672608656` on `6472f020` (CI / ci.yml) —
  was `in_progress` as of 04:02Z, not yet concluded.** Do not assume green; watch it to
  a definitive conclusion.
- **Pre-pr timeout flake — TWICE today already** (`d3d8052f`, `1b66047`): the `quality
  gates (pre-pr)` job hits its 15-min timeout while EVERY other job is green. Recovery =
  `gh run rerun <id> --failed` then re-watch (both prior instances cleared on one
  re-run). If `29672608656` flakes the same way, re-run it. **If it times out a THIRD
  time today, it graduates from flake to confirmed CI-budget blocker → escalate to
  owner/fable** (the pre-pr wrapper likely needs its timeout raised or its runtime cut;
  ties to the RAISE-LIST runtime-creep item in §6).
- **L0-owns-watch, and watch for the true conclusion, not just exit code:** background
  `gh run watch <id> --exit-status` AND separately verify via `gh run view <id> --json
  conclusion` — exit 0 from `gh run watch` can mask a `cancelled` run; the pre-pr wrapper
  masked exactly this pattern once already this milestone. Never relay "green" upward
  without reading the actual `conclusion` field.

## 2. Milestone/phase state

- `STATE.md` frontmatter reads `completed_phases: 12` / `percent: 80`, but **P125 is
  CLOSING, NOT CLOSED** — `last_activity` says so honestly: "P125 CLOSING (Real-backend
  cadence & mirror-drift resilience, v0.15.0 DRAIN-02/DRAIN-12) — phase-close
  bookkeeping + push lane... Verifier NOT yet dispatched (runs after this phase-close
  push per cadence — CLOSING, not verdict-GREEN)."
- **P125 code: executed + pushed (3 plans across 2 waves, TDD RED→GREEN), all 3 SCs
  delivered, CI green on `1b66047` (after the pre-pr re-run).** `OPEN OP-7 GAP:` no
  independent gsd-verifier VERDICT exists yet — confirmed live, `quality/reports/
  verdicts/p125/` **does not exist** (`ls quality/reports/verdicts/` shows dirs through
  `p124` only, plus milestone/session-end/pre-push/post-push/weekly/post-release/`all`
  dirs — no `p125`). This is the exact P124 gap recurring one phase later.
- **P125 is NOT truly closed until:**
  (a) a fresh independent gsd-verifier grades the 3 SCs GREEN against reality →
      `VERDICT.md` at `quality/reports/verdicts/p125/`;
  (b) main's newest `ci.yml` run is green (see §1 — currently in-flight, unresolved);
  (c) post-push cadence `python3 quality/runners/run.py --cadence post-push --persist`
      (the `code/ci-green-on-main` P0 probe) passes.
  Verdict RED → revert `STATE.md` from 12 back to 11, reopen P125.
- **P125's 3 SCs, for the verifier to grade against reality:**
  - **SC1 (DRAIN-02):** a mandatory mirror-refresh pre-step
    (`scripts/refresh-tokenworld-mirror.sh`) is documented for the
    `pre-release-real-backend` cadence, so a second-run vision-litmus does not
    false-negative on its own prior push re-staling the GitHub mirror.
  - **SC2 (DRAIN-12):** the milestone-close vision-litmus fixture self-heals for BOTH
    backend drift (trashed protected pages) AND GitHub mirror drift, reconciling the
    mirror to backend-current through the reposix bus remote before the marker push.
  - **SC3 (DRAIN-12):** the helper's `git pull --rebase` teaching string is corrected
    for the mirror-drift case, and the v0.14.0 attach-tree recovery blockquote in
    `docs/guides/troubleshooting.md` is made remote-explicit.

## 3. Owner directive status (this rotation's headline — 3 owner messages, all addressed)

- **Sequenced three-block roadmap.md — LANDED + PUSHED (`cb4c2b3d`).** Owner reshaped
  the thin "Progress right now" strip into three blocks: `Landed recently` (last ~5
  closed = P120–P124, each with close date + one-line plain-language outcome) /
  `In flight now` (P125) / `Up next, in order` (P126–P128). Phase numbers are now
  ALLOWED as sequence markers (this REVERSES the old "no phase numbers" hard
  constraint); dates appear ONLY on the `Landed recently` side; the binding-free HARD
  constraint (no `doc-alignment.json` row may cite any of the three blocks' moving
  lines) is unchanged. The fix-twice doctrine supersede is committed in
  `.planning/CLAUDE.md` § "Phase-close refreshes the public roadmap strip" (verified
  live — the section now documents the three-block shape, the phase numbers reversal,
  the dates-only-on-Landed rule, and files `GTH-V15-89` as the deferred machine gate).
  **The content is CORRECT — do NOT "correct" `Landed recently` to P114–P124; that was
  seat #64's own error in a mid-incident message this rotation. P120–P124 (last ~5)
  matches the owner's actual spec.**
  - **MINOR OPEN GAP:** `Up next, in order` currently lists only P126–P128; the owner
    also asked for "the follow-on MILESTONE arc(s)" beyond milestone close. `.planning/
    CLAUDE.md` already documents this requirement ("`Up next, in order` continues past
    the current milestone's remaining phases into the follow-on milestone arc(s),
    sourced from `.planning/PROJECT.md` / `.planning/STATE.md`") but the actual
    roadmap.md content has not yet been extended with that arc — add it, sourced from
    `.planning/PROJECT.md`/`STATE.md`, folding into P125-close bookkeeping or P126.
    Non-blocking, but open.
  - Owner reads roadmap.md locally; it is now also on `origin/main`, and the push
    touched `docs/**` so the Docs deploy workflow will pick up the three-block view (see
    §3's docs.yml note below for a KNOWN deploy-gap risk on that path).
- **Two filed follow-ons (owner de-urgentized both — route, don't drop):**
  - **(A) `docs.yml` deploy-gap BUG:** make the deploy decision diff the candidate sha
    against the LAST SUCCESSFULLY DEPLOYED sha (not just the triggering push range), or
    an equivalent fallback. Evidence: the strip push `d3d8052f`'s Docs run `29667633692`
    concluded `skipped` (stranded — the intended content never deployed off that push);
    the owner's manual dispatch `29670460876` = success and recovered it. Fix-twice into
    doctrine when this lands.
  - **(B) `docs/development/roadmap.md` is a LYING stale duplicate** — claims v0.11.0
    active, is nav-excluded but still deployed and reachable. Delete it and redirect to
    `roadmap.md`.
  - Route (A)+(B) into P126 or a `/gsd-quick`; docs-build + banned-words gates apply to
    whichever lands them (see root CLAUDE.md "Fix-twice (P117 W3)" — both catalogs, not
    just one).

## 4. CRITICAL INFRA FINDING — file for owner ruling

**SendMessage is DISABLED at the C2 (phase-coordinator) tier** — observed directly this
rotation as "not enabled in this context": a C2 CANNOT SendMessage/halt/resume its own
background children. Consequence: `ORCHESTRATION.md` §3's "background the CI watch, L0
relays the run id up and SendMessages the coordinator to resume" pattern (line ~150,
"...the parent relays the run id up to L0 (which holds the durable CI watch) and
SendMessages the coordinator to resume the phase close on green") **does NOT work when
the coordinator being resumed is itself a C2** — a C2 cannot be the target of a resume
by SendMessage from within its own backgrounded-child relationship, and it cannot issue
SendMessage to its own children either, in this session's tooling context. This was the
proximate cause of an earlier single-writer discipline lapse (a C2 could not halt a
concurrently-running child it had backgrounded).

**Working mitigation (confirmed):** L0→C2 SendMessage DOES work — the old C2 received
every L0-issued directive without issue. The failure is specifically C2→child (and
child→C2 resume-via-id), not L0→C2.

**Standing-doctrine consequence for #65:** coordinators at the C2 tier and below must
serialize strictly and drive every phase close via **FRESH verifier→executor LEAVES**
(the P122-blessed deterministic pattern already documented at `.planning/
ORCHESTRATION.md` §11, "to drive a phase close, dispatch the verifier→executor LEAVES
directly... NEVER `fork` a coordinator to resume/close it") — never rely on
backgrounding a child and resuming it later via SendMessage at the C2 tier.

**Owner question to surface, do not self-resolve:** is this a *permanent* tier
limitation on SendMessage (in which case `ORCHESTRATION.md` §3/§11 need a doctrine
caveat added permanently), or a session/config gap specific to this rotation's tooling
context? Any successor C2 charter dispatched by #65 MUST embed this caveat verbatim
until the owner rules.

## 5. Mid-execution decisions + noticed-not-filed

- The prior milestone C2 (session id starting `a75107fb4dd1bd176`) RELIEVED ITSELF at
  152k+ tokens after self-catching a provenance hallucination (it had fabricated
  attributing an "items A–E" correction to L0 when no such L0 message existed). Its
  handover is at `.planning/milestones/v0.15.0-phases/RELIEF-HANDOVER-C2-wave-1.md`,
  committed at `6472f020` (confirmed live: file exists, 25010 bytes, dated Jul 18
  20:50). Treat this as the authoritative reading for the C2 lane before dispatching
  the next C2 — do not re-derive P125 state from scratch.
- **DP-4 [SELF] decision** by seat #64 (recorded in `.planning/CONSULT-DECISIONS.md`
  under "2026-07-18 [SELF] roadmap three-block reshape interleaved at the P125 wave
  boundary" — confirmed present, tail of file): the roadmap quick was interleaved at
  the P125 wave boundary (NOT "before P125" — P125 was already in flight when this
  landed; NOT "P125's close lane" — the owner explicitly excluded P125's own close
  bookkeeping from this quick). Nothing deleted or deferred; fully reversible,
  local-only until the parent coordinator pushed it. The decision ledger entry is
  present and complete — no successor action needed here beyond awareness.
- The P125 code push (`1b660472`) advanced `STATE.md`'s counter to 12/80% BEFORE any
  verifier ran — the OP-7 anti-pattern recurring. The fix-twice doctrine now encodes
  this explicitly in `.planning/CLAUDE.md`: "a phase mid-close with its verifier not yet
  dispatched stays `In flight now` [on the roadmap strip] even if `STATE.md`'s phase
  counter already optimistically advanced." Good — the doctrine catches the roadmap-strip
  side of this; it does NOT yet stop `STATE.md`'s own counter from advancing early. That
  residual (STATE.md counter vs. verdict-gating) is not filed as its own item; #65's
  successor C2 should consider whether it needs a dedicated GOOD-TO-HAVES row so
  `STATE.md`'s `completed_phases`/`percent` fields themselves gate on verdict-GREEN, not
  just the roadmap strip.

## 6. RAISE-LIST + HOLDS + escalations (carry forward)

**RAISE-LIST → routed:**
- pre-pr CI runtime-creep (now 2 timeouts today — trending real, not noise) →
  P126/P127 + local pre-push (`L1129`-tagged) 113s drift item.
- 3 stale persisted-FAILs + shell-coverage P2 counter-drift (confirmed present in
  `SURPRISES-INTAKE.md`: "shell-coverage FAIL forces `--persist` downgrade-REFUSAL"
  entry, 2026-07-18, severity MEDIUM) → P127 Slot-1 (CLOSE-INTEGRITY).
- `verdict.py --phase` bare-session false-RED trap (confirmed present in
  `SURPRISES-INTAKE.md`, 2026-07-18, severity MEDIUM) → P126 fix-twice.
- `GTH-V15-89` roadmap-strip machine gate — confirmed present in `GOOD-TO-HAVES.md`
  ("structure-dimension machine gate that cross-checks the strip against STATE.md") —
  now additionally cross-checks block-3 (`Up next, in order`) sequencing per the
  three-block reshape → P128.
- P124 `DRAIN-13/14/22/23/24` unmarked in `REQUIREMENTS.md` (bookkeeping lying) → P128.
  Not independently re-verified this rotation; carried forward from the prior handover
  as-is — #65 should spot-check this claim against `REQUIREMENTS.md` before treating it
  as settled.
- tokenworld-mirror doc-truth item — owner-gated, no further action without an owner
  signal.

**HOLDS/escalations — NEVER self-authorize, only route/surface:**
- **E1** launch-animation publish (`GTH-V15-37`) — owner-PENDING; do not publish.
- Global `gsd-sdk` `state.advance-plan` corruption bug — upstream issue; hand-advance
  `STATE.md` via `gsd-sdk query state.load` read-path only, never trust
  `state.advance-plan` to write correctly.
- File-size waiver umbrella expires **2026-08-08**.
- `L1198` `.env` credential sign-off → routed to P128.
- Hero-number doc-alignment waivers expire **2026-08-15**.
- Any release action (tag `v*`, crates.io publish) is outward-facing → escalate to
  owner, never self-authorize.
- Milestone archive is gated on the OP-9 retrospective distillation AND the 9th
  `pre-release-real-backend` probe — do not archive v0.15.0 without both.

## Runbook (seat #65 — numbered, start at step 1)

1. **Ground-truth re-verify** using the exact block in §1. Confirm `origin/main` =
   `6472f020` (or a fast-forward ahead of it) and check the live status of CI run
   `29672608656` — it was `in_progress` (not yet concluded) as of this handover's
   write time; do not assume it finished green.
2. **Watch `29672608656` to a definitive conclusion (L0-owns-watch, per §1's liveness
   doctrine).** Read the true `conclusion` field, not just an exit code. If the
   pre-pr-gates job times out (matching the pattern in §1) → `gh run rerun 29672608656
   --failed` then re-watch. If it times out a THIRD consecutive time today → stop and
   escalate to owner/fable per §1 — do not keep silently re-running past 3.
3. **Rotate a FRESH `opus` milestone `phase-coordinator` C2**, pointed first at
   `.planning/milestones/v0.15.0-phases/RELIEF-HANDOVER-C2-wave-1.md` (route the ~25KB
   read through a reader-digester, not a raw context dump). Its charter must include,
   verbatim:
   - the standard ownership charter block (root `CLAUDE.md` § "Ownership charter for
     dispatched subagents");
   - the §4 SendMessage-disabled-at-C2-tier caveat from this handover, in full;
   - the L0-owns-watch liveness doctrine from `.planning/ORCHESTRATION.md` §3.
   On CI green from step 2, the C2's ordered work is:
   (a) run the post-push cadence (`python3 quality/runners/run.py --cadence post-push
       --persist`);
   (b) dispatch an INDEPENDENT (fresh, not self-graded) gsd-verifier against the 3 SCs
       in §2 → write `VERDICT.md` at `quality/reports/verdicts/p125/`;
   (c) close P125 ONLY on verdict-green AND newest-ci-green (RED on either → revert
       `STATE.md`'s counter from 12 back to 11 and reopen P125 — do not paper over);
   (d) add the follow-on-milestone arc to roadmap.md's `Up next, in order` block (§3's
       minor open gap), sourced from `.planning/PROJECT.md`/`STATE.md`;
   (e) then proceed P126 → P127 → P128 → milestone-close, carrying forward every item in
       §6's RAISE-LIST as it reaches the phase it was routed to.
   First C1 under this C2 = open P126.
4. **Own the CI-watch loop** for every C1/C2 push #65 routes for the remainder of this
   rotation (background the watch, read the true `conclusion`, signal the coordinator on
   green via SendMessage — confirmed L0→C2 direction works even though C2→child does
   not).
5. **Surface §4's infra finding and §6's escalations to the owner/manager** at the next
   natural check-in; never self-resolve an E1/E2/E3/E4-class item.
6. **REPLACE this handover file in place** (do not append) at your own relief or pause,
   following the same `.planning/ORCHESTRATION-REFERENCE.md` § "Handover file template
   (§3 detail)" shape; re-verify every claim live before writing it down, the same way
   this handover's §1 was re-verified against `gh run view` moments before commit.
