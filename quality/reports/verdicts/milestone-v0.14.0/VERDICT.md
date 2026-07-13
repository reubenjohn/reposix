---
milestone: v0.14.0
date: 2026-07-12
verdict: RED
status: NOT READY TO TAG
verifier: unbiased milestone-close verifier subagent (no session context — graded from the recorded probe artifacts, not a re-run)
head_at_grade: 9890a67
blocking_cadence: pre-release-real-backend (9th probe) — exit 1
aggregate_real_backend: 1 PASS / 3 FAIL / 2 NOT-VERIFIED
regrade_condition: "re-grades ONLY after `python3 quality/runners/run.py --cadence pre-release-real-backend` exits 0 AND an unbiased ratification passes"
constraint_notes:
  - "No cargo invoked; the probe was NOT re-run — this verdict grades the recorded artifact + transcript only."
  - "No tag cut, no git push. Explicit-path add of this VERDICT.md only."
  - "Real-backend probe EXECUTED (preflight PASS 3/3, no env wall) and failed behaviorally → hard RED per OD-2, NOT the substrate-absent NOT-VERIFIED SLOT state."
---

# Milestone v0.14.0 — Aggregate Milestone-Close Verdict

## Status: RED — NOT READY TO TAG

The owner-gated, non-skippable 9th probe (`pre-release-real-backend` cadence,
`python3 quality/runners/run.py --cadence pre-release-real-backend`) **EXECUTED for
real against the sanctioned Confluence backend and exited 1.** Preflight passed 3/3
(Confluence REPOSIX reachable, HTTP 200) — there was **no environment wall**, so this
is not the legitimate substrate-absent `NOT-VERIFIED` SLOT state. Per **OD-2
(89-OWNER-DECISIONS.md, binding)** an *executed-and-failed* real-backend probe at
milestone-close is a **hard RED**: no owner-waiver, no `until_date`, no
PASS-with-comment, no skip-counts-as-pass. The milestone cannot close green.

> **OD-2 quoted (root `.planning/CLAUDE.md` "Milestone-close 9th probe (RBF-FW-03)" +
> `agent-ux.json` row `agent-ux/milestone-close-vision-litmus-real-backend`
> `claim_vs_assertion_audit`):** *"Any milestone-close missing
> `python3 quality/runners/run.py --cadence pre-release-real-backend` exit 0 grades RED.
> … once the substrate exists, inability to EXECUTE this probe at milestone-close
> (creds/targets missing or unreachable) is hard RED per OD-2 — no waiver, no
> until_date, no PASS-with-comment."* The probe here did the stronger thing: it
> executed and **failed behaviorally** (the mass-delete guard correctly refused a
> delete-shaped push). Executed + failed = hard RED, distinct from the substrate-absent
> NOT-VERIFIED state.

The RED is **isolated to the real-backend cadence.** The other 8 milestone-close probes
and every non-real-backend cadence + wave-2 phase (P102–P112) were GREEN at the P111
phase-close (`quality/reports/verdicts/p111/VERDICT.md`, HEAD `b1c4b74`). Nothing in this
verdict re-opens those; the block is the 9th probe alone.

## Probe results (milestone-close skeleton, `quality/dispatch/milestone-close-verdict.md`)

| # | Probe | Status | Evidence |
|---|---|---|---|
| 1 | Catalog rows GREEN-or-WAIVED for milestone scope | ✅ GREEN | `quality/reports/verdicts/p111/VERDICT.md` (4/4 P111 rows PASS + p110 guardrail) |
| 2 | Dark-factory simulator arm GREEN | ✅ GREEN | P102–P112 wave-2 closes (ref p111 verdict) |
| 3 | Dark-factory DVCS third arm GREEN | ✅ GREEN | ref p111 verdict |
| 4 | Shipped REQ-IDs traceable to catalog row + verifier artifact | ✅ GREEN | ref p111 verdict |
| 5 | RETROSPECTIVE.md v0.14.0 section distilled (OP-9) | ✅ GREEN | `agent-ux/p111-retrospective-v0.14.0-section` PASS (OP-9 ratification) |
| 6 | Tag-script clean-tree + signed-tag guards passing | ⬜ N/A | tag NOT cut; `tag-v0.14.0.sh` not authored — correct, milestone is RED |
| 7 | No expired waivers without follow-up | ✅ GREEN | ref p111 verdict (no expired-waiver FAIL rows) |
| 8 | CLAUDE.md milestone-shipped subsection landed | ✅ GREEN | ref p111 verdict |
| **9** | **Vision litmus vs real backend (`pre-release-real-backend`)** | **🔴 RED** | **exit 1** — see cadence sub-table below |

