---
phase: 122-remote-init-hardening
plan: 04
wave: 4
subsystem: quality gates / reposix-remote read path (agent-ux)
tags: [agent-ux, stateless-connect, protocol-v2, GTH-V15-04, DRAIN-07, RBF-LR-03, git-2.34, rebase-recovery]
requires:
  - "122-01 catalog-first extension (agent-ux/rebase-recovery-reconciles +2 SC1 asserts, transiently UNMET)"
  - "P105 RBF-LR-03 fix (import-path parent chaining + two-namespace remote-helper contract)"
provides:
  - "modern-git stateless-connect (protocol-v2) READ-path coverage in rebase-recovery-reconciles.sh (git >= 2.34)"
  - "P105 §5 resolution — Branch A (convergence); RBF-LR-03 is import-path-only / git-version-scoped"
  - "GIT_TRACE_PACKET wire-level transport proof pattern for read-path assertions"
  - "_provenance_note on the two P122 W1 agent-ux rows (Task Z hygiene)"
affects:
  - "quality/gates/agent-ux/rebase-recovery-reconciles.sh"
  - "quality/catalogs/agent-ux.json"
  - "crates/CLAUDE.md"
tech-stack:
  patterns:
    - "GIT_TRACE_PACKET set on the pull only (push does not inherit) → isolate the READ-path wire trace while preserving the documented `pull && push` single-command chain"
    - "wire-signature transport proof (command=fetch + version 2, zero import-stream) instead of a ref-namespace discriminator"
    - "lift the protocol.version=0 forcing in the main shell for the modern-git legs (safe: after all import legs run)"
key-files:
  created:
    - ".planning/phases/122-remote-init-hardening/122-04-SUMMARY.md"
  modified:
    - "quality/gates/agent-ux/rebase-recovery-reconciles.sh"
    - "quality/catalogs/agent-ux.json"
    - "crates/CLAUDE.md"
    - ".planning/milestones/v0.14.0-phases/105-rbf-lr-03-rebase-recovery/105-PLAN.md"
    - ".planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md"
decisions:
  - "SC1 = Branch A (convergence): the stateless-connect read path fast-forwards off the cache's chained history; RBF-LR-03 is import-path-only. NO second cache-side fix site."
  - "refs/reposix-import/main is a PUSH (export) artifact, NOT a read-path discriminator — replaced the ref-absence check with a GIT_TRACE_PACKET wire proof after reality contradicted the ref heuristic."
  - "Task Z _provenance_note added after `kind` to match the sibling agent-ux hand-edit ordering; row status left untouched (verifier's job)."
  - "Did NOT advance STATE/ROADMAP/REQUIREMENTS or push — wave executor; phase-close (verifier grade + push) is the C1 coordinator's (OP-7)."
metrics:
  tasks: 3  # Task 1 (gate legs + fix-twice), Task 2 (Branch A adjudication), Task Z (catalog hygiene)
  duration: "~1 session"
  completed: 2026-07-18
---

# Phase 122 Plan 04 (Wave 4): stateless-connect read-path verification (GTH-V15-04 / DRAIN-07) Summary

Closes GTH-V15-04 / DRAIN-07 and resolves P105 PLAN §5. RBF-LR-03's rebase-recovery fix
was proven only on the legacy `import` (git ≤ 2.25) path because the gate force-set
`protocol.version=0` for every git subprocess — so even on CI's modern git the
`stateless-connect` (protocol-v2) READ path was never exercised; it only logged a TODO.
Wave 4 extends `rebase-recovery-reconciles.sh` so that on git ≥ 2.34 it ALSO runs both
drift scenarios via the REAL stateless-connect path (forcing lifted), and adjudicates each
deterministically. **Result: Branch A — both scenarios converge**, proven at the wire level
on this VM's git **2.50.1**. RBF-LR-03 is `import`-path-only (git-version-scoped); no second
cache-side fix site exists.

## What shipped

