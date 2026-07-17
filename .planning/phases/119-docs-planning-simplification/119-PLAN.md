---
phase: 119-docs-planning-simplification
plan: 119
type: execute
wave: 1
depends_on: []
files_modified:
  - .planning/milestones/v0.13.0-phases/ROADMAP.md   # SC-4 link fix
  - .planning/ROADMAP.md                             # Item 3 progress-table reconcile
  - .planning/PROJECT.md                             # Item 4 stale self-flag
  - .planning/ORCHESTRATION.md                        # SC-3 split source + Item 1b path fix
  - .planning/ORCHESTRATION-REFERENCE.md              # SC-3 split target (existing sibling)
  - CLAUDE.md                                         # Item 1b OP-8 path fix-twice
autonomous: false        # Wave 3 is a checkpoint:decision (deletion dispositions the audit flagged as live/referenced)
requirements: [DOCS-08, DRAIN-11, DRAIN-25]
user_setup: []

must_haves:
  truths:
    - "ORCHESTRATION.md is under the 20000B structure/file-size-limits ceiling (was 20480B) and ORCHESTRATION-REFERENCE.md stays under 20000B"
    - "The six P79-P84 **Plan:** links in v0.13.0-phases/ROADMAP.md resolve to an existing file (the NN-PLAN-OVERVIEW/index.md directory form)"
    - "The top-level ROADMAP.md Progress table no longer contradicts the phase index for P115/P116/P117"
    - "root CLAUDE.md + ORCHESTRATION.md OP-8 intake references point at the milestone-scoped path form, not a bare black-holing filename"
    - "PROJECT.md no longer self-flags a REQUIREMENTS.md staleness that has already been fixed"
    - "No live/referenced file was deleted; every deletion candidate the audit surfaced is either grep-clean-with-cascade (owner-gated) or explicitly LEFT with evidence"
  artifacts:
    - path: ".planning/ORCHESTRATION.md"
      provides: "orchestration doctrine, back under 20000B"
      contains: "## 3. Context budget + relief/handover protocol"
    - path: ".planning/ORCHESTRATION-REFERENCE.md"
      provides: "moved Handover file template full text"
      contains: "Handover file template"
    - path: ".planning/milestones/v0.13.0-phases/ROADMAP.md"
      provides: "P79-P84 Plan links pointing at the real overview dirs"
      contains: "79-PLAN-OVERVIEW/index.md"
  key_links:
    - from: ".planning/ORCHESTRATION.md §3"
      to: ".planning/ORCHESTRATION-REFERENCE.md § Handover file template"
      via: "pointer replacing the inlined block (preserves every '§3 template' cross-ref)"
      pattern: "ORCHESTRATION-REFERENCE.*Handover file template"
    - from: "CLAUDE.md OP-8 / ORCHESTRATION.md §5"
      to: ".planning/milestones/<active>-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md"
      via: "milestone-scoped intake path"
      pattern: "milestones/.*-phases/(SURPRISES-INTAKE|GOOD-TO-HAVES)"
---

<objective>
Delete stale/superseded legacy planning content OUTRIGHT (git history is the archive — no
redirect stubs, no "moved to…" placeholders) AND fix the doc-truth drift the P118 close
surfaced. This is the "P112 RAISE" cleanup phase.

**Critical audit outcome (verify-against-reality, 2026-07-17):** the concrete inventory
below found that ALMOST EVERY ROADMAP-named deletion target is LIVE or REFERENCED, not the
clean stale delete SC-1/SC-2 assumed (those criteria were written from the 5-day-stale
2026-07-12 reality-check audit). The two handover files became live rotation state; both
catalog JSONs still have live consumers; the 999.* dirs are live backlog homes. Deleting
any of them is a regression under the phase's own hard constraint. Therefore this plan
BAKES the unambiguous clean wins (SC-3, SC-4, fold-items 1b/3/4) and puts every
still-referenced deletion candidate behind an owner-decision gate with a fully-designed
cascade — it does NOT bake a guess.