## Probe 9 — `pre-release-real-backend` cadence: exit 1 (1 PASS / 3 FAIL / 2 NOT-VERIFIED)

| Row | blast_radius | Grade | Cause |
|---|---|---|---|
| `agent-ux/cadence-pre-release-real-backend` | P1 | ✅ PASS | cadence wiring self-test (exit 0) |
| `agent-ux/milestone-close-vision-litmus-real-backend` | **P0** | 🔴 **FAIL** | mass-delete guard refused delete-shaped push (`matched=2 no_id=1 backend_deleted=1`) |
| `agent-ux/p93-partial-failure-recovery-real-confluence` | **P0** | 🔴 **FAIL** | not root-caused this run — investigate |
| `agent-ux/attach-sync-real-backend` | P1 | 🔴 **FAIL** | not root-caused this run — investigate |
| `agent-ux/t4-conflict-rebase-ancestry-real-backend` | **P0** | 🟠 **NOT-VERIFIED** | verifier script absent AND never git-tracked → missing-verifier → NOT-VERIFIED unconditionally; at milestone-close on a P0 = RED |
| `agent-ux/github-front-door-real-backend` | P1 | 🟠 **NOT-VERIFIED** | same cause — verifier script absent/never-tracked |

**Artifacts / transcripts cross-referenced:**

- P0 vision-litmus (fresh run): `quality/reports/verifications/agent-ux/milestone-close-vision-litmus-real-backend.json` (exit_code 1) + transcript `quality/reports/transcripts/milestone-close-vision-litmus-real-backend.txt` (ts 2026-07-13T00:50:54Z; env includes `REPOSIX_ALLOWED_ORIGINS`, preflight `Confluence REPOSIX reachable (HTTP 200)`).
- P0 t4-conflict: `quality/reports/verifications/agent-ux/t4-conflict-rebase-ancestry-real-backend.json` (`error: verifier not found at quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh`).
- P1 github-front-door: `quality/reports/verifications/agent-ux/github-front-door-real-backend.json` (`error: verifier not found at quality/gates/agent-ux/github-front-door-real-backend.sh`).
- P0 p93-partial-failure: `quality/reports/verifications/agent-ux/p93-partial-failure-recovery-real-confluence.json`.
- P1 attach-sync: `quality/reports/verifications/agent-ux/attach-sync-real-backend.json`.

## Blocking findings

### B1 (P0) — Vision-litmus refused: mass-delete guard on stale-mirror drift

**What.** `reposix attach confluence::REPOSIX --remote-name reposix --mirror-name origin`
reconciled a **delete-shaped** diff: `matched=2 no_id=1 backend_deleted=1`. Page
`id=2818063` (`pages/2818063.md`) is present in the GitHub mirror
(`reposix-tokenworld-mirror`) but **absent from live Confluence** → classified
`BACKEND_DELETED`. The mass-delete guard **correctly refused to push a delete-shaped
diff** and the verifier exited 1. **TokenWorld was NOT mutated** — the guard aborted
before any write.

**Root-cause hypothesis.** Stale-mirror drift: the `reposix-tokenworld-mirror` still
carries a page (`2818063`) that has since been deleted from Confluence. Page `2818063`
is **NOT** a protected fixture (`PROTECTED_IDS` did not shield it). **The guard is
behaving correctly** — this is a data-drift blocker, not a code defect.

**Recommended remediation (OWNER-GATED external mutation).** Whether `2818063` was
*legitimately deleted* from Confluence (mirror is stale → reconcile the mirror down) or
was *erroneously deleted* (Confluence should be restored) is **not determinable from the
probe** and requires an owner decision — it is an external-mutation call, owner-named
target required. Candidate move once the owner rules: mirror-drift reconcile
(`reposix sync --reconcile`) to bring the mirror head into coherence with live
Confluence, then re-run the cadence. **Do not** auto-push the delete.

### B2 (P0) — `p93-partial-failure-recovery-real-confluence`: FAIL (not root-caused)

Exit 1. Not root-caused in this run. A lead worth checking first: the freshest on-disk
artifact (2026-07-06 capture) shows this row's sanctioned-target guard rejecting a
**non-TokenWorld** space (`FAIL: non-sanctioned Confluence space 'REPOSIX' -- only
TokenWorld is sanctioned for this mutating row`). If the current run targeted space
`REPOSIX` while this row is hard-pinned to TokenWorld, that guard — not a partial-failure
regression — may be the FAIL. **Recommended:** dispatch a focused investigation to
confirm the guard-vs-regression distinction before treating it as a P93 code defect.

