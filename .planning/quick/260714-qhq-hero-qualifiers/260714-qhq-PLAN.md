---
quick_id: 260714-qhq
title: "Q1c interim hero qualifiers — honest caveats on hero token/latency numbers"
status: ready
created: 2026-07-14
---

# Quick Task 260714-qhq — Q1c interim hero qualifiers

Post-audit queue **item 2** (`.planning/milestones/audits/2026-07-12-reality-check.md`
§4 Lane 4, lines ~265–277). Small, honesty-sensitive edit to PUBLIC headline numbers.
**No re-measurement, no number changes, no section rewrites** — tight inline interim/
synthetic-baseline qualifiers only, so a reader isn't misled into thinking all three
hero figures are live-backend measurements.

## Ground truth (from the audit — authoritative, not invented)

- `89.1%` is a REAL Anthropic `count_tokens` number, but the MCP side it's compared
  against is a **hand-modeled 35-tool catalog, not a live MCP transcript** — full catalog
  vs a ~3-tool task maximizes the gap ("synthesized/synthetic baseline").
- `8 ms` / `27 ms` are **simulator-measured**, not live-backend — interim, pending live
  re-measurement.
- Audit's specific complaint: the honest "synthesized baseline" disclosure sits ~141
  lines below the hero in long-form prose (docs/index.md:158), not inline at the hero
  (docs/index.md:17); README already discloses inline (line 25) but the section header
  ("## Three measured numbers") still implies uniform live-measurement.

## The two edits

1. **`README.md`** — "Three measured numbers" section (lines ~21–25). Add a tight
   inline qualifier (single italic caveat line under the header) covering: 8ms/27ms are
   simulator-measured pending live-backend re-measurement; 89.1% is vs a synthesized
   MCP-tool-catalog baseline, not a live MCP transcript. Numbers stay verbatim; existing
   89.1% bullet's synthesized-baseline detail is untouched (already present since
   2026-04-26, commit d067b49c).
2. **`docs/index.md` line 17** — hero grid-card 89.1% bullet has NO caveat today. Add a
   brief inline parenthetical naming the MCP side as a synthesized tool-catalog baseline
   (interim). Hero card — stay terse.

## Acceptance

- Both edits land, numbers unchanged verbatim, wording matches surrounding style.
- `bash scripts/banned-words-lint.sh` passes (docs/index.md is Layer 1 — plumbing words +
  "replace" banned).
- `mkdocs build --strict` clean (docs/index.md renders).
- Cold-reader pass attempted via `/doc-clarity-review`; if the harness's nested
  `claude -p` doesn't return real file content in this environment, STOP before push,
  commit locally, and report "cold-reader NOT run — needs orchestrator" rather than
  improvising an alternate invocation.
- Atomic commit(s), conventional-commit message, `Co-Authored-By: Claude Opus 4.8`
  trailer. **Do NOT push** — L0 gates the push on reviewing final wording.

## Constraints

- Reality-check arc is NOT owner-ratified for defect-fixing — this quick does ONLY the
  interim qualifier edit. Other audit findings (false-provenance fixture, missing CI
  enforcement, FUSE-era north-star figure, latency-table staleness, etc.) are OUT of
  scope — notice and file, don't fix.
- No cargo needed.
- Stay on `main`; no new branch.
- No `--no-verify`.
