# RELIEF-HANDOVER-C2-wave-3.md — v0.15.0 "Floor" C2 relief @ Cycle-1-(e)-landed / Cycle-2-not-started boundary, 2026-07-19

Written by the outgoing v0.15.0 C2 milestone coordinator-of-coordinators, relieving at a
**clean boundary**: Cycle 1 (bundled `/gsd-quick` fixing the pre-pr CI timeout) is landed
and pushed, its CI run is IN-FLIGHT (L0's watch, not this coordinator's), and Cycle 2 /
P126 have NOT been dispatched — no tree-writer is mid-flight. Predecessor chain, newest
first: `RELIEF-HANDOVER-C2-wave-2.md` (P125-closed / P126-not-started boundary) →
`RELIEF-HANDOVER-C2-wave-1.md` (P125 close-drive history) → `C2-MILESTONE-HANDOVER.md`
(background doctrine only, stale since `c267f0e8`). Each wave file supersedes its
predecessor for state; read older waves only for narrative you need, not current ground
truth — **this file is self-contained for resuming work.**

**Successor's required first reads, in order:** this file in full →
`.planning/ORCHESTRATION.md` §3 + §11 (relief/liveness/SendMessage-tier doctrine) →
`.planning/STATE.md` Current-Position prose → `quality/reports/verdicts/p125/VERDICT.md`
(what GREEN covered) → `.planning/CONSULT-DECISIONS.md` 2026-07-18 SendMessage entry
(ratified, do not re-file) → `RELIEF-HANDOVER-C2-wave-2.md` (P125-close history,
optional/background).

**Do-not-touch guardrails:** do NOT self-watch CI run `29678587237` (or its
release-plz/CodeQL siblings) — that is L0's watch; do NOT open Cycle 2 or P126 until L0
has relayed green on this run; do NOT re-diagnose the pre-pr timeout as "contention" —
that narrative is SUPERSEDED (§1); do NOT re-file the SendMessage-tier finding (it is
RATIFIED standing doctrine, ledger closed); do NOT self-authorize anything in §6's
HELD/ESCALATE list.

---

## 1. Ground truth (git)

Verified live this rotation via `git log`, `git status`, `gh run list` — not carried from
a stale snapshot.