Purpose: shrink and de-stale the planning surface without regressing a single live link,
rotation file, tool, or gate.
Output: ORCHESTRATION.md back under ceiling; 6 fixed links; a truthful progress table;
milestone-scoped OP-8 paths; a de-staled PROJECT.md; and a concrete, evidence-backed
deletion disposition for the owner.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/CLAUDE.md

# Single-tree-writer discipline: this is a top-level, single-plan phase (like P115).
# Execute tasks in wave order; ONE tree-writer; commit at each coherent boundary.
# NONE of the files touched here is under docs/** or mkdocs.yml, so mkdocs/docs-build
# gates do not fire — BUT every commit is still subject to structure/file-size-limits,
# structure/banned-words, and structure/no-loose-planning-docs. Run
# `python3 quality/runners/run.py --cadence pre-push` before any push.
</context>

<audit_findings>
## Concrete inventory (done during planning — this is the auditable evidence base)

### SC-1 — loose/stale phase-dir artifacts
- `.planning/phases/999.2 … 999.6` — each holds ONLY `.gitkeep`. **NOT stale-deletable —
  LIVE backlog homes.** ROADMAP §Backlog lines 346/355/368/377 define 999.2/999.3/999.5/999.6
  as open backlog phases; `.planning/REQUIREMENTS.md:286-287` explicitly names
  "phases `999.5 docs-crates-md-zero-coverage` / `999.6 docs-alignment-coverage-climb`" as
  the cross-referenced deferred-work home. 999.4 = RESOLVED but cited historically in
  RETROSPECTIVE.md:284,328. → **LEAVE all.**
- `.planning/phases/114 … 118` — active v0.15.0 phases. Not stale. → LEAVE.
- Shipped-milestone transient handovers (the real "archival cascade" candidates):
  `97-HANDOVER.md`(1 ref), `98-HANDOVER.md`(0 ref), `RELIEF-HANDOVER-C2-wave-2.md`(3),
  `-2b.md`(1), `-f.md`(2), `-c.md`(0), `tag-v0.13.0.sh.disabled`(7). Most are cross-referenced
  by intake / good-to-haves / surprises narrative rows (several inside the SC-5 untouchable
  v0.15.0 files). Only `98-HANDOVER.md` and `RELIEF-HANDOVER-C2-wave-c.md` are 0-live-ref —
  marginal value, inside shipped dirs, already archived in git history. → **owner-gated (Wave 3).**

### SC-2 — catalog JSONs + handover transients
- `.planning/v0.11.1-catalog.json` (79662B) — **REFERENCED, NOT clean.** Live consumer:
  `scripts/catalog.py` (declares it "the source of truth", L5/L24/L334) + generated doc
  `.planning/research/v0.11.1/CATALOG-v3.md` + `quality/catalogs/README.md:229`. `catalog.py`
  is NOT invoked by any CI/justfile/runner/hook (grep-confirmed) → dead-subsystem-stale.
  Remaining mentions (CHANGELOG:222, research/v0.12.0, milestone audit) are historical
  narrative (git-history-is-the-archive → do not block deletion). → **owner-gated: retire the
  catalog.py subsystem as a bounded cascade, or defer.**
- `.planning/docs_reproducible_catalog.json` (25916B) — **REFERENCED, NOT clean.** "DEPRECATED
  seed" per `quality/gates/docs-repro/README.md:33`. Live `source:` provenance breadcrumbs in
  `quality/catalogs/release-assets.json` (×6) + `quality/catalogs/docs-reproducible.json` (×2).
  These are provenance pointers, NOT gate-validated (no runner checks source-file existence —
  grep-confirmed). Deleting it leaves 8 dangling breadcrumbs in gate-sensitive catalogs. →
  **owner-gated: delete + repoint the 8 breadcrumbs in the same commit, or defer.**
- `.planning/MANAGER-HANDOVER.md` (19101B) — **LIVE ROTATION STATE, DO NOT DELETE.** Header:
  "live state only". Driven by `.planning/manager-rotate.sh` (the successor prompt reads it
  FIRST). SC-2's "transient" premise is stale (true at the 2026-07-12 audit; false now). → LEAVE.
- `.planning/SESSION-HANDOVER.md` (10907B) — **LIVE, DO NOT DELETE.** Referenced as
  current-rotation ground truth by STATE.md:111,161, ORCHESTRATION.md:177 (§3), PROGRESS.md:51,65,
  RUNBOOK-TO-V1/index.md. → LEAVE.

