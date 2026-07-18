---
phase: 122-remote-init-hardening
plan: 02
subsystem: infra
tags: [git-remote-helper, rpx-codes, error-ux, resolve_import_parent, fetch, import, teach_coded]

# Dependency graph
requires:
  - phase: 122-01
    provides: "Wave-1 catalog-first rows (agent-ux/import-parent-resolve-fails-loud GREEN-CONTRACT, 5 asserts) minted at b49a0527"
  - phase: 121-error-code-registry
    provides: "RPX-xxxx registry + teach_coded/ExplainEntry; RPX-0507 import_unreachable_detail pattern to mirror"
provides:
  - "resolve_import_parent lifted to anyhow::Result<Option<ImportParent>> with loud tri-state (Ok(Some)/Ok(None)/Err)"
  - "RPX-0508 (HELPER_IMPORT_PARENT_RESOLVE) registered + emitted + indexed"
  - "5 injected-git-runner regression tests locking the loud path (exit-128, spawn-fail, exit-0-empty) and the benign ref-absent Ok(None)"
  - "quality/gates/agent-ux/import-parent-resolve-fails-loud.sh verifier gate (row grades GREEN)"
affects: [122-remote-init-hardening W3/W4, v0.15.0 milestone-close, helper fetch/import path]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Injectable subprocess seam: resolve_import_parent_with<F: Fn(&str)->io::Result<RevParseRun>> + real_rev_parse, so tri-state git-exit semantics are unit-testable with no real git on PATH"
    - "git exit-code tri-state: exit 1 = benign absence (Ok(None)); any other non-zero / spawn-fail / signal / exit-0-empty = loud coded Err"

key-files:
  created:
    - quality/gates/agent-ux/import-parent-resolve-fails-loud.sh
  modified:
    - crates/reposix-remote/src/main.rs
    - crates/reposix-core/src/codes.rs
    - docs/reference/error-codes.md
    - crates/CLAUDE.md

key-decisions:
  - "exit-0-with-empty-stdout is a LOUD failure (broken-git signal), not Ok(None) — git rev-parse --verify --quiet always prints the oid on a success exit (plan fix D3)"
  - "Err routed through fail_push (protocol error line + teaching diag), NOT bare ? — mirrors the RPX-0507 backend-unreachable arm; avoids a torn pipe and the RBF-LR-03 does-not-contain regression"
  - "Teaching built in a helper (import_parent_resolve_detail) mirroring RPX-0507's import_unreachable_detail; the thin anyhow! wrapper carries a // teach-exempt: ok marker (plan-sanctioned for an internal wrapper)"
  - "Error string carries no remote byte / git stderr — only a static ref name, exit-code integers, and a local spawn io::Error (T-122-04); out.stderr is deliberately never read"

patterns-established:
  - "Pattern: silent-degradation-to-loud-coded-error via an injectable runner seam + a per-code _detail teaching helper"

requirements-completed: [DRAIN-08]

# Metrics
duration: ~40min
completed: 2026-07-18
---

# Phase 122 Plan 02: resolve_import_parent hardening (loud RPX-0508) Summary

**`resolve_import_parent` now fails LOUD with the coded RPX-0508 teaching on any non-absence `git rev-parse` fault (spawn failure / non-1 non-zero exit / signal / anomalous exit-0-empty-stdout) instead of silently degrading to the parentless overlay — while a genuine ref-absent first fetch still returns `Ok(None)` and bootstraps.**

## Performance

- **Duration:** ~40 min
- **Completed:** 2026-07-18T06:57Z
- **Tasks:** 3
- **Files modified:** 4 (+1 created)

## Accomplishments
- Closed GTH-V15-05 / DRAIN-08: the silent `None`-on-any-git-error degrade in `resolve_import_parent` (main.rs:450-469) is now a tri-state that errors loudly on non-absence faults.
- Registered RPX-0508 with a full `rustc --explain`-grade ExplainEntry (cause/fix/alternative/recovery), emitted via `import_parent_resolve_detail` (mirrors RPX-0507), indexed in `docs/reference/error-codes.md`, and noted in `crates/CLAUDE.md`.
- 5 bin-target regression tests inject a fake git runner and prove each branch; the Wave-1 SC2/DRAIN-08 row now grades GREEN (F-K4b congruence 5/5 verified against the real `asserts_congruent`).

## Task Commits

1. **Task 1: Register RPX-0508 + rewrite resolve_import_parent (tri-state) + tests** — `4557c20c` (feat)
2. **Task 2: Verifier gate for import-parent-resolve-fails-loud** — `744a6104` (test)
3. **Task 3: Fix-twice — error-codes.md + crates/CLAUDE.md** — `560d010a` (docs)

_Regression tests live in main.rs alongside the impl (same file), so they landed in the Task 1 commit; the gate wrapping them is the Task 2 commit._

