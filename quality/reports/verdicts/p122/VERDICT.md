---
phase: 122-remote-init-hardening
verdict: GREEN
verified: 2026-07-18T09:00:00Z
verifier: gsd-verifier (opus tier), unbiased phase-close gate
graded_state: 985e7dc2 (origin/main == HEAD)
ci: run 29637816791 @ 985e7dc2 concluded success (verified via gh; newest ci.yml on main)
score: 3/3 (SC1 DRAIN-07, SC2 DRAIN-08, SC3 DRAIN-09) PASS
p0_probe: code/ci-green-on-main PASS (0.78s) — "latest ci.yml run on main is GREEN"
persist_downgrade: NONE (569 catalog rows byte-identical before/after --persist)
methodology: goal-backward — ran the phase-close post-push cadence, then graded each
  SC against reality: drove the real prebuilt binary to a LIVE RPX-0406 refusal + the
  /tmp dark-factory flow + `reposix explain`, ran all three verifier gates to exit 0,
  and ran the underlying bin-target + integration regression tests directly to confirm
  the 5 resolve_import_parent arms and the 5 init-refusal cases (a)-(e) execute (not
  filtered to 0) and assert their namesakes. SUMMARY/REVIEW claims were not taken on faith.
---

# Phase 122: `reposix-remote` + `init` hardening — VERDICT

**Phase goal:** The git helper and `reposix init` close two HIGH-severity carry-forward
robustness gaps from the v0.14.0 intake (GTH-V15-04/05/06 → DRAIN-07/08/09).

**Verdict: GREEN.** All three success criteria are genuinely met on the current pushed
tree (`985e7dc2`), the P0 ci-green-on-main probe passes on main's newest ci.yml run, no
`--persist` catalog downgrade occurred, and all three SHIP-WITH-NITS review warnings are
fixed in the shipped code (confirmed live, not just claimed).

---

## Phase-close cadence + P0 probe

`python3 quality/runners/run.py --cadence post-push --persist` → **exit 0**, `1 PASS`.
The only in-scope row is the P0 gate:

```
[PASS] code/ci-green-on-main  (P0, 0.78s)
```

Artifact `quality/reports/verifications/code/ci-green-on-main.json`: `exit_code: 0`,
assert "latest ci.yml run on main concluded success". Cross-checked against reality with
`gh run list --workflow=ci.yml --branch=main`: newest run is **29637816791**, headSha
**985e7dc2** (== HEAD), conclusion **success**. The P0 probe attests the correct run.

**`--persist` downgrade check:** snapshotted all 569 catalog rows before and after the
persist run; `diff` is empty. No GREEN→worse downgrade. (The known-HIGH persist-rewrite
risk did not materialize this run.)

---

## Per-SC grade (goal-backward, against reality)

### SC1 — DRAIN-07: rebase-recovery gate exits 0 on modern git (≥2.34) OR stateless-connect divergence documented+gated — **PASS**

`bash quality/gates/agent-ux/rebase-recovery-reconciles.sh` → **exit 0** on git **2.50**.
Beyond the legacy import legs (Scenarios A/B/C, parent-chaining + clobber + deletion
guards all PASS), the STATELESS-CONNECT legs now run the **real protocol-v2 read path**
(`protocol.version` UNSET) for both drift scenarios and converge:

- Stateless Scenario A (peer git-push drift): `git pull --rebase && git push` exits 0,
  issue4 v1→v2 converged; **GIT_TRACE_PACKET transport proof** = `command=fetch` +
  `version 2`, **ZERO** fast-import/reposix-import signatures (the real stateless-connect
  READ path, not the legacy import path).
- Stateless Scenario B (external REST PATCH drift): same, issue5 v1→v2.
- Deterministic verdict: **Branch A convergence** — "no bare TODO-skip remains on modern
  git … never a silent skip or a faked green" (P105 §5 resolved, RBF-LR-03 is
  import-path/git-version-scoped; no second cache-side fix site). The `rebase-recovery`
  catalog row also runs at `pre-pr` cadence, so CI's ubuntu-latest exercises these legs.

### SC2 — DRAIN-08: `resolve_import_parent()` errors loudly (not silent-degrade) on a non-ref-absence git failure, proven by a regression test — **PASS**

`bash quality/gates/agent-ux/import-parent-resolve-fails-loud.sh` → **exit 0**. The 5
bin-target regression tests (graded via the bare `cargo test -p reposix-remote`, per the
bin-target seam rule) all pass and assert the full tri-state:

```
resolve_import_parent_tests::ref_absent_exit_1_returns_ok_none ... ok          (benign absence → Ok(None))
resolve_import_parent_tests::non_absence_exit_128_errors_loud_with_rpx0508 ... ok
resolve_import_parent_tests::spawn_failure_errors_loud_with_rpx0508 ... ok
resolve_import_parent_tests::anomalous_exit_0_empty_stdout_errors_loud_with_rpx0508 ... ok
resolve_import_parent_tests::present_ref_returns_some_import_parent ... ok
```

The loud-arm tests assert the `Err` carries `RPX-0508` + `Fix:` + `Recovery:` +
`Explain: reposix explain RPX-0508` (init.rs test module) — a genuine coded teaching,
never `Ok(None)`. RPX-0508 is registered in codes.rs and emitted via `fail_push`. These
regression tests run in CI's cargo-test job independent of the on-demand gate row.

### SC3 — DRAIN-09: subprocess/worktree `reposix init` targeting the shared tree is refused (Rust-compiler-grade), while /tmp dark-factory + `attach` still succeed — **PASS**

