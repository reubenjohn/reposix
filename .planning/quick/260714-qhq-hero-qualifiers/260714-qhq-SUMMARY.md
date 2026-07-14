---
quick_id: 260714-qhq
title: "Q1c interim hero qualifiers — honest caveats on hero token/latency numbers"
status: complete
completed: 2026-07-14
---

# Quick Task 260714-qhq — SUMMARY (interim hero qualifiers)

Post-audit queue item 2. Added tight inline interim/synthetic-baseline qualifiers to the
hero numbers on two surfaces per `.planning/milestones/audits/2026-07-12-reality-check.md`
§4 Lane 4. No numbers changed, no sections rewritten — caveats only.

## Edit 1 — README.md "Three measured numbers" (line 21 onward)

Added a single italic caveat line under the header, above the existing three bullets
(numbers/bullets untouched verbatim):

```
*Interim figures — `8 ms` and `27 ms` are simulator-measured, pending live-backend
re-measurement; `89.1%` is measured against a synthesized MCP-tool-catalog baseline,
not a live MCP transcript.*
```

The existing 89.1% bullet already carried inline synthesized-baseline detail (since
commit `d067b49c`, 2026-04-26) — left as-is; the new line addresses the header's
"measured" framing implying uniform live-backend measurement.

## Edit 2 — docs/index.md line 17 (hero grid card)

Before:
```
-   **`89.1%`** fewer tokens vs MCP for the same 3-issue read+edit+push workflow ([token economy](benchmarks/token-economy.md))
```
After:
```
-   **`89.1%`** fewer tokens vs a synthesized MCP-tool-catalog baseline (interim) for the same 3-issue read+edit+push workflow ([token economy](benchmarks/token-economy.md))
```
Hero card kept terse; the fuller "not a live MCP transcript" / Anthropic Forge-catalog
detail stays one click away at `benchmarks/token-economy.md` and in the footer disclosure
(`docs/index.md:158`, unchanged).

## Verification

- `bash scripts/banned-words-lint.sh` — PASSED (default mode; docs/index.md is Layer 1 —
  no plumbing words, no "replace" added).
- `mkdocs build --strict` — `OK: docs site clean` (1.42s build, no warnings).
- Cold-reader (`/doc-clarity-review README.md docs/index.md`): Skill tool loaded and ran
  its prescribed `claude -p` subprocess invocation against isolated copies in
  `/tmp/.../doc-clarity-review-WudCSu/`, but the subprocess reported it received "no file
  content" — only ambient session context, not the copied files. This matches the
  known `.planning/CLAUDE.md` caveat that "subscription users cannot fall back to
  `claude -p`". Per charter, did NOT improvise an alternate invocation.
  **Cold-reader NOT run — needs orchestrator** (Task-tool/Path-A dispatch).

## Commits

- `<see git log>` — `docs(hero): add interim/synthetic-baseline qualifiers to hero numbers`
  (README.md + docs/index.md)
- `<see git log>` — `docs(260714-qhq): file GTH-V15-12 doc-clarity-review nested-claude-p gap + complete hero-qualifiers quick`
  (GOOD-TO-HAVES.md + PLAN.md + this SUMMARY)

## Noticing (OD-3 ownership charter — REPORTED)

- **FILED — GTH-V15-12** (`.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md`,
  severity LOW-MEDIUM): the `doc-clarity-review` skill's nested `claude -p` invocation
  returns a confusing non-error ("no file content was included") instead of a hard fail
  when it can't see the target files in this environment — a less careful agent skimming
  for a CLEAR/NEEDS WORK/CONFUSING verdict could misread that reply as an actual review
  outcome. Fix-sketch: canary probe + hard-fail, or a subscription-caveat note pointing at
  Task-tool dispatch (mirrors the existing doc-alignment skill caveat). Lives in the
  user-global skill dir, outside this repo.
- **Not filed (already tracked by the audit itself, explicitly out of scope for this
  quick):** the 27ms vs 24ms cold-init inconsistency (`docs/index.md:18` vs
  `docs/concepts/...:15`) noted while reading the hero card — POLISH-severity per the
  audit, a separate queue item. Left untouched per charter (only line 17 in scope).
- **Not filed (audit-accuracy nuance, not a code/doc defect):** the audit's HIGH finding
  cites `README.md:21` alongside `docs/index.md:17` as lacking inline disclosure, but
  README's 89.1% bullet (line 25) has carried synthesized-baseline detail inline since
  2026-04-26 (commit `d067b49c`), predating the audit. The audit's actual complaint reads
  as being about the section *header's* "measured" framing, not a missing bullet-level
  caveat — Edit 1 above addresses that reading. No action needed beyond the edit made.

## Self-Check: PASSED

- Both edits land, numbers verbatim, style matches surrounding markdown.
- Banned-words lint PASSED; mkdocs --strict PASSED.
- Cold-reader attempted per the prescribed skill invocation; environment limitation
  surfaced and reported per charter, not improvised around.
- Quick record under `.planning/quick/260714-qhq-hero-qualifiers/` (PLAN + SUMMARY).
- NOT pushed — local commit(s) only, per charter (L0 gates the push).