### Task 1 — modern-git stateless-connect legs + fix-twice header (`feat`, e572a8ae)
- The STATELESS-CONNECT block (previously a bare TODO on modern git) now, when
  `git --version` ≥ 2.34, `unset`s `GIT_CONFIG_COUNT/KEY_0/VALUE_0` (safe in the main shell —
  all import scenarios A/B/C already ran) and re-runs two drift scenarios via the real
  protocol-v2 path:
  - **Stateless Scenario A** — peer git-push drift (SP edits issue3 + pushes; SA holds an
    unpushed edit on issue4 → clean rebase replay).
  - **Stateless Scenario B** — external REST PATCH drift (PATCH issue3; SB holds issue5).
  - Fresh records (3/4/5) — issue2 was deleted by import Scenario C; no collision.
- Each leg adjudicates deterministically: convergence (`git pull --rebase && git push`
  exits 0, SoT version before+1, no FATAL/REFLOCK) **and** a wire-level transport proof.
- Import legs (protocol.version=0 forced) are UNTOUCHED — they still guard old git.
- **fix-twice:** the stale "only git 2.25.1 is installed / protocol.version=2 errors with
  `bad line length 2`" header comments are corrected — the real floor is git ≥ 2.34,
  verified on 2.50.1 (CURRENT STATUS + IMPORT-PATH FORCING blocks rewritten).

### Task 2 — Branch A adjudication (`docs`, 01e53750)
- `crates/CLAUDE.md`: the modern-git stateless-connect READ path is now PROVEN to reconcile
  after SoT drift (not just the import path) — GIT_TRACE_PACKET wire proof, Branch A.
- `105-PLAN.md §5`: annotated RESOLVED (Branch A). The historical "git 2.25.1 only" note is
  preserved as accurate-at-authoring but marked STALE (real floor git ≥ 2.34) — the plan is
  an archived artifact, so history is annotated, not rewritten.

### Task Z — catalog `_provenance_note` hygiene (`chore`, e58c0803)
- Added `_provenance_note` to `agent-ux/import-parent-resolve-fails-loud` and
  `agent-ux/init-refuses-nested-in-shared-tree` (the two P122 W1 rows omitted it while every
  other hand-edited agent-ux row carries one). Notes match the sibling style
  (documented-gap / GOOD-TO-HAVES-01), attributed to P122 W1 (2026-07-17).
