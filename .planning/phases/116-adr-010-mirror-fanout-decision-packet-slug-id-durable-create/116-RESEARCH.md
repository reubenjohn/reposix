# Phase 116: ADR-010 mirror-fanout decision packet + slug→id durable-create design - Research

**Researched:** 2026-07-16
**Domain:** Docs/design-record execution of two already-ruled decisions (no code lands)
**Confidence:** HIGH (every claim below is grep/read/jq-verified against the live repo tree; no training-data guesses about file contents)

## Summary

The ROADMAP goal text for Phase 116 ("produce a decision packet... for the owner to RULE
on") is **stale**. Both rulings already happened — two verbatim `[MANAGER]` entries dated
2026-07-16 in `.planning/CONSULT-DECISIONS.md` (commit `8212373`), encoded as a locked
contract in `116-CONTEXT.md`. This research treats `116-CONTEXT.md` as authoritative and
investigates only the **execution** of the two rulings: an ADR-01 doc-truth rewrite +
ADR-010 §2 amendment + intake-row retirement, and an ADR-010 §3 amendment recording the
FIX-03 target design (no build). No code changes, no new tests against `crates/`.

The most load-bearing finding: **the "false claim" is less broken than the packet's own
framing suggests.** I read every live doc location the ruling names and confirmed with
`grep`/`jq` that root `CLAUDE.md`'s "Mirror-head refresh promise" section and
`docs/concepts/dvcs-topology.md`'s L1/L2/L3 section already correctly scope
`reposix sync --reconcile` to the **local cache** only — they do not claim it heals the
external mirror. What's missing is not a lie to delete but an **explicit blessing**:
neither file names "webhook + 30-min cron" as the *authoritative* external-mirror
convergence mechanism, and neither uses the packet's own disambiguating vocabulary
("(a) cache observability ref" vs "(b) external mirror repo"). The genuinely false,
unambiguous claim — "`--reconcile` is the manual catch-up for a stale mirror" — lives only
in an **archived** v0.14.0 milestone artifact
(`.planning/milestones/v0.14.0-phases/surprises-intake/part-02.md:299-329`, STATUS
DEFERRED), not in any live doc and not in `quality/catalogs/doc-alignment.json` (verified:
zero doc-alignment rows are bound to `CLAUDE.md` at all). `116-CONTEXT.md`'s own
canonical-refs bullet mislabels this as "the doc-alignment part-02 row" — it is a
SURPRISES-INTAKE row, not a catalog row. See § Doc-truth rewrite map for the precise
correction.

Second load-bearing finding: **doc-alignment rebind risk is much lower than the generic
warning implies**, because every existing catalog row bound to the four touched files
anchors at or above the line ranges this phase's content changes, and none of them sit
inside the sections that change. Verified via `jq` query of `doc-alignment.json` (details
in § Gotchas).

Third: `docs/decisions/010-l2-l3-cache-coherence.md` is **already 27,684 bytes**, over the
20,000-char `*.md` file-size-limits ceiling (currently warn-only, waived until
2026-08-08) — amendments must stay terse. `docs/concepts/dvcs-topology.md` is at 18,171
bytes (90.9% of budget) — new prose risks tipping it into the same over-budget (but still
warn-only) tier, growing a tracked debt list; not blocking, but plan for it explicitly.

**Primary recommendation:** treat this as three narrow, surgical edits (not a rewrite):
(1) tighten CLAUDE.md + dvcs-topology.md to explicitly bless webhook+cron and name the two
"mirror" senses distinctly, citing the ruling; (2) append dated amendment blocks to ADR-010
§2 and §3 rather than rewriting the ratified Decision text; (3) satisfy packet
co-location via a backtick-quoted cross-reference from ADR-010 (matching the ADR's own
existing citation style), not a file move — avoiding new mkdocs nav entries and the
`structure/no-orphan-docs` gate entirely.

## Architectural Responsibility Map

This phase has no runtime tiers (no browser/API/DB) — it operates entirely on the
**docs / planning-record surfaces**. Mapping capability → surface for planner sanity-check:

| Capability | Primary surface | Secondary surface | Rationale |
|------------|-----------------|--------------------|-----------|
| Mirror-truth doc correction (ADR-01) | `docs/` (product docs, mkdocs-served) | root `CLAUDE.md` (repo-root grounding, not mkdocs-served) | Both are read by humans/agents; only `docs/` is gated by mkdocs-strict/mermaid/banned-words |
| ADR-010 §2/§3 amendments | `docs/decisions/010-l2-l3-cache-coherence.md` | — | ADRs are the durable, ratified-decision record; amendments append, never rewrite ratified Decision prose |
| Intake-row retirement | `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` (live milestone) | — | Live milestone ledger; NOT the archived v0.14.0 twin |
| FIX-03 next-milestone proposal | `.planning/GOOD-TO-HAVES.md` (GOOD-TO-HAVES-09) | ADR-010 §3 amendment text (cross-ref only) | GTH ledger is what next-milestone roadmapping actually reads at boundary time (OP-8 convention) |
| Packet co-location | `docs/decisions/010-l2-l3-cache-coherence.md` (pointer) | `.planning/phases/115-.../P116-ADR-010-DECISION-PACKET.md` (packet stays put) | See § Packet co-location options |
| Catalog-row correctness (any new doc-alignment rows) | `quality/catalogs/doc-alignment.json` (via `reposix-quality doc-alignment bind`, never hand-edited) | `quality/gates/docs-alignment/` verifier scripts | catalog-first rule (quality/CLAUDE.md) |

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ADR-01 | ADR-010 mirror-fanout decision packet + doc-truth rewrite + §2 amendment (ROADMAP `.planning/REQUIREMENTS.md:151-156`) | § Doc-truth rewrite map, § ADR-010 §2 amendment approach, § Packet co-location options |
| FIX-03 | Slug→id durable-create design-only amendment, NO v0.15 build (`.planning/REQUIREMENTS.md:56-69`, GOOD-TO-HAVES-09 at `.planning/GOOD-TO-HAVES.md:44-78`) | § ADR-010 §3 amendment approach |
</phase_requirements>

<user_constraints>
## User Constraints (from 116-CONTEXT.md)

**116-CONTEXT.md is itself the ruling-derived contract for this phase** (no separate
discuss-phase CONTEXT.md exists — the PRD Express Path produced `116-CONTEXT.md` directly
from the ruled packet + the two `[MANAGER]` entries). Copied verbatim below.

### Locked Decisions

**Decision 1 — ADR-01 mirror fan-out = Option B with A folded in (ruled 2026-07-16):**
- Doc-truth rewrite of the conflated "mirror" docs: `docs/concepts/dvcs-topology.md`, root
  `CLAUDE.md` (§ "Mirror-head refresh promise"), and the false "`sync --reconcile` heals the
  external mirror" claim (location corrected below — see § Doc-truth rewrite map).
- Webhook + 30-min-cron is BLESSED as the **authoritative** external-mirror convergence
  mechanism — docs must present it as such, not as a workaround.
- `scripts/refresh-tokenworld-mirror.sh` is **manual op-recovery only** — docs must not
  present it as a convergence mechanism.
- Option D REJECTED: keep the `files_touched > 0` mirror-refresh gate exactly as is.
- Option C NOT sanctioned: `GTH-V15-38` holds the verbatim pull-forward trigger; do not
  implement, do not delete/weaken that GTH row.
- Retire the litmus-non-idempotency `SURPRISES-INTAKE.md` row during this phase's
  execution (terminal STATUS with rationale pointing at the ruling, not silent deletion).

**Decision 2 — FIX-03 slug→id = Option A this milestone, design-only (ruled 2026-07-16):**
- ADR-010 §3 amendment records Option B (slug→id durable-create reconciliation) as
  SANCTIONED TARGET DESIGN.
- The §3 waiver STAYS, qualified (amended wording, not removal).
- Explicitly NO v0.15 build of FIX-03.
- Propose a dedicated design+build phase at the next milestone boundary (durable record —
  e.g. a GOOD-TO-HAVES/backlog row or explicit ADR amendment text — planner picks the
  placement that next-milestone roadmapping actually reads).
- Option D = incident-only stopgap (record as such in the amendment).

**Cross-cutting constraints (standing project rules):**
- Docs edits must pass `quality/gates/docs-build/mkdocs-strict.sh` + `mermaid-renders.sh`
  and the reposix-banned-words layer rules for `docs/**`.
- Editing `docs/concepts/dvcs-topology.md` / root `CLAUDE.md` WILL shift line-anchored
  doc-alignment catalog rows — shifted rows must be mechanically rebound (precedent: quick
  `260716-fmt`, commit `97fad0d`) and the pre-push docs-alignment walk must exit 0.
- The part-02 row fix is a claim-text/binding correction — never hand-break catalog JSON;
  follow `quality/CLAUDE.md` / `quality/catalogs/README.md` conventions.
- Root `CLAUDE.md` § "Mirror-head refresh promise" was already partially corrected (ADR-010
  RBF-LR-04 qualification); the rewrite must reconcile with (not duplicate or contradict)
  that text.
- Targeted staging only; push cadence per phase close; Bash timeout ≥300s on pushes.
- The 11-row `RETIRE_PROPOSED` human gate (P115) is untouched by this phase.

### Claude's Discretion
- Exact mechanical form of satisfying criterion 1 "alongside" (move vs copy vs ADR
  cross-link + pointer stub), as long as a first-time reader of ADR-010 finds the packet.
- Exact amendment structure inside ADR-010 (§2/§3 amendment blocks vs appended
  "Amendments (2026-07-16)" section) — follow the ADR file's existing conventions.
- Whether `docs/guides/dvcs-mirror-setup.md` / `docs/guides/troubleshooting.md` need
  consistency touch-ups after the rewrite (fix if they contradict the blessed mechanism;
  don't rewrite them wholesale).
- Task/wave decomposition.

### Deferred Ideas (OUT OF SCOPE)
- FIX-03 Option B build — explicitly ruled NO for v0.15; dedicated design+build phase
  proposed at the next milestone boundary.
- ADR-01 Option C (code-level pull-forward) — trigger held verbatim in `GTH-V15-38`.
</user_constraints>

## 1. Doc-truth rewrite map (ADR-01)

### 1.1 What is ALREADY correct (verified by reading the live files)

| File | Lines | Current state |
|------|-------|----------------|
| `CLAUDE.md` (repo root) | 92-100 | § "Mirror-head refresh promise" ALREADY says `git fetch <bus-remote> && git rebase && git push` (re-driving an SoT-changing push) is the recovery, and explicitly states this is "NOT `reposix sync --reconcile`, which rebuilds only the LOCAL cache and leaves the external mirror head byte-identical." **This sentence is already true and not misleading.** |
| `docs/concepts/dvcs-topology.md` | 174-182 (§ "Cache coherence: L1/L2/L3") | L1 bullet says "Recovery: `reposix sync --reconcile` (a full `list_records` walk that rebuilds **the cache** from the SoT)" — scoped to cache, not mirror. Separately mentions the `files_touched > 0` mirror-head gate. Not conflated in the current text. |
| `docs/concepts/dvcs-topology.md` | 84-91 (stderr hint block) | `hint: run \`reposix sync --reconcile\` to refresh **your cache** against the SoT, then \`git pull --rebase\`` — also correctly scoped to cache. |
| `docs/guides/troubleshooting.md` | 233-283 (### Bus-remote `fetch first` rejection) | Line 253: "the `git pull --rebase` fetch reconciles **your cache** against the SoT" — line 255-257 explicitly demotes `sync --reconcile` to optional ("no longer a prerequisite for recovery"). Already correctly scoped. |

**Conclusion:** none of the four canonical rewrite targets currently assert the false
claim "`sync --reconcile` heals the external mirror." Do not plan this as "purge lying
prose" — plan it as "add the missing explicit blessing + vocabulary + citation."

### 1.2 What is MISSING (the actual gap to close)

1. **No file names webhook+cron as *authoritative*.** `docs/guides/dvcs-mirror-setup.md:9`
   describes the mechanism ("a GitHub Action workflow... runs on `repository_dispatch`...
   plus a 30-minute cron safety net") but neither `CLAUDE.md` nor `dvcs-topology.md` cross-
   references it as *the* blessed external-mirror convergence path. Add one sentence to
   each, citing `dvcs-mirror-setup.md` and the ruling (date `2026-07-16`, commit `8212373`).
2. **No file uses the packet's disambiguating vocabulary.** The packet
   (`P116-ADR-010-DECISION-PACKET.md:30-38`) names two distinct things both called
   "mirror": (a) the cache-internal `refs/mirrors/<sot>-head` **observability ref**, and
   (b) the **external GitHub mirror repo**. Neither `CLAUDE.md` nor `dvcs-topology.md`
   uses this (a)/(b) framing explicitly — both use "mirror-head" and "external mirror"
   somewhat interchangeably. Adding the explicit (a)/(b) split (even just as a
   parenthetical) closes the residual ambiguity the ruling exists to close.
3. **`scripts/refresh-tokenworld-mirror.sh`** is not mentioned by name in either
   `CLAUDE.md` or `dvcs-topology.md` as the manual op-recovery tool (it is currently
   documented only in its own header comment and referenced from
   `.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md`). Decision 1(ii)
   requires docs to name it as "manual op-recovery only."

### 1.3 The false-claim historical record — location CORRECTED vs 116-CONTEXT.md

`116-CONTEXT.md`'s canonical-refs section says: *"`quality/catalogs/doc-alignment.json` —
part-02 row with the false '`sync --reconcile` heals the external mirror' claim."* This is
**imprecise** — I verified with `jq` that `doc-alignment.json` has:
- **Zero rows bound to `CLAUDE.md`** (root) — confirmed by querying every row's
  `source.file` for an exact match; none exist.
- **Exactly one row bound to `docs/concepts/dvcs-topology.md`**:
  `docs-alignment/dvcs-topology-three-roles-bound`, anchored at **L5-11** (the "Three
  roles" intro table) — unrelated to the mirror-truth content.
- **Exactly one row bound to `docs/guides/dvcs-mirror-setup.md`**:
  `docs-alignment/dvcs-mirror-setup-walkthrough-bound`, anchored at **L5-11** — also
  unrelated.

The actual "false claim" narrative lives at
**`.planning/milestones/v0.14.0-phases/surprises-intake/part-02.md:299-329`** — an
**archived** (v0.14.0, milestone-closed) SURPRISES-INTAKE entry titled *"Root CLAUDE.md §
'Mirror-head refresh promise' conflates two distinct 'mirrors'; the `reposix sync
--reconcile` manual-catch-up prose is empirically wrong"*, **STATUS: DEFERRED — pending
the B1 mirror-refresh manager decision** (line 328-329). This entry predates the partial
CLAUDE.md fix mentioned in `116-CONTEXT.md` line 73-75 — i.e., it describes an
**older wording** that has since been improved, and its own STATUS field is stale relative
to reality.

**Recommendation:** the planner does NOT need to edit the archived
`v0.14.0-phases/surprises-intake/part-02.md` (archived milestones are frozen; git history
is the archive per this repo's own philosophy). No live artifact needs "correcting" for
this specific claim — it never existed in the current doc-alignment catalog. Treat
`116-CONTEXT.md`'s bullet as informally pointing at 1.2's gap-closure above, not at a
literal catalog-row edit. If completeness is wanted, a **one-line addendum** noting
resolution (with a forward pointer to the P116 commit) could be added to the archived
part-02.md entry — this is optional, low-priority, and should not block phase close.

### 1.4 The row to ACTUALLY retire (verified location + exact lines)

The row `116-CONTEXT.md` really means for retirement is
**`.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md:108-116`** (the LIVE v0.15.0
milestone ledger, not the archived v0.14.0 one):

```
## 2026-07-14 20:42 | discovered-by: L0 rotation #22 (t4 real-backend re-run, pre-release-real-backend cadence) | severity: MEDIUM

**What:** `pre-release-real-backend` cadence needs a documented mirror-refresh pre-step.
The vision-litmus non-idempotency (documented in
`.planning/milestones/v0.14.0-phases/surprises-intake/part-03.md` § litmus non-idempotency,
Manager Ruling #2): the litmus's own successful push re-stales the GitHub mirror it reads,
so a SECOND-run vision-litmus is RED unless `scripts/refresh-tokenworld-mirror.sh` runs
FIRST. ...

**STATUS:** OPEN
```

This is the litmus-non-idempotency row. Retire it by flipping line 116's `**STATUS:**
OPEN` to a terminal value (e.g. `**STATUS:** RESOLVED — see ruling
.planning/CONSULT-DECISIONS.md "2026-07-16 [MANAGER] P116 ADR-01..." — webhook+cron
blessed as authoritative external-mirror convergence; the pre-step remains documented
op-recovery, not a product fix; GTH-V15-38 tracks the elective Option C upgrade`), per the
entry-format template at the top of that same file (line 27: `STATUS: OPEN (← drain phase
updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)`).

Do **not** confuse this with the entries at lines 34-64 (F-K4b tautology, unrelated),
46-64, 66-136 (other unrelated rows) in the same file — I read the full file; line
108-116 is the only mirror/litmus-related OPEN row in the live v0.15.0 ledger.

## 2. ADR-010 §2 amendment approach (mirror)

**Current §2 text to amend (verbatim, `docs/decisions/010-l2-l3-cache-coherence.md:208-219`):**

> 2. **Refresh-path redesign (RBF-LR-02).** `refresh_for_mirror_head` stays as the
>    full-rebuild `build_from` (already coherent). The *honest* reconciliation of the L1
>    promise is the **RBF-LR-04 "keep + qualify"** branch: retain the `files_touched > 0`
>    skip (`write_loop.rs:295`)... but re-document it in `docs/concepts/dvcs-topology.md`
>    as a *semantic* no-op... (The fix wave MAY instead choose "always refresh, remove the
>    asterisk"... **the ADR leaves that lever to the fix-wave**, since both are honest.
>    Either way, no lying doc.)

This §2 text explicitly "leaves that lever to the fix-wave" — Decision 1(iii) of the
2026-07-16 ruling **closes that lever**: keep `files_touched > 0` (Option D REJECTED). The
amendment should:

1. **Not rewrite the ratified 2026-07-05 Decision prose.** Append a dated amendment block
   (the file's own convention already uses dated inline blockquote markers — see the
   "RESOLVED in v0.14.0 (Phase 105)" blockquote inserted into §3 at lines 259-279 as
   direct precedent for how this ADR incorporates a later ruling without rewriting the
   original decision).
2. **State the closure explicitly:** "RBF-LR-04 lever CLOSED (2026-07-16 manager ruling,
   `.planning/CONSULT-DECISIONS.md`, commit `8212373`): `files_touched > 0` gate STAYS
   unconditionally; Option D (unconditional refresh) REJECTED — the no-op perf-skip
   assertion (`perf_l1.rs:386-390`) stands."
3. **Name the authoritative external-mirror mechanism** (this is new information §2 never
   had — §2 was scoped to the cache-internal observability ref only): "webhook + 30-min
   cron (`docs/guides/dvcs-mirror-setup.md`) is BLESSED as the authoritative external-
   mirror convergence mechanism; `scripts/refresh-tokenworld-mirror.sh` remains manual
   op-recovery only."
4. **Record Option C's disposition:** "Option C (post-write snapshot fan-out) NOT
   sanctioned for v0.15 — filed as `GOOD-TO-HAVES.md` `GTH-V15-38` with pull-forward
   trigger 'a real incident or recurring operational friction from the litmus pre-step'
   (already filed, do not re-file or restate the trigger differently)."

**Verified: no code changes required.** `files_touched > 0` at `write_loop.rs:295` (per
ADR-010's own reference list) is UNCHANGED by this ruling — Option D rejection means the
gate stays exactly as shipped. The planner should NOT create any task touching
`crates/reposix-remote/src/write_loop.rs`.

## 3. ADR-010 §3 amendment approach (FIX-03 design-only)

**Current §3 waiver text to qualify, NOT remove (`docs/decisions/010-l2-l3-cache-coherence.md:238-257`):**

> `SotPartialFail` recovery semantics... > **KNOWN LIMITATION — WAIVED for v0.13.0
> (RBF-LR-03).** ... Result: **one duplicate record** the owner hand-deletes... The real
> fix is a design pivot, not a point patch: model create... as a **commit sequence with
> slug→id translation**... That reconciliation redesign is the **v0.14.0 headline
> milestone** (owner directive, `.planning/CONSULT-DECISIONS.md` "RBF-LR-03 pivot"). ADR-010
> §3's convergence contract is revised only after that exploration converges; until then
> this marker is the honest boundary.

And the "RESOLVED in v0.14.0" blockquote immediately after (lines 259-279) already states:
*"STILL OPEN: the duplicate-record-on-interrupted-create waiver above... remains the
v0.14.0 reconciliation-redesign pivot until that exploration converges."* — this is the
exact sentence the new amendment supersedes (the v0.14.0 exploration DID converge, per
`CONSULT-DECISIONS.md` "2026-07-06 [OWNER] RBF-LR-03 pivot" entry, status line: "the
create-partial-fail half... is still unbuilt" as of 2026-07-14, and the 2026-07-16 ruling
now sets its v0.15 depth to design-only).

**What the new §3 amendment block must say:**
1. **Cite the ruling:** "2026-07-16 manager ruling (`.planning/CONSULT-DECISIONS.md`,
   commit `8212373`): OPTION B (durable slug→id map alongside `oid_map`) is the SANCTIONED
   TARGET DESIGN."
2. **Describe the design in build-ready terms** (from the packet, `P116-ADR-010-DECISION-
   PACKET.md:113-120`, Option B row): mint a stable local slug pre-push → model the create
   as "slug X → (pending) → backend id N" → persist this mapping alongside `oid_map`
   (today's non-append-only cache tables are `oid_map` + `meta`,
   `crates/reposix-cache/src/meta.rs:1`) → an interrupted create leaves a pending-slug-
   no-confirmed-id state the retry reconciles against instead of re-creating.
3. **Explicitly state the waiver STAYS**, reworded to reflect the sanctioned direction
   rather than an open question: e.g. "The known limitation above remains WAIVED for
   v0.15.0 — no build lands this milestone. The waiver is now qualified by a chosen
   target design (Option B) rather than an open design question."
4. **State SC4 depth explicitly: NO v0.15 build.** Option D (pending-create intent-log,
   packet row `P116-ADR-010-DECISION-PACKET.md:120`) is recorded as "sanctioned
   REDUCED-SCOPE STOPGAP only if the duplicate hazard materializes in a real incident
   before B lands — decide then, not now" (verbatim from the ruling).
5. **Point at where the next-milestone build proposal durably lives** — see below.

### Where the next-milestone build proposal should live

Verified: `GOOD-TO-HAVES-09` already exists at **`.planning/GOOD-TO-HAVES.md:44-78`**
(root ledger, NOT the v0.14.0 milestone twin at
`.planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md:232-248`, which is an archived
cross-reference only). Its current fields:
- `**TAG:** v0.15.0` (line 75)
- `**STATUS:** DEFERRED — owner scope call, 2026-07-12` (line 77-78)

Both are now stale relative to the 2026-07-16 ruling (which sanctions the design and
explicitly proposes a next-milestone build, not an open "deferred" scope call). No
`v0.16.0-phases/` milestone directory exists yet (only `v0.15.0-phases/` is live;
`.planning/ROADMAP.md` Phase 116-128 are all still inside the CURRENT v0.15.0 milestone).
**Recommendation:** update GTH-09's `STATUS` to something like `SANCTIONED TARGET DESIGN
(Option B) — build proposed at the next milestone boundary, 2026-07-16 ruling` and its
`TAG` to a boundary-relative phrase rather than a hardcoded version — precedent already
exists in this exact file at `GTH-V15-38` (`.planning/milestones/v0.15.0-phases/
GOOD-TO-HAVES.md:300-301`: *"If triggered, propose as a phase at the **then-current
milestone boundary**"*) — matching this wording avoids hardcoding a version number
(`v0.16.0`) that may not match the eventual milestone name. Cross-reference the ADR-010
§3 amendment from the GTH-09 `Pointer:` line (it already cites ADR-010 §3).

## 4. Packet co-location options

**Current state:** the ruled packet lives at
`.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`
(148 lines, committed `da41d7d`). ROADMAP criterion 1 wants it discoverable "alongside"
`docs/decisions/010-l2-l3-cache-coherence.md`.

### Option A — Move the packet file into `docs/decisions/`
E.g. `git mv` to `docs/decisions/010-appendix-mirror-fanout-slug-id-packet.md`.
- **Requires an `mkdocs.yml` nav entry** — `mkdocs build --strict` fails the build on
  files under `docs/` that aren't in the nav (confirmed: this is exactly what
  `quality/gates/docs-build/mkdocs-strict.sh`'s own header comment says --strict catches:
  "missing nav entries"; the `structure/no-orphan-docs` freshness-invariant row wraps the
  same script). New nav entry = new maintenance surface + a place for the "furnished
  product" cold-reader bar (owner ruling `.planning/CONSULT-DECISIONS.md` "2026-07-16
  [OWNER] docs site must read as a furnished product") to judge harshly — an internal
  options-A/B/C/D-with-manager-ruling packet reads as **process documentation**, not
  product documentation, and would look out of place in the public docs nav next to
  install guides and tutorials.
- Breaks the old `.planning/phases/115-.../` path (git history preserves it via `mv`, but
  any other doc/commit message referencing the old path — e.g. `116-CONTEXT.md` itself,
  `CONSULT-DECISIONS.md`'s two `[MANAGER]` entries — would now point at a dead path unless
  also updated).
- **Verified: no doc-alignment row currently binds the old `.planning/phases/115-.../
  P116-ADR-010-DECISION-PACKET.md` path** (doc-alignment's eligible file set is `docs/**/
  *.md` + `README.md` + a short allowlist of archived REQUIREMENTS.md — a `.planning/
  phases/` path is out of scope), so moving it does not orphan a catalog row.

### Option B — Copy the packet into `docs/decisions/`, keep the original
Same nav-entry requirement as A, plus a duplication-drift risk (two copies of ruled,
frozen content). Since neither copy is expected to be edited again (the packet is a
point-in-time ruling record), drift risk is low in practice — but it is still two sources
of truth for one piece of history, which contradicts this repo's stated "git history IS
the archive" philosophy (used repeatedly to justify NOT duplicating content, e.g. root
CLAUDE.md's DOCS-08 requirement, the 2026-07-16 "strip retirement-history narrative"
ruling).

### Option C — Cross-link only, no file move (RECOMMENDED)
Add a short pointer inside `docs/decisions/010-l2-l3-cache-coherence.md` — e.g. a new
bullet in the existing "## References" section (which already cites non-`docs/` planning
paths as **backtick code spans, not markdown hyperlinks** — precedent at line 425:
`` `.planning/milestones/v0.13.0-phases/93-cache-coherence/93-DP2-REPRO-NOTES.md` — executed
repro + root cause`` — this is the ADR's own established convention for citing planning
provenance):

```markdown
- `.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`
  — the ruled options+tradeoffs packet for the §2/§3 amendments below (ruling:
  `.planning/CONSULT-DECISIONS.md`, 2026-07-16, commit `8212373`).
```

- **Zero new files under `docs/`** → no nav entry, no `structure/no-orphan-docs` risk, no
  mermaid/mkdocs-strict surface added.
- **Zero file-size-budget impact** beyond ~2 lines (ADR-010 is already over budget at
  27,684 bytes — Option A/B would add ~5-8KB of packet content or a whole new nav'd page;
  Option C adds a citation line).
- **Matches existing convention exactly** — the file already links out to `.planning/`
  planning artifacts this way for provenance; a first-time reader who reaches the
  References section (which every ADR reader eventually does, since it's where the
  D-P92-03 repro notes and RUNBOOK live) finds the packet path immediately, copy-pasteable
  into `cat`/`git show`.
- **Trade-off:** the packet is not clickable from the *rendered* mkdocs site (a `.planning/`
  path isn't served) — a reader browsing the public docs site sees inert text, not a link.
  This is the SAME trade-off the ADR already accepts for `93-DP2-REPRO-NOTES.md`, so it is
  not a new precedent, just a consistent one. If ROADMAP criterion 1's "alongside" is
  judged by a strict verifier subagent to require more than this, the fallback is to also
  add one sentence to the ADR's own body (not References) naming the packet near the
  Decision-1/Decision-2 amendment blocks themselves, so it surfaces before a reader would
  need to scroll to References.

**Recommendation: Option C.** It satisfies "a first-time reader of ADR-010 finds the
packet" (the discretion note's actual bar) at zero new-file risk, matches the file's own
convention, and avoids conflating a ruling-process artifact with permanent product docs
during the exact milestone window (P117/P119) the owner just set a "furnished product"
quality bar for.

## 5. Gotchas / landmines

### 5.1 File-size-limits ceiling (structure/file-size-limits, `quality/CLAUDE.md`)

Measured (this session, `wc -c`):

| File | Bytes | `*.md` ceiling | % of budget | Status |
|------|-------|----------------|-------------|--------|
| `docs/decisions/010-l2-l3-cache-coherence.md` | 27,684 | 20,000 | 138% | Already OVER-BUDGET (warn-only, waived until 2026-08-08) |
| `docs/concepts/dvcs-topology.md` | 18,171 | 20,000 | 90.9% | EARLY-WARNING band (print-only, never blocks) — new prose risks tipping it over 20,000 |
| `docs/guides/troubleshooting.md` | 27,312 | 20,000 | 137% | Already OVER-BUDGET (same waiver) |
| `CLAUDE.md` (root) | 21,003 | 40,000 | 52.5% | Comfortable — `CLAUDE.md` gets a 40,000 ceiling, not 20,000 |
| `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` | 61,039 | 20,000 | 305% | Already OVER-BUDGET, already tracked (own SURPRISES row notes it grew from 47,291→now larger) |
| `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` | 56,624 | 20,000 | 283% | Already OVER-BUDGET, already tracked |

None of this **blocks** the phase — the over-budget tier is globally waived
(`--warn-only`) until 2026-08-08 (`quality/catalogs/freshness-invariants.json:667`). But:
keep ADR-010's §2/§3 amendments and the CLAUDE.md/dvcs-topology.md prose additions
**terse** (dated blockquote-style amendment blocks, not rewrites) — both to respect
progressive disclosure and because dvcs-topology.md crossing 20,000 bytes silently grows
an already-tracked debt list without a new SURPRISES/GOOD-TO-HAVES filing. If the plan's
edits do push dvcs-topology.md over 20,000, that is a real (if non-blocking) new debt item
— the ownership-charter noticing duty means the executing agent should note it, not let it
pass silently (either fold it into the existing file-size-drain GOOD-TO-HAVES entry or
file a one-line addendum).

### 5.2 Doc-alignment rebind risk — verified LOWER than the generic warning

`116-CONTEXT.md` warns generically that editing these files "WILL shift line-anchored
doc-alignment catalog rows." I verified the actual anchors via `jq` against
`quality/catalogs/doc-alignment.json`:

| File | Existing row(s) | Anchor lines | Relative to this phase's edit zone |
|------|------------------|--------------|--------------------------------------|
| `CLAUDE.md` (root) | **none** | — | No risk — zero bound rows exist |
| `docs/concepts/dvcs-topology.md` | `dvcs-topology-three-roles-bound` | L5-11 | Edit zone is L52-93 + L169-198 — strictly BELOW the anchor; content at L5-11 is untouched, so the row's `source_hash` (computed over that exact span) does not change |
| `docs/guides/dvcs-mirror-setup.md` | `dvcs-mirror-setup-walkthrough-bound` | L5-11 | This file is on the "must stay intact" list — expected untouched |
| `docs/guides/troubleshooting.md` | 6 rows: `fetch-first-error` (L21-46), `conflict-recovery` (L40-42), `reposix-doctor` (L9-19, L11), `blob-limit-recovery` (L61-69), `blob-limit-env-override` (L71-75), `dvcs-troubleshooting-matrix-bound` (L227-227) | All ≤ L75 except the L227 heading anchor | The DVCS mirror-lag section a discretionary touch-up would target is `### Bus-remote fetch first rejection`, **lines 233-283** — strictly below every existing anchor (including L227, which only pins the H2 heading text itself). Edits confined to L233+ carry **zero** rebind risk for any of these 6 rows. |

**Practical upshot:** if the plan keeps all new prose *below* the existing anchors in each
file (which is naturally where the mirror-truth content lives anyway), **no rebind is
required**. Still run `bash quality/gates/docs-alignment/walk.sh` after edits as a
defensive check (never skip it), and if it DOES flag `STALE_DOCS_DRIFT` (e.g. because a
plan choice inserts lines above an anchor), rebind via `/reposix-quality-refresh <doc>` —
never hand-edit `doc-alignment.json` (quality/CLAUDE.md's catalog-first rule).

### 5.3 Banned-words layer scoping

`docs/.banned-words.toml`: `docs/concepts/*.md` (→ `dvcs-topology.md`) and
`docs/guides/*.md` (→ `troubleshooting.md`, `dvcs-mirror-setup.md`) fall under
`[layer.below_fold]`, which bans `FUSE`, `fusermount`, `kernel`, `syscall`, `daemon`,
`inode`, `partial-clone`, `promisor`, `stateless-connect`, `fast-import`, `protocol-v2` in
default-mode scans. `docs/decisions/*.md` (ADR-010) is **not covered by any layer glob**
(only `[p1]`'s global ban on the exact word `replace` applies, everywhere in `docs/`).
Verified: ran `bash quality/gates/structure/banned-words.sh` (default) and `--all` — both
currently exit 0 clean. Concretely: when writing the ADR-010 amendment, avoid the bare
word "replace" (confirmed the linter does exact-word matching, not substring — "replaced"
at ADR-010:215 already passes clean — so prefer "supersedes"/"amends" only if you want to
be safe, but "replaced"/"replaces" as inflected forms are fine). When writing
dvcs-topology.md prose, avoid the Layer-2-banned plumbing vocabulary — none of it is
naturally needed for a mirror-convergence-mechanism explanation, so this should be a
non-issue in practice, but re-run the gate after editing since it's cheap.

### 5.4 Mermaid diagram — already consistent, no change expected

`docs/concepts/dvcs-topology.md:23-48` (the "One picture" mermaid flowchart) already shows
`SoT -. "edit (browser)" .-> Action` and `Action -- "git push --force-with-lease" -->
Mirror` — i.e. it **already** visually depicts the webhook+cron GH Action as the mirror
writer. No diagram edit is needed for this ruling. If the plan does touch text inside the
mermaid code fence for any reason, re-run `mermaid-renders.sh` (HTML-entity leakage into
mermaid blocks is this repo's most-cited docs-build regression class, per the script's own
header comment).

### 5.5 ROADMAP's stale "Execution mode: top-level" annotation

`.planning/ROADMAP.md:120` currently reads: `**Execution mode**: top-level (produce
options + tradeoffs, gather owner/manager ruling; explicitly NOT an implement-and-ship
phase)`. That description matches the OLD (pre-ruling) shape of this phase. The remaining
work (edit 4-5 docs, append 2 ADR sections, flip 1 status field) is a "write → verify →
commit" shape, not a "fan out live sessions → gather → interpret" shape — per
`.planning/CLAUDE.md`'s own routing rule, this now reads as **normal `/gsd-execute-phase`
shape**, not a top-level-only phase. This is a genuine ROADMAP staleness the planner
should flag (not silently work around) — if the plan proceeds as a standard
`/gsd-execute-phase` with waves/tasks, note the mismatch explicitly in the plan so a
reviewer doesn't assume the executor skipped the top-level fan-out step by mistake.

### 5.6 Stray duplicate lines in `GOOD-TO-HAVES.md` near `GTH-V15-38`

Noticed while reading `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md:288-305`
(the `GTH-V15-38` entry this phase must NOT touch/weaken): lines 303-304 contain a
duplicate "Fix-sketch"/"Effort" pair that is clearly copy-paste bleed from the *previous*
unrelated entry (`GTH-V15-37`, the launch-animation checklist — mentions "P117 planner",
"bundle precompile → font self-host", nothing to do with mirror fan-out). This is a doc
hygiene defect adjacent to (but not part of) the row this phase must leave untouched. See
§ Noticed below — filed, not fixed (read-only research + "do not weaken GTH-V15-38" both
argue against touching this file).

## 6. Validation Architecture

### Test framework
This is a docs/design-record phase — there is no `cargo test` surface. "Tests" here are
the existing quality-gate shell verifiers plus (optionally) new `doc-alignment` catalog
rows binding the corrected claims.

| Property | Value |
|----------|-------|
| Framework | `quality/runners/run.py` (catalog-driven shell/mechanical verifiers) — no code-test framework applies |
| Config file | `quality/catalogs/doc-alignment.json`, `quality/catalogs/freshness-invariants.json` |
| Quick run command | `bash quality/gates/docs-alignment/walk.sh && bash quality/gates/docs-build/mkdocs-strict.sh && bash quality/gates/structure/banned-words.sh --all` |
| Full suite command | `python3 quality/runners/run.py --cadence pre-push` |

### Phase Requirements → Verification Map

| Req ID | Behavior | Verification | Command | Exists today? |
|--------|----------|---------------|---------|----------------|
| ADR-01 | CLAUDE.md + dvcs-topology.md explicitly bless webhook+cron and name the two mirror senses | grep for new phrase fragments (e.g. `"webhook + 30-min cron"`, `"authoritative"`) in both files | `grep -F "webhook" CLAUDE.md && grep -F "webhook" docs/concepts/dvcs-topology.md` | ❌ — new grep-level check, not yet a bound catalog row |
| ADR-01 | No doc-alignment regression | pre-push docs-alignment walk exits 0, no new `STALE_DOCS_DRIFT` | `bash quality/gates/docs-alignment/walk.sh` | ✅ — gate exists, wrapper documented in `quality/CLAUDE.md` |
| ADR-01 | mkdocs/mermaid/banned-words still clean after edits | full docs-build + structure sweep | `bash quality/gates/docs-build/mkdocs-strict.sh && bash quality/gates/docs-build/mermaid-renders.sh && bash quality/gates/structure/banned-words.sh --all` | ✅ — all three gates exist and currently pass clean (verified this session) |
| ADR-01 | Litmus-non-idempotency row retired with terminal status | grep for `STATUS:` != `OPEN` at the specific entry | `sed -n '108,116p' .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md \| grep -v 'STATUS:.*OPEN'` | ❌ — row exists (verified), status flip is this phase's deliverable |
| ADR-01 | `files_touched > 0` gate unchanged (Option D rejected → no code diff) | assert zero diff under `crates/` | `git diff --stat -- crates/ \| wc -l` returns 0 for this phase's commits | ✅ — trivially checkable, no new script needed |
| ADR-01 | ADR-010 §2 amendment cites the ruling + Option C/D dispositions | grep for `GTH-V15-38`, `8212373`, `files_touched > 0 gate STAYS`-equivalent text | `grep -F "GTH-V15-38" docs/decisions/010-l2-l3-cache-coherence.md` | ❌ — amendment doesn't exist yet; this is the phase's deliverable |
| FIX-03 | ADR-010 §3 amendment records Option B as sanctioned, waiver stays | grep for `SANCTIONED TARGET DESIGN` AND the still-present WAIVED-limitation substring | `grep -F "SANCTIONED TARGET DESIGN" docs/decisions/010-l2-l3-cache-coherence.md && grep -F "WAIVED for v0.13.0" docs/decisions/010-l2-l3-cache-coherence.md` | ❌ / partial — WAIVED text exists today (verified L238), SANCTIONED text is the deliverable |
| FIX-03 | GOOD-TO-HAVES-09 status/tag updated, next-milestone build proposal recorded | grep for updated STATUS field | `grep -A2 "GOOD-TO-HAVES-09" .planning/GOOD-TO-HAVES.md` | ❌ — row exists (verified L44-78, currently STATUS DEFERRED), update is the deliverable |
| FIX-03 | No v0.15 build lands (no code diff) | assert zero diff under `crates/` | same as ADR-01's code-diff check above | ✅ |
| Both | Packet discoverable from ADR-010 | grep for the packet's exact path string inside ADR-010 | `grep -F "P116-ADR-010-DECISION-PACKET.md" docs/decisions/010-l2-l3-cache-coherence.md` | ❌ — this is the co-location deliverable (§4) |
| Both | RETIRE_PROPOSED 11-row gate untouched | count unchanged | `jq '[.rows[] \| select(.last_verdict=="RETIRE_PROPOSED")] \| length' quality/catalogs/doc-alignment.json` returns `11` before and after | ✅ — verified 11 today; re-run post-edit as a regression check |

### Sampling rate
- **Per task commit:** the "Quick run command" above (walk + mkdocs-strict + banned-words), plus the specific grep assertions for whichever file that task touched.
- **Per wave merge / phase gate:** `python3 quality/runners/run.py --cadence pre-push` full sweep, then the push cadence per `.planning/CLAUDE.md` (`git push origin main` → `run.py --cadence post-push --persist` → `code/ci-green-on-main`).

### Wave 0 gaps
- No new test framework needed (docs-only phase).
- **Recommend (not required) minting a new `doc-alignment.json` row** binding the
  corrected CLAUDE.md/dvcs-topology.md mirror-truth claim (via `reposix-quality
  doc-alignment bind`, never hand-edited) — today NOTHING programmatically guards this
  claim from silently regressing again in the future (zero existing rows cover it, per
  §5.2). This is the single highest-leverage "Don't Hand-Roll" opportunity in this phase:
  without a bound row, a future edit could re-introduce the conflation with no gate
  catching it. If the planner accepts this recommendation, it must land as this phase's
  FIRST commit per the catalog-first rule (`quality/CLAUDE.md`).
- If the planner does NOT mint a new catalog row, the grep-based ad-hoc checks in the
  table above are the fallback verification — weaker (no drift detection over time) but
  sufficient for phase-close grading.

*(If minted: one new catalog row + one new `quality/gates/docs-alignment/*.sh` verifier,
grep-based, mirroring the style of `dvcs-topology-three-roles.sh` read in this session —
phrase-fragment presence checks, not exact-hash matching, "so prose can be reflowed
without breaking the verifier.")*

## Noticed (not fixed)

Per the ownership charter, everything below is surfaced, not silently skipped, and NOT
edited (this research task is read-only):

1. **MEDIUM — `116-CONTEXT.md` canonical-refs mislabels the false-claim location.** It
   names `quality/catalogs/doc-alignment.json` as holding "the part-02 row," but the real
   artifact is `.planning/milestones/v0.14.0-phases/surprises-intake/part-02.md:299-329`
   (a SURPRISES-INTAKE row, not a catalog row) — verified zero doc-alignment rows bind to
   `CLAUDE.md`. Low actual impact (the planner can route around it using § 1.3 above), but
   worth a one-line correction the next time `116-CONTEXT.md` or a successor doc is
   touched, so a future reader doesn't waste time grepping `doc-alignment.json` for a
   phrase that was never there.
2. **LOW — `GOOD-TO-HAVES.md` (v0.15.0-phases) has copy-paste bleed at lines 303-304.**
   Two stray "Fix-sketch"/"Effort" lines belonging to the unrelated `GTH-V15-37`
   (launch-animation) entry are duplicated onto the end of `GTH-V15-38` (mirror-fanout
   Option C). Cosmetic, does not change `GTH-V15-38`'s meaning, but is a genuine hygiene
   defect a future editor of that file should clean up (sketch: delete lines 303-304,
   verify `GTH-V15-38`'s block still parses as one coherent entry, verify `GTH-V15-37`
   above it isn't missing content it needs).
3. **LOW — `docs/concepts/dvcs-topology.md` is at 90.9% of its file-size ceiling** before
   this phase adds anything. Any future addition (not just this phase's) risks tipping it
   into the tracked over-budget list. Not actionable now beyond keeping this phase's own
   addition terse (§5.1); flagging so the next editor of this file has the context.
4. **LOW — ROADMAP Phase 116's "Execution mode: top-level" + success-criteria 1-4 text is
   stale** relative to the ruled reality (§5.5). Not this research's job to fix ROADMAP.md,
   but the planner/executor should not silently "satisfy" the stale criteria text without
   noting the mismatch, since a future reader diffing ROADMAP vs. what actually shipped
   would otherwise be confused about why success criteria don't map cleanly onto the
   actual commits.
5. **INFO — verified, not a defect:** `docs/guides/dvcs-mirror-setup.md` and the mermaid
   diagram in `dvcs-topology.md` are both ALREADY consistent with the ruling (no changes
   needed) — flagging this positively so the planner doesn't spend task budget re-checking
   or re-deriving something already confirmed clean this session.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | A plain backtick-quoted `.planning/` path citation inside ADR-010's References section will satisfy a verifier subagent's reading of ROADMAP criterion 1 ("alongside") without requiring a clickable/nav'd docs-site link. | § 4 Packet co-location, recommendation | If a stricter reading is enforced at phase-close, the plan-checker/verifier could grade this criterion RED, requiring a follow-up task to add an mkdocs nav entry (Option A) — low cost to recover, but would add a wave. |
| A2 | The archived `.planning/milestones/v0.14.0-phases/surprises-intake/part-02.md` entry does not need editing (archived milestones are frozen/git-history-is-the-archive) and `116-CONTEXT.md`'s reference to it is a labeling imprecision, not a literal instruction to edit an archived file. | § 1.3, § Noticed #1 | If the verifier subagent (or the owner) actually wants that archived entry's STATUS field updated for completeness, this is a trivial one-line addendum to add — low risk, easily absorbed as an extra task if flagged wrong. |
| A3 | Updating `GOOD-TO-HAVES-09`'s `TAG`/`STATUS` fields (root `.planning/GOOD-TO-HAVES.md`) is an acceptable durable home for "propose a dedicated design+build phase at the next milestone boundary," rather than requiring new prose in the top-level `.planning/ROADMAP.md` itself. | § 3, next-milestone proposal placement | If next-milestone roadmapping conventionally reads ROADMAP.md rather than GOOD-TO-HAVES.md at boundary time, the proposal could be missed; low risk since GTH-09 is already the established cross-reference for this exact hazard and next-milestone drain phases explicitly consume GOOD-TO-HAVES.md per OP-8. |

## Open Questions

1. **Should the ADR-010 §2/§3 amendments be literal edits to the existing numbered
   Decision list, or appended dated blockquotes (matching the existing "RESOLVED in
   v0.14.0 (Phase 105)" precedent at §3 lines 259-279)?**
   - What we know: the file already has ONE precedent for appending a dated, clearly-
     marked blockquote after a numbered Decision item without rewriting the original
     ratified prose (used for the v0.14.0 RBF-LR-03 partial resolution).
   - What's unclear: whether the planner prefers a NEW top-level "## Amendments
     (2026-07-16)" section instead (cleaner single-location diff, but breaks the file's
     established in-place-blockquote convention).
   - Recommendation: follow the existing in-place blockquote precedent for consistency —
     this is explicitly left as Claude's Discretion in `116-CONTEXT.md`, so this is not a
     blocker, just worth the planner picking deliberately rather than defaulting.
2. **Does the plan-checker / verifier subagent expect a NEW doc-alignment catalog row for
   the corrected mirror-truth claim, or is grep-level verification (§6 table) sufficient
   for phase close?**
   - What we know: zero existing rows cover this claim; minting one is optional per the
     catalog-first rule (not mandated by `116-CONTEXT.md`, which doesn't mention catalog
     rows for the CLAUDE.md/dvcs-topology.md text itself, only for the "part-02 row" it
     mislabels).
   - What's unclear: whether "verify against reality" / "north star" bar implicitly wants
     the stronger guarantee (a bound test that prevents regression) vs. the phase staying
     minimal-scope (docs+design only, no new quality-infra work).
   - Recommendation: mint the row (§6 Wave 0 gaps) — it's cheap (one grep-based verifier,
     following the exact style already used by `dvcs-topology-three-roles.sh`), closes a
     real gap, and directly serves the "noticing is a deliverable" / "don't let a metric
     you generate but don't watch silently decay" standing principles.

## Sources

### Primary (HIGH confidence — read/grepped/jq'd directly this session)
- `.planning/phases/116-adr-010-mirror-fanout-decision-packet-slug-id-durable-create/116-CONTEXT.md` — the locked contract
- `.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md` — full 148-line packet
- `.planning/CONSULT-DECISIONS.md` — both 2026-07-16 `[MANAGER]` entries, verbatim
- `docs/decisions/010-l2-l3-cache-coherence.md` — full file (432 lines)
- `docs/concepts/dvcs-topology.md` — full file (214 lines)
- `CLAUDE.md` (repo root) — Mirror-head refresh promise section + surrounding context
- `docs/guides/troubleshooting.md` — lines 1-90, 225-290 (DVCS section)
- `docs/guides/dvcs-mirror-setup.md` — lines 1-20, 120-150
- `quality/catalogs/doc-alignment.json` — queried via `jq` for `CLAUDE.md`, `dvcs-topology.md`, `dvcs-mirror-setup.md`, `troubleshooting.md` bindings; summary block; RETIRE_PROPOSED count (399 rows total, 11 RETIRE_PROPOSED, alignment_ratio 0.81, coverage_ratio 0.173)
- `quality/gates/docs-alignment/dvcs-topology-three-roles.sh` — verifier implementation (grep-based, phrase-fragment)
- `quality/gates/docs-build/mkdocs-strict.sh` — ran directly; confirms nav/mermaid checks
- `quality/gates/structure/banned-words.sh` — ran directly (default + `--all`), both exit 0
- `docs/.banned-words.toml` — full layer/glob rules
- `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` — lines 1-140 (full litmus row + entry-format template)
- `.planning/milestones/v0.14.0-phases/surprises-intake/part-02.md` — lines 290-333 (the archived false-claim entry)
- `.planning/GOOD-TO-HAVES.md` — lines 1-90 (GOOD-TO-HAVES-09 full entry)
- `.planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md` — grep for GOOD-TO-HAVES-09 (archived cross-ref)
- `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` — lines 280-305 (GTH-V15-38 full entry + stray-lines finding)
- `.planning/REQUIREMENTS.md` — lines 50-70 (FIX-03), 145-157 (ADR-01/Lane 5), 300-320 (status table)
- `.planning/ROADMAP.md` — lines 60-121 (Phase 116-119 + full Phase 116 entry)
- `quality/catalogs/freshness-invariants.json` — grep for `no-loose-roadmap-or-requirements`, `no-orphan-docs`, file-size-limits waiver text
- `quality/CLAUDE.md` — catalog-first rule, docs-alignment dimension, file-size-limits table
- `quality/catalogs/README.md` — lines 85-224 (docs-alignment dimension full spec)
- `mkdocs.yml` — grep for `decisions/` nav entries (confirms ADR-010 is nav'd, no packet entry exists)
- `scripts/refresh-tokenworld-mirror.sh` — header comment (manual op-recovery framing, confirmed)
- `git log` — confirmed commit subjects for `8212373`, `da41d7d`, `97fad0d`, `5a5dd29`
- `wc -c` on all six files in §5.1 (this session, exact byte counts)

### Secondary / Tertiary
None — every claim above traces to a primary source read or command run in this session.
No WebSearch/Context7 lookups were needed (this phase has zero external-library surface).

## Metadata

**Confidence breakdown:**
- Doc-truth rewrite map: HIGH — every cited claim is a direct file read or grep run this session; the "already correct" finding was cross-checked against three separate files independently.
- ADR-010 §2/§3 amendment approach: HIGH — full ADR-010 text read; exact line numbers verified.
- Packet co-location: HIGH (mechanics) / MEDIUM (which option the planner/verifier ultimately prefers is a judgment call, flagged as Open Question 1... actually Assumption A1).
- Gotchas: HIGH — every gate mentioned was either read or actually executed (mkdocs-strict, banned-words ran clean; doc-alignment queried via jq).
- Validation Architecture: MEDIUM — the grep-level checks are HIGH confidence (directly executable today); the "mint a new catalog row" recommendation's exact verifier shape is a design proposal, not yet built.

**Research date:** 2026-07-16
**Valid until:** 30 days (docs-only phase; low churn expected, but the 2026-08-08 file-size waiver expiry and the ongoing v0.15.0 milestone phases 117-128 could shift line numbers in the touched files before this phase executes — re-verify anchors if execution is delayed past ~1 week).