### B3 (P1) — `attach-sync-real-backend`: FAIL (not root-caused)

Exit 1, empty `asserts_passed`/`asserts_failed` in the artifact.
**Recommended:** investigate against the paired transcript
(`quality/reports/transcripts/attach-sync-real-backend-*.txt`) to determine whether this
is downstream of the same stale-mirror drift as B1 or an independent attach/sync defect.

### B4 (P0) + B5 (P1) — Two verifier scripts absent AND never git-tracked

`agent-ux/t4-conflict-rebase-ancestry-real-backend` (P0) and
`agent-ux/github-front-door-real-backend` (P1) both point at verifier scripts that **do
not exist on disk and were never tracked in git** — the catalog rows reference scripts
that never shipped. Per the framework's **"missing verifier script → NOT-VERIFIED
unconditionally"** rule these grade NOT-VERIFIED (never skip-as-pass, never preserve a
prior PASS). At milestone-close, a **P0** row stuck at NOT-VERIFIED counts as RED.

**Recommended remediation.** Author the two missing verifier scripts
(`quality/gates/agent-ux/t4-conflict-rebase-ancestry-real-backend.sh` and
`quality/gates/agent-ux/github-front-door-real-backend.sh`), commit them, and re-run the
cadence so both rows can execute against a real backend.

**This class was predicted.** The v0.15.0 GOOD-TO-HAVES entry **GTH-V15-03** ("no gate
checks a row's `verifier.script` exists + is executable",
`.planning/milestones/v0.15.0-phases/GOOD-TO-HAVES.md`) — filed 2026-07-12 — describes
exactly this false-positive-contract window: a row can carry a `verifier.script` path
that does not exist on disk, and no pre-commit/pre-push gate structurally verifies it.
B4/B5 are two live instances of that class. The proposed
`quality/gates/structure/verifier-script-exists.sh` gate would have blocked these rows
from landing PASS-eligible with no backing script.

## Evidence-freshness caveat (honesty note — accuracy over optimism)

Of the six `pre-release-real-backend` rows, **three** on-disk artifacts are freshly dated
2026-07-13 and independently corroborate this verdict: the P0 vision-litmus FAIL (B1)
and the two missing-verifier NOT-VERIFIED rows (B4/B5). The remaining three
(`cadence-pre-release-real-backend`, `p93-partial-failure-recovery-real-confluence`,
`attach-sync-real-backend`) carry a **2026-07-06** capture on disk showing an
`env-missing` skip — they do **not** yet corroborate a fresh no-env-wall FAIL from their
own re-persisted artifacts. **This does not change the verdict:** RED is
**over-determined** by the freshly-captured B1 (P0 executed-and-failed) alone, plus the
freshly-captured B4 (P0 NOT-VERIFIED). It IS a recommendation for the fix cycle:
re-persist the p93-partial-failure + attach-sync per-row artifacts from the fresh run so
the audit trail for B2/B3 is complete before ratification.

## Re-grade condition

This milestone **re-grades only after both** of the following hold:

1. `python3 quality/runners/run.py --cadence pre-release-real-backend` **exits 0** — every
   P0/P1 row in the sub-table above PASS or legitimately WAIVED (the P0 vision-litmus row
   carries **no waiver mechanism** by design, so it must genuinely PASS), and
2. an **unbiased ratification** (author ≠ this verifier, per F-K5 / `quality/CLAUDE.md`
   § Verifier-subagent dispatch) passes against the re-run cadence.

Until both hold, **no tag is cut.** No `tag-v0.14.0.sh` guard sequence should be run;
the milestone stays RED.

---

**VERDICT: RED — NOT READY TO TAG.** The 9th probe executed against a real backend and
failed (mass-delete guard correctly refused a stale-mirror delete-shaped diff; two P0/P1
verifier scripts never shipped). Per OD-2 an executed-and-failed real-backend probe at
milestone-close is a hard RED with no waiver path. The block is isolated to the
real-backend cadence; wave-2 phases P102–P112 remain GREEN (p111 verdict). Remediation
is owner-gated (mirror-drift reconcile decision for `2818063`) plus framework work
(author the two missing verifiers; investigate the p93 + attach-sync FAILs). No mutation
of TokenWorld occurred.

_Verifier: Claude (unbiased milestone-close). Graded from recorded probe artifacts +
transcripts; probe NOT re-run, no cargo, no tag, no push._