### SC-3 — ORCHESTRATION.md over ceiling
`wc -c .planning/ORCHESTRATION.md` = **20480B** (480B over the 20000B ceiling; waiver lapses
2026-08-08). A companion `.planning/ORCHESTRATION-REFERENCE.md` (13163B) already exists as the
established progressive-disclosure sibling (D-CONV-7 pointer contract). Split design in
`<split_design>` below. Both files land under ceiling; §-number map preserved.

### SC-4 — six broken P79-P84 links
`.planning/milestones/v0.13.0-phases/ROADMAP.md` links (lines 28/39/45/51/57/63) point at
`NN-…/NN-PLAN-OVERVIEW.md` (FILE form) but the real artifact is a DIRECTORY
`NN-…/NN-PLAN-OVERVIEW/` containing `index.md` (all six dirs verified present). Fix = repoint
each to `NN-PLAN-OVERVIEW/index.md`. NOTE: P78's link (line 22) is CORRECT — `78-PLAN-OVERVIEW.md`
is a real 13949B FILE, not a dir. Scope is exactly P79-P84 (six links); P78 needs no change.

### FOLD Item 1a — top-level GOOD-TO-HAVES.md → BLOCKED by SC-5
`.planning/GOOD-TO-HAVES.md` (11970B) holds 9 LIVE, undrained entries
(GOOD-TO-HAVES-01/09/10/11/12/13/14/15/16; 8 STATUS:OPEN, 1 SANCTIONED). None is mirrored in
v0.15.0-phases/GOOD-TO-HAVES.md (grep-confirmed 0 hits for all 9 short-ids). FOLD Item 1a says
migrate them INTO `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` — but that file is
EXACTLY the 101274B SC-5 UNTOUCHABLE. **Direct conflict: the only migration target is off-limits,
and deleting the top-level file without migrating loses live content.** → **owner-gated (Wave 3);
recommend defer to v0.17 with the split mechanic.** (Item 1b is unaffected and is baked.)

### FOLD Item 3 — progress-table contradiction: CONFIRMED REAL
Top-level ROADMAP.md Progress table (lines 293-295) shows P115 "0/TBD Not started", P116 "0/3
Planned", P117 "0/7 Planned" — but the phase index (lines 68-70) marks all three `[x]` complete,
and P114 (2/2) + P118 (1/1) are shown Complete around them. P115's detail block (line 107) also
still says "Plans: TBD" and P117's detail block (lines 145-151) has 7 unchecked `[ ]` sub-bullets.
→ **eager-fix baked (Task 2).** (The filed SURPRISES-INTAKE row lives in the 78k SC-5 untouchable —
it stays filed; not marked resolved to avoid touching that file.)

### FOLD Item 4 — PROJECT.md stale self-flag: CONFIRMED
PROJECT.md lines 30-33 self-flag that `.planning/REQUIREMENTS.md` "still lists v0.14.0 as
'Active milestone'". REQUIREMENTS.md:1 now reads "Active milestone: v0.15.0 Floor" — the flag is
stale. → **eager-fix baked (Task 2).** LEAVE the "FUSE on Linux only" Out-of-Scope bullet
(line ~44, accurate historical scope).

### docs-alignment binding check (STALE_DOCS_DRIFT trap)
`grep -nE '"(source|file)": "[^"]*(CLAUDE.md|ORCHESTRATION|ROADMAP.md|PROJECT.md|GOOD-TO-HAVES)'
quality/catalogs/doc-alignment.json` → **ZERO line-pinned bindings** into any line this plan
edits. No rebind needed. (The catalog's CLAUDE.md/ROADMAP mentions are `claim`/`rationale` prose
about FUSE-removal and git-2.34 docs — different lines, not touched here.)
</audit_findings>

<split_design>
## SC-3 — ORCHESTRATION.md split (deterministic)

**Move unit:** the "Handover file template" block in §3 — ORCHESTRATION.md lines **141-174**
(intro line "**Handover file template** …" + the fenced ```…``` template + the closing sentence
"The writer confirms the commit SHA in its report.") = **1561 bytes** (measured).

