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

## Live state (refresh at every rotation)

- **v0.14.0 wave-2 CLOSED 11/11 GREEN at the OWNER tag boundary** (2026-07-12).
  Milestone-close verification = `quality/reports/verdicts/p111/VERDICT.md` (GREEN,
  OP-9 GREEN, unbiased fresh-execution verifier). Aggregate
  `milestone-v0.14.0/VERDICT.md` deliberately NOT minted yet — the owner-gated 9th
  probe would force RED on its P0 row. Board fully green at `bda849d`.
- **v0.14.0 TAG DELEGATED TO MANAGER (owner, 2026-07-12) — currently ⛔ BLOCKED on a
  RED 9th probe (2026-07-13Z).** The probe ran for real (no env wall): 1 PASS / 3 FAIL
  / 2 NOT-VERIFIED → hard RED per OD-2; honest RED verdict minted
  `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md` (`563095f`). Sim-only
  coverage had masked real-backend failures. Blockers: **B1** (P0) vision-litmus —
  mass-delete guard refused a delete-shaped reconcile (page 2818063 in mirror, absent
  from live TokenWorld; stale-mirror-drift vs legitimate-delete diagnosis needed;
  reconcile mutation = manager's call after read-only diagnosis); **B2** (P0)
  p93-partial-failure-recovery + **B3** (P1) attach-sync — stale 2026-07-06 artifacts,
  re-run + re-persist; **B4** (P0) t4-conflict-rebase + **B5** (P1) github-front-door
  — verifier scripts never shipped → NOT-VERIFIED. Remediation checklist:
  `.planning/SESSION-HANDOVER.md`. Re-grade needs cadence exit 0 + unbiased
  ratification; then tag script → manager pushes tag. NO softening, NO waiver.
- **Serialization discipline (self-correction, 2026-07-13Z):** the manager violated
  the single-writer ruling by committing handover updates mid-workhorse-run
  (6d0f94f, 2bc29f1, 7dabbbf). New rule: manager edits may be drafted anytime but are
  COMMITTED only at workhorse idle/wave boundaries (monitor events mark them).
- **Priority order (owner, 2026-07-12): big SURPRISES-INTAKE drain FIRST**
  (`.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md`, 43,988B / 44,000B
  ceiling, ~20 entries incl. several HIGH — triage/drain substantially, not
  byte-relief), then the tag sequence above.
- **Owner ask (2026-07-12): 75% early-warning on file-size ceilings.** Extend the
  `structure/file-size-limits` gate (or add a sibling row) so any tracked file at
  ≥75% of its ceiling emits a non-blocking WARN in pre-commit/pre-push output;
  ≥100% keeps the existing block contract. Mind the current waiver (until
  2026-08-08, warn-now/block-later) interplay; fix-twice into quality/CLAUDE.md.
  Route via workhorse /gsd-quick — queued in the charter expansion.
- **Hygiene lane CLOSED green (2026-07-13Z):** 3 splits landed — SURPRISES-INTAKE
  `dc3c21a` (43,988→3,797B, all 17 entries terminal, index preserved), STATE.md
  `3491d24` (21,137→10,846B), ROADMAP.md `068bcfc` (37,764→13,015B); CI fully green
  at `a47bd20` (CI/release-plz/CodeQL/Docs all success, manager-verified). Spent
  workhorse rotated out at ~180k tokens; its flaky-subagent-file-tooling RAISE is in
  SESSION-HANDOVER.md.
- Workhorse (w1:p5): **successor #2** launched 2026-07-13Z, xhigh, entry
  `.planning/SESSION-HANDOVER.md` + this file. Expanded charter (in order): ① record
  serialization decision (CONSULT-DECISIONS `[OWNER]` + ORCHESTRATION fix-twice);
  ② foreign-tree triage; ③ 75% file-size early-warning gate; ④ DEFERRED-entry
  routing (7 entries → concrete v0.15.0/GTH landing spots); ⑤ v0.14.0 tag sequence
  (probe → aggregate verdict → tag script → STOP at READY-TO-TAG; manager pushes the
  tag); ⑥ v0.13.0 pre-tag actions queued after. Hard-barred from tag-push and from
  unrecorded foreign-tree moves. VERIFY each item lands green.
- **Owner decision (2026-07-12): shared-tree contention RESOLVED — session
  serialization** (no parallel sessions writing the shared tree; no new worktree
  infra). Route via workhorse: `[OWNER]` disposition row in CONSULT-DECISIONS.md +
  fix-twice into ORCHESTRATION.md doctrine. Foreign uncommitted work still in the
  tree (code.json delta + phases/21, phases/22, scripts/demos, scripts/dev,
  verifications/docs-repro) — triage/land-or-drop it as part of the serialization
  cleanup, via workhorse.
- **OWNER DECISION (2026-07-12, reality-check §5 Q1): FUND LIVE MCP RE-MEASUREMENT
  against a real backend.** (a) Retire/relabel the FUSE-era ~150k→~2k (98.7%) figure
  everywhere it anchors — docs/research/initial-report/performance.md:9,
  initial-report.md:78, .planning/PROJECT.md Context. (b) Commission honest
  re-measurement: a real MCP server (e.g. Atlassian/GitHub MCP) driven against a
  sanctioned real backend (TokenWorld Confluence / reubenjohn-reposix GitHub issues;
  creds verified, targets pre-authorized), real count_tokens pipeline, vs the
  equivalent reposix session on the SAME backend — wired into CI per the §4
  benchmark-honesty milestone (live baseline is now FUNDED, not optional). Egress
  allowlist applies to the MCP server too. (c) INTERIM (before the live number
  exists): hero surfaces — README "Three measured numbers" header + docs/index.md:17
  card — get the synthetic-baseline qualifier; never let the old framing ride to
  launch. Record as `[OWNER]` in CONSULT-DECISIONS via workhorse (with the other §5
  answers when ratified). Manager sequencing call: record now; docs-touching interim
  edits AFTER the v0.14.0 tag lands (avoid churning probe/CI state pre-tag).
- **Reality-check audit (2026-07-12):** owner moving the file to
  `.planning/milestones/audits/2026-07-12-reality-check.md` themselves (manager backup:
  `/home/reuben/reposix-reality-check-2026-07-12.bak.md`; watcher armed). Working arc
  recommendation = §4 arc D (ratchet-first), manager endorses; ARC SELECTION IS THE
  OWNER'S — no defect-fixing lanes until ratified. §5 status: Q1 + Q2 owner-decided;
  Q3–Q9 have manager-proposed answers awaiting ratification (see session transcript /
  final scoping doc). Fold chosen arc + §5 answers into PROJECT.md re-anchor +
  launch-readiness /gsd-new-milestone.
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
