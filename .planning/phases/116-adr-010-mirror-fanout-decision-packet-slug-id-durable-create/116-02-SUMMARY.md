---
phase: 116-adr-010-mirror-fanout-decision-packet-slug-id-durable-create
plan: 02
subsystem: docs/decisions
tags: [adr-010, adr-01, fix-03, mirror-fanout, slug-to-id, durable-decision-record]
requires:
  - "P116 manager rulings (2026-07-16, commit 8212373) in .planning/CONSULT-DECISIONS.md"
  - "Plan 116-01 mirror-convergence blessing (commits a1cc2d4 / 7412833)"
provides:
  - "ADR-010 §2 durable record: RBF-LR-04 lever CLOSED (files_touched>0 STAYS, Option D REJECTED)"
  - "ADR-010 §3 durable record: Option B = SANCTIONED TARGET DESIGN, design-only, waiver qualified-not-removed"
  - "ADR-010 References cross-link to the P116 decision packet (closes ROADMAP criterion 1)"
affects:
  - docs/decisions/010-l2-l3-cache-coherence.md
tech-stack:
  added: []
  patterns:
    - "Dated-amendment-blockquote convention (append, never rewrite ratified prose)"
    - "backtick .planning/ path (not markdown hyperlink) for non-mkdocs-served cross-links"
key-files:
  created:
    - .planning/phases/116-adr-010-mirror-fanout-decision-packet-slug-id-durable-create/116-02-SUMMARY.md
  modified:
    - docs/decisions/010-l2-l3-cache-coherence.md
decisions:
  - "Criterion 1 satisfied by cross-link, NOT a file move — packet stays in the P115 dir (avoids an mkdocs nav entry / product-vs-process-artifact conflation during the P117/P119 furnished-product window)"
  - "FIX-03 is design-only: zero crates/ diff asserted; no v0.15 build"
metrics:
  duration: ~15m
  completed: 2026-07-16
---

# Phase 116 Plan 02: ADR-010 durable decision record (ADR-01 + FIX-03) Summary

Three terse, append-only dated blockquotes to `docs/decisions/010-l2-l3-cache-coherence.md`
durably record both 2026-07-16 manager rulings and close ROADMAP criterion 1 — ratified
`## Decision` prose stays byte-unchanged; zero `crates/` diff.

## What shipped

- **T1 — §2 RBF-LR-04 lever CLOSED (ADR-01).** A dated blockquote appended after Decision
  item 2 records that the ruling (commit `8212373`) settles the "leaves that lever to the
  fix-wave" question: `files_touched > 0` STAYS unconditionally, Option D REJECTED, the no-op
  perf-skip (`perf_l1.rs:386-390`) stands. Names webhook + 30-min cron
  (`docs/guides/dvcs-mirror-setup.md`) as the BLESSED authoritative external-mirror
  convergence mechanism (new info §2 never had — §2 was scoped to the cache-internal
  observability ref), `refresh-tokenworld-mirror.sh` as manual op-recovery only, and records
  Option C's disposition as `GTH-V15-38` without restating its trigger. Cites Plan 116-01's
  `mirror-convergence-blessed.sh` guard.
- **T2 — §3 Option B SANCTIONED TARGET DESIGN, design-only (FIX-03).** A dated blockquote
  appended after the RESOLVED block supersedes the "STILL OPEN … v0.14.0 pivot" sentence by
  naming Option B (durable slug→id map alongside `oid_map`) as the SANCTIONED TARGET DESIGN,
  with the build-ready slug→(pending)→backend-id shape. The known limitation stays WAIVED for
  v0.15.0 (SC4 depth: design-only, NO build), now qualified by a chosen direction; Option D is
  the incident-only reduced-scope stopgap; cross-refs `GOOD-TO-HAVES-09`. The existing L238
  "WAIVED for v0.13.0" marker is untouched (still greps).
- **T3 — References cross-link.** One backtick-code-span bullet to
  `.planning/phases/115-live-mcp-benchmark-re-measurement/P116-ADR-010-DECISION-PACKET.md`
  (NOT a markdown hyperlink — `.planning/` is not mkdocs-served; a link would 404 the docs
  build). Satisfies ROADMAP criterion 1 via cross-link; no file move.

## Verification (all GREEN)

- T1: `grep -F GTH-V15-38` + `grep -F 8212373` + `files_touched > 0 STAYS` / `Option D … REJECTED` all hit.
- T2: `grep -F "SANCTIONED TARGET DESIGN"` + `grep -F "WAIVED for v0.13.0"` + `grep -F GOOD-TO-HAVES-09` all hit.
- T3: `grep -F P116-ADR-010-DECISION-PACKET.md` hits.
- `git diff --stat -- crates/ | wc -l` == 0 (design-only; no build).
- Diff is pure-insertion: 0 deletion lines, 36 addition lines → ratified `## Decision` prose byte-unchanged.
- `bash quality/gates/docs-build/mkdocs-strict.sh` exit 0.
- `bash quality/gates/structure/banned-words.sh` (`--all`) exit 0 (additions use "supersedes"/"amends", never the bare word "replace").

## Deviations from Plan

None — plan executed exactly as written. All three appends landed at the specified insertion
points; no architectural changes, no auto-fixes required.

## Noticed (ownership deliverable)

1. **LOW — banned-words scope gap (informational, not a defect this plan owns).**
   `scripts/banned-words-lint.sh` scans only `index.md`, `concepts/`, `tutorials/`, `guides/`
   (and `how-it-works/` under `--all`). `docs/decisions/**` is entirely out of scope, so the
   P1 "replace" ban is NOT enforced on ADRs — which is why the pre-existing "replaced" at
   `010-…:215` passes today. The charter's "avoid replace" rule was honored by convention
   here, but a reviewer expecting the gate to catch it on decisions/ would be surprised. Not
   filing — it may be intentional (ADRs are Layer-3-equivalent reference prose) — but flagging
   for the phase verifier's awareness.
2. **LOW (confirms PATTERNS Noticed #3).** `GTH-V15-38`'s block in
   `.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md` still carries copy-paste-bled
   duplicate "Fix-sketch"/"Effort" lines bled from `GTH-V15-37` above it. Adjacent to, not
   part of, this plan's edit set — left for the GOOD-TO-HAVES drain lane (already filed in
   PATTERNS/RESEARCH Noticed, no new filing needed).

## Self-Check: PASSED

- `docs/decisions/010-l2-l3-cache-coherence.md` — FOUND, all three appends present (greps above).
- `.planning/phases/116-…/116-02-SUMMARY.md` — FOUND (this file).
- Commit hash recorded in the final report.
