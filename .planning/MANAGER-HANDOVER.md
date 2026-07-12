# MANAGER-HANDOVER.md — outer-loop session manager (herdr) — live state only

For the incoming MANAGER session (the herdr outer loop in pane **w1:p7**), NOT the
reposix L0 orchestrator. The manager herds the workhorse agent in pane **w1:p5**; it
never does reposix work itself. Bound to live state; git history is the archive.

## Role & standing owner instructions (2026-07-11)

- You are the **outer loop**: monitor w1:p5 with `herdr agent wait w1:p5 --status idle
  --timeout 3600000` (≈1h, catches hangs), inspect on wake, nudge/answer/rotate. Load
  the `/herdr-manager` skill knowledge: never `agent send` blind; check
  `herdr agent explain w1:p5 --json` + `agent read` first; "idle" can mean
  background subagents still running — check for `●`/`◯` subagent lines.
- **Never get involved in low-level concerns** (including CI breaking on main).
  Delegate days-sized scopes to the workhorse; use your own Explore/reader-digester
  subagents for research so your context stays lean.
- **Keep your own context under ~200k hard** (soft line ~150k; owner relaxed
  2026-07-11). When approaching:
  update THIS file to live state, commit+push, then run the rotation script
  (§ Rotation) to /clear w1:p7 and start your successor. Do this DELICATELY — the
  script must be running detached BEFORE your final turn ends.
- **Real-backend mutations are PRE-AUTHORIZED** by the owner (2026-07-11): Confluence
  TokenWorld, GitHub reubenjohn/reposix issues, JIRA TEST are playgrounds. The
  workhorse knows (relayed). Credentials/spend beyond those targets still owner-gated.
- **Workhorse budget**: 1M-context agent; instruct it to stay under ~100k own-context,
  hard wrap-up ~150k, then REPLACE `.planning/SESSION-HANDOVER.md` (live state only),
  commit+push, report, END TURN. Then you `/clear` w1:p5 and launch its successor
  pointing at the fresh SESSION-HANDOVER.md.
- **Owner intent**: keep this chugging for days; the project should evolve, get "on
  the map" (OD-4 launch-readiness: asciinema demo, honest headline numbers, install
  excellence, Show-HN kit), and be excellent. Ensure the workhorse delegates heavily
  (router, not worker).

## Rotation procedure (self-succession, w1:p7)

1. Update this file to live state; commit+push (`docs(planning): refresh manager handover`).
2. Start the detached rotation script (gitignored, local-only):
   `setsid nohup bash .planning/manager-rotate.sh w1:p7 >/tmp/manager-rotate.log 2>&1 &`
3. END YOUR TURN immediately (script waits for the pane to go idle before clearing).
   The script sends `/clear`, then `/herdr-manager <successor prompt pointing here>`.
4. If the script is missing (fresh clone), recreate it from § Script contract below.

### Script contract (recreate if absent)
`.planning/manager-rotate.sh <manager-pane-id>`: waits ~15s, waits for pane idle,
**empties the input box first (Ctrl-U — pending user-typed text otherwise swallows
the /clear; learned from the 2026-07-11 dry run)**, sends `/clear` + Enter, waits
~10s, verifies the clear landed (context gauge reset), sends the successor
`/herdr-manager` prompt (read MANAGER-HANDOVER.md, resume herding w1:p5) + Enter,
verifies via `herdr agent read`, logs to /tmp/manager-rotate.log.

## Live state (refresh at every rotation)

- **2026-07-11 ~23:00** — v0.13.1 shipped+green; SESSION-HANDOVER.md replaced at
  `4bf544b`. Workhorse (w1:p5) successor launched with WAVE-2 charter: D2 self-safe
  hardening FIRST (reject-t@t hook, worktree isolation, config-write guard), then
  GitHub-v09 404, RBF-LR-03, tutorial repro (waiver deadline 2026-09-15), RUSTSEC
  PRs #64-66, prune_oid_map data-loss hazard; cheap wins: doc-alignment --persist
  split, file-size waiver (expires 2026-08-08). Launch-readiness milestone: scope
  after wave 2, don't start.
- Strategic synthesis (explore run 2026-07-11): adoption is the north star; ladder is
  v0.13.2 → launch-readiness → observability/multi-repo → v1.0.0.
- Open watch items: none blocked on owner.
