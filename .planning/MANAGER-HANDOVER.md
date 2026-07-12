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
- **v0.14.0 TAG DELEGATED TO MANAGER (owner, 2026-07-12)** — owner authorized the
  manager to make and execute the tag call end-to-end, tag push included (external
  mutation pre-approved under this delegation). Sequence (route work through the
  workhorse/GSD, manager verifies each artifact, never over a red main): ① 9th probe
  `python3 quality/runners/run.py --cadence pre-release-real-backend` with non-default
  `REPOSIX_ALLOWED_ORIGINS` (exit 0) → ② mint+ratify aggregate milestone-v0.14.0
  verdict (skeleton `quality/dispatch/milestone-close-verdict.md`) → ③ author/run tag
  script (pattern `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh`) → ④ push
  v0.14.0 tag. Ground truth: `scripts/preflight-real-backends.sh` = PASS 3/3, verified
  firsthand by manager 2026-07-12 — the old "no real-backend creds" claim was stale.
  Probe runs honestly first; only a genuine env wall justifies a recorded caveat call.
- **Priority order (owner, 2026-07-12): big SURPRISES-INTAKE drain FIRST**
  (`.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md`, 43,988B / 44,000B
  ceiling, ~20 entries incl. several HIGH — triage/drain substantially, not
  byte-relief), then the tag sequence above.
- Workhorse (w1:p5): **FRESH successor** launched post-`bda849d`, xhigh, entry point
  `.planning/SESSION-HANDOVER.md`. Charter: hygiene lane — /gsd-quick
  progressive-disclosure splits of over-20k STATE.md + ROADMAP.md; charter expansion
  queued (send at next idle): OP-8-style big intake drain + record owner decisions.
  Hard-barred from tag-push and foreign tree work. VERIFY its quicks land green.
- **Owner decision (2026-07-12): shared-tree contention RESOLVED — session
  serialization** (no parallel sessions writing the shared tree; no new worktree
  infra). Route via workhorse: `[OWNER]` disposition row in CONSULT-DECISIONS.md +
  fix-twice into ORCHESTRATION.md doctrine. Foreign uncommitted work still in the
  tree (code.json delta + phases/21, phases/22, scripts/demos, scripts/dev,
  verifications/docs-repro) — triage/land-or-drop it as part of the serialization
  cleanup, via workhorse.
- **v0.13.0 tag ALSO DELEGATED (owner, 2026-07-12, via AskUserQuestion):** same
  end-to-end delegation — execute the v0.13.0 OWNER PRE-TAG ACTIONS
  (§ Workstream A of the v0.13.0 ROADMAP/STATE) → verify → push the v0.13.0 tag,
  sequenced AFTER the v0.14.0 tag lands.
- **RAISEs standing for the owner:** P112 ROADMAP prose-vs-artifact reconcile at
  /gsd-new-milestone; D5 fold-release-plz-into-CI still open (CONSULT-DECISIONS).
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
