---
phase: 103-early-cheap-wins
verified: 2026-07-12T00:00:00Z
status: passed
score: 3/3 items GREEN
verifier: unbiased phase-close verifier (no session context)
constraint: NO CARGO (machine-wide cargo slot held by another lane) — honored
commits:
  - 12abdfb  # item 1: doc-alignment grade/persist split
  - 1136bb3  # item 2: F-K4b false-demote robustness
  - dad227e  # item 3: file-size OP-8 split
---

# Phase 103 "early cheap wins" — Verification Report

Graded against reality (ran the gates), not the executors' word. Both
`reposix-quality` binaries pre-existed (`target/{release,debug}`); no rebuild.

## Item 1 — doc-alignment grade/persist split — GREEN
- `walk.sh` grade path (default) `cp`s the committed catalog to a `mktemp` and
  walks the copy (`--catalog $TMP`); only `--persist` writes `$DEFAULT_CATALOG`.
- Ran `bash quality/gates/docs-alignment/walk.sh` → exit 0; `git status --short
  quality/catalogs/doc-alignment.json` empty after. No self-mutation.
- Grade/persist contract assert present on the `docs-alignment/walk` row
  (freshness-invariants.json line 578).

## Item 2 — F-K4b false-demote robustness — GREEN
- `python3 -m pytest quality/runners/test_audit_field.py -q` → 47 passed.
- Guard honest: transcript-fail still flips `status=FAIL` + appends
  `asserts_failed` BEFORE the new unconditional `return`; the return only skips
  F-K4b congruence, not the transcript-evidence check.
- `TestShellSubprocessFK4bExemption` asserts what its name promises: empty PASS,
  non-mapping asserts_passed PASS, missing transcript FAIL.
- Ran all 3 P0 `agent-ux/fleet-safety-*` verifier scripts → exit 0 (PASS).

## Item 3 — file-size OP-8 split — GREEN
- `bash quality/gates/structure/file-size-limits.sh` → SURPRISES-INTAKE.md /
  GOOD-TO-HAVES.md ABSENT from the 56-file violation list.
- Byte-exact round-trip vs pre-split original (`dad227e^`): entry bytes identical
  — SURPRISES 106856==106856, GOOD-TO-HAVES 127321==127321 (50→50, 58→58).
- All 15 part files ≤20K (largest 19079). INDEX rewrites link 7+8 parts.
- Waiver narrowed honestly: intakes off-waiver, residual re-counted 56 (live gate
  agrees), tracked_in → GOOD-TO-HAVES-02 (filed). `--warn-only` → exit 0.
- `scripts/split_ledger.py` carries a round-trip self-check that raises on
  mismatch (line 154).

## Verdict: GREEN — all 3 items pass. Phase 103 achieves its goal.

_Verifier: Claude (unbiased phase-close). Verdict skeleton (SC4
`quality/reports/verdicts/p103/VERDICT.md`) is the executor's deliverable, not
yet present — noted, not graded RED (verifier artifact only)._
