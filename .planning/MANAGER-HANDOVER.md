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

## Live state (refresh at every rotation) — 2026-07-13, manager rotation #4

- **v0.14.0 TAG critical path: item 5 APPROVED-RESTORE (execute) — workhorse successor
  #8 IN FLIGHT in w1:p5** (launched post-`6938024`, charter =
  `.planning/SESSION-HANDOVER.md`, execution-ready spec + 6 binding guardrails):
  RESTORE sacrificial page `2818063` (untrash via `scripts/confluence_tokenworld.py
  restore`, NO mirror surgery) → litmus re-green on the UNMODIFIED Pattern-C harness →
  item 8 §4 mechanicals (OP-9 triage of the 8 OPEN intake items + RETROSPECTIVE
  distillation BLOCKS ratification; ci-green-on-main probe has a KNOWN headSha race —
  cross-check manually) → **STOP at READY-TO-TAG**. Manager then VERIFIES against
  reality (probe exit 0, verdict, ratification, CI green headSha-matched, no tag
  pushed, TokenWorld = 2 protected + 1 sacrificial current) and PUSHES the v0.14.0
  tag under standing authority.
- **Fixture doctrine CORRECTED (manager ruling, recorded `c46ae55`):** TokenWorld =
  2 PROTECTED pages never deleted (`7766017`+`7798785`) + **1 SACRIFICIAL EDITABLE**
  (`2818063`, current or trashed between runs). "Exactly 2 pages" was WRONG — the
  unmodified litmus needs the editable 3rd (`litmus-flow.sh:64-72`). The earlier DROP
  ruling was superseded by RESTORE after successor #7 HALTED execution with
  code-verified proof (exemplary prove-before-mutate; nothing was mutated under the
  bad ruling).
- Items 4a/4b/6 SHIPPED GREEN @ `22a7777` (DP-2 repro-first honored; §8 real-backend
  arm ran); item 7 = RESOLVED-DEFER (p93 = architectural RBF-LR-03, owner-waived
  ADR-010 — flag PROMINENTLY in the READY-TO-TAG report); intake recount honest at
  8 OPEN (`466b6a6`).
- **Manager monitor:** task `bbz6wred9` (60s poll; ORIGIN-MOVED / BLOCKED /
  IDLE-STABLE / one-shot STALL / CI-RED). Incoming manager: TaskStop it, re-arm your
  own (script recoverable via TaskStop output or git history of this file).
- **Ops lessons (rotation #3):** commit the manager-handover refresh BEFORE launching
  a workhorse successor (committing after raced its first commit — harmless near-miss,
  different files). Relaying a ruling to a WORKING workhorse via queued `agent send`
  works — consumed at its next tool boundary; no need to wait for idle when it's an
  unblock the workhorse is waiting on.
- **OWNER RULING (2026-07-13, `b773c04`): fix-first.** Tag-blocking product bugs
  default to FIX BEFORE TAG — no owner consult needed unless the fix is architectural
  ("this was something you didn't need my input on"). Calibrate future escalations UP.
- **After v0.14.0 tag lands:** ① v0.13.0 sequence (OWNER PRE-TAG ACTIONS, v0.13.0
  ROADMAP § Workstream A → same READY-TO-TAG stop; delegation extended by owner);
  ② post-tag queue: Q1c interim hero qualifiers (README "Three measured numbers" +
  index.md:17 synthetic-baseline caveat), `.playwright-mcp/audit-03..08` droppings
  sweep, `/gsd-cleanup` archival cascade (tags unblock it), ORCHESTRATION.md >100%
  size split.
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
- **Standing RAISEs for the owner:** P112 ROADMAP prose-vs-artifact reconcile at
  /gsd-new-milestone; D5 fold-release-plz-into-CI (CONSULT-DECISIONS). Monitor
  craft + P112 launch-scope spine: see git history of this file (`61af3ba`).
