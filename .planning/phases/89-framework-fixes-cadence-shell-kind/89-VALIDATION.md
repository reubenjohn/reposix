---
phase: 89
slug: framework-fixes-cadence-shell-kind
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-08
---

# Phase 89 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

Source: `89-RESEARCH.md` § "Validation Architecture" — sampled here as the executable contract.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Python 3 stdlib `unittest` (runner-side) + bash (verifier scripts). Existing tests at `quality/runners/test_freshness.py` + `test_freshness_synth.py` are the precedent. |
| **Config file** | none — `python3 -m unittest discover quality/runners` works as-is |
| **Quick run command** | `python3 -m unittest discover -s quality/runners -p "test_*.py"` |
| **Full suite command** | `python3 quality/runners/run.py --cadence pre-push` |
| **Estimated runtime** | ~30 seconds (unit tests <2s; full pre-push sweep ≤30s) |

---

## Sampling Rate

- **After every task commit:** Run `python3 -m unittest discover -s quality/runners`
- **After every plan wave:** Run `python3 quality/runners/run.py --cadence pre-push`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 89-01-01 | 01 | 1 | RBF-FW-01..05, RBF-FW-11 | T-89-01 | Catalog rows mint NOT-VERIFIED before implementation; runner refuses to grade rows missing required fields | mechanical | `python3 quality/runners/run.py --cadence pre-push --dimension agent-ux,structure --dry-run` | ❌ W0 | ⬜ pending |
| 89-02-01 | 02 | 2 | RBF-FW-04 | T-89-04 / Banned tokens leak phase IDs to user-facing stderr | Regex `\bP\d{2,3}-\d+\b` over `crates/**/*.rs` (NOT tests, CHANGELOG.md, planning) BLOCKs commits/pushes | mechanical | `bash quality/gates/structure/banned-production-tokens.sh` | ❌ W0 | ⬜ pending |
| 89-03-01 | 03 | 2 | RBF-FW-01 | — | New `cadence: pre-release-real-backend` env-gates on `REPOSIX_ALLOWED_ORIGINS` + creds; default-skips when unset (NOT-VERIFIED, not RED) | unit | `python3 -m unittest quality.runners.test_realbackend` | ❌ W0 | ⬜ pending |
| 89-03-02 | 03 | 2 | RBF-FW-01 | — | Catalog row tagged with new cadence runs only when env set | integration | `python3 quality/runners/run.py --cadence pre-release-real-backend` | rows ❌ W0 | ⬜ pending |
| 89-04-01 | 04 | 2 | RBF-FW-02 | T-89-02 / Tests claim transport coverage but only invoke wiremock | New `kind: shell-subprocess` verifier drives `reposix init/attach/sync/push` as actual subprocess against real backend; produces transcript artifact at `quality/reports/transcripts/<row>-<ts>.txt` containing argv + env keys (NOT values) + exit + stdout + stderr | smoke | `bash quality/gates/agent-ux/shell-subprocess-example.sh && test -f quality/reports/transcripts/*.txt` | ❌ W0 | ⬜ pending |
| 89-05-01 | 05 | 3 | RBF-FW-05 | T-89-05 / Phase IDs in `crates/` referencing non-existent downstream PLANs silently rot | Pre-push linter greps three patterns (`not yet wired in P\d+`, `lands? (alongside\|in) P\d+`, `substrate-gap-deferred`); cross-references named phase's `.planning/phases/N-*/PLAN*.md`; BLOCKs if no PLAN | mechanical | `bash quality/gates/structure/deferral-pointer-linter.sh` | ❌ W0 | ⬜ pending |
| 89-06-01 | 06 | 3 | RBF-FW-03 | T-89-03 / Milestone-close verdict graded GREEN without real-backend probe | 9th probe SLOT verifier exists + executable; returns NOT-VERIFIED until P91+ substrate; verdict template lives at `quality/dispatch/milestone-close-verdict.md` | smoke | `test -x quality/gates/agent-ux/milestone-close-vision-litmus.sh && test -f quality/dispatch/milestone-close-verdict.md` | ❌ W0 | ⬜ pending |
| 89-07-01 | 07 | 3 | RBF-FW-11 | T-89-06 / Catalog rows ship without claim-vs-assertion accountability | Runner refuses catalog with row dated ≥2026-05-08 missing `claim_vs_assertion_audit` field (≥50 chars); date-cutoff gates legacy P78–P88 rows | unit | `python3 -m unittest quality.runners.test_audit_field` | ❌ W0 | ⬜ pending |
| 89-08-01 | 08 | 4 | all | — | Catalog rows status flips PASS via verifier subagent; CLAUDE.md updated; verdict at `quality/reports/verdicts/p89/VERDICT.md` GREEN | integration | `python3 quality/runners/run.py --cadence pre-push` + verifier dispatch | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `quality/runners/test_realbackend.py` — covers RBF-FW-01 env-gate semantics
- [ ] `quality/runners/test_audit_field.py` — covers RBF-FW-11 cross-check
- [ ] `quality/gates/structure/banned-production-tokens.sh` — RBF-FW-04 verifier
- [ ] `quality/gates/structure/deferral-pointer-linter.sh` — RBF-FW-05 verifier
- [ ] `quality/gates/agent-ux/shell-subprocess-example.sh` — RBF-FW-02 worked example
- [ ] `quality/gates/agent-ux/milestone-close-vision-litmus.sh` — RBF-FW-03 SLOT verifier
- [ ] `quality/dispatch/milestone-close-verdict.md` — verdict template (NEW per RESEARCH § Q-LOC-1)
- [ ] `quality/gates/agent-ux/lib/transcript.sh` — shared transcript-writing helper (per RESEARCH § Q-SHELL-1)
- [ ] 6 catalog rows minted NOT-VERIFIED in `agent-ux.json` + `freshness-invariants.json` (3+3 split per RESEARCH § Q-CATALOG-DIM-1)
- [ ] No framework install needed — Python stdlib + bash already established

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| 9th-probe SLOT semantics: NOT-VERIFIED outcome correctly threads through runner exit codes + verifier subagent grading at milestone-close. | RBF-FW-03 | The full grading path involves a verifier subagent reading the transcript + verdict template — the substrate doesn't exist yet (P91+ delivers it). Manual sanity-check at phase close that the SLOT skeleton is shaped to receive the real probe later. | After 89-06-01 ships, run `bash quality/gates/agent-ux/milestone-close-vision-litmus.sh` and confirm exit code maps to NOT-VERIFIED in runner, not RED. Document the manual confirmation in 89-VERIFICATION.md. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
