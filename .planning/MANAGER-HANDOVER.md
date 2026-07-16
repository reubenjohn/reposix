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
  refresh this file, commit+push, run § Rotation. Workhorse: instruct ~100k soft /
  ~150k hard, then it REPLACES `.planning/SESSION-HANDOVER.md`, commits+pushes, ends
  turn; you `/clear` w1:p5 and launch its successor pointing at that file.
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

## Live state (manager #10 on station; mid-shift refresh 2026-07-15 ~17:40 PT)

- **SHIFT SUMMARY (#10 so far):** Workhorses #35 and #36 launched+closed clean.
  #35: roadmap-diagram quick SHIPPED (`1db48e4`/`16fb356`/`fa58ad6`), T5
  JSONL-usage methodology ENCODED (`9be5439`: 115-PLAN.md amendment +
  CONSULT-DECISIONS `[SELF]`), noticing filed (`4b38e62`). #36: Rovo MCP auth
  check — #34's "API-token-endpoint blocker" **REFUTED** (`5374fe0`:
  ATLASSIAN_API_KEY authenticates `mcp.atlassian.com/v1/mcp` via BOTH Basic and
  Bearer; 401 no-auth control; probing stopped at initialize) → **T4→T6 has NO
  remaining owner-gated item**; pre-push 109s spike root-caused+filed
  (`fcddf90`); relief handover #36→#37 pushed (`1780641`); GTH-V15-25 filed
  (`e1c71c4`, owner-approved token-bloat CI tripwire). CI GREEN on tip
  `e1c71c4` (run 29461500358). Seat IDLE at the T4 capture gate.
- **#37 RUNNING (launched 2026-07-15 ~20:40 PT — owner directive "finish it
  all"; reset-wait overridden, and reset-gating retired entirely per the
  standing instruction above):** full P115 charter = read
  `.planning/SESSION-HANDOVER.md` (`1780641`, authoritative) → T4 captures
  (≤18 sessions, session-spend ledger ≤50 ceiling, throwaway `/tmp` clone for
  the reposix arm, MCP arm via verified ATLASSIAN_API_KEY auth `5374fe0`; on
  any cap-hit: commit+push progress, update handover, clean turn end,
  successor resumes) → **FOLD-IN (owner timing note 2026-07-15): during T4,
  extract the agent command list from the captured session JSONL into a
  committed trajectory fixture — GTH-V15-25 step 1, <1h byproduct while the
  data is fresh; the rest of that row stays a post-T4 lane** → T5 (JSONL-usage
  path per `9be5439`, token-economy.md regen) → T6 (honest-headline reframe +
  second latency.md refresh; delete all FOUR `[SELF]` entries: A1,
  T2-latency-canonical, T6-headline, T5-JSONL-methodology) → close P115 per
  push cadence → then P116 ADR-010 packet routed to MANAGER for ruling, no
  pre-ruling implementation. Watch items: latency.md regen-clobber tension;
  doc-alignment 14-row re-drift budget in T6.
- **Seat-rotation craft (learned #35→#36):** do NOT pane-clear a workhorse
  whose wrap says "awaiting CI green" — its final turn hasn't ended; killing
  its gh-run-watch shell injects a failure event and costs an investigation
  detour. Wait for manager-poll TRUE-IDLE (idle + zero shells) first. A /clear
  sent mid-turn QUEUES ("Press up to edit queued messages") and fires at turn
  end — harmless but confusing. Workhorse charters now include "never end your
  final turn with a background shell running" (#36 complied via bounded
  in-turn polls). Also retained from #9: never load the claude-api skill at
  manager tier (~300k context); delegate API-mechanics to a subagent.
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