- **Local HEAD = `c09f1d72`.** `git status --porcelain` — clean. `main`, up to date with
  `origin/main` (`0 0` ahead/behind immediately before this handover's own commit). This
  handover's commit advances HEAD one further and needs its own push (§6 step 1).
- **CI on `c09f1d72` is IN-FLIGHT, not yet confirmed green.** `gh run list --branch main`:
  `CI` run **`29678587237`** and `release-plz` run `29678587245` both `in_progress`
  (started `2026-07-19T07:43:43Z`, ~3m17s elapsed of the new 28-min cap); a `Push on
  main` → `CodeQL` run (`29678587015`) already `success`. **Live push→CI-in-flight
  boundary — L0's job to watch.** Do not treat `c09f1d72` as confirmed-green until L0
  relays it.
- **CI on the prior commit `592ae4c0` IS confirmed green** — `CI` run `29677846998`
  success (5m29s), `release-plz` run `29677846962` success (8m5s), `Docs` success. Solid
  ground.
- **Commit chain since the wave-2 boundary (`4cc9836a`), newest first:**
  - `c09f1d72` — **Cycle 1 (e), this rotation's headline commit.** Fixed the pre-pr CI
    timeout: `.github/workflows/ci.yml` `quality-gates-pre-pr` `timeout-minutes` 15→28;
    added explicit `timeout-minutes` to 3 previously-unbounded sibling jobs (`test`=6,
    `coverage`=6, `shell-coverage`=3, each ~2x observed green-run duration); replaced the
    stale "warm finishes inside 10" comment with the real forensic driver. Filed
    **GTH-V15-93** (`good-to-haves/part-10.md:59`) for the REAL fix (rust cache hit-rate)
    — the timeout raise is an explicitly-labeled band-aid, not a cure.
  - `592ae4c0` — doctrine-attribution fix: the SendMessage C2-tier ratification is a
    **MANAGER decide-and-disclose** ruling (owner veto window open), not owner-authored —
    corrected in `ORCHESTRATION.md` §3/§11 + `CONSULT-DECISIONS.md`.
  - `a19eb4cb` — quick-task self-check SUMMARY append (260718-x7j).
  - `495b8357` — SendMessage C2-tier limitation ratified as standing doctrine (substantive
    doctrine commit; `592ae4c0` only fixed its attribution).
  - `91d819e6` — relief-handover amendment: pre-pr CI timeout escalated from "watch item"
    to "confirmed budget blocker" (2x `cancelled`, cleared on 3rd rerun) — the finding
    `c09f1d72` fixed.
  - `394ebc3a` — GTH-V15-92 intake (milestone-close session-retrospective ceremony step).
  - `19a5f9f4`, `d2d1786b`, `4cc9836a` — prior rotation's relief + RAISE#1 eager-fix,
    already covered in `RELIEF-HANDOVER-C2-wave-2.md`.
- **CORRECTED root cause for the pre-pr CI timeout — supersedes the "queue contention"
  narrative in `SESSION-HANDOVER.md`/`91d819e6`.** L0-relayed correction, re-encoded here
  — do not revert to "contention." Forensics (from `c09f1d72`'s commit body, sourced from
  the `91d819e6` trace): of 4 recent `cancelled` `ci.yml` runs, **3/4 were genuine COLD
  rust-cache timeouts** — `quality-gates-pre-pr`'s `cargo build --workspace --bins` step
  hit the 15-min wall at 15m51s–20m00s while siblings finished in minutes; GitHub reports
  a `timeout-minutes` kill as `conclusion: cancelled` with **no** `timed_out` flag, which
  is exactly why it masqueraded as a concurrency-cancel event. Only **1/4** was a benign
  `cancel-in-progress` auto-cancel. Warm-vs-cold proof: `394ebc3a`'s 3rd attempt = 3m39s
  (warm) vs. attempts 1 & 2 on the SAME commit = both killed at 20:00 (cold).
- **L0 approvals live this rotation (fix-first authority, active):** (1) pivot from
  "raise the timeout as a stopgap" to "diagnose+fix the real cold-cache cause, timeout
  raise as an explicitly-labeled band-aid"; (2) sequencing below — run Cycle 2
  (hermetic-test fix + scaffolding propagation) as ONE bundled `/gsd-quick` BEFORE
  opening P126, not folded into it.
- **Cycle-1-(e)-executor noticing, carried forward:** pre-push hook took 119s — near the
  documented ~90-120s WARN ceiling (pre-existing, watch-item, not a regression from this
  diff); `code/shell-coverage` validate-only FAIL at 61s/exit 0 — pre-existing
  counter-drift (§4), unrelated to this diff.

## 2. Wave/cycle state

| Phase / Cycle | Plans/SCs | State | Commits / verdict |
|---|---|---|---|
| P114–P124 | — | DONE, CLOSED GREEN | unchanged, see wave-2 + `C2-MILESTONE-HANDOVER.md` |
| **P125 Real-backend cadence & mirror-drift resilience** | 3/3 SC | **DONE, CLOSED GREEN** | verdict `55d66378`; close `21b89fe3`; RAISE#1 `4cc9836a` |
| **Cycle 1 (e) — pre-pr CI timeout fix** | n/a (`/gsd-quick`-shaped) | **DONE, LANDED, CI IN-FLIGHT** | `c09f1d72`; run `29678587237` awaiting L0 green |
| **Cycle 2 (d)+(f) — hermetic-test fix + scaffolding propagation** | 2 tasks, bundled | **NOT STARTED — next** | — |
| P126 Docs-alignment tooling polish (DRAIN-15..21) | 0/TBD (4-lane slicing proposed) | **NOT STARTED — after Cycle 2** | — |
| P127 Surprises absorption (OP-8 Slot 1) | 0/TBD | NOT STARTED | — |
| P128 Good-to-haves polish + milestone close (OP-9 Slot 2) | 0/TBD | NOT STARTED | — |

No named incident this rotation — Cycle 1 (e) was a clean root-cause diagnosis → targeted
fix → push, no reopen, no confabulation, no orphaned build. The only correction of note is
upstream narrative drift (contention → cold-cache), captured in §1 so it does not recur.

## 3. Binding constraints (unchanged from wave-2 — embed verbatim in every dispatch)

- **ONE cargo invocation machine-wide, FOREGROUND-only** (never `run_in_background`;
  orphans the build, evades `cargo-mutex.sh`, OOM risk). Prefer `-p <crate>`.
- **Leaf test setup runs in a `/tmp` clone, `cd`-ing in the SAME Bash invocation — NEVER
  the shared repo.** Mechanically enforced (`leaf-isolation-guard.sh` + pre-commit
  backstop + binary-side `reposix init` refusal RPX-0406).
- **Uncommitted = didn't happen.** Commit before ending any turn. No `--no-verify`, ever.
  One tree-writer at a time.
- **Push cadence:** `git fetch origin && git rebase origin/main`, then `git push origin
  main` **BEFORE** the verifier-subagent dispatch, THEN `python3 quality/runners/run.py
  --cadence post-push --persist` — P0 `code/ci-green-on-main` must show main's NEWEST
  `ci.yml` run = success. **Never open the next phase over a red main.**
- **Tainted-by-default / `REPOSIX_ALLOWED_ORIGINS` egress allowlist.** Sim is the default
  backend everywhere.
- **Model tiering:** every C1 gets an EXPLICIT `model` override — opus
  security/genuinely-complex, sonnet default, haiku mechanical. Never fable at a leaf.
- **Commit-trailer format:** `Co-Authored-By: Claude Opus 4.8 (1M context)
  <noreply@anthropic.com>` on every non-trivial commit.
- **Phase-close refreshes `docs/roadmap.md`'s three-block strip** (Landed recently / In
  flight now / Up next, in order) in the SAME close-bookkeeping commit; phase numbers OK
  in all three blocks, **dates ONLY on "Landed recently."** Binding-free constraint HARD —
  never let a `doc-alignment.json` row cite the moving lines (P117-W3 `STALE_DOCS_DRIFT`
  cascade is the cautionary precedent).
