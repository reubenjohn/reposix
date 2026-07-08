# v0.13.1 "Front door actually works" — phase-close verification (OP-7)

**Verifier:** unbiased gsd-verifier subagent (zero session authorship of the work graded).
**Verified:** 2026-07-08T00:10Z
**Repo:** `/home/reuben/workspace/reposix`, branch `main`.
**Ground truth:** `HEAD == origin/main == a374a4b` (verified via `git rev-parse`), committed
tree clean, `git config user.email == reubenvjohn@gmail.com`.
**Overall verdict:** **GREEN — tag-ready.** All 6 milestone DoD items PASS on independently
re-run gates/tests/flow. One release-decision item (`pre-release-real-backend` 9th probe) is
surfaced to L0 below — it is L0's call, not a verifier RED.

The core fix (`ee5f909`) — `reposix sim` runs the simulator **in-process** with a compiled-in
offline builtin seed — is present and demonstrably works: independently reproduced this session
on **git 2.25.1, fully offline**, serving 6 issues with no `--seed-file`/curl.

---

## DoD grades

| # | DoD item | Verdict | Evidence (command / file / exit) |
|---|----------|---------|----------------------------------|
| 1 | Zero-shot onboarding gate GREEN (+ dark-factory-sim honest) | **GREEN** | `quality/gates/agent-ux/zero-shot-onboarding.sh` → **exit 0**, 18 asserts pass, `seed_source=builtin`, artifact `quality/reports/verifications/agent-ux/zero-shot-onboarding.json` written this session. Flow ran offline on git 2.25.1: `builtin seed loaded inserted=6`, init→checkout→cat→edit→commit→push all exit 0, zero fixups. `dark-factory.sh sim` → **exit 0**, artifact `dark-factory-sim.json` (3 asserts, conflict+blob-limit teaching strings present). Row `agent-ux/dark-factory-sim` re-minted honestly (`5122ebb` added `REPOSIX_SIM_ORIGIN` export closing the phantom-green). |
| 2 | `reposix init` exits non-zero on unreachable backend | **GREEN** | `crates/reposix-cli/src/init.rs:269-314` — "An unreachable backend is a HARD ERROR, not a warning" + `bail!` on no `refs/reposix/origin/*` synced; best-effort-always-`Ok` doc-comment removed (doc-comment at `:182-185` now states non-zero-exit contract). Test `crates/reposix-cli/tests/no_truncate.rs:100 no_truncate_flag_exits_nonzero_when_backend_unreachable`; `agent_flow.rs:79-93` asserts the hard-error path. `cargo test -p reposix-cli` = 69/69 + all integration suites, **0 failed**. |
| 3 | No documented command errors on copy-paste (outputs match) | **GREEN** | Ran the documented flow verbatim in a `/tmp` leaf. `first-run.md:70` "Next:" line now includes `(or git sparse-checkout set <pathspec> first)` — matches real output. `:153-156` push shown as fast-forward `<old>..<new> main -> main` with prose (was the false `[new branch]`). `:168-170` audit timestamps now nanosecond `+00:00` form. Item (c) undocumented "Secret scan: clean ✓" resolved: string is **absent from `crates/`**; it is a global git hook (`core.hooksPath=/home/reuben/.git-hooks`) on the test box — a real external user won't see it, so correctly left undocumented (false-fail artifact, not a doc-lie). `mkdocs-strict.sh`+`mermaid-renders.sh` exit 0. |
| 4 | B5 resolved honestly | **GREEN** | `.planning/CONSULT-DECISIONS.md:53` dated **2026-07-07 [SELF]**: verdict = KNOWN LIMITATION (RBF-LR-03 class), rationale recorded, honest troubleshooting prose landed (`:166` external-REST-write caveat + `docs/guides/troubleshooting.md` cross-link). v0.14.0 follow-up filed: `SURPRISES-INTAKE.md:905 S-260707-rbf-lr03-external-write-crash | severity: HIGH | tag: v0.14.0 RBF-LR-03 pivot`. |
| 5 | Clean tree, pushed, touched-crate tests + docs gates green | **GREEN** | `HEAD==origin/main==a374a4b`; committed tree clean (the transient `M doc-alignment.json` is the known `docs-alignment/walk` non-persisted `last_walked`/coverage-counter side-effect — filed GTH, no substantive change; `git checkout --` restores clean). `cargo test -p reposix-cli` 0 failed; `cargo test -p reposix-sim` 33+ tests 0 failed. `mkdocs-strict.sh` exit 0 ("docs site clean"); `mermaid-renders.sh` exit 0 (6/6 pages). |
| 6 | Catalog rows PASS (pre-push cadence) | **GREEN** | `python3 quality/runners/run.py --cadence pre-push` → **exit 0**, summary **55 PASS, 0 FAIL, 0 PARTIAL, 1 WAIVED, 0 NOT-VERIFIED**. Sole WAIVED = `structure/file-size-limits` (until 2026-08-08, pre-existing, acceptable). No RED row. B7: both binstall rows in `doc-alignment.json` (`docs/index/install-cargo-binstall:1934`, `polish-07-binstall:1959`) `last_verdict: BOUND` (not STALE_TEST_DRIFT), rationale documents the PR#70 PASS_SIGNALS broadening. |