**Destination:** append to `.planning/ORCHESTRATION-REFERENCE.md` as a new section
`## Handover file template (§3 detail)` — matches that file's existing `## <topic> (§N detail)`
header convention (it already hosts §2/§4/§6/§8/§11/§12 details).

**Replacement pointer** left in ORCHESTRATION.md §3 (≈2 lines, ~180B):
> The `relief-handover-writer` agent writes+commits this handover. Full template:
> `.planning/ORCHESTRATION-REFERENCE.md` § "Handover file template (§3 detail)".

**Projected sizes:** ORCHESTRATION.md 20480 − 1561 + ~180 ≈ **19099B** (~900B under ceiling).
ORCHESTRATION-REFERENCE.md 13163 + 1561 + header ≈ **~14.8KB** (well under). Executor MUST verify
with `wc -c` that ORCHESTRATION.md ≤ **19500B** (margin target) and REFERENCE ≤ 20000B.
**Fallback if margin is tight:** also move the §11 "Budgets" elaboration paragraph (lines 272-281,
850B — it already has a REFERENCE pointer) into `## Budget over-fan anti-pattern (§11 detail)`.

**§-number map preserved:** NO section renumbering. §3 stays §3 and still describes the template +
points to REFERENCE. All existing "§3 template" cross-refs therefore STILL resolve through the
pointer chain: `.claude/agents/phase-coordinator.md:7,52`, `.claude/agents/relief-handover-writer.md:11`,
`.planning/CLAUDE.md:66`, `.claude/skills/decision-procedures/SKILL.md:27`,
`.claude/skills/coordinator-dispatch/SKILL.md:87,91`. **No cross-ref file requires editing.**
(Optional polish: append "(full text: ORCHESTRATION-REFERENCE.md § Handover file template)" to
relief-handover-writer.md:11 — nice-to-have, not required; skip if it risks scope creep.)

**No new loose-planning-doc violation:** the destination sibling already exists at
`.planning/ORCHESTRATION-REFERENCE.md` (an allowed path that passes structure/no-loose-planning-docs);
no new file is created.
</split_design>

<deletion_manifest>
## DELETION MANIFEST (tiered — evidence-backed)

**Tier BAKED (unconditional clean delete):** — **NONE.** The audit found no ROADMAP-named target
that is both stale AND unreferenced. This is the honest outcome, not an omission.

**Tier LEAVE (live/referenced — deleting = regression):**
| Path | Why not deletable |
|------|-------------------|
| `.planning/MANAGER-HANDOVER.md` | live rotation state (manager-rotate.sh + header "live state only") |
| `.planning/SESSION-HANDOVER.md` | live current-rotation ground truth (STATE/ORCHESTRATION §3/PROGRESS/RUNBOOK) |
| `.planning/phases/999.2 … 999.6` | live backlog homes (ROADMAP §Backlog + REQUIREMENTS.md:286-287) |
| `.planning/v0.11.1-catalog.json` | live consumer scripts/catalog.py (unless subsystem retired — see gate) |
| `.planning/docs_reproducible_catalog.json` | 8 live source-breadcrumbs in quality catalogs |

**Tier OWNER-GATED (Wave 3 checkpoint — cascade pre-designed, deterministic post-approval):**
1. `v0.11.1-catalog.json` subsystem retirement: delete the JSON + `scripts/catalog.py` +
   `.planning/research/v0.11.1/CATALOG-v3.md`; edit `quality/catalogs/README.md:229` to drop the
   "reads v0.11.1-catalog.json" sentence; leave CHANGELOG/research historical mentions (git-archive).
2. `docs_reproducible_catalog.json` delete: repoint the 8 `source:` breadcrumbs in
   `quality/catalogs/release-assets.json` (×6) + `quality/catalogs/docs-reproducible.json` (×2)
   to a git-history note in the SAME commit; then delete the JSON.
3. Shipped-milestone 0-ref transients: `.planning/milestones/v0.13.0-phases/98-HANDOVER.md` +
   `.planning/milestones/v0.13.1-phases/RELIEF-HANDOVER-C2-wave-c.md` (both 0 live-ref).
