# MANAGER-HANDOVER.md — outer-loop session manager (herdr) — live state only

For the incoming MANAGER session (the herdr outer loop in pane **w1:p7**), NOT the
reposix L0 orchestrator. The manager herds the workhorse in pane **w1:p5**; it never
does reposix work itself. Keep this file lean; git history is the archive.

## Role & standing owner instructions

- **Outer loop — polling model (owner directive 2026-07-15)**: watch w1:p5 per the
  `/herdr-manager` skill § "Watching a pane over time" — one-shot background poll
  that EXITS on the first event, **every wait capped ≤1h**, re-arm on every wake;
  never event-stream monitors or a long `herdr agent wait`. Concrete loop ships
  WITH the skill: `bash ~/.claude/skills/herdr-manager/scripts/manager-poll.sh
  w1:p5 3300 "<handled CI run ids>" /home/reuben/workspace/reposix` (no local
  copy — the skill's is canonical). On wake, inspect (`herdr agent explain/read`,
  ghost-text trap) before acting. Never `agent send` blind.
- **WAKE-SOURCE RULE (incident 2026-07-18 — all-day stall):** NEVER end a manager
  turn without a verified freshly-armed poll (the one-shot is CONSUMED by the wake
  that delivered it). Three liveness layers now stand: L1 = re-arm poll every wake;
  L2 = in-session CronCreate deadman tick (~2x/hr; SESSION-ONLY — every successor
  re-creates it, see first actions); L3 = external
  `~/.claude/skills/herdr-manager/scripts/liveness-watchdog.sh` (setsid, flock,
  crontab respawn `*/17min`, log `~/.local/state/herdr-watchdog/watchdog.log`) —
  nudges the MANAGER pane when either pane's screen is static ≥40min or a
  `[Pasted text]` block is stuck ≥15min, 45min cooldown. L2/L3 are insurance,
  not substitutes for L1.
- **Workhorse seat rotation — never bare `/clear` (proven 2026-07-15)**: `/clear`
  does NOT stop background shells/monitors; orphans survive (same PIDs) and their
  later events INJECT into the successor session, which cannot even enumerate them.
  Rotate via `bash ~/.claude/skills/herdr-manager/scripts/pane-clear.sh w1:p5 --yes`
  (kills leftovers by exact PID, re-verifies, then /clears; dry-run without `--yes`;
  read-only check: `scripts/pane-tasks.sh w1:p5`). Then verify gauge reset + charter
  submission per skill craft (§ Clearing a session).
- **Ownership mandate**: the manager OWNS everything end-to-end — maintainability,
  code/architectural elegance, end-user experience. Heavy delegation and context-lean
  constraints stand, with one exception: at rare boundaries (only after very
  significant milestones), run your own highly selective probes to ground
  understanding, complementing delegate reports.
- **Eyes-and-ears baseline (every wake)**: (1) `gh run list --branch main -L 3` — a
  red main is owner-visible health, never a "low-level concern"; dispatch a fix
  immediately. (2) origin/main sync + dirty-tree check. (3) Spot-verify one
  load-bearing claim from any wrap report before relaying it. Workhorse self-reports
  are verified, not trusted.
