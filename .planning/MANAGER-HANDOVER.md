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

- **2026-07-12 (workhorse session COMPLETE at clean boundary).** The post-incident
  workhorse ended its turn at ~100k own-context; SESSION-HANDOVER.md replaced+pushed at
  `5cbfcbb`, tracked tree clean, origin/main in sync. Outcomes: D2 re-sealed at BOTH
  layers (hook hardening + `3206a2b` binary-side `reposix init` refusal — the real cut
  for the subprocess bypass); live D2 repro filed in v0.14.0 SURPRISES-INTAKE (HIGH,
  "D2 re-seal Wave 1"); 8 lanes GREEN + pushed, unbiased-verifier PASS; P108 paperwork
  closed; lost-update fix preserved on `backup-lost-update-424d367`.
- **FOUR owner decisions parked (none block the READY queue):** (1) land 424d367 to
  GitHub main (rec: yes, clean /tmp-clone push); (2) close dependabot #64-66 as
  redundant (rec: yes — cargo audit 0 live advisories); (3) gh404 live-GitHub verify
  (rec: defer); (4) GTH-09 ADR-010 slug→id durable-create — ship in v0.14.0 or defer.
- **Rotation of w1:p5 is BLOCKED by owner-staged input text** — the owner EDITED it
  (now "land 424d367 and close the dependabot PRs") but did not submit. Do NOT press
  Enter (external mutation) and do NOT /clear (would destroy it). Natural flow: owner
  submits → workhorse (14% context, plenty) executes → then rotate to successor
  pointing at SESSION-HANDOVER.md `5cbfcbb`. Owner push-notified 2026-07-12.

- **2026-07-12 (post-incident)** — wave-2 progressing: **D2 (P102), P103, P105 all
  closed GREEN** on origin/main (P105 verifier-graded at `f2d527a`, incl. CR-01
  deletion-propagation fix + tests). Workhorse (w1:p5) resumed under a fresh
  "v0.14.0 wave-2 C2 (post-incident)" coordinator. Main agent context ~13%. In flight:
  P106 (held local commit `424d367`, was behind pre-push gates) + gh404/RUSTSEC lanes.
- **SHARED-TREE CORRUPTION INCIDENT (2026-07-12, RESOLVED).** A P106 leaf subagent ran
  `reposix init`/sim-seed in the SHARED repo instead of a /tmp clone — flipped
  `.git/config` core.bare=true, repointed origin to the sim (127.0.0.1:7988), thrashed
  HEAD to `e18df81`, polluted refs/reposix/*. The exact D2 hazard this milestone hardens.
  The workhorse **misdiagnosed the source as "a concurrent herdr session"** (the manager)
  and blocked asking the absent owner to "stop herdr." Manager intervention: audited all
  panes (confirmed manager never writes .git/config; w1:pE = idle Sonnet skill-md editor,
  not a corruptor; no active writer → safe to repair now), Escape-cancelled the block,
  interrupted a HUNG gsd-executor (323k tokens, frozen 40+min, git-cat-file integrity
  check), and delivered ground-truth correction. Workhorse then **repaired the tree**
  (core.bare=false, origin→GitHub, HEAD→main/f2d527a) and spawned the post-incident C2.
  Asked it to capture the leak as a live D2 repro in SURPRISES-INTAKE — SUCCESSOR: verify
  that repro landed + that the offending leaf's init path is fixed so it can't recur.
- **Owner-gated, do NOT action:** staged text sits in w1:p5's input box —
  "land 424d367 and close dependabot 64-66 as redundant" (likely owner-typed). Closing
  dependabot #64-66 is an owner-gated external mutation; leave it for the owner. Also
  pending owner: gh404 read-only GitHub named-target ask; RUSTSEC reframe confirm.
- **TOOLING BUG (manager rotation):** `herdr pane send-keys <pane> C-u` is REJECTED
  ("unsupported key C-u") in this herdr version — only `Enter`/`Escape` validated.
  `.planning/manager-rotate.sh` lines 18 uses `C-u` and will fail to clear the box.
  Also: a long single-line `agent send` becomes a "[Pasted text #1]" block that `Enter`
  won't submit while background subagents hold input — interrupt subagents first, THEN
  Enter. Successor: fix rotate script's clear step (or drop it) before relying on it.
- Strategic synthesis (explore run 2026-07-11): adoption is the north star; ladder is
  v0.13.2 → launch-readiness → observability/multi-repo → v1.0.0.
- Open watch items: post-incident stabilization (D2 repro capture, no re-corruption),
  P106 landing 424d367 through gates.
