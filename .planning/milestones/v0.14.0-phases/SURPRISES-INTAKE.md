# v0.14.0 Surprises Intake (P110 source-of-truth)

> **Append-only intake for surprises discovered during P102–P109 execution.**
> Each entry is something the discovering phase chose NOT to fix eagerly because it was
> massively out-of-scope. P110 drains this file (per CLAUDE.md OP-8 — Slot 1 of the v0.14.0
> +2 reservation).
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering
> phase without doubling the phase's scope (rough heuristic: < 1 hour incremental work, no
> new dependency introduced, no new file created outside the phase's planned set), do it
> there. The intake file is for items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN,
> RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md` (drained by P111, Slot
> 2).

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: P<N> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for P<N>:** Why eager-resolution wasn't possible (scope, time, dependency).

**Sketched resolution:** One paragraph proposing how P110 should resolve.

**STATUS:** OPEN  (← P110 updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

## Split index (OP-8 file-size drain)

This ledger approached the milestone-hygiene 44000 B ceiling (`quality/gates/agent-ux/p111-milestone-hygiene.sh` assert E) and was split into 2 per-part child files under `surprises-intake/` via `scripts/split_ledger.py` (byte-exact round-trip verified). Every entry is preserved verbatim; append new entries to the last part (or a new part) and add the title here. All 20 v0.14.0 entries are terminal (RESOLVED / DEFERRED) — zero OPEN.

- [`surprises-intake/part-01.md`](surprises-intake/part-01.md) — 10 entries:
  - 2026-07-11 23:00 | discovered-by: P102 (adversarial code-review) | severity: HIGH | RESOLVED-in-P102
  - 2026-07-11 23:00 | discovered-by: P102 (adversarial code-review) | severity: HIGH | RESOLVED-in-P102
  - 2026-07-11 23:00 | discovered-by: P102 (adversarial code-review) | severity: MEDIUM | RESOLVED-in-P102
  - 2026-07-11 23:00 | discovered-by: P102 (adversarial code-review) | severity: HIGH | RESOLVED-in-P102
  - 2026-07-11 23:00 | discovered-by: P102 (adversarial code-review) | severity: MEDIUM | RESOLVED-in-P102
  - 2026-07-12 07:13 | discovered-by: P104 (github-helper-path 404 fix verifier) | severity: MEDIUM | DEFERRED-TO-v0.15.0
  - 2026-07-12 07:35 | discovered-by: v0.14.0 health-triage lane (main gate sweep) | severity: MEDIUM | DEFERRED
  - 2026-07-12 07:40 | discovered-by: v0.14.0 health-triage lane (main gate sweep) | severity: LOW | RESOLVED
  - 2026-07-12 07:13 | discovered-by: P104 (github-helper-path 404 fix verifier) | severity: MEDIUM | DEFERRED-TO-v0.15.0
  - 2026-07-12 08:10 | discovered-by: P105 (RBF-LR-03 rebase-recovery research) | severity: HIGH | RESOLVED
- [`surprises-intake/part-02.md`](surprises-intake/part-02.md) — 9 entries:
  - 2026-07-12 08:35 | discovered-by: P105 (RBF-LR-03 rebase-recovery gate, Lane 2) | severity: HIGH | RESOLVED-in-P105
  - 2026-07-12 09:40 | discovered-by: P105 (RBF-LR-03 docs fix-twice lane, ownership noticing) | severity: MEDIUM | DEFERRED-TO-v0.15.0
  - 2026-07-12 | discovered-by: D2 re-seal Wave 1 (shell/planning lane) | severity: HIGH | DEFERRED-TO-v0.15.0
  - 2026-07-12 15:57 | discovered-by: C2-wave-2 (CI-gate fix-twice) | severity: MEDIUM | RESOLVED
  - 2026-07-12 | discovered-by: GSD-quick (release-plz RED fix) | severity: MEDIUM | DEFERRED
  - 2026-07-12 | discovered-by: GSD-quick (fleet-safety untrack fix) | severity: MEDIUM | RESOLVED
  - 2026-07-12 20:59 | discovered-by: P111 (milestone-close CI-wait) | severity: MEDIUM | RESOLVED
  - 2026-07-13 | discovered-by: B1 (tag-remediation, mirror-reconcile investigation) | severity: MEDIUM | DEFERRED (pending B1 mirror-refresh manager decision)
  - 2026-07-13 | discovered-by: B1 (tag-remediation, mirror-reconcile investigation) | severity: MEDIUM | DEFERRED (pending B1 mirror-refresh manager decision)
- [`surprises-intake/part-03.md`](surprises-intake/part-03.md) — 7 entries:
  - 2026-07-13 | discovered-by: B2 (tag-remediation, p93 harness fix, verify-against-reality) | severity: HIGH | DEFERRED (dedicated real-backend recovery-convergence fix phase; active p93 tag-blocker, owner decision pending)
  - 2026-07-12 21:15 | discovered-by: B1 (litmus self-heal proof, verify-against-reality) | severity: HIGH | OPEN (documented recovery doc-lie #2, entangled with B1 owner decision)
  - 2026-07-12 21:15 | discovered-by: B1 (litmus self-heal proof, verify-against-reality) | severity: HIGH | OPEN (ADF-unparse silently empties page body — real-backend data-loss risk)
  - 2026-07-12 21:15 | discovered-by: PRIORITY-ZERO red-CI sweep | severity: MEDIUM | OPEN (contract_confluence_live_hierarchy self-seed fallback misses TRASHED status)
  - 2026-07-12 21:15 | discovered-by: B1 (litmus self-heal implementation, d413432) | severity: MEDIUM | OPEN (litmus-flow.sh at 97.9% of file-size ceiling)
  - 2026-07-12 21:15 | discovered-by: B1 (litmus self-heal implementation, d413432) | severity: LOW | OPEN (litmus marker-strip hygiene — stale T2-attach-* markers accumulate)
  - 2026-07-13 | discovered-by: B3 (tag-remediation, attach-sync re-run, verify-against-reality) | severity: MEDIUM | OPEN (verdict asserted a FAIL not backed by a fresh artifact — phantom B3 failure; fresh re-run is a clean PASS → v0.15.0 verdict-rigor guard)
