# 57-05 — SIMPLIFY-03 audit memo

> **Wave E close.** Confirms Wave A's boundary assessment in
> `quality/catalogs/README.md` § "SIMPLIFY-03 boundary statement". Read
> by Wave F's verifier subagent as evidence for SIMPLIFY-03 closure.

## 1. What does `scripts/catalog.py` do?

Per-FILE disposition tracker (430 lines, stdlib-only). Reads
`.planning/v0.11.1-catalog.json` (one row per tracked file from
`git ls-files`). Subcommands: `init / set / coverage / render / query /
stats`. Status set: `KEEP / TODO / DONE / REVIEW / DELETE / REFACTOR`.
`SKIP_PATH_PATTERNS` excludes `Cargo.lock`, `target/`, `runtime/`,
`.planning/milestones/v0.X.0-phases/`. On `init`, scans audit markdown
files for verdict keywords near each path and seeds dispositions.
Domain: "what files do I touch this session, what work-state?"

## 2. What does `quality/runners/verdict.py` do?

Per-CHECK gate-state collator (272 lines, stdlib-only). Reads per-row
artifacts under `quality/reports/verifications/<dim>/*.json` + catalogs
under `quality/catalogs/*.json`. Status set: `PASS / FAIL / PARTIAL /
WAIVED / NOT-VERIFIED`. Subcommands: `--cadence <name>`, `--phase <N>`,
`session-end`. Writes markdown verdict to
`quality/reports/verdicts/<cadence-or-phase>/<ts>.md` + shields.io
badge at `quality/reports/badge.json`. Domain: "which gates GREEN this run?"

## 3. Overlap

None across these axes:

- **Source artifact paths:** `.planning/v0.11.1-catalog.json` vs
  `quality/catalogs/*.json` + `quality/reports/{verdicts,badge}.json`.
- **Row IDs:** `git ls-files` paths (e.g. `crates/reposix-core/src/lib.rs`)
  vs gate slugs (e.g. `structure/no-version-pinned-filenames`).
- **Cadence:** manual planning-session invocation vs pre-push / CI per
  cadence tag.

## 4. Divergence

- **Unit:** file vs check.
- **State:** work-state (planner-set TODO/DONE/REFACTOR) vs gate-state
  (verifier-set PASS/FAIL/WAIVED).
- **Lifecycle:** session-bounded (one v0.X.X catalog per session,
  archived per milestone) vs infinite (gates persist across milestones).

## 5. Recommendation

Keep separate in v0.12.0. The two systems answer different questions;
conflating them would degrade both. A future `Catalog<T>` abstraction
(`T` = `FileDisposition` | `GateOutcome`) is plausible in v0.13.x if
shared infra accumulates more callers; v0.12.0 boundary holds.

## 6. SIMPLIFY-03 closure

- Wave A shipped the boundary statement in `quality/catalogs/README.md`.
- Wave E (this memo) confirms via end-to-end re-read.
- `scripts/catalog.py` stays in place — no migration, no shim, no waiver.

SIMPLIFY-03: **CLOSED**.
