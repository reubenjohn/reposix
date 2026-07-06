---
phase: 260706-rbf-rbf-lr-03-known-limitation
plan: 01
type: docs
autonomous: true
subsystem: docs (ADR-010 + troubleshooting + dvcs-topology)
requirements: []
---

# Quick 260706-rbf: RBF-LR-03 honest known-limitation across three docs

## Objective

Make the v0.13.0 "ship now" (T1, `.planning/CONSULT-DECISIONS.md` lines 59-75) tag
decision honest by documenting RBF-LR-03 as an out-loud WAIVED-for-v0.13.0 known
limitation. No code change — this is a documentation-only honesty edit. The existing
clean-convergence contract in ADR-010 §3 is CORRECT for the sim and for the normal
retry path; the gap is narrow (real backend + create interrupted mid-batch before
slug→ID reconciliation completes → one hand-deletable duplicate on retry) and is
slated for the v0.14.0 reconciliation-redesign pivot (CONSULT-DECISIONS.md lines 13-57).

## Context

- @.planning/CONSULT-DECISIONS.md — RBF-LR-03 pivot (13-57) + T1 tag-timing (59-75): canonical wording.
- @docs/decisions/010-l2-l3-cache-coherence.md — §3 clean-convergence contract (do NOT touch).

## Tasks

1. **type=auto** — ADR-010 §3: ADD a WAIVED-for-v0.13.0 known-limitation marker
   ALONGSIDE (not replacing) the existing convergence contract. Real backend +
   mid-batch create network drop → one hand-deletable duplicate on retry, because
   reconciliation is not yet commit-sequence/slug-based. Point to the v0.14.0 pivot.
   Optional light pointer at the ~lines 310-313 RBF-LR-03 mention.
2. **type=auto** — troubleshooting.md new subsection under "## DVCS push/pull issues":
   user-facing operational phrasing — on a real backend a create interrupted mid-batch
   may leave a duplicate on retry; check before re-pushing; recovery = hand-delete the
   duplicate. Teach recovery.
3. **type=auto** — dvcs-topology.md "Out of scope (intentionally)" list: ONE line
   cross-referencing this as a documented v0.13.0 known limitation slated for v0.14.0.

## Verification / success criteria

- `bash quality/gates/docs-build/mkdocs-strict.sh` exit 0 (SEEN, not assumed).
- `bash quality/gates/docs-build/mermaid-renders.sh` exit 0.
- `scripts/banned-words-lint.sh` PASS on concepts + guides edits (Layer 2: no
  plumbing words in the two edited pages).
- ADR-010 §3 clean-convergence contract prose unchanged; marker is ADDITIVE.
- Commit atomically, then `git push origin main` before reporting.

## Output

`.planning/quick/260706-rbf-rbf-lr-03-known-limitation/260706-rbf-SUMMARY.md`.
