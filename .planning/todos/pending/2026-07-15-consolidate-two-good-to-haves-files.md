---
created: 2026-07-15T18:22:22Z
title: Resolve two coexisting GOOD-TO-HAVES.md files (doctrine call)
area: planning
files:
  - .planning/GOOD-TO-HAVES.md
  - .planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md
---

## Problem

RAISE #2 (intake-canonicalization ambiguity), filed at P114 closeout, 2026-07-15.

TWO `GOOD-TO-HAVES.md` files coexist with DIFFERENT, non-overlapping numbering schemes:

- `.planning/GOOD-TO-HAVES.md` — uses the spelled-out `GOOD-TO-HAVES-NN` prefix
  (live ids: `GOOD-TO-HAVES-01`, `GOOD-TO-HAVES-09` … `GOOD-TO-HAVES-16`).
- `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` — uses the `GTH-V15-NN`
  milestone-scoped prefix (live ids: `GTH-V15-01` … `GTH-V15-20`).

(Numbering verified against the live files at filing time. The originating RAISE note
approximated these as "GTH-01, GTH-09..16" vs "GTH-V15-01..08"; the real prefixes and
upper bounds differ — see the correction, which strengthens the ambiguity rather than
weakening it, since the two files use genuinely distinct prefixes.)

The canonical-intake destination is AMBIGUOUS:
- Root `CLAUDE.md` says nice-to-haves → `GOOD-TO-HAVES.md` (reads as the root file).
- Milestone-layout doctrine says per-milestone artifacts live inside `*-phases/`
  (which is where the milestone-scoped file lives).

An agent capturing a new nice-to-have today has no unambiguous rule for which file to
append to, risking split/duplicate intake and drift between the two numbering spaces.

## Solution

NEEDS A MANAGER/OWNER DOCTRINE CALL before consolidating — do NOT merge unilaterally
(risks erasing intended structure).

The doctrine call must decide and record:
1. Which file is canonical (root vs milestone-scoped), or whether both are canonical for
   different scopes (durable cross-milestone vs current-milestone).
2. How the two numbering spaces reconcile (`GOOD-TO-HAVES-NN` vs `GTH-V15-NN`).
3. Whether one file should become a pointer/stub to the other, and how root `CLAUDE.md`
   + the milestone-layout doctrine should be reworded so the intake destination is
   unambiguous going forward.

Do NOT touch either file's contents until the doctrine call lands.
