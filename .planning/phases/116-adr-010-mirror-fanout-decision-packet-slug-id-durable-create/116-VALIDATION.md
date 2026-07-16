---
phase: 116
slug: adr-010-mirror-fanout-decision-packet-slug-id-durable-create
status: ready
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-16
---

# Phase 116 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> **DOCS + DESIGN-RECORD phase (no `crates/` code lands this milestone).** Verification is
> **gate/grep-based**, not test-suite-based. Every deliverable below maps to a concrete
> `grep`/gate assertion (from RESEARCH §6) proving the corrected doc/record now tells the
> truth. The single piece of new "test infrastructure" is one optional-but-recommended
> `doc-alignment` regression-guard row + verifier (Plan 01 Task 1, catalog-first).
>
> **Grep-gate hygiene (verified this session):** the bare word `webhook` already appears in
> both live docs (3× CLAUDE.md, 11× dvcs-topology.md) — a `grep webhook` gate is
> TAUTOLOGICAL. The discriminating token is `authoritative` (ZERO occurrences in either file
> today). All live-doc gates below key on `authoritative` + the new phrase fragments.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Doc/gate-based — `quality/runners/run.py` (catalog-driven shell verifiers). No `cargo test` surface. |
| **Config file** | `quality/catalogs/doc-alignment.json`, `quality/catalogs/freshness-invariants.json` |
| **Quick run command** | `bash quality/gates/docs-alignment/walk.sh && bash quality/gates/docs-build/mkdocs-strict.sh && bash quality/gates/structure/banned-words.sh --all` |
| **Full suite command** | `python3 quality/runners/run.py --cadence pre-push` |
| **New verifier (this phase)** | `quality/gates/docs-alignment/mirror-convergence-blessed.sh` (Plan 01 T1 — grep phrase-fragment on `authoritative`+`refresh-tokenworld-mirror.sh`; reflow-tolerant; MUST fail before Task 2, pass after) |
| **Estimated runtime** | ~55–60s (whole-repo pre-push, per quality/CLAUDE.md) |

---

## Sampling Rate

- **After every task commit:** the relevant single-gate check + the task's own `grep`
  assertion (see Per-Task Verification Map). For live-doc edits: `mkdocs-strict.sh` +
  `banned-words.sh --all` + the docs-alignment `walk.sh` on the touched doc.
- **After every plan wave:** full `python3 quality/runners/run.py --cadence pre-push`.
- **Before `/gsd-verify-work` / phase close:** full pre-push cadence green; then the push
  cadence per `.planning/CLAUDE.md` (`git push origin main` → `run.py --cadence post-push
  --persist` → `code/ci-green-on-main` P0). **NOTE: this planner commits locally only — the
  L0 coordinator owns the push at its relief boundary.**
