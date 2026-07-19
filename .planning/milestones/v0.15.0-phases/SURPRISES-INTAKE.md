# v0.15.0 Surprises Intake

> **Append-only intake for surprises discovered during v0.15.0-era execution
> (and items routed forward from prior milestones).**
> Each entry is something the discovering session chose NOT to fix eagerly because it was
> out-of-scope. A v0.15.0 drain phase (OP-8 Slot 1) closes this file.
>
> **Eager-resolution preference:** if a surprise can be closed inside its discovering
> phase without doubling scope (rough heuristic: < 1 hour incremental work, no new
> dependency, no new file outside the phase's planned set), do it there. This file is for
> items that genuinely don't fit.
>
> **Distinction from `GOOD-TO-HAVES.md`:** entries here fix something that's BROKEN,
> RISKY, or BLOCKING. Improvements/polish go in `GOOD-TO-HAVES.md`.

## Entry format

```markdown
## YYYY-MM-DD HH:MM | discovered-by: <source> | severity: BLOCKER|HIGH|MEDIUM|LOW

**What:** One-paragraph description of what was found.

**Why out-of-scope for the discovering session:** Why eager-resolution wasn't possible.

**Sketched resolution:** One paragraph proposing how the drain phase should resolve.

**STATUS:** OPEN  (← drain phase updates to RESOLVED|DEFERRED|WONTFIX with rationale or commit SHA)
```

---

## Entries

## Split index (OP-8 file-size drain)

This ledger exceeded the *.md 20k budget and was split into 7 per-part child files under `surprises-intake/`. Every entry is preserved verbatim; append new entries to the last part (or a new part) and add the title here.

- [`surprises-intake/part-01.md`](surprises-intake/part-01.md) — 7 entries:
  - 2026-07-13 14:00 | discovered-by: quick 260713-q0e (RED-main honest-rework, Manager Ruling #5 Option A) | severity: MEDIUM
  - 2026-07-13 20:30 | discovered-by: b773c04 RED-main arc (SESSION-HANDOVER successor #16 noticing, routed by item-0 cursor refresh) | severity: MEDIUM
  - 2026-07-13 20:30 | discovered-by: b773c04 RED-main arc (SESSION-HANDOVER successor #16 noticing, routed by item-0 cursor refresh) | severity: MEDIUM (verify — provenance unconfirmed, not a proven defect)
  - 2026-07-14 21:00 | discovered-by: quick 260714-rcv (L0 rotation #21 post-tag cursor refresh — carried noticing from rotation #20) | severity: MEDIUM
  - 2026-07-14 21:05 | discovered-by: quick 260714-rcv (L0 rotation #21 post-tag cursor refresh — carried noticing from rotation #20; scope corrected against reality) | severity: LOW
  - 2026-07-14 20:40 | discovered-by: L0 rotation #22 (t4 real-backend re-run, agent-ux/t4-conflict-rebase-ancestry-real-backend cadence) | severity: HIGH
  - 2026-07-14 20:41 | discovered-by: L0 rotation #22 (t4 real-backend re-run, same session as the oid-drift defect above) | severity: MEDIUM
- [`surprises-intake/part-02.md`](surprises-intake/part-02.md) — 9 entries:
  - 2026-07-14 20:42 | discovered-by: L0 rotation #22 (t4 real-backend re-run, pre-release-real-backend cadence) | severity: MEDIUM
  - 2026-07-14 20:43 | discovered-by: L0 rotation #22 (t4 real-backend re-run — preflight vs runner env-loading gap) | severity: HIGH
  - 2026-07-14 20:44 | discovered-by: L0 rotation #22 (t4 real-backend re-run — `--persist` write review) | severity: HIGH
  - 2026-07-15 06:30 | discovered-by: L0 rotation #26 intake-filing leaf (carried forward across workhorse #24→#25→#26 handovers, 2 rotations un-filed) | severity: MEDIUM
  - 2026-07-15 06:35 | discovered-by: manager amendment 4 to L0 rotation #26 (measured on this rotation's push; corroborates workhorse #25's 101s WARN) | severity: LOW-MEDIUM
  - 2026-07-15 21:45 | discovered-by: P115-T2 (BENCH-01 live latency re-measurement) | severity: MEDIUM
  - 2026-07-15 22:00 | discovered-by: P115 roadmap gsd-quick noticing (OD-3) | severity: LOW
  - 2026-07-15 17:18 | discovered-by: L0 rotation #36 (read-only pre-push-spike diagnosis, charter item 2) | severity: LOW
  - 2026-07-16 05:00 | discovered-by: P115 Task-4 capture executor (L0 #38) | severity: BLOCKER
- [`surprises-intake/part-03.md`](surprises-intake/part-03.md) — 10 entries:
  - 2026-07-16 06:05 | discovered-by: P115 Task-4 capture executor (L0 #39) | severity: MEDIUM
  - 2026-07-16 | discovered-by: P117 W1 (noticed during execution; filed by L0 #55 intake triage) | severity: MEDIUM
  - 2026-07-16 07:47 | discovered-by: P115-T5 close-out executor (SURPRISES-INTAKE filing pass) | severity: MEDIUM
  - 2026-07-16 12:00 | discovered-by: P115-T6 Wave 2 item 2 executor (115-UNWAIVE-PATH.md inventory pass) | severity: MEDIUM
  - 2026-07-16 07:50 | discovered-by: P115-T5 close-out executor (SURPRISES-INTAKE filing pass, relaying a T5 executor mid-task noticing) | severity: MEDIUM
  - 2026-07-16 | discovered-by: P115 owner-directive lane doc sweep | severity: MEDIUM
  - 2026-07-16 | discovered-by: P115 owner-directive Wave-1 executor | severity: MEDIUM
  - 2026-07-16 | discovered-by: P115 phase-close cold-reader pass (L0 #44) | severity: MEDIUM
  - 2026-07-16 | discovered-by: P115 cold-reader dispatch (L0 #44) | severity: MEDIUM (tooling, silent-failure class)
  - 2026-07-16 | discovered-by: quick 260716-fmt (GTH-V15-35) | severity: MEDIUM
- [`surprises-intake/part-04.md`](surprises-intake/part-04.md) — 6 entries:
  - 2026-07-16 | discovered-by: manager (w1:p7) finding to L0 #47, verified live | severity: MEDIUM (HIGH-visibility — uncatalogued hero-number surfaces)
  - 2026-07-16 | discovered-by: L0 #48 doc-alignment bind executor (OD-3 no-false-bound noticing) | severity: LOW
  - 2026-07-16 | discovered-by: L0 #48 doc-alignment bind executor | severity: LOW
  - 2026-07-16 23:50 | discovered-by: L0 #53 (P116 phase-close relief writer) | severity: HIGH
  - 2026-07-16 23:59 | discovered-by: P117 W2 push-blocker Step A executor (catalog-corruption unblock) | severity: BLOCKER
  - 2026-07-16 23:59 | discovered-by: P117 W3 sub-lane 117-06 (docs/social freshness gate + dead-code + CLAUDE.md sweep) | severity: MEDIUM
- [`surprises-intake/part-05.md`](surprises-intake/part-05.md) — 10 entries:
  - 2026-07-17 09:35 | discovered-by: P117 W5 phase-close push executor (final-commit tree-writer) | severity: MEDIUM-HIGH
  - 2026-07-17 10:15 | discovered-by: P118 close intake-filing (pre-existing drift surfaced, NOT a P118 defect) | severity: MEDIUM
  - 2026-07-17 | discovered-by: P119 executor (docs/planning-simplification, SC-1/SC-2 audit) | severity: MEDIUM
  - 2026-07-17 | discovered-by: P119 executor (deletion-candidate ref-eval) | severity: LOW
  - 2026-07-17 | discovered-by: P119 executor (RAISE-2a/2b deletion gates) | severity: LOW
  - 2026-07-17 | discovered-by: P119 close (phase-close push cadence) | severity: MEDIUM (velocity/health regression)
  - 2026-07-17 | discovered-by: P119 phase-close tree-writer (executor-dispatch race NOTICED) | severity: LOW (no damage — orchestration-process candidate)
  - 2026-07-17 | discovered-by: P120 CLOSE Wave A (machine-wide build-mutex hazard, EXECUTED evidence) | severity: HIGH
  - 2026-07-17 | discovered-by: P120 phase-close verifier (OD-3 adversarial credential-flow sweep) | severity: MEDIUM
  - From P121 W1 (2026-07-17, registry authoring)
- [`surprises-intake/part-06.md`](surprises-intake/part-06.md) — 4 entries:
  - From P122 W2 (2026-07-18, remote-init hardening)
  - 2026-07-18 11:09 | discovered-by: gsd-executor 123-01 (catalog-first Wave 1) | severity: MEDIUM
  - 2026-07-18 06:00 | discovered-by: 123-06 (SC4/DRAIN-06, structure/verifier-script-exists gate) | severity: MEDIUM
  - 2026-07-18 | discovered-by: 123-07 close-wave Lane 2 (PLANNING-CLOSE + AUDIT, security audit tasked by the coordinator) | severity: MEDIUM
- [`surprises-intake/part-07.md`](surprises-intake/part-07.md) — 12 entries:
  - 2026-07-18 | discovered-by: P123 close wave (push cadence measurement, coordinator-filed) | severity: MEDIUM
  - 2026-07-18 | discovered-by: gsd-verifier P123 phase-close verdict (`quality/reports/verdicts/p123/VERDICT.md`) | severity: LOW-MEDIUM
  - 2026-07-18 | discovered-by: gsd-verifier P123 phase-close verdict, NOTICED #1 (`quality/reports/verdicts/p123/VERDICT.md`) | severity: LOW / INFO
  - 2026-07-18 15:47 | discovered-by: quick-260718-fork (fork-anti-pattern doctrine + intake-filing lane) | severity: MEDIUM
  - 2026-07-18 | discovered-by: P124 W1a + W2 (source=P124, migrated at phase close from phase-local `deferred-items.md`) | severity: LOW-MEDIUM
  - 2026-07-18 | discovered-by: P124 close-bookkeeping lane (shell-coverage FAIL forces `--persist` downgrade-REFUSAL — augments `L1166` / the two drift entries) | severity: MEDIUM
  - 2026-07-18 | discovered-by: P124 close-bookkeeping lane (verdict.py `--phase` bare-session collation reads a misleading RED) | severity: MEDIUM
  - 2026-07-18 | discovered-by: gsd-executor 125-01 (SC3/DRAIN-12 — troubleshooting.md v0.14.0 blockquote shows bare attach-tree recovery that reads the stale mirror) | severity: LOW-MEDIUM | **RESOLVED** by 125-03 PART 2
  - 2026-07-19 | discovered-by: P125 planning + C1 close-bookkeeping lane (gsd-sdk STATE.md corruption, EXECUTED evidence) | severity: MEDIUM
  - 2026-07-19 | discovered-by: P125 planning + C1 ground-truth (tokenworld-mirror doc-truth, verified against reality) | severity: MEDIUM
  - 2026-07-19 | discovered-by: P125 verifier NOTICED (VERDICT #4 — weak OR-assert push_conflict.rs:352-354) | severity: LOW-MEDIUM
  - 2026-07-19 | discovered-by: P125 verifier NOTICED (VERDICT #5 — refresh/litmus env-var divergence) | severity: LOW-MEDIUM
- [`surprises-intake/part-08.md`](surprises-intake/part-08.md) — 5 entries:
  - 2026-07-19 | discovered-by: Cycle-2 bundled `/gsd-quick` task (d) executor | severity: MEDIUM | RESOLVED same commit
  - 2026-07-19 | discovered-by: Cycle-2 task (d) executor (own `--persist` verification pass) | severity: HIGH | CLOSED (P126 W1) — agent-ux/real-git-push-e2e minted_at load-crash landmine defused
  - 2026-07-19 | discovered-by: P126 close-bookkeeping lane | severity: LOW | RESOLVED (confirmed flake) — container-rehearse-sigkill-safe SIGKILL was a flake; clean rerun GREEN
  - 2026-07-19 | discovered-by: P126 close-bookkeeping lane | severity: HIGH | OPEN — container-rehearse-sigkill-safe leaked-process-group-kill took down all ~83 gates + leaked orphan `reposix sim` PID 11014 (same class as cef3a2ea); P127 Slot 1
  - 2026-07-19 | discovered-by: P126 gsd-verifier (WARN-1) | severity: MEDIUM | OPEN — structure/hermetic-test-network-isolation stale local PASS mint vs deterministic ~0.02s CI-sandbox fast-fail (lying catalog row); P127 Slot 1