- Placed after `kind` (sibling ordering); **status untouched** (verifier's job).
- Catalog parses (`python3 -m json.tool`); the real `load_catalog()` honesty gate
  (`validate_row` on all 71 rows) accepts it.

## SC1 branch: A (convergence) — the concrete evidence

The verdict hung on WHY `refs/reposix-import/main` appeared in the recovery clones. A
leaf-isolated `/tmp` probe (git 2.50.1, protocol.version unset) settled it against reality:

| Step (protocol.version UNSET) | Result |
|---|---|
| after `reposix init` | `refs/reposix-import/main` **ABSENT** (only `refs/reposix/origin/main`) |
| after `git pull --rebase` (pull only) | exit 0, **converged**, import ref still **ABSENT**; `GIT_TRACE_PACKET` = `2× command=fetch`, `1× command=ls-refs`, `2× version 2`, **0** fast-import/reposix-import lines |
| after `git push` | `refs/reposix-import/main` **appears** (614971…) |

So the READ path (init + pull) genuinely uses protocol-v2 stateless-connect and
fast-forwards; the private import ref is written by the **EXPORT/push** path, not the fetch.
The `+refs/heads/*:refs/reposix/origin/*` force refspec means git owns the tracking-ref
placement and the cache's linear `refs/heads/main` history fast-forwards cleanly. **RBF-LR-03
does not recur on modern git.**

## Gate result

`bash quality/gates/agent-ux/rebase-recovery-reconciles.sh` → **exit 0 (PASS)**, 19/19
asserts, 0 failed. Both new SC1 `expected.asserts` (122-01) map to emitted `asserts_passed`
(F-K4b congruent):
- assert[9] "runs both drift scenarios via the real stateless-connect path … no bare
  TODO-skip" → `STATELESS-CONNECT MODERN-GIT (git 2.50 >= 2.34) … proven by the
  GIT_TRACE_PACKET protocol-v2 command=fetch/version 2 wire signatures …`.
- assert[10] "deterministic per-scenario verdict: both converge … OR labelled known-
  divergence" → `STATELESS-CONNECT VERDICT: … both … CONVERGE … [Branch A] …`.
`test-name-vs-asserts.sh` honesty gate: PASS. `cargo build -p reposix-remote -p reposix-cli
-p reposix-sim` (the gate's own foreground build): GREEN (no Rust source changed this wave).

## Deviations from Plan

### Auto-fixed (Rule 1 — bug in my own first cut)

**1. [Rule 1 - Bug] The initial ref-namespace PATH-DISCRIMINATOR was wrong.**
- **Found during:** Task 1 first gate run — convergence PASSED but the discriminator FAILed:
  `refs/reposix-import/main` was present in both stateless recovery clones, which the check
  read as "import path taken."
- **Root cause:** the import ref is a PUSH (export) artifact, not a fetch signal — an
  incorrect assumption in my first design, contradicted by the `/tmp` probe.
- **Fix:** replaced the ref-absence heuristic with a `GIT_TRACE_PACKET` wire proof on the
  pull only (`command=fetch` + `version 2`, zero import-stream) — a direct, honest transport
  signal. Documented the why-not in the gate header + 105-PLAN §5 (fix-twice).
- **Files:** `quality/gates/agent-ux/rebase-recovery-reconciles.sh`. **Commit:** `e572a8ae`
  (the corrected discriminator shipped in the same Task 1 commit after the reality check).

### Filed (OD-3 — else file)

**GTH-V15-78** (`.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md`): the gate is ~42k
chars (4× the 10k `.sh` ceiling) — pre-existing, grown by W4's legs. Sketch: extract shared
helpers into a sourced lib (mirror `lib/litmus-flow.sh`). Effort ~1-2h on a load-bearing
green gate → out of this lane's charter. File-size tier is WAIVED until 2026-08-08, so
non-blocking today.

## Noticing (OD-3 #2)

- **The ONLY `protocol.version=0` masking site is this gate's own (intentional) import legs.**
  Swept `crates/`, `quality/gates/`, `.github/` — no CI config or Rust test force-sets v0 to
  hide modern-git behavior. The POC scripts under `.planning/research/**` force v2 (they
  exercise, not mask). So closing this gate's TODO closes the whole masking gap; CI
  (ubuntu-latest, git ≥ 2.43) now runs the same modern-git legs.
- **A ref-namespace read-path discriminator is a false-negative trap** (verified). The write
  path (export) creates `refs/reposix-import/*`; the read path (stateless-connect) does not.
  Anyone reasoning about "which transport ran" from ref presence will be misled — the gate
  header + 105-PLAN §5 now warn about this explicitly.
- **The gate re-uses the sim SoT across import + stateless legs** (issue2 deleted mid-run;
  stateless legs use pristine 3/4/5). This works but is fragile — a future scenario editing
  3/4/5 in the import legs would collide. A per-leg fresh sim (or a fresh project slug) would
  isolate cleanly; folded into the GTH-V15-78 refactor sketch, not done here.
- **`git 2.50` version parse.** The gate's `GIT_VER` is `2.50` (major.minor) — correct for
  the ≥ 2.34 comparison; the transcript logs `git 2.50` where a reader might expect `2.50.1`.
  Cosmetic; the full `git --version` (2.50.1) is also logged at transcript top.

## Self-Check: PASSED

`122-04-SUMMARY.md`, the modified gate, catalog, `crates/CLAUDE.md`, `105-PLAN.md`, and
`GOOD-TO-HAVES.md` all exist on disk; commits `e58c0803` (Task Z), `e572a8ae` (Task 1),
`01e53750` (Task 2) are present in git history; the gate exits 0 with both SC1 asserts in
`asserts_passed`.
