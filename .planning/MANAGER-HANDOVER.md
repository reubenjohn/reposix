# MANAGER-HANDOVER.md — outer-loop session manager (herdr) — live state only

For the incoming MANAGER session (the herdr outer loop in pane **w1:p7**), NOT the
reposix L0 orchestrator. The manager herds the workhorse in pane **w1:p5**; it never
does reposix work itself. Keep this file lean; git history is the archive.

## Role & standing owner instructions

- **Outer loop**: monitor w1:p5 (`herdr agent wait w1:p5 --status idle --timeout
  3600000`), inspect on wake (`herdr agent explain/read` — see the `/herdr-manager`
  skill, incl. the ghost-text trap), nudge/answer/rotate. Never `agent send` blind.
- **Ownership mandate (owner, 2026-07-12)**: the manager OWNS everything end-to-end —
  maintainability, code/architectural elegance, end-user experience. Heavy delegation
  and context-lean constraints stand, with one exception: at rare boundaries (only
  after very significant milestones), run your own highly selective probes to ground
  understanding, complementing delegate reports.
- **Eyes-and-ears baseline (every wake)**: (1) `gh run list --branch main -L 3` — a
  red main is owner-visible health, never a "low-level concern"; dispatch a fix
  immediately. (2) origin/main sync + dirty-tree check. (3) Spot-verify one
  load-bearing claim from any wrap report before relaying it. Workhorse self-reports
  are verified, not trusted.
- **Context budget** (owner loosened 2026-07-12): self under ~250k hard (soft ~200k) —
  refresh this file,
  commit+push, run § Rotation. Workhorse: instruct ~100k soft / ~150k hard, then it
  REPLACES `.planning/SESSION-HANDOVER.md`, commits+pushes, ends turn; you `/clear`
  w1:p5 and launch its successor pointing at that file.
- **Real-backend mutations PRE-AUTHORIZED** (owner 2026-07-11): Confluence TokenWorld,
  GitHub reubenjohn/reposix issues, JIRA TEST. Credentials/spend beyond those still
  owner-gated.
- **Owner intent**: multi-day autonomous chug toward OD-4 launch-readiness (asciinema
  demo, honest headline numbers, install excellence, Show-HN kit); workhorse routes,
  doesn't work.

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

- **2026-07-12 ~08:00** — workhorse session COMPLETE at clean boundary
  (SESSION-HANDOVER at `5cbfcbb`; origin/main in sync). Wave-2: P102–P105, P108,
  P109(a) GREEN. Shared-tree corruption incident (P106 leaf, subprocess bypass)
  resolved with zero data loss; re-sealed at product layer (`3206a2b` init refuses an
  existing worktree root). D2 repro filed in v0.14.0 SURPRISES-INTAKE.
- **CI on main is RED** (since ~09:26Z): `code/shell-coverage` ratchet 12.54% vs 13%
  floor (guard shell code diluted the corpus). Fleet pushed over it all night —
  systemic hole: phase-close verifies push-landed but NOT ci-green; docs deploy also
  ungated on CI. Successor workhorse charter: fix CI honestly FIRST, fix-twice the
  gate hole, then P106/P107 (P110–P112 chained behind P102–P109 ALL GREEN).
- **Owner decisions PARKED (do not action)**: land `424d367` (lost-update fix, local
  branch `backup-lost-update-424d367`); close dependabot #64-66; gh404 live verify;
  GTH-09 ship-or-defer.
- **Pending**: rotate w1:p5 to successor with the CI-first charter (input box is
  empty — the earlier "staged text" was ghost text, rule now in `/herdr-manager`).
