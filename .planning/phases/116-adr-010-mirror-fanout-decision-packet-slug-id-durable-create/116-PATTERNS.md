---
phase: 116
slug: adr-010-mirror-fanout-decision-packet-slug-id-durable-create
kind: docs-and-planning-record (no code lands)
mapped: 2026-07-16
---

# Phase 116: ADR-010 mirror-fanout decision packet + slug→id durable-create design - Pattern Map

**Mapped:** 2026-07-16
**Files analyzed:** 5 edit targets (no new files — every target already exists; this phase
amends/extends established prose, it does not invent a new doc shape)
**Analogs found:** 5 / 5 (every edit item has a same-file or sibling-file analog — this is a
docs-only phase with no code/component/service files, so the usual role/data-flow taxonomy
is replaced below with doc-edit-type: amendment-append, cross-link-insert, status-flip,
prose-extend)

## File Classification

| Edit target | Doc-edit-type | Closest analog (same or sibling file) | Match quality |
|---|---|---|---|
| `docs/decisions/010-l2-l3-cache-coherence.md` §2 (ADR-01 mirror-fanout amendment) | amendment-append (dated blockquote after a ratified Decision item) | Same file: `RESOLVED in v0.14.0 (Phase 105)` blockquote at lines 259-279 (appended after item 3, not a rewrite of ratified prose) | exact |
| `docs/decisions/010-l2-l3-cache-coherence.md` §3 (FIX-03 sanctioned-design amendment) | amendment-append (qualify a WAIVED marker, don't remove it) | Same file: the `KNOWN LIMITATION — WAIVED for v0.13.0 (RBF-LR-03)` blockquote (lines 238-257) immediately followed by the same `RESOLVED in v0.14.0` blockquote (259-279) — i.e. the file's own precedent for "waiver stays, qualified by a later ruling" | exact |
| `docs/decisions/010-l2-l3-cache-coherence.md` `## References` (packet cross-link) | cross-link-insert (backtick `.planning/` path, not a markdown hyperlink) | Same file: existing bullet at line 425 — `` `.planning/milestones/v0.13.0-phases/93-cache-coherence/93-DP2-REPRO-NOTES.md` — executed repro + root cause `` | exact |
| root `CLAUDE.md` § "Mirror-head refresh promise" (lines 92-100) | prose-extend (add one sentence, don't rewrite the already-correct claim) | Same section: the bullet immediately above it, "Bus URL / mirror fan-out" (lines 89-91), which already models "compress detail out of root, point at `dvcs-topology.md` + `dvcs-mirror-setup.md`" — the exact move needed to add the webhook+cron blessing sentence | exact |
| `docs/concepts/dvcs-topology.md` § "Cache coherence: L1/L2/L3 (ADR-010)" L1 bullet (lines 174-182) | prose-extend (add authoritative-mechanism sentence + (a)/(b) vocabulary) | Same file: § "Two refs — and where they actually live" (lines 52-93), which already disambiguates "cache-internal ref" vs "external GH mirror repo" in prose (just not with the packet's explicit (a)/(b) labels) — copy its disambiguation move, not its exact wording | role-match |
| `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md:108-116` (litmus-non-idempotency row retirement) | status-flip (terminal STATUS + rationale, never delete) | Archived sibling ledger conventions (see § Pattern Assignments below) — two concrete STATUS strings to copy the shape of | exact |
| `.planning/GOOD-TO-HAVES.md:44-78` (GOOD-TO-HAVES-09 TAG/STATUS update) | status-flip (STATUS + TAG field rewrite, entry body stays) | Sibling row in the SAME repo convention family: `GTH-V15-38` at `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md:288-305` (its "propose as a phase at the then-current milestone boundary" phrasing is the version-agnostic wording GTH-09's stale `TAG: v0.15.0` should be rewritten to match) | exact |
| *(optional, not required)* new `doc-alignment.json` row for the corrected mirror-truth claim | new-catalog-row (only if planner accepts RESEARCH.md §6 Wave-0 recommendation) | `docs-alignment/dvcs-topology-three-roles-bound` row (spot-verified below) + its verifier `quality/gates/docs-alignment/dvcs-topology-three-roles.sh` | exact |

## Pattern Assignments

### 1. `docs/decisions/010-l2-l3-cache-coherence.md` §2 amendment (ADR-01)

**Analog:** same file, `## Decision` section (heading at line 195), item 3's existing
dated-blockquote convention.

**Section structure** (`grep -n "^## \|^### "`, full heading map):
```
195:## Decision
207:  2. **Refresh-path redesign (RBF-LR-02).** ...            <- §2 target: append after this item
221:  3. **`SotPartialFail` recovery semantics (RBF-LR-03).** ...
238:  > KNOWN LIMITATION — WAIVED for v0.13.0 (RBF-LR-03) ...   <- §3 target: qualify this
259:  > RESOLVED in v0.14.0 (Phase 105) — ...                   <- the append-after-ratified-decision precedent
300:## Reversibility / blast-radius assessment
423:## References
```

**Precedent blockquote to copy the shape of** (lines 259-279, verbatim structure):
```markdown
   > **RESOLVED in v0.14.0 (Phase 105) — the *rebase-recovery deep-reconciliation*
   > half of RBF-LR-03 only.** RBF-LR-03 carried two distinct limitations under one
   > tag. The one **now resolved** is ...
   > ... regression-guarded by `quality/gates/agent-ux/rebase-recovery-reconciles.sh`
   > (catalog row `agent-ux/rebase-recovery-reconciles`, graded GREEN at Phase 105
   > close). **STILL OPEN:** the *duplicate-record-on-interrupted-create* waiver
   > above ... remains the v0.14.0 reconciliation-redesign pivot until that
   > exploration converges.
```
Copy this exact shape for the §2 amendment: bold dated header naming the ruling, state
what's now closed, name the regression-guard/catalog row if one exists, explicitly restate
what remains open (Option C / `GTH-V15-38`). RESEARCH.md §2 already drafted the four content
points to land inside this shape — the planner's job is the *placement* (append after item 2,
lines 207-219), not re-deriving the content.

**§2 text being amended (lines 207-219, do not rewrite, only append after):**
```
  2. **Refresh-path redesign (RBF-LR-02).** `refresh_for_mirror_head` stays as the
     full-rebuild `build_from` (already coherent). ... (The fix wave MAY instead choose
     "always refresh, remove the asterisk" ... **the ADR leaves that lever to the
     fix-wave**, since both are honest. Either way, no lying doc.)
```
The 2026-07-16 ruling closes exactly this "leaves that lever to the fix-wave" sentence —
the amendment's first job is stating that closure.

### 2. `docs/decisions/010-l2-l3-cache-coherence.md` §3 amendment (FIX-03)

**Analog:** same file, the WAIVED blockquote (lines 238-257) immediately followed by the
RESOLVED blockquote (lines 259-279) — this pair IS the "qualify, don't remove" precedent.

**WAIVED text being qualified (lines 238-257, verbatim opening + closing):**
```markdown
   > **KNOWN LIMITATION — WAIVED for v0.13.0 (RBF-LR-03).** The clean-convergence
   > contract above holds against the simulator and against any backend that lets
   > the client name its own record id. ... Result: **one duplicate record** the owner
   > hand-deletes on the backend. ... The real fix is a design pivot, not a point patch:
   > model create ... as a **commit sequence with slug→id translation** ... That
   > reconciliation redesign is the **v0.14.0 headline milestone** (owner directive,
   > `.planning/CONSULT-DECISIONS.md` "RBF-LR-03 pivot"). ADR-010 §3's convergence
   > contract is revised only after that exploration converges; until then this marker
   > is the honest boundary.
```
Copy the "cite `.planning/CONSULT-DECISIONS.md` by name + quoted entry label" citation
style (`"RBF-LR-03 pivot"`) for the new amendment's citation of the 2026-07-16 ruling — the
file's own convention is a quoted short-label, not a raw commit SHA alone. Combine with the
commit-SHA convention seen in the P105 blockquote (`commit bd5b9cb` style, from the archived
intake — see § Shared Patterns) since the ruling has both a date and a commit (`8212373`).

### 3. `docs/decisions/010-l2-l3-cache-coherence.md` — packet cross-link

**Analog:** same file's `## References` section (lines 423-431), which already mixes two
citation conventions — copy the correct one for a `.planning/` path:

```markdown
423:## References
424:
425:- `.planning/milestones/v0.13.0-phases/93-cache-coherence/93-DP2-REPRO-NOTES.md` — executed repro + root cause (repro commit `9c46e49`).
426:- `crates/reposix-cache/tests/delta_sync.rs::delta_sync_tree_references_only_resolvable_oids` — ...
...
431:- [ADR-007 — Time-travel via git tags](007-time-travel-via-git-tags.md), [ADR-009 — Stability commitment](009-stability-commitment.md) — cache-is-rebuildable + partial-clone precedent.
```

**The rule to copy:** `.planning/` paths → **backtick code span**, no markdown link (line
425 style — the packet is not mkdocs-served, a `[label](path)` link would 404 on the
rendered site). `docs/` sibling ADR pages → markdown link (line 431 style). The new packet
bullet MUST use the line-425 form:
```markdown
- `.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md` — the ruled options+tradeoffs packet for the §2/§3 amendments above (ruling: `.planning/CONSULT-DECISIONS.md`, 2026-07-16, commit `8212373`).
```

### 4. root `CLAUDE.md` § "Mirror-head refresh promise" (lines 92-100)

**Analog:** the bullet immediately above it in the same list, "Bus URL / mirror fan-out"
(lines 89-91):
```markdown
89:- **Bus URL / mirror fan-out / webhook sync** (`reposix::<sot>?mirror=<url>`, SoT-first
90:  then mirror-best-effort, mirror-lag refs, webhook-driven sync, p95 ≤ 120s): compressed
91:  out of root — see `docs/concepts/dvcs-topology.md` + `docs/guides/dvcs-mirror-setup.md`.
```
This bullet already models the exact move the phase needs for the section below it: name
the mechanism inline, then point at the two detail docs rather than inlining detail (root
CLAUDE.md's progressive-disclosure convention per its own header — "compressed out of
root"). Apply the same move inside the "Mirror-head refresh promise" bullet: add one clause
naming webhook+30-min-cron as authoritative, citing `docs/guides/dvcs-mirror-setup.md`, then
leave the existing (already-correct, per RESEARCH.md §1.1) sentence about `sync --reconcile`
untouched — do not duplicate or contradict it (explicit constraint in CONTEXT.md).

**Existing text NOT to rewrite (lines 92-100, already true):**
```markdown
92:- **Mirror-head refresh promise (qualified, ADR-010 RBF-LR-04).** The mirror-head ref
93:  refreshes on every push that changes the SoT (`files_touched > 0`); a push that
94:  changes nothing in the SoT is a semantic no-op — skipped because there is nothing new
95:  to refresh, not a coherence shortcut. If the external mirror ever lags the SoT (an
96:  out-of-band write moved the backend), catch it up by re-driving an SoT-changing push
97:  through the documented recovery — `git fetch <bus-remote> && git rebase && git push`
98:  (the mirror fan-out refreshes the mirror head on that successful push), NOT `reposix
99:  sync --reconcile`, which rebuilds only the LOCAL cache and leaves the external mirror
100:  head byte-identical. Detail: `docs/concepts/dvcs-topology.md`.
```

### 5. `docs/concepts/dvcs-topology.md` § "Cache coherence: L1/L2/L3 (ADR-010)" (lines 169-198)

**Analog:** same file's § "Two refs — and where they actually live" (lines 52-93), which
already disambiguates the cache-internal ref from the external mirror repo in prose:
```markdown
73:> **Where these refs live and how they travel ...:** `refs/mirrors/...` are always
    written into the **local reposix cache's own bare repo** ...
75:> - **The bus-remote push leg does NOT put them on the mirror.** ...
76:> - **The sync GitHub Action DOES put them on the mirror.** The mirror-sync workflow
    runs a **separate, plain** `git push mirror refs/mirrors/<sot-host>-head
    refs/mirrors/<sot-host>-synced-at` ...
```
Copy this same-file disambiguation move (bold-lead-in + bullet split) for the packet's (a)
cache-ref / (b) external-mirror-repo vocabulary inside the L1 bullet at lines 174-182 — the
file already knows how to write this distinction, it just hasn't applied the packet's exact
labels there yet. The L1 bullet itself (verbatim, current state, do not remove — extend):
```markdown
174:- **L1 — trust the cache as prior.** ... Recovery: `reposix
177:  sync --reconcile` (a full `list_records` walk that rebuilds the cache from the SoT).
179:  Also gates the mirror-head refresh: `refresh_for_mirror_head` runs on every push
180:  that changes the SoT (`files_touched > 0`); a push that changes nothing is a
181:  semantic no-op — skipped because there is nothing new to refresh, not a coherence
182:  shortcut (RBF-LR-04).
```
**Cross-link precedent already in this file** for citing the blessed mechanism doc:
`docs/guides/dvcs-mirror-setup.md` is already linked at line 211 (`## See also`) and at
line 76 inline — reuse that exact path + anchor style, don't invent a new citation form.

### 6. `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md:108-116` — row retirement

**Analog:** this project has NOT yet retired any row inside the *live* v0.15.0 ledger itself
(grepped `STATUS:\*\* RESOLVED|DEFERRED|WONTFIX` against the live file — zero hits; every
row is still `OPEN`). The retirement-string convention must instead be copied from the
**archived** v0.14.0 ledger, which has multiple closed rows using two distinct shapes:

```
part-02.md:63:  **STATUS:** RESOLVED-in-P105, fix commit `bd5b9cb` (`fix(105): helper import writes private ref ns...
part-02.md:195: **STATUS:** RESOLVED, commit `0d05d7f` (`fix(ci): gate release.yml publish on green CI for the...
part-03.md:313: **STATUS:** RULED-DEFER→v0.15.0 (2026-07-13 [MANAGER] Ruling #2, E2/ADR valve,
```

Two reusable shapes:
1. `RESOLVED-in-<milestone/phase>, fix commit `<sha>` (`<commit subject prefix>...`)` — used
   when a code/doc commit closes the row.
2. `RULED-DEFER→<milestone>` / `RULED-<VERDICT>` `(<date> [MANAGER] Ruling #N, ...)` — used
   when a manager ruling (not a fix commit) closes the row's open question.

This row is closed by a **ruling**, not a fix commit landing in `crates/`, so shape 2 is the
closer analog — the planner should write something in the family of:
```markdown
**STATUS:** RESOLVED — 2026-07-16 [MANAGER] ruling (`.planning/CONSULT-DECISIONS.md`,
commit `8212373`): webhook+cron blessed as authoritative external-mirror convergence; the
litmus pre-step remains documented op-recovery, not a product fix; `GTH-V15-38` tracks the
elective Option C upgrade if the pre-step becomes a real incident.
```
matching the file's own entry-format template requirement (line 27: `STATUS: OPEN (← drain
phase updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)`) — this is a
`RESOLVED` (not `DEFERRED`/`WONTFIX`), since the ruling makes a final call rather than
pushing the decision further out.

### 7. `.planning/GOOD-TO-HAVES.md:44-78` — GOOD-TO-HAVES-09 field update

**Analog:** sibling row `GTH-V15-38` (`.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md:288-305`),
which already uses version-agnostic boundary phrasing instead of a hardcoded version tag:
```markdown
- **Fix-sketch:** packet § Decision 1, Option C. If triggered, propose as a phase at the
  then-current milestone boundary; do not fold into an unrelated lane.
```
Current GTH-09 fields to update (verbatim, lines 75-78):
```markdown
75:**TAG:** v0.15.0
76:
77:**STATUS:** DEFERRED — owner scope call, 2026-07-12 (explicit deferral past v0.14.0
78:milestone-close, not a silent slip).
```
Copy `GTH-V15-38`'s "then-current milestone boundary" phrasing for the TAG (avoid
hardcoding `v0.16.0`, which may not match the eventual milestone name), and flip STATUS to
name the 2026-07-16 ruling + "SANCTIONED TARGET DESIGN" per RESEARCH.md §3's recommended
wording. The `**Pointer:**` line (already citing ADR-010 §3, lines 26-28) needs no
structural change — just stays accurate once §3's amendment lands.

### 8. *(optional)* new `doc-alignment.json` row — only if planner accepts the Wave-0 recommendation

**Analog, spot-verified this session** (`jq` query re-run, confirms RESEARCH.md §5.2's
finding is still accurate):
```json
{
  "id": "docs-alignment/dvcs-topology-three-roles-bound",
  "claim": "docs/concepts/dvcs-topology.md ships and explains the three DVCS roles ...",
  "source": { "file": "docs/concepts/dvcs-topology.md", "line_start": 5, "line_end": 11 },
  "tests": ["quality/gates/docs-alignment/dvcs-topology-three-roles.sh"],
  "rationale": "P85 DVCS-DOCS-01 — presence-check verifier asserts the three role tokens + Q2.2 phrase fragments. Body-hash drift on either source or verifier fires STALE_DOCS_DRIFT.",
  "last_verdict": "BOUND",
  "next_action": "BIND_GREEN"
}
```
If minted, follow: `id` = `docs-alignment/<slug>-bound`; `source` anchored at the NEW
prose's own line range (must sit below existing anchors per RESEARCH.md §5.2, i.e. below
L11 in dvcs-topology.md and — since zero rows exist for CLAUDE.md today — anywhere in
CLAUDE.md is safe); `tests` = a new grep-based verifier script mirroring
`quality/gates/docs-alignment/dvcs-topology-three-roles.sh`'s phrase-fragment-presence
style (not exact-hash matching, so prose can be reflowed without breaking the verifier).
Never hand-edit `doc-alignment.json` — mint via `reposix-quality doc-alignment bind` per
`quality/CLAUDE.md`'s catalog-first rule.

## Shared Patterns

### Ruling-citation convention (applies to every amendment/status-flip in this phase)
**Source:** `docs/decisions/010-l2-l3-cache-coherence.md:249-250` and `:254-256` (existing
citations of `.planning/CONSULT-DECISIONS.md` entries)
```markdown
... **owner-signed as WAIVED for v0.13.0 (tag-timing decision T1,
`.planning/CONSULT-DECISIONS.md`)** ...
That reconciliation redesign is the **v0.14.0 headline milestone** (owner directive,
`.planning/CONSULT-DECISIONS.md` "RBF-LR-03 pivot").
```
**Apply to:** every new amendment block and the SURPRISES-INTAKE/GOOD-TO-HAVES status
flips — cite `.planning/CONSULT-DECISIONS.md` by a **quoted short-label** for the entry
(e.g. `"2026-07-16 [MANAGER] P116 ADR-01..."`), plus the date and commit `8212373` per
CONTEXT.md's own instruction ("Ruling text should be cited by date + commit... wherever the
amendments claim authority"). Do not paraphrase the ruling into new meaning — quote it.

### Dated-amendment-blockquote convention (applies to both ADR-010 §2 and §3 amendments)
**Source:** `docs/decisions/010-l2-l3-cache-coherence.md:259-279` (`RESOLVED in v0.14.0
(Phase 105)` blockquote)
**Apply to:** append, never rewrite, ratified `## Decision` prose. Bold dated header →
state what's now closed → name any regression-guard/catalog row if one exists → explicitly
name what remains open/deferred.

### `.planning/`-path citation convention (References sections)
**Source:** `docs/decisions/010-l2-l3-cache-coherence.md:425` vs `:431`
**Apply to:** the packet cross-link. `.planning/` paths = backtick code span (not
mkdocs-served, a markdown link would be dead on the rendered site); sibling `docs/` pages =
markdown hyperlink.

### SURPRISES-INTAKE terminal-status convention
**Source:** `.planning/milestones/v0.14.0-phases/surprises-intake/part-02.md:63,195` (fix-
commit shape) and `part-03.md:313` (ruling shape); template at
`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md:27`
**Apply to:** flip `**STATUS:** OPEN` in place at lines 108-116 to a `RESOLVED` line citing
the ruling — never delete the entry.

### Version-agnostic "next milestone boundary" phrasing
**Source:** `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md:300-301` (`GTH-V15-38`)
**Apply to:** GOOD-TO-HAVES-09's `TAG` field — avoid hardcoding `v0.16.0`.

## No Analog Found

None. Every edit target in this phase's scope is an existing, already-conventioned file;
there are no new files and no code files, so there is no case where the codebase lacks a
precedent to extend. (Contrast with a typical code phase's "No Analog Found" table — not
applicable here.)

## Metadata

**Analog search scope:** `docs/decisions/010-l2-l3-cache-coherence.md` (full file, 432
lines), `docs/concepts/dvcs-topology.md` (lines 1-100, 160-214), root `CLAUDE.md` (lines
85-104), `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (lines 1-140, full
entry-format template + all 2026-07-13/14/15 entries), `.planning/milestones/v0.14.0-
phases/surprises-intake/{part-02,part-03}.md` (STATUS-line grep only), `.planning/GOOD-TO-
HAVES.md:44-78`, `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md:280-310`,
`quality/catalogs/doc-alignment.json` (jq query, `docs/concepts/dvcs-topology.md` binding
re-verified).
**Files scanned:** 9 (5 primary edit targets + 4 analog-only reads)
**Pattern extraction date:** 2026-07-16

## Noticed (not fixed — read-only pattern-mapping pass)

1. **MEDIUM (confirms RESEARCH.md § Noticed #1):** the live v0.15.0
   `SURPRISES-INTAKE.md` has **zero** rows with a terminal STATUS today (all entries
   lines 1-140+ are `OPEN`) — this phase's retirement of the litmus row (lines 108-116)
   will be the **first** terminal-status row in this specific ledger. There is no
   in-ledger precedent to copy; the closest convention comes from the archived v0.14.0
   ledger (two shapes identified above). Worth noting explicitly in the plan so a
   reviewer doesn't expect an in-file precedent that doesn't exist yet.
2. **LOW:** `doc-alignment.json`'s rows are not uniformly shaped — a raw `.source.file`
   jq filter without a type guard throws (`Cannot index array with string "file"`) because
   at least one row's `source` field is an array, not an object, elsewhere in the 399-row
   catalog. Not a defect this phase touches, but any planner/executor script querying this
   catalog directly (rather than via `reposix-quality` tooling) should guard with
   `select((.source|type)=="object")` to avoid a jq crash on unrelated rows.
3. **LOW (confirms RESEARCH.md § Noticed #2):** `GTH-V15-38`'s block in
   `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` has copy-paste-bled duplicate
   "Fix-sketch"/"Effort" lines from the unrelated `GTH-V15-37` entry directly above it —
   confirmed still present this session (same file read for the GTH-V15-38 analog above).
   Adjacent to, not part of, this phase's edit set — filed, not fixed.
