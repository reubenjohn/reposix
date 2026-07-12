---
phase: quick-260712-mhb
plan: 01
subsystem: planning-hygiene
tags: [op-8, progressive-disclosure, milestone-hygiene, surprises-intake]
dependency-graph:
  requires: []
  provides:
    - "SURPRISES-INTAKE.md under the p111 44000 B ceiling (3797 B, was 43988 B)"
    - "surprises-intake/part-01.md + part-02.md (byte-exact verbatim relocation of all 17 v0.14.0 terminal entries)"
    - "p110-surprises-absorption.sh split-aware drained-invariant counting"
  affects:
    - ".planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md"
    - "quality/gates/agent-ux/p110-surprises-absorption.sh"
    - "quality/catalogs/agent-ux.json"
tech-stack:
  added: []
  patterns:
    - "OP-8 file-size-drain via scripts/split_ledger.py (same tool + layout as the v0.13.0 intake split)"
key-files:
  created:
    - ".planning/milestones/v0.14.0-phases/surprises-intake/part-01.md"
    - ".planning/milestones/v0.14.0-phases/surprises-intake/part-02.md"
  modified:
    - ".planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md"
    - "quality/gates/agent-ux/p110-surprises-absorption.sh"
    - "quality/catalogs/agent-ux.json"
decisions: []
metrics:
  duration: "~20 minutes"
  completed: "2026-07-12"
---

# Quick Task 260712-mhb: Progressive-disclosure relief split for SURPRISES-INTAKE.md Summary

One-liner: relocated all 17 terminal v0.14.0 surprise entries verbatim into two OP-8-drain
part files via `scripts/split_ledger.py` (byte-exact round-trip proven twice — the tool's
own internal check plus an independent `cmp`), shrinking the live ledger from 43988 B to
3797 B (12 B headroom to 43988x under the p111 44000 B ceiling), and made the
`p110-surprises-absorption.sh` drained-invariant gate split-aware so it keeps truthfully
counting OPEN/terminal STATUS across the relocated part files instead of silently
regressing to FAIL.

## What Was Built

- **Task 1** — ran `python3 scripts/split_ledger.py .../SURPRISES-INTAKE.md
  --first-entry-line 35 --budget 24000`. Produced exactly 2 parts: `part-01.md` (10 entries,
  21516 B) and `part-02.md` (7 entries, 21574 B). The tool's own built-in round-trip check
  passed (exit 0, "17 entries preserved").
- **Task 2** — rewrote the top-level `## Split index (OP-8 file-size drain)` section to
  add the `date | discovered-by | severity | terminal-status-word` one-liner format (17
  bullets, matching the v0.13.0 convention plus the status-word column). Preamble (lines
  1-33 through `## Entries`) verified byte-identical via `cmp` before installing the
  rewrite. Ran an independent byte-exact proof: concatenating the two part BODIES (each
  minus its 4-line header) reproduces the git-pristine 42688 B entries block exactly
  (`cmp` exit 0, "BYTE-EXACT: part bodies == original entries block"). Final top-level
  file: 3797 B.
- **Task 3** — made `p110-surprises-absorption.sh` split-aware: added a `SCAN_FILES` array
  (top-level file + `surprises-intake/part-*.md` glob, falls back gracefully pre-split) and
  a `FNR==1 { in_fence=0 }` fence-reset rule in both the OPEN_COUNT and TERMINAL_COUNT awk
  blocks so a fence in one file cannot bleed into the next. Updated the
  `agent-ux/p110-surprises-absorption` catalog row: added the part-glob to `sources`
  (the plan called this "evidence" informally — the actual schema field is `sources`,
  confirmed against `quality/catalogs/README.md`) and appended one sentence to `comment`
  documenting the relocation. Did NOT flip `status` or add a `waiver`, per the plan.

## Verification (all required gates green)

- `bash quality/gates/agent-ux/p110-surprises-absorption.sh` → exit 0, `PASS:
  SURPRISES-INTAKE drained (0 OPEN, 17 terminal); honesty spot-check artifact present`.
- `bash quality/gates/agent-ux/p111-milestone-hygiene.sh` → exit 0, `PASS: P111
  milestone-close hygiene — 103-109 terminal markers, Phase 113 reconciled, p93 untracked,
  pre-push doctrine present, ledgers bounded, SURPRISES 0-OPEN`.
- `python3 -c "import json; json.load(open('quality/catalogs/agent-ux.json'))"` →
  `agent-ux.json valid JSON`.
- Byte-exact proof: `cmp` of the recombined part bodies against the git-HEAD-pristine
  entries block → exit 0 (no content lost, nothing summarized).
- SURPRISES-INTAKE.md size: **before 43988 B → after 3797 B** (well under the 44000 B
  p111-hygiene ceiling and the plan's <10000 B target).

## Deviations from Plan

None — plan executed exactly as written, all 3 tasks' verify blocks passed on first run,
no bugs, no blocking issues, no architectural questions. The one minor terminology note:
the plan's Task 3 action text says "add ... to its `evidence` array" but the actual catalog
row schema (and the row itself) uses the field name `sources` — added the part-glob there,
which is the correct field per `quality/catalogs/README.md`'s documented schema (`sources`
is listed; there is no `evidence` field). This is a plan-wording vs. schema-naming mismatch,
not a deviation in substance.

## Commit

`dc3c21a` — `docs(planning): progressive-disclosure split — SURPRISES-INTAKE.md, archive 17
terminal entries (RAISE hygiene)` — 5 files changed (SURPRISES-INTAKE.md rewrite, 2 new part
files, p110-surprises-absorption.sh, agent-ux.json). **Local only, NOT pushed** per task
instructions (no `git push` was run). HEAD (`dc3c21a`) is 1 commit ahead of
`origin/main` (`118478b`).

## Known Stubs

None.

## Threat Flags

None — this is a planning-artifact relocation + a shell health-gate maintenance edit
(per the plan's own threat model: no new runtime code path, no external/tainted input,
no egress surface, no `Tainted<T>` handling).

## Self-Check: PASSED

- `FOUND: .planning/milestones/v0.14.0-phases/surprises-intake/part-01.md`
- `FOUND: .planning/milestones/v0.14.0-phases/surprises-intake/part-02.md`
- `FOUND: .planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md`
- `FOUND: commit dc3c21a` (`git log --oneline --all | grep dc3c21a`)