- **Context budget**: self under ~400k hard (soft ~350k; owner raise 2026-07-12) —
  refresh this file, commit+push, run § Rotation. Workhorse: relieve at **~18%
  gauge soft / ~22% hard** (Opus 1M seat; fresh-seat baseline is ~6% ≈ 60k system
  overhead — the old "100k/150k absolute" phrasing read as 10–15% gauge and caused
  premature 15-min legs #55/#59), then it REPLACES `.planning/SESSION-HANDOVER.md`,
  commits+pushes, ends turn; you `/clear` w1:p5 and launch its successor pointing
  at that file. Charter the gauge numbers, not raw token counts.
- **Real-backend mutations PRE-AUTHORIZED**: Confluence TokenWorld, GitHub
  reubenjohn/reposix issues, JIRA TEST. Credentials/spend beyond those still
  owner-gated.
- **NEVER gate work on weekly subscription resets (owner, 2026-07-15):** do not
  defer, hold, or schedule around reset timing or usage percentages — launch work
  when it is ready. If a cap-hit ever pauses a session, it wraps cleanly
  (commit+push+handover) and the successor resumes; that is hygiene, not a
  scheduling input and not an owner-notify event by itself.
- **STANDING AUTHORITY (owner, 2026-07-12): milestone release cuts are the manager's.**
  The manager makes and executes tag/release-cut calls end-to-end (tag push included)
  for milestone closes, without per-milestone re-approval — ALWAYS through the honest
  gate sequence (9th probe exit 0 or a recorded caveat call, aggregate verdict minted +
  ratified, tag script, never over a red main). Workhorse executes artifacts; manager
  verifies and pushes the tag.
- **Owner intent**: multi-day autonomous chug toward OD-4 launch-readiness (asciinema
  demo, honest headline numbers, install excellence, Show-HN kit); workhorse routes,
  doesn't work.
- **UX mandate**: end-user experience is the north star all tooling serves. The manager
  makes strong UX decisions on the owner's behalf — docs, error-messages-with-fix-hints,
  onboarding friction. Bar: Rust-compiler-grade UX (teach the fix, suggest the
  alternative, copy-paste recovery). UX polish is a first-class lane, never a leftover.
- **DELEGATION DEPTH (owner directive 2026-07-17):** the relay drifted to depth-0/1
  work with ~25–45 min legs (measured: #46–#52 median ~42m vs ~2h50m on the earlier
  Fable legs). Owner rule of thumb: **~1h+ of work per workhorse handover, with most
  work executed by agents nested TWO levels below the seat** — L0 as ROUTER →
  `phase-coordinator` C1 per phase/wave (explicit model tiers: opus complex / sonnet
  default / haiku mechanical) → gsd leaves. L0's own window is reserved for routing,
  gate checks, verification; all >100-line reads via `reader-digester`. Rationale
  (owner, verbatim intent): reduce low-level detail at BOTH tiers so the manager keeps
  mental capacity for meta-level judgment. MANAGER TIER: templated 3-line charters
  (handover carries content; charter = seat number + handover pointer + deltas), ONE
  spot-verify per leg, no CI babysitting a script already covers. Encode workhorse-side
  into ORCHESTRATION.md via tracked quick — routed to #54's charter; verify it landed.

## Rotation procedure (self-succession, w1:p7)

1. Refresh this file; commit+push (`docs(planning): refresh manager handover`).
2. `setsid nohup bash .planning/manager-rotate.sh w1:p7 >/tmp/manager-rotate.log 2>&1 &`
   (gitignored, local-only; recreate per contract below if absent), then END TURN
   immediately — the script waits for pane idle, sends `/clear`, verifies, sends the
   successor `/herdr-manager` prompt pointing here.
3. **Known tooling bugs**: `send-keys C-u` is unsupported (script's clear step must not
   rely on it — fix or drop before use); a long single-line `agent send` becomes a
   "[Pasted text]" block that Enter won't submit while background subagents hold input;
   rotation #6→#7 the successor prompt never submitted at all (pane showed only a stray
   "a" — likely the pasted-block/Enter race) — owner had to bootstrap manually; verify
   the send actually submitted (agent read: input emptied + turn started) before the
   outgoing manager's script exits. HARDENED 2026-07-15 (#7): the script now verifies
   gauge-reset after /clear (3 retries) and successor-turn-started after the prompt
   send (4 Enter retries, loud FAILED log line if unsubmitted).

## Live state (refreshed at rotation #14→#15, 2026-07-19 ~14:45 UTC)

