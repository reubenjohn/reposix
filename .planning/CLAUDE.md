# .planning/CLAUDE.md — planning-surface rules (auto-loaded under .planning/)

Extends root `CLAUDE.md`. Orchestration doctrine (delegation, relief, cadence, durable
state): **`.planning/ORCHESTRATION.md` — read before dispatching any agent.**

## Do not hand-edit

`.planning/` is GSD-tracked state. **Always enter through a GSD command** (`/gsd-quick`,
`/gsd-execute-phase <n>`, `/gsd-debug`, `/gsd-progress`); never hand-edit code or
planning artifacts outside a GSD-tracked phase or quick. Entry point for "where are we":
`.planning/STATE.md`.

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
  dispatch; the verifier grades RED if the phase shipped without the push landing.