4. Top-level `.planning/GOOD-TO-HAVES.md`: BLOCKED by SC-5 (migration target is the 101k untouchable).
   Recommend defer to v0.17 with the split mechanic (co-locate with the file that gets sharded anyway).
</deletion_manifest>

<edit_manifest>
## EDIT MANIFEST (baked — auto tasks)
| # | File | Change | Rebind? |
|---|------|--------|---------|
| SC-4 | `.planning/milestones/v0.13.0-phases/ROADMAP.md` L28/39/45/51/57/63 | `NN-PLAN-OVERVIEW.md` → `NN-PLAN-OVERVIEW/index.md` (×6) | none |
| Item3 | `.planning/ROADMAP.md` L293/294/295 | P115→"1/1 \| Complete \| 2026-07-17"; P116→"3/3 \| Complete \| 2026-07-16"; P117→"7/7 \| Complete \| 2026-07-17" | none |
| Item3 | `.planning/ROADMAP.md` L107 (P115 detail) + L145-151 (P117 detail) | "Plans: TBD"→"1 (115-PLAN.md) — CLOSED GREEN 2026-07-17"; check the 7 `[ ]`→`[x]` | none |
| Item4 | `.planning/PROJECT.md` L30-33 | delete/rewrite the stale "REQUIREMENTS.md still lists v0.14.0" self-flag | none |
| SC-3 | `.planning/ORCHESTRATION.md` + `-REFERENCE.md` | move template block (see split_design) | none |
| Item1b | `CLAUDE.md` L169-170, L276 + `.planning/ORCHESTRATION.md` L198 | bare `SURPRISES-INTAKE.md`/`GOOD-TO-HAVES.md` → milestone-scoped `.planning/milestones/<active>-phases/…` form | none |

docs-alignment: no line-pinned binding hits any edited line (grep-confirmed) → no STALE_DOCS_DRIFT rebind.
</edit_manifest>

