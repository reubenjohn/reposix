# v0.14.0 Surprises Intake (P110 source-of-truth) — Part 3 of 3

> Split from `SURPRISES-INTAKE.md` for the file-size gate (OP-8 drain). Index: `../SURPRISES-INTAKE.md`. Entries preserved verbatim.

## 2026-07-13 | discovered-by: B2 (v0.14.0 tag-remediation lane, p93 harness fix, verify-against-reality) | severity: HIGH

**Title:** Real-Confluence partial-failure RECOVERY does not converge — `partial_failure_recovery_real_confluence` push 2 fails deterministically with `some-actions-failed`; p93 (P0) blocks the v0.14.0 tag for a NEW reason once the self-reject is cleared. **[owner decision required]**

**What:** B2 pinned the p93 verifier to the sanctioned TokenWorld space (commit `311d7fe`),
clearing the self-reject that previously exit-1'd the row BEFORE any backend call. With the pin,
the `partial_failure_recovery_real_confluence` smoke (`crates/reposix-cli/tests/agent_flow_real.rs`)
now actually runs against real Confluence — and a DEEPER, previously-MASKED failure surfaces. Push 1
(create page A top-level + page B under an unresolvable parent) partially-fails AS DESIGNED (asserts
:657-661 PASS; page A LANDS in TokenWorld). Push 2 — the RECOVERY push (re-send already-landed A +
retry B with a valid parent) — is asserted to converge (`ok refs/heads/main`) after PRECHECK B diffs
away the content-equivalent already-landed A and replans only B. Instead it fails DETERMINISTICALLY
(identical across two credentialed runs 2026-07-13T02:45Z / 02:48Z — NOT a consistency flake):

```
crates/reposix-cli/tests/agent_flow_real.rs:690:5:
push 2 (recovery) must succeed and converge; stdout=error refs/heads/main some-actions-failed
```

Strong hypothesis (not yet proven to the failing action): PRECHECK B does NOT diff away the
already-landed page A against real Confluence, so page A gets a duplicate-title CREATE that Confluence
rejects (Confluence enforces unique titles within a space) → `some-actions-failed`. The wiremock-sim
twin `crates/reposix-remote/tests/partial_failure_recovery.rs` does NOT model unique-title
enforcement, so it stays GREEN while the real backend does not — a real-backend-only
recovery-convergence gap. Evidence (both runs, verbatim transcripts):
`.planning/quick/260712-phc-author-two-missing-pre-release-real-back/B2-p93-proof.txt`.

**Why out-of-scope for B2 / why FILE not eager-fix:** B2's charter was the harness space-targeting
self-reject, which is FIXED. This is a distinct defect in load-bearing helper / PRECHECK-B export
protocol + real-backend semantics (unique-title / read-after-write). A correct fix needs its own
diagnostic (confirm WHICH action fails via the cache/confluence audit rows) + design (does PRECHECK B
match on title? id? content-hash? how does it dedupe an already-landed create on retry?) + real-backend
regression — >1h, load-bearing protocol code, Rule-4 architectural. Folding it into B2 would double
scope.

**Sketched resolution:** (1) Diagnose the failing action deterministically — instrument
`run_helper_export_real` to surface the helper's per-action stderr, or read `audit_events_cache` /
the confluence adapter audit rows for push 2, to confirm A-duplicate vs B-fail. (2) If A-duplicate:
make PRECHECK B's replan recognise the already-landed content-equivalent page A (match on the record
`id` / content-hash, not a blind re-create) so retry does not re-POST it. (3) Add the real-backend
regression back through `quality/gates/agent-ux/p93-partial-failure-recovery-real-confluence.sh`
(now correctly TokenWorld-pinned) — it is written to flip PASS (exit 0) once push 2 converges.
Test-hygiene sub-item: on the recovery-fail path the smoke leaves an orphan
`p93 smoke A (kind=test <ts>)` page in TokenWorld each run (page A lands, recovery never converges to
clean up) — a human title-sweep per `docs/reference/testing-targets.md` is needed; two orphans exist
from the B2 verification runs.

**OWNER DECISION REQUIRED (tag-blocking):** the p93 row carries `blast_radius: P0` and runs at the
`pre-release-real-backend` cadence, so with creds present it grades **exit 1 (FAIL, not
NOT-VERIFIED)** — the v0.14.0 tag remains BLOCKED by p93 for THIS new reason, not the (now-cleared)
self-reject. The owner must choose: (a) fix the recovery-convergence gap before the tag (dedicated
phase), or (b) explicitly waive p93 with a documented `until_date` per the coverage-kind honesty rule
(never a bare PASS-with-comment). Do NOT cut the tag treating p93 as green.

**STATUS:** DEFERRED — dedicated real-backend recovery-convergence fix phase (Rule-4 architectural).
Distinct from the two B1 mirror-refresh doc-gaps in `part-02.md`. Tracked as the active p93 blocker
for the v0.14.0 tag; owner decision (fix-before-tag vs documented waiver) pending.
