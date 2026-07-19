# .planning/CLAUDE.md — planning-surface rules (auto-loaded under .planning/)

Extends root `CLAUDE.md`. Orchestration doctrine (delegation, relief, cadence, durable
state): **`.planning/ORCHESTRATION.md` — read before dispatching any agent.**

## Do not hand-edit

`.planning/` is GSD-tracked state. **Always enter through a GSD command** (`/gsd-quick`,
`/gsd-execute-phase <n>`, `/gsd-debug`, `/gsd-progress`); never hand-edit code or
planning artifacts outside a GSD-tracked phase or quick. Entry point for "where are we":
`.planning/STATE.md`.

The 2026-04-13 auto-mode bootstrap set `mode: yolo`, `granularity: coarse`, and enabled
all workflow gates (research / plan_check / verifier / nyquist / code_review). **Do not
silently downgrade these.**

## Subagent-dispatch specifics

Full doctrine: `.planning/ORCHESTRATION.md`. Project-specific rules:

- **Never delegate `gh pr checkout` to a bash subagent without isolation.** Bash
  subagents share the coordinator's working tree; `gh pr checkout` switches the branch
  behind its back (caused the cherry-pick mess at `5a91ae2`). Spawn a worktree
  (`git worktree add /tmp/pr-N pr-N-branch`) or have the subagent operate in
  `/tmp/<branch>-checkout`.
- **Orchestration-shaped phases run at top-level, not under `/gsd-execute-phase`.** When
  the work is "fan out → gather → interpret → resolve" rather than "write code → test →
  commit," the top-level coordinator IS the executor (`gsd-executor` lacks `Task`;
  depth-2 spawning is forbidden). Mark such phases `Execution mode: top-level` in ROADMAP.
  Docs-alignment backfill / retroactive audits / stale-doc refresh runs
  (`/reposix-quality-refresh <doc>`) are canonical examples — a pre-push that BLOCKS
  mid-`gsd-execute-phase` is resolved by checkpointing the executor and invoking the slash
  command from a fresh top-level session.
- **Milestone-close 9th probe (RBF-FW-03) is non-skippable.** Any milestone-close missing
  `python3 quality/runners/run.py --cadence pre-release-real-backend` exit 0 grades RED.
  As of P123/DRAIN-03, run.py **self-sources `./.env`** when present (present-only,
  non-clobbering; via `quality/runners/_env_load.py`), so this cadence exercises
  creds-in-`.env` without a manual `set -a; . ./.env; set +a` prefix — closing the
  silent-skip false-green (preflight sourced `.env` but the runner did not). The OP-1
  egress gate is unchanged: a real backend is still hit only when creds are present AND
  `REPOSIX_ALLOWED_ORIGINS` is non-default.
  It runs the vision litmus against the sanctioned real backend (TokenWorld); the catalog
  row `agent-ux/milestone-close-vision-litmus-real-backend` carries `blast_radius: P0` and
  NEVER carries a `waiver`. Verifier: `quality/gates/agent-ux/milestone-close-vision-litmus.sh`;
  verdict skeleton: `quality/dispatch/milestone-close-verdict.md`. Reads NOT-VERIFIED (never
  FAIL/skip-as-pass) when env unset or substrate absent. Exit-code + OD-2 conventions:
  `quality/PROTOCOL.md`.
- **Subjective-rubric dispatch** (cold-reader, install-positioning, headline-numbers):
  `/reposix-quality-review` skill (`bash .claude/skills/reposix-quality-review/dispatch.sh
  --rubric <id>` / `--all-stale` / `--force`). Path A (Task tool) preferred for unbiased
  grading; Path B (`claude -p`) fallback.

## Milestones layout (HANDOVER §0.5 / Option B)

Per-milestone planning artifacts live INSIDE the matching `*-phases/` dir, never loose
at `.planning/milestones/` top level:
```
.planning/milestones/v0.8.0-phases/
├── ARCHIVE.md  ├── ROADMAP.md  ├── REQUIREMENTS.md  └── tag-v0.8.0.sh
```
`freshness-invariants.py` `no-loose-roadmap-or-requirements` BLOCKS any loose
`*ROADMAP*`/`*REQUIREMENTS*` at `.planning/milestones/v*.0-*.md`.
`no-loose-top-level-planning-audits.sh` BLOCKS `*MILESTONE-AUDIT*`/`SESSION-END-STATE*`
loose at `.planning/` top level — such files go under `.planning/milestones/audits/` or
`.planning/archive/`.

## Intake / handover conventions

- Surprises → `SURPRISES-INTAKE.md`; nice-to-haves → `GOOD-TO-HAVES.md` (OP-8 drains
  them in a milestone's last two phases). Milestone-close distills into
  `.planning/RETROSPECTIVE.md` (OP-9).
- Relief/pause handovers use the template in `.planning/ORCHESTRATION.md` §3; the
  `relief-handover-writer` agent writes + commits them. Exemplars:
  `.planning/phases/90-*/90-PAUSE-HANDOFF.md`, `.planning/phases/91-*/91-HANDOVER.md`.
- Push cadence is per-phase: `git push origin main` BEFORE the verifier-subagent
  dispatch; the verifier grades RED if the phase shipped without the push landing —
  AND if main's LATEST CI run is not GREEN after the push. After pushing, run
  `quality/runners/run.py --cadence post-push --persist`; the `code/ci-green-on-main`
  (P0) probe asserts main's newest `ci.yml` run concluded success and rolls the phase
  verdict RED otherwise. Push-landed is the floor; CI-green-on-main-after is the bar.
  Never open the next phase over a red main.
- **Phase-close refreshes the public roadmap strip (dimension: structure).** Every phase
  close MUST refresh the `docs/roadmap.md` "Progress right now" strip — bump the
  phases-closed **fraction + percent**, the milestone **name** if it rolled, the one
  plain-language **capability line**, and the **last-updated date** — in the SAME
  close-bookkeeping commit that advances `STATE.md`/`ROADMAP.md`, so the OWNER sees
  mid-milestone progress instead of a page frozen since the last milestone close. This is
  the *fast* cadence; the mermaid **arcs** still re-color only at *milestone* close — two
  documented cadences (see the SYNC comment atop `docs/roadmap.md`). HARD CONSTRAINTS: the
  strip carries **no phase numbers** (user-facing progressive disclosure — capability
  language only), and its moving lines stay **binding-free** — never let a
  `quality/catalogs/doc-alignment.json` row cite the fraction/percent/capability-line/date,
  or every close re-drifts the binding (P117 W3 `STALE_DOCS_DRIFT` cascade). Tag: this is a
  **structure**-dimension freshness invariant (`quality/gates/structure/`); a machine gate
  that cross-checks the strip against `STATE.md`'s phase count is filed as **GTH-V15-89**
  (prose doctrine here is the enforcement floor until that gate lands).
