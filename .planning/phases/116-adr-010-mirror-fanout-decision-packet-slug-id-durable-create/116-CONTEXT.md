# Phase 116: ADR-010 mirror-fanout decision packet + slug→id durable-create design - Context

**Gathered:** 2026-07-16
**Status:** Ready for planning
**Source:** PRD Express Path — ruled decision packet
(`.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`)
+ the two verbatim `[MANAGER]` rulings dated 2026-07-16 in `.planning/CONSULT-DECISIONS.md`
(commit `8212373`). No discuss-phase was run: the owner/manager decisions this phase needs
are already recorded — this file encodes them as locked decisions.

<domain>
## Phase Boundary

ROADMAP success criteria 2–4 are **already materially satisfied before planning**: the
decision packet exists (rescued + committed in the P115 phase dir), the slug→id
durable-create design is co-located inside it, both rulings are recorded verbatim in
`.planning/CONSULT-DECISIONS.md`, and FIX-03's v0.15 depth is explicitly scoped
(design-only, NO v0.15 build). Criterion 1 says the packet exists **alongside**
`docs/decisions/010-l2-l3-cache-coherence.md` — the packet currently lives in the P115
phase dir, NOT alongside the ADR; closing that gap (move/copy or explicit
ADR-side cross-link, whichever is cleanest) IS in scope.

The remaining scope of this phase is **executing the two rulings**:

1. **Decision 1 follow-through (ADR-01, Option B with A folded in):** doc-truth rewrite
   of the conflated "mirror" docs; ADR-010 §2 amendment; retirement of the
   litmus-non-idempotency `SURPRISES-INTAKE.md` row.
2. **Decision 2 follow-through (FIX-03, Option A design-only):** ADR-010 §3 amendment
   recording Option B as SANCTIONED TARGET DESIGN, qualifying (not removing) the §3
   waiver, and recording the next-milestone design+build phase proposal.

Explicitly OUT of scope: any implementation of a mirror fan-out option (Option C code,
retry queues, gate changes), any FIX-03 build work, any change to the
`files_touched > 0` gate (Option D REJECTED — the gate STAYS).
</domain>

<decisions>
## Implementation Decisions

### Decision 1 — ADR-01 mirror fan-out = Option B with A folded in (LOCKED, ruled 2026-07-16)
- Doc-truth rewrite of the conflated "mirror" docs: `docs/concepts/dvcs-topology.md`,
  root `CLAUDE.md` (§ "Mirror-head refresh promise"), and the doc-alignment part-02
  row's false claim that "`sync --reconcile` heals the external mirror".
- Webhook + 30-min-cron is BLESSED as the **authoritative** external-mirror convergence
  mechanism — docs must present it as such, not as a workaround.
- `scripts/refresh-tokenworld-mirror.sh` is **manual op-recovery only** — docs must not
  present it as a convergence mechanism.
- Option D REJECTED: keep the `files_touched > 0` mirror-refresh gate exactly as is.
- Option C NOT sanctioned: `GTH-V15-38` holds the verbatim pull-forward trigger;
  do not implement, do not delete/weaken that GTH row.
- Retire the litmus-non-idempotency `SURPRISES-INTAKE.md` row DURING this phase's
  execution (terminal STATUS with rationale pointing at the ruling, not silent deletion).

### Decision 2 — FIX-03 slug→id = Option A this milestone, design-only (LOCKED, ruled 2026-07-16)
- ADR-010 §3 amendment records Option B (slug→id durable-create reconciliation) as
  SANCTIONED TARGET DESIGN.
- The §3 waiver STAYS, qualified (amended wording, not removal).
- Explicitly NO v0.15 build of FIX-03.
- Propose a dedicated design+build phase at the next milestone boundary (durable record —
  e.g. a GOOD-TO-HAVES/backlog row or explicit ADR amendment text — planner picks the
  placement that next-milestone roadmapping actually reads).
- Option D = incident-only stopgap (record as such in the amendment).

### Cross-cutting constraints (LOCKED, standing project rules)
- Docs edits must pass `quality/gates/docs-build/mkdocs-strict.sh` + `mermaid-renders.sh`
  and the reposix-banned-words layer rules for `docs/**`.
