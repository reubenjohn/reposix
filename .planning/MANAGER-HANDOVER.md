# MANAGER-HANDOVER.md — outer-loop session manager (herdr) — live state only

For the incoming MANAGER session (the herdr outer loop in pane **w1:p7**), NOT the
reposix L0 orchestrator. The manager herds the workhorse in pane **w1:p5**; it never
does reposix work itself. Keep this file lean; git history is the archive.

## Role & standing owner instructions

- **Outer loop**: monitor w1:p5 (`herdr agent wait w1:p5 --status idle --timeout
  3600000`), inspect on wake (`herdr agent explain/read` — see the `/herdr-manager`
  skill, incl. the ghost-text trap), nudge/answer/rotate. Never `agent send` blind.
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
   "[Pasted text]" block that Enter won't submit while background subagents hold input.

## Live state (refresh at every rotation) — 2026-07-14 late, rotation #6→#7 (OWNER ACTIVE IN-PANE)

- **SUCCESSOR #7 FIRST ACTIONS:** (1) The OWNER is actively conversing in THIS pane —
  greet briefly, continue seamlessly; do not re-litigate tonight's decisions (all
  recorded: audit ADDENDUM + bullets below). (2) TaskStop monitor `bjsloa9qo`, re-arm
  your own (script in TaskStop output / git history of this file). (3) Workhorse #21
  LIVE in w1:p5: item-0 cursor refresh DONE (3d0184f); now /gsd-new-milestone re-anchor
  → v0.15 floor planning; two queued directives: creds-loaded pre-release-real-backend
  cadence (t4 REAL destructive run — git 2.50.1 + preflight exit 0 verified) and
  scratch-repo-policy doc (testing-targets.md). (4) Standing commitments made to the
  owner tonight: ADR-class decisions = manager decide-and-disclose (ledger + owner veto
  window); PUSH-NOTIFY the owner (PushNotification tool) at any owner-blocking moment
  or planned pause — silence must always mean "working"; owner-only = interactive sudo,
  new creds/scopes/spend, outward publishing. (5) Owner intent: 24/7 chug through
  v0.15→v0.25 this week; milestone cuts flow under standing release authority;
  2026-08-15 waiver expiry is a hard scheduling deadline inside v0.15.
  (6) CREDENTIAL AUDIT COMPLETE (2026-07-14, documented as comments in gitignored
  .env): GitHub = gh keyring OAuth non-expiring (`GITHUB_TOKEN=$(gh auth token…)` —
  never grep the literal value, it's a substitution); Atlassian token-world-for-reposix
  expires 2027-01-14; JIRA_API_TOKEN=$ATLASSIAN_API_KEY (same token). gh config
  cleaned of stale github.ncsu.edu (backup hosts.yml.bak). (7) OPEN OWNER ASKS (asked
  2026-07-14, unanswered — push-notify when they become blocking, don't nag):
  benchmark-spend ceiling for the funded Q1 live re-measurement; keep-host-awake /
  pause-Windows-updates for the week; usage-plan headroom.

- **v0.14.0 TAG SHIPPED (2026-07-14T01:16Z): `refs/tags/v0.14.0` @ `bcdee07`** + all 9
  release-plz per-package tags; crates.io at 0.14.0. Cut by the manager under standing
  authority through the FULL honest gate sequence: verdict GREEN-WITH-RECORDED-CAVEATS
  (`b8e309f`) + independent ratification (`5624943`) + CI green headSha-matched on
  `bcdee07` + TokenWorld end-state live-verified (2 protected + `2818063` current) +
  bump PR #72 merged (`256bb2e`) + tag-script all-guards-green. Recorded caveats
  (verdict + RETROSPECTIVE carry them verbatim): t4 git-floor NOT-VERIFIED (Ruling #4),
  litmus non-idempotency interim op (Ruling #2), item-7 CREATE-recovery waiver. Late
  arc: bump-merge turned main RED via a release-window gate deadlock (brew gates assert
  formula==crates.io-max; crates.io auto-published on merge) — fixed window-aware at
  `970d466` (successor #14), CI green re-confirmed, THEN tagged. Full arc:
  CONSULT-DECISIONS Rulings #2/#3/#4 + RETROSPECTIVE v0.14.0 + intake part-04.
- **ROTATION-#6 SESSION UPDATE (2026-07-14, mid-session):** All successor first-actions
  DONE. release.yml tag run SUCCESS (5 platform archives + installers + SHA256SUMS on
  releases/latest; crates.io 0.14.0) — v0.14.0 fully public. THEN a post-release
  regression arc, now CLOSED GREEN @ `8e2aae5` (all 5 workflows, manager-watched):
  quality-post-release went RED because P106 (804eedc+c4f1261, 07-12) hand-minted
  PASSes the runner's F-K4b congruence check rightly rejects → **MANAGER RULING #5
  (Option A, ledgered 05aa23c)**: honest bounded fix (03e7a6f — honest emission for
  fail-loud 01/02/04; example-05 asserts reworded to truth) + v0.15.0 intake for the
  F-K4b container-tautology redesign + example-05 deeper fix (3775075); NO waivers,
  F-K4b untouched → last red was example-04 TIMEOUT-BUDGET (unused apt toolchain in
  container SETUP; trimmed + 300→600s @ 8e2aae5). Workhorse #15 relieved (~143k),
  #16 relieved (~100k, handover ffb9d25), **#17 LIVE on the queue**.
- **MANAGER RULING #2 (E2/ADR valve, 2026-07-13): litmus non-idempotency = DEFER;
  tag proceeds.** The ADR-010 RBF-LR-04 inline fan-out pushes the PRE-write client
  tree — the mirror never converges to SoT after a push (executed proof, intake
  part-03); litmus repeatability needs `refresh-tokenworld-mirror.sh` before each
  run (documented interim op). Pre-existing v0.13.0-shipped behavior, SoT never
  wrong, mirror best-effort by design → NOT a v0.14.0 regression → not
  tag-blocking. Product fix (POST-write materialized-snapshot fan-out) is
  ADR-class → v0.15.0 + owner RAISE. Caveat carried VERBATIM in READY-TO-TAG
  report + RETROSPECTIVE. Doc-truth: the "bus-push catches the mirror up" claim
  (root CLAUDE.md / dvcs-topology.md) is proven non-convergent — correction
  bundled WITH the v0.15.0 ADR decision; truth meanwhile lives in the intake row +
  RETROSPECTIVE.
- **MANAGER RULINGS #3+#4 (2026-07-13):** #3 (E3) = fix both 9th-probe harness gaps
  + destructive t4 vs TokenWorld pre-authorized w/ protected-pair guardrails —
  EXECUTED `cb8ad11`, both gaps proven fixed. Cadence re-run then hit a THIRD gap:
  **VM git 2.25.1 < t4's legitimate 2.34 floor → t4 NOT-VERIFIED (exit 75,
  precondition-not-met, bailed pre-mutation), 5 PASS / 0 FAIL otherwise.**
  #4 (recorded caveat call under standing release authority) = **Option B: tag
  proceeds**; t4 row stays runner-minted NOT-VERIFIED (NO waiver, NO catalog/gate
  surgery); the named non-skippable probe (vision litmus P0) + p93 PASSED live;
  t4 sim twin green in CI; `reposix doctor` treats sub-2.34 as WARN. Option A
  (VM git upgrade) needs interactive sudo = owner-only → RAISEd. Full options +
  rationale: CONSULT-DECISIONS entry at `c27fd06`.
- TokenWorld fixture doctrine: 2 PROTECTED never deleted (`7766017`+`7798785`) +
  1 SACRIFICIAL EDITABLE (`2818063`, at v11 after the fix lane); orphan `9994241`
  DELETED. Item 7 = RESOLVED-DEFER (owner-waived CREATE-recovery RBF-LR-03 — flag
  VERBATIM in the READY-TO-TAG report); 8 OPEN intakes route v0.15.0, none
  tag-blocking (+ new rows from the diagnosis/fix lanes, all routed).
- **Manager monitor:** task `bjsloa9qo` (60s poll; ORIGIN-MOVED / BLOCKED /
  IDLE-STABLE / one-shot STALL / CI-RED). Incoming manager: TaskStop it, re-arm your
  own (script recoverable via TaskStop output or git history of this file).
- **Ops lessons (rotation #3):** commit the manager-handover refresh BEFORE launching
  a workhorse successor (committing after raced its first commit — harmless near-miss,
  different files). Relaying a ruling to a WORKING workhorse via queued `agent send`
  works — consumed at its next tool boundary; no need to wait for idle when it's an
  unblock the workhorse is waiting on.
- **Ops lessons (rotation #5, the v0.14.0 cut):** (1) ALWAYS `/clear` + verify the
  context gauge RESET before sending a successor charter — a charter sent into a
  stood-down-at-ceiling session ran on degraded context (recovered via Escape →
  queued-/clear fires → resend); never send text right after Escape without
  confirming the interrupt landed (it concatenates into the input box). (2)
  READY-TO-TAG must include a tag-script guards DRY-RUN — guard 3 (version bump
  unmerged, PR #72) was caught only by the manager's own pre-check. (3) The
  bump-merge→crates.io-publish→red-main window recurs EVERY release — the
  window-aware brew gate (`970d466`) should hold; verify steady-state next cut.
  (4) Workhorse token-absolute relief pushback (successor #8) was CORRECT — the %
  gauge misleads on a 1M window; don't waive rotations off the gauge.
- **OWNER RULING (2026-07-13, `b773c04`): fix-first.** Tag-blocking product bugs
  default to FIX BEFORE TAG — no owner consult needed unless the fix is architectural
  ("this was something you didn't need my input on"). Calibrate future escalations UP.
- **POST-TAG QUEUE CLOSED GREEN (2026-07-14, workhorses #17–#20; relief handover
  61da012).** All six items landed + CI green manager-verified through 6f44acb:
  hero-number interim qualifiers live (8 rows waived time-boxed to funded Q1
  re-measurement, expiry 2026-08-15; 2 rows re-bound to real tests), droppings swept,
  /gsd-cleanup archival cascade (21 phase dirs), ORCHESTRATION.md split <20k,
  CONSULT-DECISIONS pruned to live entries. **PIPELINE PAUSED awaiting owner Arc-D
  confirm** → then /gsd-new-milestone re-anchor (fold audit ADDENDUM rulings +
  simplification mandate + per-milestone deep surveys) → v0.15 floor planning.
  Workhorse seat w1:p5 idle after #20's wrap; launch #21 only after arc confirm.
  Historical queue detail (superseded): ⓪ DONE (ff7be56 — cursor de-staled, 4 new
  noticings routed to v0.15.0 intake, CI green + post-push cadence PASS);
  ① DONE (re-scoped then closed @ 6dc47a3, CI green): the original "v0.13.0 tag
  sequence" was STALE — v0.13.0/v0.13.1 already tagged+released 07-07/07-08 (#17
  caught it, manager corroborated). Landed instead: make_latest hardening (370310d —
  release.yml computes highest published semver, passes --latest=true|false explicitly
  on create+edit; hazard PROVEN by executed steal-demo, proof artifact committed) +
  riders (Workstream-C de-stale, PROJECT.md truth banner; wholesale PROJECT.md
  re-anchor stays deferred to /gsd-new-milestone); ② Q1c interim hero qualifiers
  (README "Three measured numbers" + index.md:17 synthetic-baseline caveat);
  ③ `.playwright-mcp/audit-03..08` droppings sweep; ④ `/gsd-cleanup` archival cascade;
  ⑤ ORCHESTRATION.md >100% size split + RIDER (CORRECTED after #17's provenance
  flag): trim CONSULT-DECISIONS.md per decision-procedures doctrine — DELETE closed
  entries, git history IS the archive; do NOT create .planning/archive copies (the
  manager's earlier "move to archive/" wording was wrong; ~54k vs 20k guideline).
- **OWNER RULINGS (2026-07-14): Q3/Q4/Q5/Q7/Q8/Q9 DECIDED + a 10-survey calibration
  mandate — canonical record is the ADDENDUM in
  `.planning/milestones/audits/2026-07-12-reality-check.md` (owner chose that home).
  Headlines: real-backend launch gate; DELETE legacy files, no banners — docs/planning
  simplification is a first-class roadmap goal; keep ~6-milestone scale; one survey
  pass ≈ 10% of latent work, ~10 passes to convergence. **Arc D RATIFIED 2026-07-14
  under explicit owner delegation ("Your call") — pipeline pause LIFTED; #21 launched
  into STATE.md cursor refresh → /gsd-new-milestone re-anchor → v0.15 floor planning.**
- **Reality-check audit (2026-07-12): LANDED at `8e36e62`** —
  `.planning/milestones/audits/2026-07-12-reality-check.md` (verbatim vs the owner bak,
  manager-diff-verified). Owner decided §5 Q1 (live MCP re-measurement FUNDED; FUSE-era
  98.7% figure retire/relabel) + Q2 (tags). **PENDING WITH OWNER: arc ratification
  (§4 arc D ratchet-first recommended+endorsed) + §5 Q3–Q9** (manager-proposed
  answers in session transcript ~2026-07-12). No defect-fixing lanes beyond
  tag-blockers until arc ratified. Fold ratified arc + answers into PROJECT.md
  re-anchor at /gsd-new-milestone.
- **Ops lessons (this manager's session):** (1) Claude Code survey/UI prompts can
  block the workhorse pane AND freeze the subagent progress display — on a stall
  alarm, read the pane; if a y/n/d survey prompt shows, answer `n` (consent is the
  owner's); a dirty tree is a liveness signal even when the display is frozen.
  (2) STALL alarms false-positive on long cargo/real-backend lanes — inspect before
  nudging; nudge = message the workhorse to health-check its child, never Escape
  first. (3) Background monitors SURVIVE `/clear` — the outgoing manager's monitor
  `bgh6ujkic` is still running: TaskStop it, then re-arm your own (poll 60s:
  `git ls-remote origin main` → ORIGIN-MOVED; `herdr agent explain w1:p5` →
  BLOCKED / IDLE-STABLE at 3 consecutive idle / one-shot STALL at 20 working-min;
  every 5th poll `gh run list --branch main` → CI-RED). (4) Serialization: draft
  manager-handover edits anytime, COMMIT only at workhorse idle/wave boundaries.
  (5) herdr: a digit/letter alone answers menus; long `agent send` needs a second
  Enter after ~2s; text after `❯` is often ghost-text — never treat it as pending
  input.
- **RETIRED RAISEs (owner, 2026-07-14):** (1) VM git upgraded to **2.50.1** (verified;
  t4 env floor satisfied — one creds-loaded `pre-release-real-backend` cadence
  dispatched to #21 so t4 executes its destructive scenario for real; product-FAIL
  there = v0.15 fix-first). (2) Scratch repo `reubenjohn/reposix-scope-test-DELETEME`:
  owner KEEPS it as a REUSABLE scratch test target — no delete_repo scope by design;
  reset policy = force-push; currently archived, unarchive via API on first reuse;
  record in docs/reference/testing-targets.md at GSD-quick scale.
- **Standing RAISEs for the owner:** ADR-010 RBF-LR-04
  mirror fan-out redesign (decision packet in v0.15 planning; MANAGER decides under
  delegated authority, decide-and-disclose)
  (push POST-write materialized snapshot; litmus non-idempotency, intake part-03)
  + the entangled dvcs-topology/root-CLAUDE.md "bus-push catch-up" doc correction;
  P112 ROADMAP prose-vs-artifact reconcile at /gsd-new-milestone; D5
  fold-release-plz-into-CI (CONSULT-DECISIONS). Monitor craft + P112 launch-scope
  spine: see git history of this file (`61af3ba`).