<left_untouched>
## LEFT UNTOUCHED (explicit)
- **SC-5:** `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` (101274B) and
  `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (77976B) — the milestone-scoped bloat
  files, deferred to v0.17 (split mechanic, would compete with this phase). **DO NOT TOUCH.**
- **GTH-V15-37** (E1 launch-animation, owner-gated) and **GTH-V15-54** — no touch; no `gh release`.
- `PROJECT.md` "FUSE on Linux only" Out-of-Scope bullet (~L44) — accurate history, leave.
- All Tier-LEAVE files in the deletion manifest.
</left_untouched>

<tasks>

<task type="auto">
  <name>Task 1: SC-4 — fix the six broken P79-P84 Plan links</name>
  <files>.planning/milestones/v0.13.0-phases/ROADMAP.md</files>
  <action>
On lines 28, 39, 45, 51, 57, 63 change each **Plan:** link target from the FILE form to the
DIRECTORY-index form (the real artifact is a `NN-PLAN-OVERVIEW/` directory containing `index.md`):
- L28  `79-poc-reposix-attach-core/79-PLAN-OVERVIEW.md`  → `79-poc-reposix-attach-core/79-PLAN-OVERVIEW/index.md`
- L39  `80-mirror-lag-refs/80-PLAN-OVERVIEW.md`          → `80-mirror-lag-refs/80-PLAN-OVERVIEW/index.md`
- L45  `81-l1-perf-migration/81-PLAN-OVERVIEW.md`        → `81-l1-perf-migration/81-PLAN-OVERVIEW/index.md`
- L51  `82-bus-remote-url-parser/82-PLAN-OVERVIEW.md`    → `82-bus-remote-url-parser/82-PLAN-OVERVIEW/index.md`
- L57  `83-bus-write-fan-out/83-PLAN-OVERVIEW.md`        → `83-bus-write-fan-out/83-PLAN-OVERVIEW/index.md`
- L63  `84-webhook-mirror-sync/84-PLAN-OVERVIEW.md`      → `84-webhook-mirror-sync/84-PLAN-OVERVIEW/index.md`
Do NOT touch line 22 (P78 — `78-PLAN-OVERVIEW.md` is a real file, its link is correct).
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && ok=1; for n in 79 80 81 82 83 84; do d=$(ls -d .planning/milestones/v0.13.0-phases/${n}-*/ | head -1); grep -q "${n}-PLAN-OVERVIEW/index.md" .planning/milestones/v0.13.0-phases/ROADMAP.md && test -f "${d}${n}-PLAN-OVERVIEW/index.md" || ok=0; done; test $ok -eq 1 && echo PASS || { echo FAIL; exit 1; }</automated>
  </verify>
  <done>All 6 links point at `NN-PLAN-OVERVIEW/index.md`; each target file exists; P78 unchanged.</done>
</task>

<task type="auto">
  <name>Task 2: Item 3 + Item 4 — doc-truth reconcile (progress table + stale self-flag)</name>
  <files>.planning/ROADMAP.md, .planning/PROJECT.md</files>
  <action>
ITEM 3 (`.planning/ROADMAP.md`): the Progress table contradicts the phase index. Update rows:
- L293 P115: `0/TBD | Not started | -`  → `1/1 | Complete | 2026-07-17`
- L294 P116: `0/3 | Planned | -`        → `3/3 | Complete | 2026-07-16`
- L295 P117: `0/7 | Planned | -`        → `7/7 | Complete | 2026-07-17`
Then remove the residual detail-block staleness that would otherwise contradict the fixed table:
- P115 detail block (L107): `**Plans**: TBD` → `**Plans**: 1 plan (115-PLAN.md) — CLOSED GREEN 2026-07-17`
- P117 detail block (the 7 `- [ ] 117-0N-PLAN.md …` sub-bullets, ~L145-151): flip each `- [ ]` → `- [x]`.
(If a date differs from source-of-truth in STATE.md, prefer STATE.md's recorded completion date.
Do NOT touch the already-filed SURPRISES-INTAKE row — it lives in the SC-5 untouchable 78k file;
leave it filed.)

ITEM 4 (`.planning/PROJECT.md`, L30-33): REQUIREMENTS.md:1 now reads "Active milestone: v0.15.0
Floor", so the self-flag "The cross-milestone index `.planning/REQUIREMENTS.md` is due for its own
refresh — as of 2026-07-14 it still lists v0.14.0 as 'Active milestone' and v0.13.0 with a stale
shipped-date/phase-range (see Noticing)." is now STALE. Delete that sentence (or rewrite to
"`.planning/REQUIREMENTS.md` was refreshed to Active milestone: v0.15.0 Floor" if surrounding prose
needs a connective). Leave the rest of the blockquote and the "FUSE on Linux only" bullet intact.
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && grep -Eq '115.*1/1 *\| *Complete' .planning/ROADMAP.md && grep -Eq '116.*3/3 *\| *Complete' .planning/ROADMAP.md && grep -Eq '117.*7/7 *\| *Complete' .planning/ROADMAP.md && ! grep -q 'Plans\*\*: TBD' <(sed -n '98,109p' .planning/ROADMAP.md) && ! grep -q 'still lists v0.14.0 as' .planning/PROJECT.md && echo PASS || { echo FAIL; exit 1; }</automated>
  </verify>
  <done>Progress table rows for P115/116/117 read Complete with dates; P115/P117 detail blocks no longer contradict; PROJECT.md stale REQUIREMENTS self-flag removed; FUSE bullet + SC-5 files untouched.</done>
</task>

<task type="auto">
  <name>Task 3: SC-3 split + Item 1b OP-8 fix-twice (grouped doctrine commit)</name>
  <files>.planning/ORCHESTRATION.md, .planning/ORCHESTRATION-REFERENCE.md, CLAUDE.md</files>
  <action>
SC-3 split (see <split_design>): cut ORCHESTRATION.md lines 141-174 (the "**Handover file
template**" intro + fenced block + closing sentence, 1561B). Append them verbatim to
`.planning/ORCHESTRATION-REFERENCE.md` under a new `## Handover file template (§3 detail)` heading.
In ORCHESTRATION.md §3, replace the cut range with a 2-line pointer:
"The `relief-handover-writer` agent writes+commits this handover. Full template:
`.planning/ORCHESTRATION-REFERENCE.md` § \"Handover file template (§3 detail)\"." Do NOT renumber
any §. If `wc -c .planning/ORCHESTRATION.md` > 19500, ALSO move the §11 Budgets elaboration
(lines 272-281, 850B) to `## Budget over-fan anti-pattern (§11 detail)` in REFERENCE, leaving its
existing pointer.

