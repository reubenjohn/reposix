---
phase: 116
slug: adr-010-mirror-fanout-decision-packet-slug-id-durable-create
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-16
---

# Phase 116 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> **NOTE (skeleton):** P116 is a DOCS + DESIGN-RECORD phase (no code lands this
> milestone) — verification here is **gate/grep-based**, not test-suite-based. The
> planner MUST populate the Per-Task Verification Map from `116-RESEARCH.md` § "6.
> Validation Architecture" (line ~515), which enumerates the concrete gate / grep
> assertions that prove each deliverable's doc now tells the truth. Frontmatter filled
> per plan-phase step 5.5; body is planner-owned.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Doc/gate-based (no unit test framework — this phase edits docs + design records) |
| **Config file** | none — verification runs the existing quality gates |
| **Quick run command** | `bash quality/gates/docs-build/mkdocs-strict.sh` (per touched doc) |
| **Full suite command** | `python3 quality/runners/run.py --cadence pre-push` (doc-alignment walk + banned-words + mermaid + freshness-invariants + mkdocs-strict) |
| **Estimated runtime** | ~55–60s (whole-repo pre-push, per quality/CLAUDE.md) |

*Planner: replace/extend from RESEARCH.md §6 — each deliverable maps to a specific gate
or `grep` assertion on the corrected doc text.*

---

## Sampling Rate

- **After every task commit:** Run the relevant single-gate check (e.g. `mkdocs-strict.sh`, `banned-words.sh`, `doc-alignment` walk on the touched doc).
- **After every plan wave:** Run the full pre-push cadence.
- **Before `/gsd-verify-work`:** Full pre-push cadence must be green.
- **Max feedback latency:** ~60 seconds.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 116-01-01 | 01 | 1 | ADR-01 / FIX-03 | — | N/A (docs) | gate/grep | *(planner: from RESEARCH.md §6)* | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*Planner: expand one row per deliverable — (1) ADR-01 doc-truth + §2 amendment, (2) FIX-03 §3 design-only amendment, (3) packet co-location cross-link — with the exact `grep`/gate command from RESEARCH.md §6 that asserts the corrected claim is present.*

---

## Wave 0 Requirements

- [ ] *(planner: none expected — no new test framework; existing quality gates cover verification. Confirm against RESEARCH.md §6.)*

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| First-time reader can find the decision packet from ADR-010 | ADR-01 (criterion 1) | Cross-link discoverability is a human-legibility property | Open `docs/decisions/010-l2-l3-cache-coherence.md`, confirm the References section links the packet path; follow the link and confirm it resolves |

*Planner: reconcile against RESEARCH.md §6 — the packet co-location deliverable is partly a legibility check.*

---

## Validation Sign-Off

- [ ] Every deliverable has a gate/grep verification or is listed as manual-only with rationale
- [ ] Sampling continuity: no deliverable ships without a green gate proving the corrected doc text
- [ ] Wave 0 covers all MISSING references (expected: none)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
