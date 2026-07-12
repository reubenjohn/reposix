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

- Board fully GREEN (CI, CodeQL, docs deploy gated-on-CI, release-plz). Phase-close
  requires ci-green-on-main (P0 post-push probe) — never open a phase over a red main.
- Workhorse (w1:p5): running at **xhigh effort** (owner-directed). Milestone C2
  autonomous, currently P110 OP-8 drain; P106 GREEN rows minted; P107 artifact
  committed; P113 (renumbered lost-update landing) executing under recorded owner
  authorization (`ed42ece`). **Do NOT /clear w1:p5 while its C2 runs**; monitor with
  `wait --status working` (idle-wait returns immediately while lanes run).
- Owner decisions ALL RESOLVED and relayed: land `424d367` (yes — VERIFY it reached
  origin/main), close dependabot #64-66 (yes — VERIFY closed), gh404 (defer), GTH-09
  (DEFER to v0.15.0, explicit named-headline deferral — VERIFY filed).
- Successor commitments made to the owner: (1) verify the UX-mandate fix-twice landed
  in the CLAUDE.md contract files + the all-subcommands error-message audit runs as a
  real lane; (2) error codes + `reposix explain <code>` go into next-milestone headline
  scope; (3) when wave-2 closes, draft P112 launch scope around the spine: agent-vs-MCP
  side-by-side demo (token counts on screen), the dark-factory/incident meta-story
  writeup, 90-second zero-install sim trial, then agent-ecosystem distribution
  (Claude Code skill, MCP directories, llms.txt).
- Watch: three planning files over 20k soft limit (P110 drain should split);
  fold-release-plz-into-CI-bar decision open (CONSULT-DECISIONS D5); runner unit tests
  uncollected by CI (MEDIUM).
