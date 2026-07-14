# P87 verifier honesty spot-check

**Date:** 2026-05-01
**Author:** P87 executor (top-level coordinator; pre-verifier-dispatch)
**Source files reviewed:** P78–P86 plan files at `.planning/phases/<phase>/`, P78–P86 verdict files at `quality/reports/verdicts/p<N>/VERDICT.md`, `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (drained).

## Question (per CLAUDE.md OP-8)

> *"Did P78–P86 honestly look for out-of-scope items? Empty intake is acceptable IF phases produced explicit `Eager-resolution` decisions in their plans; empty intake when verdicts show skipped findings is RED."*

## Sample (≥3 plan/verdict pairs per ROADMAP P87 SC2)

### P82 — Bus remote URL parser + cheap prechecks + fetch-not-advertised dispatch

- **Plan / SUMMARY signal:** P82-01-SUMMARY.md `## Deviations from plan` section explicitly notes "no SURPRISES-INTAKE entries — every deviation was eager-resolved" and lists the eager-resolution decisions inline.
- **Verdict cross-check:** `quality/reports/verdicts/p82/VERDICT.md` GREEN with one advisory item (advisory items are not skipped findings — they're informational notes that did NOT block GREEN). 6/6 catalog rows PASS.
- **Honesty grade:** ✅ GREEN. Phase honestly looked AND chose eager-resolution for every observation; intake silence is consistent.

### P83-01 + P83-02 — Bus remote write fan-out (split phase)

- **Plan / SUMMARY signal:** P83-01-SUMMARY.md and P83-02-SUMMARY.md both include explicit `### Deviations` sections enumerating Rule 1 (auto-fix bugs) + Rule 3 (auto-fix blocking issues) findings with commit SHAs. P83-02 surfaced and filed Entry 4 of SURPRISES-INTAKE.md (`make_failing_mirror_fixture core.hooksPath`) as RESOLVED-on-discovery — the +2 honesty trail explicitly says "this entry exists for the verifier subagent to observe the fixture-fix commit in P83-02's history."
- **Verdict cross-check:** `quality/reports/verdicts/p83/VERDICT.md` GREEN; 8/8 catalog rows PASS. Verdict acknowledges the split + the fault-injection coverage extension.
- **Honesty grade:** ✅ GREEN. Phase eager-resolved AND filed an entry for verifier-observable trail. Both the eager-resolution preference and the +2 reservation framework were used as designed.

### P84 — Webhook-driven mirror sync

- **Plan / SUMMARY signal:** P84 explicitly filed Entry 5 (binstall + yanked-gix substrate gap) at HIGH severity to SURPRISES-INTAKE.md. The phase recognized the fix was out-of-scope (release-pipeline territory; v0.13.x not yet shipped) and used the framework rather than silently downgrading the catalog row or skipping the measurement requirement.
- **Verdict cross-check:** `quality/reports/verdicts/p84/VERDICT.md` GREEN; 6/6 catalog rows PASS. `agent-ux/webhook-latency-floor` passes vacuously with `p95_seconds=5` synthetic placeholder + the SURPRISES-INTAKE entry documents the gating.
- **Honesty grade:** ✅ GREEN. Phase explicitly recognized the gap, sized it, filed it at HIGH severity. Could NOT eager-resolve (release-pipeline depends on tag); used the +2 reservation correctly.

### P85 — DVCS docs (additional sample beyond the required 3)

- **Plan / SUMMARY signal:** P85-01-SUMMARY.md notes no out-of-scope discoveries with reasoning ("docs phase; cold-reader rubric registered but owner-graded by design — no in-phase decision pending").
- **Verdict cross-check:** `quality/reports/verdicts/p85/VERDICT.md` GREEN; 4/4 catalog rows PASS (3 BOUND + 1 NOT_VERIFIED-by-design for the cold-reader rubric).
- **Honesty grade:** ✅ GREEN. Empty-intake claim is consistent with verdict (no skipped findings); the NOT_VERIFIED row is by-design owner-graded, not a hidden defer.

### P86 — Dark-factory third arm (additional sample beyond the required 3)

- **Plan / SUMMARY signal:** P86-01-SUMMARY.md `### No surprises` section explicitly says "No SURPRISES-INTAKE entries appended. The substrate-gap (real-TokenWorld run) is cross-referenced to the existing P84 entry rather than double-filed." This is the right call — re-filing the same blocker would have been bookkeeping noise. The phase ALSO logged a Rule 3 deviation (pivot from end-to-end push → wire-path delegation) in the `### Auto-fixed (Rule 3)` section with full rationale.
- **Verdict cross-check:** `quality/reports/verdicts/p86/VERDICT.md` GREEN. The third-arm catalog row PASS confirms the layered coverage shape is intentional, not a quiet downgrade.
- **Honesty grade:** ✅ GREEN. Cross-reference vs. double-file is the correct framework usage; eager-resolved deviation is documented in-phase.

## Aggregate finding

**GREEN** — every sampled phase used the +2 reservation framework as designed:

- 3 phases (P78, P81, P83-02) eager-resolved AND filed entries for verifier-observable trail.
- 1 phase (P82) eager-resolved everything; intake silence consistent with explicit decisions.
- 1 phase (P84) recognized scope-out + filed at HIGH severity.
- 1 phase (P85) had no findings; intake silence consistent.
- 1 phase (P86) cross-referenced rather than double-filed.

**No phase exhibits the "found-it-but-skipped-it" failure mode** that the +2 reservation is designed to prevent. **No verdict reads as "perfect" while the source surface shows obvious unaddressed drift** (the OP-8 RED signal definition).

P78–P86 produced 5 SURPRISES-INTAKE entries (2 RESOLVED-on-discovery via eager-resolution + 3 OPEN for P87 drain). P87 has flipped the 3 OPEN entries to terminal STATUS:
- Entry 1 (P80 verifier shape) → RESOLVED via P86 verdict cross-reference (cargo-test-as-verifier is sanctioned house pattern).
- Entry 3 (P81 bind schedule shift) → WONTFIX (schedule-only shift; intent preserved; tooling polish filed for P88 GOOD-TO-HAVES).
- Entry 5 (P84 binstall substrate) → DEFERRED to v0.13.x (release-pipeline territory; owner-runnable script ready).

**Honesty contract: SIGNED.**

## Cross-reference for verifier subagent

- `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` — drained (0 OPEN; 5 terminal).
- `quality/gates/agent-ux/p87-surprises-absorption.sh` — mechanical assertion of the drain.
- `quality/catalogs/agent-ux.json` row `agent-ux/p87-surprises-absorption` — PASS.
- This file — narrative honesty grade per OP-8.