**Live reality check (prebuilt binary, hook-safe `cd /tmp` effective-target):** init into
a fresh subdir nested inside a non-/tmp git tree was **refused, exit 1, target NOT
created (pre-mutation)** with a Rust-compiler-grade RPX-0406 error — it names the
enclosing tree + target + `[RPX-0406]`, teaches the fix, suggests `reposix attach` + /tmp
as the alternative, prints two copy-paste recovery commands, and points to `reposix
explain RPX-0406`. The `/tmp` nested flow was **NOT refused** (proceeded past both latches,
`target/.git` created), and `reposix explain RPX-0406` renders the rustc-explain-grade
extended cause distinguishing latch 1 (conservative heuristic) from latch 2 (precise cut).

`bash quality/gates/agent-ux/init-refuses-nested-in-shared-tree.sh` → **exit 0**. All 5
cases execute and pass (19/19 in `errors_teach_recovery`, not filtered to 0):

```
init_nested_in_non_tmp_repo_refuses_with_rpx0406 ... ok            (a) refuse + RPX-0406 + attach + Fix + Recovery
init_fresh_subdir_under_tmp_clone_is_not_refused ... ok            (b) /tmp flow preserved (git init reached)
init_via_symlink_into_non_tmp_repo_refuses_with_rpx0406 ... ok     (c) symlink/.. smuggle defeated
attach_nested_checkout_is_not_blocked_by_init_refusal ... ok       (d) attach un-regressed (init-only refusal)
init_worktree_shared_git_dir_aborts_before_config_write ... ok     (e) latch 2: GIT_DIR shared store → RPX-0406, shared config byte-unchanged
```

The `non_tmp_dir` test helper asserts its root is genuinely non-/tmp (fails loud
otherwise), so cases (a)/(c)/(d) cannot silently pass by landing under /tmp — honest
tests that assert their namesakes. Latch 2 (case e) genuinely fires (config
byte-identical + no leaked reposix keys), not a no-op.

---

## Requirements coverage

| Req | SC | Status | Evidence |
|-----|----|--------|----------|
| DRAIN-07 | SC1 | SATISFIED | rebase-recovery gate exit 0, stateless-connect wire proof, Branch A |
| DRAIN-08 | SC2 | SATISFIED | RPX-0508 loud tri-state, 5 bin-target tests green, gate exit 0 |
| DRAIN-09 | SC3 | SATISFIED | RPX-0406 live refusal + 5 cases a-e green, /tmp + attach preserved |

---

## REVIEW.md nit disposition (SHIP-WITH-NITS: 0 critical, 2 warning, 3 info)

| Nit | Sev | Status | Evidence |
|-----|-----|--------|----------|
| WR-01 | MED | **FIXED** (`f5974ebe`) | Latch-1 mechanism reworded to "CONSERVATIVE, defense-in-depth heuristic, not the precise cut" across codes.rs cause, init.rs docstring, RPX-0406 runtime message (confirmed LIVE) + `reposix explain RPX-0406` |
| WR-02 | LOW | **FIXED** (`f5974ebe`) | `is_tmp_safe` doc (init.rs:231-234) now explicitly excludes macOS `/var/folders`, cites WR-02, keeps hook parity |
| IN-03 | NIT | **FIXED** (`f5974ebe`) | D2 narrative made four-way consistent (codes.rs + init.rs + crates/CLAUDE.md + docs/reference/error-codes.md) |
| IN-01 | LOW | No action (documented) | import path emits a push-shaped protocol line — pre-existing, mirrors RPX-0507 exactly; teaching correctly rides stderr diag. REVIEW: "Fix: None required" |
| IN-02 | LOW | No action (documented) | latch 2 guarantees "no config write reaches shared" (not "no git command touches shared") — residual `git init` re-init is provably non-destructive (case e config byte-unchanged). REVIEW: "Fix: None required" |

All warnings fixed; both info items are explicitly no-action-required in the review. None
require filing.

---

## Noticing (OD-3 §2)

- **The two NEW P122 catalog rows are `on-demand` cadence, so they sit at NOT-VERIFIED at
  tip and are NOT graded by any CI cadence (pre-push/pre-pr/post-push).** Their GREEN
  status is only reachable by running the gates directly (done here — both exit 0). This
  is NOT a goal gap: the underlying regression TESTS (`errors_teach_recovery` cases a-e;
  `resolve_import_parent_tests`) are ordinary crate tests that DO run in CI's cargo-test
  job, so a real regression would still be caught. But the contract ROWS won't flip to
  PASS without a `run.py --cadence on-demand --persist`. **Flag for the close executor /
  milestone-close:** either run on-demand-persist to flip both rows to PASS, or confirm
  on-demand-only membership is intentional (contrast: the SC1 `rebase-recovery` row is
  pre-push/pre-pr/on-demand and IS CI-covered).
- **`/tmp` safe-zone logic is duplicated in two languages** (Rust `is_tmp_safe` + bash
  `is_safe`) with no shared source of truth — they agree today (tested) but can drift.
  Already filed by W3 as a cross-language congruence-test GOOD-TO-HAVE.
- **`init.rs` ~56k / `codes.rs` ~55k chars** (2.8×/2.7× the 20k `.rs` soft ceiling) and
  **rebase-recovery-reconciles.sh ~42k** (4× the 10k `.sh` ceiling, GTH-V15-78) — all
  WAIVED until 2026-08-08. init.rs has no single-source-of-truth charter; a real split
  candidate (init/refuse.rs). Already noted by W3/W4.
- **`teach_scan.py` is on-demand cadence** — its RED rotted undetected P121→P122 (W3
  found 26 RED cli blocks, not the 5 the W2 sketch predicted). W3 resolved it and filed a
  candidate to promote `teach_scan` to a pre-push gate. Worth the promotion.

---

_Verified: 2026-07-18_
_Verifier: Claude (gsd-verifier, opus tier) — unbiased phase-close gate_