- **SUCCESSOR #15 FIRST ACTIONS:** (1) Arm your poll IMMEDIATELY:
  `bash ~/.claude/skills/herdr-manager/scripts/manager-poll.sh w1:p5 1800 ""
  /home/reuben/workspace/reposix`. NOTE: `ignore_ids` (3rd arg) is
  **SPACE-separated** — comma-separated silently fails and the poll re-fires on
  handled red runs (#14 hit this twice). (2) Re-create the L2 in-session deadman
  tick (died with #14): cron `11,53 * * * *`, recurring, prompt = "WATCHDOG TICK:
  if your last turn did not end with a freshly armed unfired manager-poll, triage
  w1:p5 + origin/CI and re-arm; else one-line ack, stop." (3) Verify L3 alive:
  `pgrep -f liveness-watchdog.sh`. (4) **Seat #69 mid-leg**: P126 CLOSED GREEN at
  handoff (verdict `7c9cc153` = 126-01-VERIFICATION.md `verdict: GREEN`;
  close-bookkeeping `13dd4ffc`; roadmap three-block moved P126→Landed correctly);
  **CI in flight on 13dd4ffc at rotation time — verify it concluded green in your
  first cycle** (workhorse holds the watch; known flakes below). Next work = P127
  planning. Relief 18% soft / 22% hard gauge; rotation flow proven 5x this shift:
  TRUE-IDLE → verify wrap+handover PUSHED (not just committed — see #68 miss) →
  pane-clear.sh --yes → gauge-reset check → charter → verify CONSUMED → re-arm.
  (5) **Owner gates OPEN (non-blocking):** E1 animation publish (owner PENDING,
  never self-authorize); PR #74 v0.14.1 cut (recommended CUT, awaiting word);
  91-file size-waiver umbrella (exp 2026-08-08); gsd-sdk state.advance-plan
  upstream bug (held); L1198 .env sign-off. **SendMessage C2-tier ruling
  (MANAGER decide-and-disclose 2026-07-19, owner veto OPEN):** C2 tool-grant
  lacks SendMessage = standing limitation; fresh-leaves mitigation ratified
  (ledger `.planning/CONSULT-DECISIONS.md:198`, doctrine `495b8357`, attribution
  honesty-corrected `592ae4c0`); owner may instead grant the tool in the registry.

- **SHIFT #14 SUMMARY (2026-07-19 03:1x → ~14:45 UTC):** Seats #64→#69, 5 clean
  rotations. **P125 + P126 CLOSED GREEN, both verdict-artifact-spot-verified**
  (milestone 73%→87%, STATE 13/15). **Owner's roadmap top ask DELIVERED**:
  three-block reshape landed `cb4c2b3d` + doctrine supersede in
  `.planning/CLAUDE.md`; both closes since kept phases honestly in "In flight"
  until verifier-GREEN. **Pre-pr CI saga (the shift's spine):** "flake" →
  "contention" (timeout 15→28min + 3-job cap `c09f1d72`) → real root cause =
  **fd-inheritance deadlock**, fixed + permanent observability net `cef3a2ea`
  (pre-pr back to ~4-6min since). Then P126's own diff regressed
  real-git-push-e2e P0 (save_catalog persist-guard) → fixed `ba13553f`; badges
  P2 pair = external shields/codecov outage, cleared on rerun; hermetic
  freshness-synth fixed `f1959373` (residual: hermetic P2 CI-portability bug
  noticed in the P126 verdict — filed, non-gating). **Manager interventions that
  mattered:** nudged the stalled roadmap quick past a passed boundary; caught
  seat #66 tagging the SendMessage ruling [OWNER] (honesty rule) → corrected;
  caught seat #68 ending its turn with 2 UNPUSHED relief commits (Stop advisory
  ignored) → manager pushed them (pre-push hook ran 264s, WARN over budget —
  possible intake; `code/shell-coverage` P2 FAIL inside it delegated to #69 for
  pre-existing-vs-new check). Charter-queue pattern (send to a working seat,
  consumed at next boundary) worked 6/6 times. Weekly limit read 83% at ~04:15Z
  (resets Jul 23 2am PT) — noted only, never gated on.

(Older prior-live-state sections — rotations #13→#14 and earlier — dropped per
the keep-lean rule; git history is the archive.)
- **MANAGER FINDING (verified, then fixed by #48): retiring hero rows had NO
  successors on 3 surfaces.** Subagent-mapped + grep-verified: docs/index.md:17
  hero bullet, README.md:27, concepts:29-31 carried the new 94.3%/74.9% figures
  unbound. Now bound (see #48 commits above). Residual noticings filed:
  concepts four-axis coverage gap (#48), row-id prefix inconsistency
  (`README-md/` legacy vs `README/` new — cost a false-negative grep; routed to
  P126 via #49's charter).
- **OWNER INTERACTION (this shift):** answered 4 owner questions with verified
  facts — docs badge v0.13.1 sighting = browser sessionStorage cache (live site
  fresh-browser reads v0.14.0; `releases/latest` = v0.14.0); short workhorse
  sessions = relay doctrine; animation ETA ≈ Jul 18–20 via P117 (GTH-V15-37);
  roadmap = docs/roadmap.md refreshes at milestone close, phase order in
  `.planning/ROADMAP.md`. **Handed the owner the 11 confirm-retire commands**
  (from 115-UNWAIVE-PATH.md § FINAL, binaries verified present) + the
  commit/push landing step — the batch may land AT ANY MOMENT; every workhorse
  charter since carries the gate re-check + STATE.md-advance instruction.
- **GHOST-TEXT INCIDENT (craft, verified live):** w1:p5's box showed "done, ran
  the 11 confirm-retire commands and pushed — close P115" — reality check:
  origin unmoved AND catalog still 11/11 ⇒ ghost text, NOT owner input. Never
  close the human gate on box text; only a real catalog-count drop closes it.
- **OPEN GATES/THREADS:** (1) **Human gate OPEN** — 11/11 RETIRE_PROPOSED;
  owner has commands in hand. (2) **P116 rulings ENCODED** (`8212373`), owner
  veto window open; execution in flight (research+validation shipped, planning
  tail → execute next). (3) PR #74 release-plz v0.14.1: OWNER-GATED, recommended
  CUT, awaiting owner word. (4) GTH-V15-34 confirm-retire --batch mode filed.
  (5) Headline stands: **~94.3% fewer output tokens / ~74.9% cheaper per
  session vs official GitHub MCP** (6/6 real sessions, `4db6b64`), now
  catalog-bound on every hero surface.
- **Liveness incident (2026-07-16, 2nd dead-watcher):** #40 froze ~30min —
  child notification chain died silently (subagent+shell gone, parent never
  resumed, dirty tree). Caught by shortened 20-min poll; nudge recovered it.
  Charters now carry: bounded backstop ≤20min on EVERY child wait; shorten the
  manager poll to 1200s when a wave looks quiet-but-alive.
- **Seat-rotation craft (learned #35→#36, extended #39→#44):** do NOT
  pane-clear a workhorse whose wrap says "awaiting CI green" — its final turn
  hasn't ended; killing its gh-run-watch shell injects a failure event and
  costs an investigation detour. Wait for manager-poll TRUE-IDLE (idle + zero
  shells) first. A /clear sent mid-turn QUEUES and fires at turn end. Ghost
  text after `❯` (e.g. a lingering "/clear" or "continue") is an autocomplete
  hint — Enter does nothing; a real send overwrites it harmlessly. KNOWN GAPS:
  `pane-tasks.sh` missed a live background CI-poll shell once (#39 rotation) —
  treat "Clean" as advisory, prefer TRUE-IDLE; `manager-poll.sh` once reported
  HEARTBEAT despite origin moving (race near cap) — `wake-triage.sh` (added
  this shift, `.home` commit `ac88eba`) re-checks origin on every wake and is
  the standard triage entry. Also retained from #9: never load the claude-api
  skill at manager tier (~300k context); delegate API-mechanics to a subagent.
- **#32 outcome: P115 Wave 1 CLOSED** — T1 preflight 3/3 backends; T4 GA
  de-risked (Rovo + GitHub remote MCP both GA → CONDITIONAL GO); T2 caught a
  real finding: sim cold-init is environment-dependent (27ms legacy dev → 278ms
  CI), ruled CI `bench-latency-v09` canonical `[SELF]`, latency.md corrected
  with provenance, sim/bench expected_version PATCH defect filed. Honest CI
  figure is ~10× the legacy hero claim → T6 reframe ruled (above). Carried into
  T6: latency.md regeneration-clobber tension (local sim-only generator would
  overwrite CI-canonical sections); un-waive-path scripts absent. Prior-session
  watch items (pre-push timing, flaky CI job, plan-refresh walk-first, file-size
  waiver 2026-08-08) live in the session handover. Weekly subscription limit was
  75% at ~2:40pm PT 2026-07-15 (resets 2am PT) — cap-hit = forced pause =
  PUSH-NOTIFY owner.
- **#29 outcome (2026-07-15): Directive-2 CLOSED** — scratch-repo KEEP-policy
  landed in `docs/reference/testing-targets.md` (spot-verified: lines 173–193,
  never-delete / force-push-reset / unarchive-API) via gsd-quick 260715-h1d,
  plus eager-fix of a stale "Phase 36 cleanup automation" forward-reference.
  Relieved at the refresh-tail boundary rather than hand-binding 11 catalog rows
  fatigued or opening P115 over-budget (clean §3 relief). CI on main GREEN (run
  29443988376). Meta-lesson (fix-it-twice candidate): editing any docs/**/*.md
  that carries doc-alignment rows drifts them and the tail surfaces only at
  pre-push — grep `quality/catalogs/doc-alignment.json` for the doc BEFORE
  editing and budget the refresh; GOOD-TO-HAVES row proposed for the quick-task
  contract.

- **MANAGER #8 SESSION (2026-07-15): P114 CLOSED GREEN.** Both fixes shipped and
  real-backend SC1/SC2 acceptance GREEN on live Confluence (`dc26302`, CI 4/4;
  root cause: list path omitted body-format → fix `9908fcc`; FIX-02 reconcile-scope
  docs corrected) — closes the t4 Confluence oid-drift defect (`builder.rs:612`
  OidDrift, owner ruling `b773c04` fix-first). Workhorses #25 (planned) → #26
  (Wave 1 + liveness incident) → #27 (Wave 2 + close) each relieved cleanly;
  #27→#28 handover `29470e2`; #28 filed intakes (GTH-V15-21 file-size-waiver
  expiry 2026-08-08) but relieved at `26ca703` with P115 UNSTARTED (plan-phase.md
  ~32k context sink — meta-lesson minted); #29 carries P115 (BENCH-01, ≤50
  sessions, waiver deadline 2026-08-15) with the anti-sink charter. Owner-approved roadmap-diagram lane captured at
  `e039bb7` (docs/roadmap.md + PROJECT.md bidirectional SYNC-comment cross-links +
  link-resolution glob extension). **Liveness doctrine minted after a dead-watcher
  stall (~2h idle):** bound every wait on a dispatched child; health-check
  self-paused children ≤30min — now in workhorse charters. **herdr craft moved to
  the /herdr-manager skill** (owner ruling) with bundled verified scripts:
  manager-poll.sh (polling model, ≤1h cap, singular-shell ghost-idle fix),
  pane-tasks.sh, pane-clear.sh (`.home` commits `f77d69e`/`0e75b26`/`5f41602`).
  Survey prompts do NOT resolve on a bare digit (unlike permission menus) —
  unverified beyond one observation.

- **v0.15.0 FLOOR RE-ANCHOR COMPLETE (#24, 2026-07-15):** PROJECT.md/STATE.md
  re-anchored (`825c449`); REQUIREMENTS+ROADMAP minted (`bb12601`..`baa3583`, 15
  phases P114–P128, 41/41 REQ-IDs mapped). Schedule facts: **BENCH-01 at P115**
  inside the 2026-08-15 waiver deadline, ≤50-session ceiling in-phase; **P116 =
  decision-only phase** — ADR-01 (mirror-fanout packet) + FIX-03 (GTH-09 slug→id
  durable-create) packets produced at execution, routed to MANAGER for ruling
  (decide-and-disclose), no pre-ruling implementation.

- **Standing owner-directed facts (2026-07-14/15):** ADR-class decisions =
  manager decide-and-disclose (ledger + owner veto window); PUSH-NOTIFY the owner
  at any owner-blocking moment or planned pause — silence must always mean
  "working"; owner-only = interactive sudo, new creds/scopes/spend, outward
  publishing. Benchmark-spend ceiling: up to 50 benchmark sessions on the existing
  subscription, no new API dollars, escalate only past 50. Owner intent: 24/7 chug
  through v0.15→v0.25; milestone cuts flow under standing release authority;
  2026-08-15 waiver expiry is a hard scheduling deadline inside v0.15.

- **TokenWorld fixture doctrine:** 2 PROTECTED ids never deleted (`7766017` +
  `7798785`) + 1 SACRIFICIAL EDITABLE (`2818063`); orphans deleted on sight.

- **OWNER RULING (2026-07-13, `b773c04`): fix-first.** Tag-blocking product bugs
  default to FIX BEFORE TAG — no owner consult needed unless the fix is
  architectural ("this was something you didn't need my input on"). Calibrate
  future escalations UP.

- **OWNER RULINGS (2026-07-12–14):** canonical record = the ADDENDUM in
  `.planning/milestones/audits/2026-07-12-reality-check.md` (landed `8e36e62`).

- **RETIRED RAISEs (owner, 2026-07-14):** VM git upgraded to **2.50.1** (t4 env
  floor satisfied). Scratch repo `reubenjohn/reposix-scope-test-DELETEME`: owner
  KEEPS as a reusable scratch test target, currently archived, reset policy =
  force-push, unarchive via API on reuse.

- **Standing RAISEs for the owner:** ADR-010 RBF-LR-04 mirror fan-out redesign
  (push POST-write materialized snapshot; MANAGER decides under delegated
  authority, decide-and-disclose) + the entangled dvcs-topology/root-CLAUDE.md
  "bus-push catch-up" doc correction; D5 fold-release-plz-into-CI
  (CONSULT-DECISIONS).

- **Durable craft (accumulated ops lessons):** commit the handover refresh BEFORE
  launching a workhorse successor; a queued `agent send` to a WORKING workhorse is
  consumed at its next tool boundary; token-absolute relief — don't waive rotations
  off the % gauge. Release craft: READY-TO-TAG needs a tag-script guards DRY-RUN;
  the bump-merge→crates.io-publish→red-main window recurs every release (the
  window-aware brew gate `970d466` should hold — verify steady-state next cut).
  STALL alarms false-positive on long subagent/cargo lanes — inspect before
  nudging. Survey/UI prompts can block the pane — answer `n`, consent is the
  owner's. Draft handover edits anytime; commit only at boundaries. (herdr
  mechanics — ghost-text, second-Enter, digit-menus — now owned by the
  `/herdr-manager` skill; not restated here.)