**Score: 6/6 DoD items GREEN.**

### Notes on non-blocking observations
- `doc-alignment.json` still carries ~10 `STALE_TEST_DRIFT` rows on OTHER docs — pre-existing
  ~180-row backfill backlog (Haiku false-BONDS), already filed as a v0.14.0 GTH per the Wave F
  handover. Outside v0.13.1 scope; the `docs-alignment/walk` gate PASSes (ratios above floor).
- The `docs-alignment/walk` catalog self-mutation on read (no `--persist` gate) is a filed GTH;
  it dirtied `doc-alignment.json` at runtime here too. Restore is clean — not a committed carry.

---

## FOR L0 (release decision — NOT executed by this verifier)

**`pre-release-real-backend` 9th probe state.** Row
`agent-ux/milestone-close-vision-litmus-real-backend` (`quality/catalogs/agent-ux.json:1323`):
`status: NOT-VERIFIED`, `last_verified: 2026-07-06T05:03:59Z`, `last_real_grade: PASS`. This is
the env-gated SLOT state (creds + `REPOSIX_ALLOWED_ORIGINS` unset on this box). It has **not**
been run for v0.13.1's close; the last real grade (PASS) dates to the v0.13.0 close window.

Per CLAUDE.md / PROTOCOL.md OD-2, milestone-close is hard-RED if this probe cannot EXECUTE at
close. **This verifier did NOT run it** — it needs real creds, a non-default allowlist, and
owner approval, all outside this agent's authority. Technical read for L0's decision: the
v0.13.1 fix is **sim-only** (in-process front door + builtin seed) and touches **no real-backend
transport path**, so on pure change-surface grounds a fresh real-backend litmus is not
technically implicated by this hotfix. Whether the 9th-probe ritual is nonetheless mandatory at
this hotfix's tag — vs. tagging under a lighter cadence given the last real grade was PASS — is
**L0's call**. Do not read this row's NOT-VERIFIED as a v0.13.1 workmanship RED; it is the
standing env-gated slot, unchanged by this milestone.

---

## Binding-constraint compliance
- ONE cargo invocation at a time (mutex hook honored); prebuilt `target/debug` binaries reused
  by gates; `cargo test -p reposix-cli` then `-p reposix-sim` run sequentially (the single
  allowed build/test budget). No `--workspace`, no `cargo build` beyond what a gate required.
- All test mutation ran in `/tmp` leaves (gate scripts self-isolate via `mktemp -d`).
- No push performed (coordinator pushes). No real-backend mutation. No tag.

_Verifier: Claude (gsd-verifier), unbiased phase-close grade._