ITEM 1b fix-twice — point OP-8 intake refs at the milestone-scoped path (bare filenames black-hole):
- `CLAUDE.md` L169-170: `Slot 1 drains \`SURPRISES-INTAKE.md\`, Slot 2 drains \`GOOD-TO-HAVES.md\``
  → `Slot 1 drains \`.planning/milestones/<active>-phases/SURPRISES-INTAKE.md\`, Slot 2 drains the
  sibling \`GOOD-TO-HAVES.md\``.
- `CLAUDE.md` L276: `\`SURPRISES-INTAKE\`/\`GOOD-TO-HAVES\`` → `the milestone-scoped
  \`.planning/milestones/<active>-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md\``.
- `.planning/ORCHESTRATION.md` §5 L198: `file to \`SURPRISES-INTAKE.md\` / \`GOOD-TO-HAVES.md\``
  → `file to the milestone-scoped \`.planning/milestones/<active>-phases/{SURPRISES-INTAKE,GOOD-TO-HAVES}.md\``.
(Confirm the exact current text before rewording — line numbers shift after the SC-3 cut, so do the
CLAUDE.md edits and the ORCHESTRATION.md §5 edit by matching the quoted strings, not by line number.)
  </action>
  <verify>
    <automated>cd /home/reuben/workspace/reposix && a=$(wc -c < .planning/ORCHESTRATION.md); b=$(wc -c < .planning/ORCHESTRATION-REFERENCE.md); test "$a" -lt 20000 && test "$b" -lt 20000 && grep -q 'Handover file template' .planning/ORCHESTRATION-REFERENCE.md && grep -q 'ORCHESTRATION-REFERENCE.md' .planning/ORCHESTRATION.md && grep -q 'milestones/<active>-phases' CLAUDE.md && grep -q 'milestones/<active>-phases' .planning/ORCHESTRATION.md && echo "PASS orch=$a ref=$b" || { echo "FAIL orch=$a ref=$b"; exit 1; }</automated>
  </verify>
  <done>ORCHESTRATION.md < 20000B; ORCHESTRATION-REFERENCE.md < 20000B and hosts the template; §-number map intact; OP-8 refs in CLAUDE.md + ORCHESTRATION.md §5 are milestone-scoped. Commit as ONE coherent doctrine commit.</done>
</task>

<task type="checkpoint:decision" gate="blocking">
  <decision>Which still-referenced deletion candidates to execute (each has a pre-designed cascade), and how to resolve the Item-1a / SC-5 conflict.</decision>
  <context>
