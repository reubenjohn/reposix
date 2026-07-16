---
quick_id: 260716-f6o
title: "Fix-it-twice for 5a5dd29 — strip retirement-history narrative from the perf-gate GENERATOR"
status: complete
date: 2026-07-16
---

# Quick Task 260716-f6o — strip retirement-history narrative from the perf-gate GENERATOR

Fix-it-twice for owner ruling `5a5dd29` ("strip retirement-history narrative from
user-facing docs"). That commit removed the `## What retired the old 89.1% / 85.5%
figures` section from `docs/benchmarks/token-economy.md`, but the GENERATOR
(`bench_token_economy_captures.py::render_token_economy_markdown`) still templated it —
the P115 phase-close gate-run regen silently re-added the section, leaving a dirty
`+12`-line working tree. **Manager-established provenance: accidental regression vector,
NOT a deliberate override of the owner ruling.**

## What was built

1. **Generator template stripped (Task 1).** Removed the `## What retired the old
   89.1% / 85.5% figures` header + its 9-line paragraph from the f-string body of
   `render_token_economy_markdown()` in `quality/gates/perf/bench_token_economy_captures.py`.
   Exact block removed (verbatim from the plan, confirmed present pre-edit):

   ```
   ## What retired the old 89.1% / 85.5% figures

   The previous token-economy figures (an **89.1%** headline and a per-backend
   **85.5%** GitHub number) came from a *different, synthetic* methodology: running
   Anthropic's `count_tokens` over a static, hand-constructed JSON fixture that
   stood in for an MCP tool catalog. That measured the size of a fixture, not the
   cost of a live agent run. It is **retired here** in favour of the live
   session-usage medians above -- real sessions, a real GitHub backend, and the
   GitHub MCP server's real tool surface. The synthetic fixtures remain in the repo
   only as provenance for that retired estimate; they no longer back any published
   number.
   ```

   Net effect: exactly one blank line now separates the `carries ... for the same
   result.` line from `## What this DOES measure`, matching the committed doc structure.
   File still parses (`ast.parse` OK); no other part of the f-string, provenance
   constants, or median math touched.

2. **Byte-for-byte proof (Task 2, HARD GATE — PASSED).** Ran
   `python3 quality/gates/perf/bench_token_economy.py --offline`. Compared:
   - `git show HEAD:docs/benchmarks/token-economy.md | sha256sum` →
     `5620699b15da82b8123404b7038ae634259891a17517b1f1cba23e598f364fcf`
   - `sha256sum docs/benchmarks/token-economy.md` (post-regen) →
     `5620699b15da82b8123404b7038ae634259891a17517b1f1cba23e598f364fcf`
   - **Match.** `git diff --stat docs/benchmarks/token-economy.md` → empty. The fix is
     proven correct: future gate-run regens will no longer re-add the stripped section.

3. **Belt-and-suspenders discard (Task 3).** Ran `git checkout -- docs/benchmarks/token-economy.md`
   as the explicit fail-safe (no-op given Task 2 already left the tree clean).
   Re-verified: empty diff, sha256 match. **The doc was never staged and is NOT part of
   this commit** — it ends byte-identical to `HEAD`.

4. **No catalog rebind required (Task 4, verified fail-closed).** Committed doc bytes
   are unchanged by construction, so no bound-claim hash can have drifted.
   `grep -n 'token-economy' quality/catalogs/doc-alignment.json` confirms: the active
   BOUND rows for this doc are the live four-axis claims
   (`docs/benchmarks/token-economy/output-reduction-94-percent`,
   `.../cost-reduction-75-percent`, `.../live-github-capture-methodology`) — none anchor
   to the removed narrative section. The `89.1%`/`85.5%`/`4883`/`531` rows were already
   `RETIRE_PROPOSED` pre-existing. `quality/catalogs/doc-alignment.json` untouched
   (`git diff --quiet` confirmed) — no mismatch found, nothing to flag.

5. **Provenance recorded (Task 5).** Appended a dated `## SHIPPED` entry to
   `.planning/PROGRESS.md` citing 260716-f6o, the generator fix, the working-tree
   discard, and the "accidental regression vector" provenance ruling.
   `.planning/MANAGER-HANDOVER.md` untouched (verified).

6. **Commit + push (Task 6).** Targeted staging only — see Commit below.

## Commit / push

- **Commit:** `fix(perf): strip retired-narrative section from token-economy generator (260716-f6o)`
- **Staged (explicit paths only, no `-A`/`.`):**
  `quality/gates/perf/bench_token_economy_captures.py`,
  `.planning/PROGRESS.md`,
  `.planning/quick/20260716-token-economy-generator-strip/PLAN.md`,
  `.planning/quick/20260716-token-economy-generator-strip/260716-f6o-SUMMARY.md`.
- **NOT staged:** `docs/benchmarks/token-economy.md` (verified `git diff --cached` excludes
  it), any `quality/catalogs/*.json` (verified excluded), `.planning/MANAGER-HANDOVER.md`.
- **Push:** `git push origin main` — pre-push hook (~2 min, includes
  `docs-alignment/walk.sh`) ran to completion; result recorded in the final report to
  the dispatcher (see below for exact outcome / commit SHA).

## Noticing (OD-3)

- **No automated test guards generator-output vs. committed-doc byte-identity.**
  `test_main_offline_regenerates_doc_from_captures` in
  `quality/gates/perf/test_bench_token_economy.py` only asserts idempotency (a second
  `--offline` run produces the same bytes as the first) against a synthetic `tmp_path`
  fixture — it never diffs the regenerated doc against the real committed
  `docs/benchmarks/token-economy.md`. This is exactly the gap that let this regression
  reach a phase-close gate run undetected until the working tree went dirty. Not fixed
  here (plan scope is fixed to generator-only + doc discard; "do not expand scope" is
  explicit) — flagged for a future phase/quick to add a real assert-doc-matches-committed
  regression test.
- **Other prose sections in the same f-string carry the same class of risk** (e.g.
  "What this does NOT measure", "Capture provenance") — free-text paragraphs that could
  independently drift from the committed doc if either is hand-edited without touching
  the other. No drift currently exists (byte-identity proven in Task 2/3); this is a
  structural observation about the template's maintenance surface, not a bug.
- **Docstring header comment (`bench_token_economy_captures.py:1-17`) has no stale
  line-number references** — checked; only file-path pointers exist elsewhere
  (`PROGRESS.md`, `MANAGER-HANDOVER.md`, this quick's own `PLAN.md`), none of them
  pin specific line numbers inside the edited file, so nothing went stale from the
  13-line removal.
- File size after edit: 249 lines / 11,191 bytes — comfortably under the 15,000-char
  `.py` ceiling (`structure/file-size-limits`); no gate risk introduced.

## Deviations from Plan

None — plan executed exactly as written. Both Task 2's hard gate (sha256 + empty diff)
and Task 4's fail-closed catalog check passed on the first attempt; no residual drift,
no catalog mismatch, no scope expansion.

## Self-check

- `quality/gates/perf/bench_token_economy_captures.py` — FOUND, no longer contains
  "What retired the old" or "89.1% headline and a per-backend"; parses as valid Python.
- `docs/benchmarks/token-economy.md` — FOUND, sha256 == committed HEAD, `git diff`
  empty, NOT staged.
- `.planning/PROGRESS.md` — FOUND, contains "260716-f6o" and "accidental regression
  vector".
- `.planning/MANAGER-HANDOVER.md` — untouched (`git diff --quiet` true).
- `quality/catalogs/doc-alignment.json` — untouched (`git diff --quiet` true).