- **Max feedback latency:** ~60 seconds.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 116-01-01 | 01 | 1 | ADR-01 | T-116-01 | catalog mutated only via bind CLI, not hand-edit; guard is a REAL gate (fails pre-edit) | gate | `test -x quality/gates/docs-alignment/mirror-convergence-blessed.sh && bash -n quality/gates/docs-alignment/mirror-convergence-blessed.sh && ! bash quality/gates/docs-alignment/mirror-convergence-blessed.sh 2>/dev/null` | ❌ W0 (created this task) | ⬜ pending |
| 116-01-02 | 01 | 1 | ADR-01 | T-116-02 | N/A (public docs) | grep+gate | `bash quality/gates/docs-alignment/mirror-convergence-blessed.sh && grep -qF "authoritative" CLAUDE.md && grep -qF "authoritative" docs/concepts/dvcs-topology.md && grep -qF "refresh-tokenworld-mirror.sh" docs/concepts/dvcs-topology.md` | ✅ (both docs exist) | ⬜ pending |
| 116-02-01 | 02 | 1 | ADR-01 | T-116-03 | ratified prose byte-unchanged (append-only) | grep | `grep -qF "GTH-V15-38" docs/decisions/010-l2-l3-cache-coherence.md && grep -qF "8212373" docs/decisions/010-l2-l3-cache-coherence.md` | ✅ | ⬜ pending |
| 116-02-02 | 02 | 1 | FIX-03 | T-116-03 | waiver stays (qualify, not remove) | grep | `grep -qF "SANCTIONED TARGET DESIGN" docs/decisions/010-l2-l3-cache-coherence.md && grep -qF "WAIVED for v0.13.0" docs/decisions/010-l2-l3-cache-coherence.md` | ✅ | ⬜ pending |
| 116-02-03 | 02 | 1 | ADR-01+FIX-03 (crit-1) | T-116-04 | provenance cited (date+SHA) | grep+gate | `grep -qF "P116-ADR-010-DECISION-PACKET.md" docs/decisions/010-l2-l3-cache-coherence.md && bash quality/gates/docs-build/mkdocs-strict.sh` | ✅ | ⬜ pending |
| 116-03-01 | 03 | 1 | ADR-01 | T-116-05, T-116-06 | correct LIVE row flipped; twin untouched | grep+awk | `awk '/^## 2026-07-14 20:42 \| discovered-by: L0 rotation #22 \(t4 real-backend re-run, pre-release-real-backend/{f=1} f&&/^\*\*STATUS:\*\*/{print; exit}' .planning/milestones/v0.15.0-phases/SURPRISES-INTAKE.md \| grep -q "RESOLVED"` | ✅ | ⬜ pending |
| 116-03-02 | 03 | 1 | FIX-03 | T-116-05 | provenance cited; boundary-relative TAG | grep | `grep -qF "SANCTIONED TARGET DESIGN" .planning/GOOD-TO-HAVES.md && grep -A34 "GOOD-TO-HAVES-09" .planning/GOOD-TO-HAVES.md \| grep -qiE "next milestone boundary\|then-current milestone boundary"` | ✅ | ⬜ pending |
| PHASE-G1 | all | close | ADR-01+FIX-03 | — | design-only: NO v0.15 build | gate | `[ "$(git diff --stat -- crates/ \| wc -l)" -eq 0 ]` | ✅ | ⬜ pending |
| PHASE-G2 | all | close | ADR-01 (constraint) | — | P115 RETIRE gate untouched | gate | `[ "$(jq '[.rows[]\|select(.last_verdict=="RETIRE_PROPOSED")]\|length' quality/catalogs/doc-alignment.json)" -eq 11 ]` | ✅ | ⬜ pending |
| PHASE-G3 | 03 | close | ADR-01 (constraint) | T-116-06 | archived v0.14.0 twin frozen | gate | `[ "$(git diff --stat -- .planning/milestones/v0.14.0-phases/ \| wc -l)" -eq 0 ]` | ✅ | ⬜ pending |
| PHASE-G4 | all | close | ADR-01 | — | no doc-alignment/mkdocs regression | gate | `bash quality/gates/docs-alignment/walk.sh && bash quality/gates/docs-build/mkdocs-strict.sh && bash quality/gates/docs-build/mermaid-renders.sh && bash quality/gates/structure/banned-words.sh --all` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Constraint → verification cross-map (L0 handover §5 hard constraints a–f):**
- **(a)** packet co-location cross-link → 116-02-03 (`grep -qF "P116-ADR-010-DECISION-PACKET.md"`)
- **(b)** retire the LIVE row (not archived twin); real gap = MISSING blessing → 116-03-01 (LIVE-row `awk`) + PHASE-G3 (twin zero-diff) + 116-01-02 (blessing ADDED — `authoritative` gate)
- **(c)** BOTH FIX-03 AND ADR-01 → ADR-01: 116-01-02/116-02-01/116-03-01; FIX-03: 116-02-02/116-03-02
- **(d)** FIX-03 DESIGN-ONLY, no code → PHASE-G1 (`crates/` zero-diff), enforced across all plans
- **(e)** terminal-status shape (first in ledger, archived analog) → 116-03-01 (RESOLVED + provenance; no in-file precedent noted in SUMMARY)
- **(f)** doc-alignment rebind risk LOW (edits below anchors) → PHASE-G4 (walk exits 0, no STALE_DOCS_DRIFT)

---

## Wave 0 Requirements

- [ ] **Plan 01 Task 1 (catalog-first):** mint `docs-alignment/mirror-convergence-authoritative-bound`
      row + `quality/gates/docs-alignment/mirror-convergence-blessed.sh` verifier as the
      phase's FIRST commit, BEFORE the Task-2 prose edits that make it GREEN. This is the
      ONLY new test infrastructure. The verifier keys on the DISCRIMINATING token
      `authoritative` (0 occurrences today) + `refresh-tokenworld-mirror.sh` — NOT the
      already-present bare `webhook` — so it provably fails before Task 2 and passes after.
      RESEARCH §6 recommends it (the corrected claim has ZERO programmatic guard today; this
      exact conflation regressed once already).
      **TOOLING-GATED:** mint via `reposix-quality doc-alignment bind` only; if that CLI is
      unavailable, do NOT hand-edit the JSON — keep the grep verifier, file a GOOD-TO-HAVES
      follow-up, and fall back to the grep-based §6 checks (sufficient for phase-close grading).

*All other verification uses existing quality gates (mkdocs-strict, mermaid, banned-words,
docs-alignment walk, freshness-invariants) — no new framework needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| First-time reader can find the decision packet from ADR-010 | ADR-01 (criterion 1) | Cross-link discoverability is a human-legibility property; the `grep` (116-02-03) proves the path string is present but not that a reader *reaches* it | Open `docs/decisions/010-l2-l3-cache-coherence.md`, scroll to `## References`, confirm the packet bullet is present as a backtick path; `cat` that path and confirm it resolves to the ruled packet |
| Live docs read as a coherent blessing (not a bolted-on sentence) | ADR-01 | Prose coherence / progressive-disclosure quality is subjective | Read the edited § in `dvcs-topology.md` and the CLAUDE.md bullet cold; confirm the webhook+cron blessing + (a)/(b) split read naturally and do not duplicate/contradict the existing `sync --reconcile` sentence |

---

## Validation Sign-Off

- [x] Every deliverable has a gate/grep verification or is listed as manual-only with rationale
- [x] Sampling continuity: no deliverable ships without a green gate proving the corrected doc/record text
- [x] Wave 0 covers all MISSING references (one: the regression-guard row/verifier, catalog-first)
- [x] Every live-doc gate keys on a discriminating token (`authoritative`), not a tautological one (`webhook`)
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter
- [x] Every hard constraint (a)–(f) maps to a named verification row (constraint cross-map above)

**Approval:** ready for execution (top-level coordinator dispatches Wave 1 plans 01/02/03;
Plan 01 T1 is the catalog-first first commit).