The inventory found NO ROADMAP-named target that is both stale AND unreferenced. Deleting any of
them without the cascade is a regression under this phase's own hard constraint. See
<deletion_manifest> Tier OWNER-GATED for the full cascade of each. `scripts/catalog.py` is
confirmed NOT CI/justfile/runner/hook-invoked (dead-subsystem). The 8 `docs_reproducible` breadcrumbs
are provenance-only (no runner validates them). The top-level GOOD-TO-HAVES.md's only migration
target is the SC-5-untouchable 101k file.
  </context>
  <options>
    <option id="v0.11.1-subsystem">
      <name>Retire the v0.11.1-catalog.json subsystem</name>
      <pros>Removes 79KB stale JSON + a genuinely dead renderer; SC-2 intent satisfied.</pros>
      <cons>Deletes a CHANGELOG-documented tool (scripts/catalog.py) + generated CATALOG-v3.md; edits quality/catalogs/README.md.</cons>
    </option>
    <option id="docs-repro-delete">
      <name>Delete docs_reproducible_catalog.json + repoint 8 breadcrumbs</name>
      <pros>Removes 26KB deprecated seed; SC-2 intent satisfied.</pros>
      <cons>Edits two gate-sensitive quality catalogs in the same commit; must re-run pre-push to confirm no catalog-integrity regression.</cons>
    </option>
    <option id="zero-ref-transients">
      <name>Delete the two 0-ref shipped-milestone transients (98-HANDOVER, RELIEF-C2-wave-c)</name>
      <pros>Trivially safe (0 live-ref, in git history).</pros>
      <cons>Marginal value; inside shipped-milestone dirs.</cons>
    </option>
    <option id="gth-defer">
      <name>Defer top-level GOOD-TO-HAVES.md to v0.17 (co-locate with the split mechanic)</name>
      <pros>Honors SC-5 (no touch of the 101k file); migrates the 9 live entries into the shards when they're created anyway.</pros>
      <cons>Top-level file lingers one more milestone.</cons>
    </option>
    <option id="leave-all">
      <name>Leave every gated candidate this phase; ship only the baked edits</name>
      <pros>Zero deletion risk; the clean wins still land.</pros>
      <cons>SC-1/SC-2 deletion intent not advanced.</cons>
    </option>
  </options>
  <resume-signal>Owner selects any subset of {v0.11.1-subsystem, docs-repro-delete, zero-ref-transients} to execute now, chooses gth-defer or an SC-5 waiver for Item-1a, or picks leave-all. For any approved deletion, the executor runs the pre-designed cascade in <deletion_manifest>, then re-runs `python3 quality/runners/run.py --cadence pre-push`.</resume-signal>
</task>

</tasks>

<threat_model>
## Trust boundaries
Docs/planning-artifact-only phase. No remote bytes, no `Tainted<T>` flow, no code execution
path, no external mutation. The single trust-relevant surface is the git working tree itself.

## STRIDE register
| Threat ID | Category | Component | Disposition | Mitigation |
|-----------|----------|-----------|-------------|------------|
| T-119-01 | Tampering | deleting a live/referenced file (regression) | mitigate | every deletion is grep-confirmed unreferenced or cascade-updated in the same commit; live files enumerated in Tier-LEAVE and gated |
| T-119-02 | Denial (gate) | ORCHESTRATION split re-introducing a file-size or loose-doc violation | mitigate | reuse existing REFERENCE sibling (no new loose doc); `wc -c` verify both < 20000B; pre-push cadence before push |
| T-119-03 | Information (doc-truth) | STALE_DOCS_DRIFT from editing a doc-alignment-bound line | accept | grep-confirmed zero line-pinned bindings into any edited line |
</threat_model>

<verification>
- `python3 quality/runners/run.py --cadence pre-push` exit 0 before any push (structure/file-size-limits,
  structure/banned-words, structure/no-loose-planning-docs are the gates that can fire here).
- `wc -c .planning/ORCHESTRATION.md .planning/ORCHESTRATION-REFERENCE.md` — both < 20000.
- Markdown link check: each P79-P84 `NN-PLAN-OVERVIEW/index.md` target file exists.
- No SC-5 file (v0.15.0-phases/{GOOD-TO-HAVES,SURPRISES-INTAKE}.md) appears in `git diff --stat`.
- No Tier-LEAVE file appears in `git diff --stat` as a deletion.
</verification>

<success_criteria>
1. ORCHESTRATION.md < 20000B; ORCHESTRATION-REFERENCE.md < 20000B; §-number map intact.
2. Six P79-P84 links resolve to an existing `NN-PLAN-OVERVIEW/index.md`.
3. ROADMAP Progress table + P115/P117 detail blocks no longer contradict the phase index.
4. root CLAUDE.md + ORCHESTRATION.md §5 OP-8 refs are milestone-scoped.
5. PROJECT.md stale REQUIREMENTS self-flag removed.
6. Deletion dispositions resolved per owner ruling; zero live/referenced file deleted; SC-5 untouched.
</success_criteria>

<output>
After completion, create `.planning/phases/119-docs-planning-simplification/119-SUMMARY.md`
recording: which baked edits landed, the owner's deletion ruling + what was deleted (with the
cascade that shipped alongside), final `wc -c` for both ORCHESTRATION files, and the RAISE-LIST
carry-forward for anything the owner deferred (Item-1a/SC-5, ungated catalog JSONs).
</output>
