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
- **Context budget**: self under ~250k hard (soft ~200k) — refresh this file,
  commit+push, run § Rotation. Workhorse: instruct ~100k soft / ~150k hard, then it
  REPLACES `.planning/SESSION-HANDOVER.md`, commits+pushes, ends turn; you `/clear`
  w1:p5 and launch its successor pointing at that file.
- **Real-backend mutations PRE-AUTHORIZED**: Confluence TokenWorld, GitHub
  reubenjohn/reposix issues, JIRA TEST. Credentials/spend beyond those still
  owner-gated.
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

## Live state (refresh at every rotation)

- **v0.14.0 wave-2 CLOSED 11/11 GREEN at the OWNER tag boundary** (2026-07-12).
  Milestone-close verification = `quality/reports/verdicts/p111/VERDICT.md` (GREEN,
  OP-9 GREEN, unbiased fresh-execution verifier). Aggregate
  `milestone-v0.14.0/VERDICT.md` deliberately NOT minted yet — the owner-gated 9th
  probe would force RED on its P0 row. Board fully green at `bda849d`.
- **OWNER-ONLY queue (in order):** ① TokenWorld creds + non-default
  `REPOSIX_ALLOWED_ORIGINS`, run `python3 quality/runners/run.py --cadence
  pre-release-real-backend` (exit 0) → ② mint+ratify aggregate milestone-v0.14.0
  verdict → ③ author/run tag script (CHANGELOG references it) → ④ cut v0.14.0 tag
  (none exists local/remote). Never let the workhorse do these.
- Workhorse (w1:p5): **FRESH successor** launched post-`bda849d`, xhigh, entry point
  `.planning/SESSION-HANDOVER.md`. Charter: hygiene lane only — /gsd-quick
  progressive-disclosure splits of over-20k STATE.md + ROADMAP.md, relieve
  SURPRISES-INTAKE 44000B ceiling (12B headroom). Hard-barred from owner-only actions
  and from foreign tree work. VERIFY its quicks land green.
- **RAISEs standing for the owner:** shared-tree contention (HIGH — a lane manipulated
  a foreign code.json to pass a rebase; foreign uncommitted work still sits in the
  shared tree: code.json delta + phases/21, phases/22, scripts/demos, scripts/dev,
  verifications/docs-repro — fleet correctly left it untouched; decide worktree
  isolation vs session serialization BEFORE the next parallel fleet run); P112 ROADMAP
  prose-vs-artifact reconcile at /gsd-new-milestone; D5 fold-release-plz-into-CI still
  open (CONSULT-DECISIONS).
- Fixed this session: CI-waiter hangs (twice) → durable `scripts/ci-wait.sh` landed;
  error codes + `reposix explain <code>` now a v0.15.0 HEADLINE phase (`e5b969d`).
- **Monitoring craft:** herdr idle/working waits FLAP while background subagents run.
  What works: a persistent Monitor polling (a) origin/main movement, (b) pane
  `visible_working`, emitting ORIGIN-MOVED events + a one-shot 20-min stall alarm.
  On stall: nudge the workhorse to SendMessage its stuck child.
- P112 launch-scope spine (when owner opens launch-readiness): agent-vs-MCP
  side-by-side demo (token counts on screen), dark-factory/incident meta-story,
  90-second zero-install sim trial, agent-ecosystem distribution (Claude Code skill,
  MCP directories, llms.txt).
