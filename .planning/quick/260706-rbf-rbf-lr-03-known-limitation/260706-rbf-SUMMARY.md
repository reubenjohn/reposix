---
phase: 260706-rbf-rbf-lr-03-known-limitation
plan: 01
subsystem: docs (ADR-010 + troubleshooting + dvcs-topology)
tags: [docs, dvcs, rbf-lr-03, known-limitation, honesty, quick]
requires: []
provides:
  - docs/decisions/010-l2-l3-cache-coherence.md (WAIVED-for-v0.13.0 known-limitation marker in §3, additive)
  - docs/guides/troubleshooting.md (user-facing duplicate-after-interrupted-create recovery subsection)
  - docs/concepts/dvcs-topology.md (one-line out-of-scope cross-reference)
affects: []
tech-stack:
  added: []
  patterns: []
key-files:
  created: []
  modified:
    - docs/decisions/010-l2-l3-cache-coherence.md
    - docs/guides/troubleshooting.md
    - docs/concepts/dvcs-topology.md
decisions:
  - Marker is ADDITIVE — the existing clean-convergence contract prose in ADR-010 §3 is byte-unchanged; the WAIVED blockquote sits after it (correct for sim + client-id backends, honest boundary for id-reassigning real backends).
  - Troubleshooting subsection leads with recovery (check-before-repush + hand-delete + sync --reconcile), not just a warning, per OD-3 "teach recovery."
  - Cross-doc anchor slug hand-verified against mkdocs toc slugify (v0.13.0 → v0130) and confirmed by link-resolution gate (0 broken links).
metrics:
  duration: ~15 minutes
  completed: 2026-07-06
---

# Quick 260706-rbf: RBF-LR-03 honest known-limitation across three docs Summary

One-liner: made the v0.13.0 T1 "ship now" tag decision honest by documenting RBF-LR-03 out loud as a WAIVED-for-v0.13.0 known limitation across ADR-010 §3, troubleshooting, and dvcs-topology — narrow (real backend + mid-batch-create network drop → one hand-deletable duplicate on retry), recoverable, and slated for the v0.14.0 reconciliation-redesign pivot. No code change.

## The three edits

1. **`docs/decisions/010-l2-l3-cache-coherence.md` §3** — ADDED a `> **KNOWN LIMITATION — WAIVED for v0.13.0 (RBF-LR-03).**` blockquote immediately after the existing convergence contract (which is unchanged). States the gap is narrow, recoverable (owner hand-deletes the duplicate; no data loss / no torn cache), owner-signed as WAIVED (T1, not suppressed), and points at the commit-sequence/slug→id v0.14.0 pivot. Also added a one-clause light pointer at the shorter RBF-LR-03 mention (Consequences item 3) tying the sim clean-convergence test to the WAIVED real-backend case.
2. **`docs/guides/troubleshooting.md`** — new `### Duplicate record after an interrupted create (real backend, v0.13.0 known limitation)` subsection under "## DVCS push/pull issues". User-facing: symptom (two copies after a dropped create + retry), what it means (documented limitation, not user error, no data loss), and a recovery recipe (check for the duplicate first, hand-delete on the backend UI, `reposix sync --reconcile` + `git fetch`), plus the v0.14.0 fix pointer.
3. **`docs/concepts/dvcs-topology.md`** — one line appended to "Out of scope (intentionally)" cross-referencing the documented v0.13.0 known limitation and the v0.14.0 pivot, linking both troubleshooting and ADR-010.

## Deviations from Plan

None — plan executed exactly as written. All edits additive; no existing contract prose modified.

## Verification (SEEN, not assumed)

- `scripts/banned-words-lint.sh` → `✓ banned-words-lint passed (default mode).` EXIT=0
- `bash quality/gates/docs-build/mkdocs-strict.sh` → `OK: docs site clean` MKDOCS_EXIT=0
- `bash quality/gates/docs-build/mermaid-renders.sh` → `✓ check-mermaid-renders: 6 source-mermaid pages all have valid artifacts.` EXIT=0
- `python3 quality/gates/docs-build/link-resolution.py` → `0 broken link(s) across 25 file(s)` EXIT=0 (validates the two new cross-doc anchor links)

## Noticing (OD-3 deliverable)

- ADR-010 Consequences item 4 (line ~319) already flags a stale/dangling cross-reference: `docs/guides/troubleshooting.md:352` points at a dvcs-topology "Out of scope" anchor and says L3 "defers to v0.14.0" which the ADR overrides. Checked line 352 as it stands now — it correctly reads "L3 (transactional cache writes) is shipped … Only L2 … remains deferred," so that specific staleness was already fixed by the P93 fix wave; the ADR's item-4 note is itself now stale (describes a defect that no longer exists). Low severity (an ADR historical-consequences note, not a live doc lie). Not eager-fixed: touching the ADR's Consequences section risks rewriting ratified decision history for a cosmetic staleness — filed below rather than silently edited.
- The T1 decision entry (`.planning/CONSULT-DECISIONS.md:75`) has an empty `**Commit:**` field ("(this entry; handover encodes the sequencing)") — the sequencing decision was never given a real SHA. Cosmetic ledger hygiene, out of scope for a docs edit.

## Eager-fixed vs filed

- Eager-fixed: none needed beyond the three planned edits (all in-scope).
- Filed: the two Noticing items above logged to `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (severity: low/cosmetic).

## Self-Check: PASSED