- Editing `docs/concepts/dvcs-topology.md` / root `CLAUDE.md` WILL shift line-anchored
  doc-alignment catalog rows — shifted rows must be mechanically rebound (precedent:
  quick `260716-fmt`, commit `97fad0d`) and the pre-push docs-alignment walk must exit 0.
- The part-02 row fix is a claim-text/binding correction in
  `quality/catalogs/doc-alignment.json` — never hand-break catalog JSON; follow the
  catalog conventions in `quality/CLAUDE.md` / `quality/catalogs/README.md`.
- Root `CLAUDE.md` § "Mirror-head refresh promise" was already partially corrected
  (ADR-010 RBF-LR-04 qualification); the rewrite must reconcile with (not duplicate or
  contradict) that text.
- Targeted staging only; push cadence per phase close (push → `run.py --cadence
  post-push --persist` → `code/ci-green-on-main` P0); Bash timeout ≥300s on pushes.
- The 11-row `RETIRE_PROPOSED` human gate (P115) is untouched by this phase — nothing in
  P116 may close, confirm, or count against it.

### Claude's Discretion
- Exact mechanical form of satisfying criterion 1 "alongside" (move vs copy vs ADR
  cross-link + pointer stub), as long as a first-time reader of ADR-010 finds the packet.
- Exact amendment structure inside ADR-010 (§2/§3 amendment blocks vs appended
  "Amendments (2026-07-16)" section) — follow the ADR file's existing conventions.
- Whether `docs/guides/dvcs-mirror-setup.md` / `docs/guides/troubleshooting.md` need
  consistency touch-ups after the rewrite (fix if they contradict the blessed mechanism;
  don't rewrite them wholesale).
- Task/wave decomposition.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Ruling + packet (the contract this phase executes)
- `.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md` — the ruled options+tradeoffs packet (incl. slug→id design)
- `.planning/CONSULT-DECISIONS.md` — the two `[MANAGER]` entries dated 2026-07-16 (authoritative ruling text; quote it, don't paraphrase it into new meaning)

### Surfaces this phase rewrites/amends
- `docs/decisions/010-l2-l3-cache-coherence.md` — ADR-010; §2 (mirror fan-out) + §3 (durable-create waiver) amendments land here
- `docs/concepts/dvcs-topology.md` — primary conflated "mirror" doc to rewrite
- `CLAUDE.md` (repo root) — § "Mirror-head refresh promise (qualified, ADR-010 RBF-LR-04)"
- `quality/catalogs/doc-alignment.json` — part-02 row with the false "`sync --reconcile` heals the external mirror" claim (grep `reconcile`) + line-anchored rows that shift on doc edits
- `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` — litmus-non-idempotency row to retire (grep `litmus` / `idempot`)

### Must stay intact / consistent
- `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` — `GTH-V15-38` (Option C pull-forward trigger, verbatim; do not execute)
- `docs/guides/dvcs-mirror-setup.md` — webhook + cron setup for the blessed mechanism
- `docs/guides/troubleshooting.md` — § DVCS push/pull recovery moves
- `quality/CLAUDE.md` + `quality/catalogs/README.md` — catalog row conventions for the part-02 fix + rebinds
</canonical_refs>

<specifics>
## Specific Ideas

- The rewrite's core truth to land: the SoT-first push fan-out refreshes the mirror head
  only on SoT-changing pushes (`files_touched > 0`, a semantic no-op skip, not a
  coherence shortcut); external-mirror convergence is owned by webhook + 30-min cron;
  `sync --reconcile` rebuilds only the LOCAL cache and never moves the external mirror
  head; the TokenWorld refresh script is a manual operator recovery tool.
- Ruling text should be cited by date + commit (`2026-07-16`, `8212373`) wherever the
  amendments claim authority.
</specifics>

<deferred>
## Deferred Ideas

- FIX-03 Option B build — explicitly ruled NO for v0.15; dedicated design+build phase
  proposed at the next milestone boundary.
- ADR-01 Option C (code-level pull-forward) — trigger held verbatim in `GTH-V15-38`.
</deferred>

---

*Phase: 116-adr-010-mirror-fanout-decision-packet-slug-id-durable-create*
*Context gathered: 2026-07-16 via PRD Express Path (ruled packet + CONSULT-DECISIONS.md rulings)*