- **Phase-close = independent fresh-verifier catalog-row PASS**, committed to
  `quality/reports/verdicts/p<n>/VERDICT.md`, in the SAME close — never defer the verdict
  (the OP-7 lesson that bit P124).

### 3a. SIX MANDATORY INJECTIONS — embed VERBATIM in every C1/leaf charter

**(a) OWNERSHIP CHARTER** (root `CLAUDE.md` § Ownership charter, OD-3, unchanged):
> 1. **Acceptance criteria are the floor, not the ceiling** — done means "I'd defend this
>    in review as excellent," not "plan executed."
> 2. **Noticing is a deliverable** — every report names what it noticed near its work
>    (lying doc claims, tests that don't assert what their names promise, error messages
>    that don't teach recovery, dead code, stale comments, missing edge cases). An empty
>    noticing section from code-touching work is itself a red flag.
> 3. **Eager-fix or file, never silently skip** — `<1h` + no new dependency → fix in
>    place; else → the active milestone's
>    `.planning/milestones/<active>-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md` with
>    severity + sketch (OP-8).
> 4. **Verify against reality** — run the thing, render the page, hit the backend; a claim
>    without an artifact is not done (OP-1).
> 5. **North star — Rust-compiler-grade UX** — end-user experience is the standing north
>    star all tooling serves. Every user-facing error must (1) teach the fix, (2) suggest
>    the alternative, (3) give a copy-paste recovery command. UX polish is scheduled as
>    first-class lanes, never leftovers. Would a skeptical dev hitting this surface for
>    the first time come away impressed?

**(b) SENDMESSAGE TIER LIMITATION — RATIFIED STANDING DOCTRINE** (`ORCHESTRATION.md`
§3/§11 + `CONSULT-DECISIONS.md` 2026-07-18 entry, commits `495b8357`/`592ae4c0`, quick
`260718-x7j` — **DO NOT re-file, re-diagnose, or re-discover; this is closed doctrine**):
> `SendMessage` is a tool-grant limitation of the `phase-coordinator` registry entry — it
> is NOT granted at the phase-coordinator (C2) tier or below. A C2 CANNOT SendMessage /
> halt / resume its own background children, and a child CANNOT resume-by-id back to its
> parent C2 — the failure is specifically **C2→child and child→C2**. **L0→C2 SendMessage
> DOES work** (confirmed repeatedly — an L0 top-level session resumes a C2 by session
> id), which is exactly why the durable CI watch and resume-on-green sit at L0, not the
> coordinator. Because a C2 cannot background-and-resume a child, coordinators at the C2
> tier and below MUST **serialize strictly** and drive every phase close via **FRESH
> verifier→executor LEAVES** (the P122-blessed deterministic pattern — dispatch leaves
> directly; never `fork` a coordinator to resume/close it, never background-and-resume a
> child at the C2 tier). Ratified under MANAGER decide-and-disclose authority
> (2026-07-18); owner veto window still open but no veto received — treat as binding.

**(c) L0-OWNS-CI-WATCH (Liveness doctrine, `ORCHESTRATION.md` §3, L0 ruling
2026-07-17)**:
> Background-task re-invocation is reliable ONLY at L0 — a coordinator's OWN backgrounded
> `gh run watch` (or any self-owned background-bash watcher) does NOT reliably re-invoke
> it; it goes dormant and stalls the phase close. A coordinator must therefore NEVER
> background its own CI watch and end its turn assuming it will wake. Everything BEFORE
> the push (plan → execute → code-review) runs straight through; the push→CI-in-flight
> boundary is the ONE stop-and-return point: STOP and RETURN to the dispatching parent a
> short status — pushed SHA + in-flight `ci.yml` run id(s) + "awaiting CI green to run
> post-push cadence + close." L0 holds the durable watch and SendMessages the coordinator
> to resume on green. A `cancelled` run that clears on rerun with NO code change is
> usually benign concurrency-cancel — **but §1's correction proved a `cancelled` run can
> ALSO be a masked cold-cache timeout kill (no `timed_out` flag either way).** Do not
> blind-rerun past ~2-3 attempts on faith; SAME job/test failing with SAME signature
> repeatedly is real — stop and check actual job-step timing before assuming contention.

**(d) HERMETIC-TEST directive** — see §5 Cycle 2 (d) for the concrete charter; standing
principle: any test reaching a live network resource (crates.io, GitHub API, etc.) inside
a CI-executed suite MUST pass deterministically OFFLINE (mocked), verified with network
denied — a flake that "usually passes" is not hermetic.

**(e) PRE-PR CI-TIMEOUT status** — FIX LANDED this rotation (`c09f1d72`, band-aid: raised
timeout + explicit sibling caps). Residual: **GTH-V15-93** (the REAL fix — rust-cache
hit-rate) is filed, not yet scheduled into a phase. Do not re-diagnose the root cause; §1
has the corrected forensics.

**(f) SCAFFOLDING PROPAGATION directive** — see §5 Cycle 2 (f) for the concrete charter;
standing principle: a ratified doctrine caveat (like (b) above) must propagate into the
auto-generating scaffolding (`.claude/skills/coordinator-dispatch/`,
`.claude/agents/phase-coordinator.md`) so future auto-generated charters inherit it
without a human/agent manually re-pasting it every rotation.

## 4. Litmus / gate / REOPEN state

- **CI run `29678587237` (`CI`) + `29678587245` (`release-plz`) on `c09f1d72`** —
  IN-FLIGHT at last read (~3m17s elapsed of a 28-min cap), no conclusion yet. **L0 holds
  this watch.** No REOPEN — nothing failed; pending-verdict, not red. This handover's own
  commit triggers a further CI run on top of `c09f1d72` — expect the successor's first CI
  check to be against a SHA newer than `c09f1d72`.
- **P125 verdict: GREEN, committed, final.** `quality/reports/verdicts/p125/VERDICT.md`
  (`55d66378`) — no reopen pending.
- **Milestone archive is GATED on BOTH of the following** (unchanged — do not archive
  v0.15.0 without both):
  1. **OP-9 retrospective distillation** — a new `.planning/RETROSPECTIVE.md` section
     written FROM intakes + run-findings BEFORE archive (ratification subagent grades RED
     if missing).
  2. **The non-skippable 9th `pre-release-real-backend` probe**
     (`python3 quality/runners/run.py --cadence pre-release-real-backend`, exit 0), per
     `.planning/CLAUDE.md` § Milestone-close 9th probe — vision litmus against the
     sanctioned real backend (TokenWorld), catalog row
     `agent-ux/milestone-close-vision-litmus-real-backend` (`blast_radius: P0`, never
     waived).
- **Open-waiver expiry clocks (unchanged, re-carry):**
  - `structure/file-size-limits` OVER-BUDGET `--warn-only` waiver — **expires
    2026-08-08T00:00:00Z**. `.planning/STATE.md` = **31846 bytes** (~1.6x the 20k soft
    ceiling), `surprises-intake/part-07.md` = **30153 bytes** (~1.5x) — both re-measured
    this rotation, unchanged from wave-2, routed to **P127** (GTH-V15-90). This handover
    file itself targets ~20-25KB, same discipline (`wave-2` was 21514 bytes).
  - Hero-number doc-alignment waivers (8 rows) — expire 2026-08-15, not re-verified fresh
    this rotation.
- **Standing `code/shell-coverage` P2 counter drift (34-vs-27, non-blocking, exit 0)** —
  unresolved, tracked at `surprises-intake/part-07.md:44` (+corroborating rows through
  line 177). Routed to P127. Cycle-1-(e)'s executor independently re-observed the SAME
  symptom (61s validate-only FAIL, exit 0) — corroborates present-and-stable, not
  worsening.
- **GTH-V15-93 filed** (`good-to-haves/part-10.md:59`) — rust-cache hit-rate real-fix,
  not yet scheduled into a phase; candidate for P127 or P128 triage.

## 5. Mid-execution decisions not yet formalized + "noticed, not yet filed"

- **Cycle 2 — bundled `/gsd-quick`, NOT yet dispatched. Charter for the successor to hand
  off verbatim:**
  - **(d) HERMETIC-TEST FIX.** `quality/runners/test_freshness_synth.py` has a stale-P2
    flake (surfaced in the PR#77 family) — its crates.io network probes are live, not
    mocked; the flake manifests as a stale-P2 assertion plus a "verifier not found at
    None" leakage message. **Locate-or-file:** this handover's own search found **no
    exact canonical tracking row** — closest adjacent items are **GTH-V15-55**
    (`good-to-haves/part-06.md:26`, a DIFFERENT live-network smell: the
    `cdn.simpleicons.org` embed, moot for the mp4 baseline) and part-06's "None-leakage"
    symptom language — **neither is the exact match.** The executor should do one more
    targeted search (`grep -rn "test_freshness_synth\|stale-P2\|verifier not found at
    None" .planning/milestones/v0.15.0-phases/`) before assuming absence, then file fresh
    if genuinely missing. **ACCEPTANCE FLOOR (non-negotiable):** the test passes
    DETERMINISTICALLY OFFLINE, verified by actually running with network denied (e.g.
    `firejail --net=none`/unshare, not just "should work offline" by inspection).
    Fix-twice: tag the hermeticity dimension in the relevant `quality/catalogs/` entry AND
    update the CLAUDE.md documenting this test (likely `quality/CLAUDE.md` or a
    `quality/runners/`-scoped doc) with the network-mock convention.
  - **(f) SENDMESSAGE-CAVEAT SCAFFOLDING PROPAGATION.** Embed §3a(b)'s caveat text into
    `.claude/skills/coordinator-dispatch/` and `.claude/agents/phase-coordinator.md`
    (both confirmed present this rotation) so auto-generated C1/C2 charters inherit it
    without manual re-pasting each rotation. Low-effort (existing, editable files), high-
    leverage (closes the "re-discovered 3 times before ratification" failure mode).
  - **Bundling rationale (L0-approved):** both are small (Python test fix + skill/agent
    markdown edits), share a low-red-ambiguity risk profile, don't need a dedicated
    multi-plan phase — ONE `/gsd-quick`, not folded into P126. **Stop at its own
    push→CI boundary and report up to L0** exactly as Cycle 1 (e) did.
- **P126 lane slicing (this rotation's proposal, L0-approved framing) — hand verbatim to
  the P126 C1 charter:**
  - **Lane A** = DRAIN-15 + DRAIN-19 (skill UX: `doc-clarity-review` canary-probe
    hard-fail/subscription-caveat; `plan-refresh <doc>` cold-invocation under-report
    note) — ~0.75h.
  - **Lane B** = DRAIN-17 + DRAIN-18 (pre-push docs-alignment block message names the
    real blocking row-STATE(s), not just the ratio, in `walk.sh`; doc-alignment grader
    compute-vs-assert reliability hardening in
    `.claude/skills/reposix-quality-doc-alignment/prompts/grader.md`) — ~3h.
  - **Lane C** = DRAIN-20 + DRAIN-21 (doc-alignment `status` gets a `waived_active`
    counter — **CARGO CYCLE**, lands in `crates/reposix-quality/src/commands/
    doc_alignment.rs`, respect the ONE-cargo-machine-wide rule; DRAIN-21 audits the 16
    pre-existing "cites out-of-eligible-file" coverage warnings, decides allowlist-expand
    vs. re-cite) — ~2-3h.
  - **Lane D** = DRAIN-16 (expand "MCP" acronym on first use in `README.md`) — ~0.25h.
  - Exact DRAIN defs verbatim: `.planning/REQUIREMENTS.md:254-272` (confirmed this
    rotation via direct line read); traceability rows at `:335-341` all read "Phase 126 |
    Pending."
- **P126 RAISE-LIST to fold into the C1 charter (do not re-diagnose, route as named):**
  1. **`verdict.py --phase` bare-session false-RED trap** — fix-twice note owed to
     `quality/PROTOCOL.md` (carried unactioned across TWO prior rotations — P126 should
     actually land it, not re-carry it a third time).
  2. **`docs.yml` deploy-gap = UNVERIFIED, verify-or-drop.** Carried across two prior
     rotations with an explicit caveat that no committed artifact substantiates it. **Do
     not treat as settled either way** — spend ≤15min trying to find a committed
     artifact; if none exists, drop the claim rather than re-carry a fourth time.
  3. **Delete the stale LYING `docs/development/roadmap.md`** (~3992 bytes,
     version-pinned content claiming v0.11.0 is "active," last touched 2026-07-07,
     carries **5 live `doc-alignment.json` rows**) → redirect readers to `docs/
     roadmap.md` (actively-maintained capability page). **REBIND every doc-alignment row
     citing the deleted file in the SAME commit** — the P117-W3 `STALE_DOCS_DRIFT` lesson
     verbatim: a later, separate reword/delete re-drifts the binding even if an earlier
     rebind was correct.
  4. **`docs/roadmap.md:3` header reading-tension** — "grouped by capability, not by
     release number or date" reads in minor tension with the now dated/numbered
     three-block strip immediately below it (same shape as RAISE#1's `4cc9836a` fix —
     mirror that scoping if reworded). Route through the mandated `/doc-clarity-review`
     cold-reader pass BEFORE shipping the next user-facing roadmap edit.
- **P126 ESCALATION GOTCHA (do not route around):** if any P126 sub-task needs a
  top-level `/reposix-quality-refresh` invocation or a doc-alignment backfill run,
  **ESCALATE to L0** rather than attempting it inside `/gsd-execute-phase` — depth-2
  subagent fan-out is structurally unreachable from inside a phase-coordinator per
  `.planning/CLAUDE.md` § Subagent-dispatch specifics.
- **P127 (Slot 1, drains SURPRISES-INTAKE)** — items confirmed present this rotation,
  unchanged from wave-2, re-verify sizes fresh before dispatch: shell-coverage 34-vs-27
  counter drift (`surprises-intake/part-07.md:44-177`); file-size splits (`STATE.md`
  31846B/1.6x, `part-07.md` 30153B/1.5x, waiver expires 2026-08-08); dead `PROTECTED_IDS`
  var at `scripts/refresh-tokenworld-mirror.sh:66`; **GTH-V15-91**
  (`good-to-haves/part-10.md:43`) — litmus self-heal has zero self-contained tests, a
  reconcile-ordering regression would escape CI entirely today. **New this rotation:**
  **GTH-V15-93** (rust cache hit-rate real-fix) is a P127-or-P128 triage candidate — not
  yet assigned, this rotation's P127 dispatch should decide.
- **P128 (Slot 2, drains GOOD-TO-HAVES + OP-9 retro + close)** — **`REQUIREMENTS.md`
  DRAIN-13/14/22/23/24 re-confirmed STILL unmarked this rotation** via direct line read:
  lines 179/191/201/247/252 all `[ ]`, traceability rows at `:333-334`/`:342-344` all
  "Phase 124 | Pending" — despite P124 CLOSED GREEN with its own committed VERDICT
  attesting delivery. **Cross-check against `p124/VERDICT.md`'s SC1-SC4 before flipping
  any checkbox** — confirm each item's delivery claim against the verdict's SC text
  first, don't blind-flip. **Trivial fold-forward, no dedicated CI cycle:** `STATE.md`'s
  `last_activity` doesn't mention the `260718-x7j` SendMessage-tier ratification — fold
  into P126's close-bookkeeping commit (it already touches STATE.md's cursor).
- **HOLDS carried unchanged (never self-authorize — route/surface only):** E1
  launch-animation publish (**GTH-V15-37**, owner-PENDING); any release action (tag `v*`,
  crates.io publish) — owner-gated; `.env` credential sign-off → **P128**; the
  `structure/file-size-limits` waiver umbrella expires 2026-08-08; hero-number
  doc-alignment waivers expire 2026-08-15.

## 6. Precise next steps (successor runbook)

1. **Push this handover commit; report the SHA + the still-open `c09f1d72` CI runs to
   L0.** `git push origin main`. Then `gh run list --branch main --limit 5` for the NEW
   run id(s) this push triggers, and report BOTH that id and the possibly-still-open
   `c09f1d72` runs (`29678587237` CI, `29678587245` release-plz) to L0. **Do not
   self-watch either — §3a(c) applies.**
2. **AWAIT L0 green on `c09f1d72`'s CI before dispatching Cycle 2.** If L0 hasn't relayed
   a verdict, STOP and wait for SendMessage-resume (L0→C2 works per §3a(b)); if nothing
   arrives in a reasonable window, escalate to L0 directly rather than self-watching. **If
   RED:** investigate the actual failure (job-step durations first, given §1's
   cold-cache-vs-contention lesson) BEFORE opening Cycle 2 — don't auto-rerun past ~2-3
   attempts on faith.
3. **Dispatch Cycle 2 as ONE bundled `/gsd-quick`** covering §5's (d) hermetic-test fix
   and (f) scaffolding propagation. Embed §3a's six mandatory injections verbatim. Stop
   at its own push→CI-in-flight boundary; report up to L0 exactly as Cycle 1 (e) did — do
   not fold it into P126.
4. **After Cycle 2 lands green, open P126** (Docs-alignment tooling polish, DRAIN-15..21)
   — dispatch ONE fresh C1 `phase-coordinator` (sonnet-default unless research surfaces
   security-judgment work; Lane C touches Rust so budget a cargo-mutex-respecting single
   invocation for it), full GSD arc (research → plan → plan-check → execute →
   code-review → phase-close push → post-push cadence → independent fresh `gsd-verifier`
   → committed `quality/reports/verdicts/p126/VERDICT.md` in the SAME close). Hand it:
   §5's 4-lane slicing, the 4-item RAISE-LIST, the escalation gotcha, and §3a's six
   injections verbatim.
5. **Continue P127 (OP-8 Slot 1) → P128 (OP-9 Slot 2 + milestone close)** in the same
   fresh-C1-per-phase pattern. Hand P127 the surprises items in §5 (including
   newly-surfaced GTH-V15-93 triage); hand P128 the DRAIN-13/14/22/23/24 cross-check
   task, the STATE.md fold-forward note, and the milestone-archive dual gate in §4 — P128
   does NOT close the milestone until BOTH the OP-9 retrospective AND the 9th
   `pre-release-real-backend` probe pass.
6. **HELD / ESCALATE-FIRST — never self-authorize, carry forward unchanged:** any
   release/tag (`v*`/crates.io); any real-backend mutation beyond the 3 sanctioned
   targets (Confluence TokenWorld / GitHub `reubenjohn/reposix` issues / JIRA `TEST`);
   the `structure/file-size-limits` waiver umbrella (expires 2026-08-08); hero-number
   doc-alignment waivers (expire 2026-08-15); the milestone ARCHIVE itself (gated per
   §4, plus report-to-L0-before-archive); E1 launch-animation publish.
7. **Watch your OWN token budget.** Relieve past ~100k tokens of your own context (hard
   stop ~150k, absolute not %) at the next CLEAN wave boundary — no in-flight
   tree-writer, no unresolved push→CI boundary you personally opened. Dispatch
   `relief-handover-writer` to write+commit `RELIEF-HANDOVER-C2-wave-4.md` (fresh file,
   do not re-edit this one past its own size). Report to L0 ONLY at: your own relief, an
   owner-decision escalation, milestone-close-ready, a 2-3-phase checkpoint, and each
   push→CI-in-flight handoff — matching wave-2's cadence, unchanged.

---

**No pointer update needed to `C2-MILESTONE-HANDOVER.md` this rotation** — it already
directs readers past `c267f0e8` into the `RELIEF-HANDOVER-C2-wave-N.md` chain; this file
continues that chain, no separate edit required.