## Files Created/Modified
- `crates/reposix-remote/src/main.rs` — tri-state `resolve_import_parent` + `resolve_import_parent_with` (injectable) + `real_rev_parse` + `RevParseRun` + `import_parent_resolve_detail` (RPX-0508 teaching) + `fail_push` routing at the call site + `resolve_import_parent_tests` (5 tests).
- `crates/reposix-core/src/codes.rs` — `ids::HELPER_IMPORT_PARENT_RESOLVE = "RPX-0508"` + ExplainEntry + resolves-test list entry.
- `quality/gates/agent-ux/import-parent-resolve-fails-loud.sh` — verifier (bare `cargo test -p reposix-remote`, anti-gutting greps, F-K4b-congruent artifact emit).
- `docs/reference/error-codes.md` — RPX-0508 in the user-facing index.
- `crates/CLAUDE.md` — one-sentence RPX-0508 / tri-state note in the registry section.

## Decisions Made
See `key-decisions` frontmatter. Notably: the SC2 helper row correctly carries `transport_claim: false` — its assertion is unit-level `resolve_import_parent` exit-code semantics graded by the bare `cargo test -p reposix-remote` (bin-target unit tests), NOT a transport-layer / real-backend claim (no reposix binary or backend endpoint is driven).

## Deviations from Plan

### Auto-fixed / eager-handled

**1. [Rule 3 - Blocking, in-scope] teach_scan marker placement for the new loud block**
- **Found during:** Task 1. The new `anyhow!(import_parent_resolve_detail(..))` block triggers `teach_scan.py --scope helper`; the terminal teaching lives in the helper (indirection the scanner can't resolve).
- **Fix:** dispositioned with a `// teach-exempt: ok` marker on the line immediately above the block (teach_scan's lookback window is 2 lines — an initial 4-line marker was silently ignored until moved). Verified teach_scan reports my block as dispositioned.
- **Committed in:** `4557c20c`.

**2. [Noticing — filed, NOT fixed] Pre-existing teach_scan RED at HEAD (P121 W3 regression)**
- `teach_scan.py --scope helper` already exits 1 at HEAD (`b49a0527`) with 5 un-dispositioned blocks (`main.rs:115`, `bus_url.rs:63`, `backend_dispatch.rs:440`, `stateless_connect.rs:96`, `bus_handler.rs:461`). Root cause: `feat(121-w3)` converted these `teach(...)` sites to `teach_coded(...)`, which `teach_scan._PASS_CALL` does not recognize (`\bteach\s*\(` misses `teach_coded(`). The `agent-ux/helper-errors-teach-recovery` gate leg (c) is therefore RED, but it is `on-demand` cadence so it does not block pre-push.
- Per the gsd-executor scope boundary (pre-existing failure in files outside the W2 change set) this was **filed, not fixed** → `.planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md` § P122-W2-01 with the one-line fix sketch (add `teach_coded\s*\(` to `_PASS_CALL`).

---

**Total deviations:** 1 in-scope handled + 1 noticed/filed. **Impact:** no scope creep; the loud path and its gate are complete and green.

## Noticing (OD-3 #2)
- **`teach_scan` vs `rpx_registry_check` disagree on the coded idiom.** `rpx_registry_check.py` leg 3 already hardcodes `teach_coded(` in its `is_teaching` set, but `teach_scan._PASS_CALL` does not — so a `teach_coded`-based helper error passes one gate and trips the other. Filed as P122-W2-01 (the fix aligns them).
- **My block needs a marker while the 5 pre-existing use inline `teach_coded`** — an intentional asymmetry: I mirrored RPX-0507's testable `_detail` helper (plan-directed), whose indirection teach_scan can't see through; the marker documents exactly that.
- **`codes.rs` (53 KB) and `main.rs` (50 KB) exceed the 20 KB file-size soft ceiling** — pre-existing, print-only + WAIVED (codes.rs is the documented single-source-of-truth, GTH-V15-68). My edits added to both but did not create the overage.
- **Tree-peel fallback is intentionally soft:** an exit-1 (absence) on `<ref>^{tree}` after the commit resolved falls back to `Ok(None)` (defensive), while a non-absence peel fault errors loudly via `?`. Documented inline; a present-commit-but-absent-tree is a should-never-happen git state, so the soft fallback is safe.

## Threat Flags
None — no new network endpoint, auth path, or trust-boundary surface introduced. The one new error string is `&'static`/local-diagnostic only (T-122-04 honored; `out.stderr` is never read).

## Issues Encountered
- rustfmt reflowed the new match arms; ran `cargo fmt -p reposix-core -p reposix-remote` and re-verified the teach-exempt marker stayed inside teach_scan's 2-line window. Resolved.

## Next Phase Readiness
- W2 complete and committed locally (NOT pushed — phase-close push is the coordinator's).
- **Left for the coordinator (C1):** STATE.md / ROADMAP.md advance-plan + REQUIREMENTS.md `mark-complete DRAIN-08` + verifier dispatch + phase-close push — not run here to avoid racing sibling waves.
- Open item flagged for C1: fast-track the one-line `teach_scan` fix (P122-W2-01) to re-green the helper-errors gate.

## Self-Check: PASSED
All created/modified files present on disk; all 3 task commits (`4557c20c`, `744a6104`, `560d010a`) resolve in git.

---
*Phase: 122-remote-init-hardening*
*Completed: 2026-07-18*
