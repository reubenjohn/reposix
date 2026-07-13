# SESSION-HANDOVER.md — v0.14.0 TAG STILL BLOCKED on the OWNER DECISION (B1) — 2026-07-13 (→ successor #6)

For the incoming top-level workhorse (L0). Map, not territory — detail lives in git + linked
files. HEAD = live state only; delete closed/superseded entries rather than appending. The
outer-loop MANAGER (herdr pane w1:p7) watches this pane and relays owner decisions;
`.planning/MANAGER-HANDOVER.md` is the live owner-directive channel. Resume an agent via
SendMessage, never fork.

## 0. 🟢 Main is GREEN. Tag is BLOCKED on ONE owner decision (B1). Do NOT fake it green.
- Since successor #5: **D2 CLOSED GREEN** (honest p93 UPDATE-recovery rewrite, unbiased-verified)
  and **B3 CLOSED GREEN** (attach-sync re-run — clean PASS; the prior "FAIL" was a phantom
  stale-skip artifact; NOT a B1-class gap).
- **THE TAG GATE IS STILL A SINGLE OWNER DECISION on B1** (§3) — no owner ruling has arrived.
  Everything else is done or cleanly queued (§4). Do NOT re-mint VERDICT GREEN / reach
  READY-TO-TAG until the owner rules. VERDICT.md remains honestly RED at
  `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md`; there is NO v0.14 tag (correct).

## 1. State (verify: `git log --oneline -8`, `git rev-parse origin/main`, `gh run list --branch main -L3`)
- main GREEN. Newest ci.yml run success = D2 commit `1c424d7` (CI run 29261754835, success).
- D2 evidence: `quality/reports/verifications/agent-ux/p93-partial-failure-recovery-real-confluence.json`
  exit 0 (real run 2026-07-13T15:28Z); catalog row `agent-ux/p93-partial-failure-recovery-real-confluence`
  honestly rewritten to UPDATE-recovery, `status: NOT-VERIFIED` LEFT BY DESIGN (flips at the §4 cadence).
- B3 evidence: `quality/reports/verifications/agent-ux/attach-sync-real-backend.json` exit 0
  (real run 15:36Z, `attach_real_confluence` + `sync_real_confluence` both ok) — GITIGNORED,
  on-disk only, nothing to commit.
- TokenWorld known-good = EXACTLY 2 pages (`7766017` + `7798785`, child.parentId=7766017);
  verified clean after both D2 and B3 runs.
- ⚠️ **STATE.md is STALE** — its `last_activity` is from successor #3 (describes a "3-page state,
  page 2818063 restored, B1 RESOLVED") that CONTRADICTS the confirmed current reality (TokenWorld =
  2 pages, B1 OPEN). THIS handover supersedes it. #6: reconcile STATE.md, but FIRST verify the
  2-vs-3-page history (was 2818063 a different space, or a since-reverted restore?) before assuming —
  do not blind-edit.

## 2. What happened (successor #5, all verified against reality)
- **D2 — CLOSED GREEN.** Executor rewrote `partial_failure_recovery_real_confluence` to
  UPDATE-recovery (mirrors the sim twin), added a RAII `TeardownGuard`, corrected the lying catalog
  assert, and fixed a `set -e`/pipefail masking bug in `quality/gates/agent-ux/lib/transcript.sh`
  (a zero-match ASSERT-grep aborted the script, masking a real exit-0). Unbiased gsd-verifier (sonnet)
  graded GREEN on all 5 criteria: real exit 0, no catalog lie, ZERO `src` touched, TokenWorld=2 pages,
  RAII teardown runs on panic/unwind. Committed `1c424d7`, pushed, CI green.
- **B3 — CLOSED GREEN.** Fresh creds-loaded run exit 0. Executor DISPROVED the "D2 fix explains it"
  hypothesis (this gate already wraps the call in `set +e`) and found the real story: the prior "FAIL"
  (VERDICT L114-119) was never artifact-backed — the on-disk artifact at verdict-time was a stale
  2026-07-06 env-missing SKIP (the verdict itself flags this at L150-156). B3 was never broken; NOT a
  B1-class gap (attach-sync ≠ the fuller B1 push-after-attach litmus). TokenWorld clean before+after.
  No commit (artifacts gitignored). Graded via coordinator ground-truth (fresh artifact + transcript
  freshness + tree-clean), proportionate to a no-commit/no-code-change item.
- 3 B3 noticings triaged + filed (§ Filed noticings).

## 3. THE OWNER DECISION (tag gate) — B1 vision-litmus [UNCHANGED — STILL OPEN]
Recorded OPEN at `.planning/CONSULT-DECISIONS.md` (2026-07-12 `[OWNER]` + `[FABLE]` entries); full
evidence `.../evidence/B1-litmus-selfheal-INSUFFICIENT-FINDINGS-2026-07-12.md`; litmus transcript
`quality/reports/transcripts/milestone-close-vision-litmus-real-backend.txt`.
- **Root cause (fable, code-cited):** `reposix attach` (`crates/reposix-cli/src/attach.rs:259`) does a
  plain `git remote add` and NEVER seeds `refs/reposix/origin/main`; `resolve_import_parent`
  (`crates/reposix-remote/src/main.rs:400-418`) chains the bus's synthetic history onto that unseeded
  ref → a real attach round-tripper hits a cross-root add/add wall, the SAME wall the litmus does. It is
  a GENUINE PRODUCT GAP, not a harness artifact. The blessed self-heal (litmus-flow.sh, `d413432`) fires
  correctly but cannot green it. Option A (harness bus-ref checkout) is DISHONEST (greens by ceasing to
  test the broken path). No honest Option C exists.
- **The bind:** the manager set B1 = **non-waivable P0, NO caveat escape** AND **no product/defect fix
  mid-tag-sequence** — and the one sanctioned mid-sequence change (the self-heal) is proven insufficient.
  Unsatisfiable together → only the OWNER can break it.
- **Owner must choose:** (A) authorize a product+harness fix now (seed `refs/reposix/origin/main` on
  attach + parseable durable fixture) — honors non-waivable, costs v0.15.0-class work mid-tag;
  **(B, fable+coord recommend)** relax B1's non-waivable constraint → ship v0.14.0
  GREEN-with-B1-documented-caveat, product fix (attach ref-seed + ADF round-trip) routed to v0.15.0
  (precedent: v0.13.0 GREEN-with-caveats). **Surface to manager/owner FIRST; it gates §4.**

## 4. Remaining work
- **Doc-lie fix (HIGH) — REQUIRED before tag, INDEPENDENT of B1 → DO NEXT.**
  `docs/guides/troubleshooting.md:259-272` + `docs/concepts/dvcs-topology.md:93` over-claim recovery for
  the attach topology. Correct honestly (attach round-trip recovery is a known v0.15.0 gap — NB B3 proved
  basic attach+sync PASSES; the gap is the fuller push-after-attach recovery relying on the unseeded
  `refs/reposix/origin/main`, i.e. the B1 litmus scenario). Docs change → MUST pass
  `quality/gates/docs-build/mkdocs-strict.sh` + `mermaid-renders.sh` + `/doc-clarity-review`.
  Bundle-able with the B1 caveat write-up. Single tree-writer.
- **§4 tag mechanicals — GATED on the §3 owner decision.** IF owner approves (B): the honest
  `pre-release-real-backend` probe WILL exit non-zero on the P0 litmus row — **never soften it**;
  re-mint VERDICT as **GREEN-with-owner-accepted-B1-caveat** (the D2 p93 row + the B3 attach-sync row
  BOTH flip from NOT-VERIFIED to their real grade in that SAME cadence run — do NOT hand-set them)
  → FRESH unbiased ratification subagent (template `quality/PROTOCOL.md` § Verifier subagent prompt /
  `quality/dispatch/milestone-close-verdict.md`) → author
  `.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh` (pattern `.../v0.13.0-phases/tag-v0.13.0.sh`)
  → STOP at READY-TO-TAG. Manager/owner pushes the tag; the coordinator NEVER does.

## Filed noticings (OP-8, successor #5)
- SURPRISES-INTAKE part-03 (MEDIUM → v0.15.0): verdict-rigor guard — every FAIL/NOT-VERIFIED row in a
  milestone VERDICT must cite a fresh session-captured artifact, never a stale skip (the B3 phantom-fail).
- GOOD-TO-HAVES-13 (S → v0.15.0): mechanical gate artifacts carry no per-assert signal (empty asserts on
  genuine PASS) — capture the cargo `test result:` summary in the artifact JSON.
- (Handled inline, not filed) D2-verifier NOTICED items were all benign/positive (fix-twice discipline,
  correct NOT-VERIFIED handling).

## 5. Constraints (unchanged) + THE LESSON
sim-first for code; real backends only via `REPOSIX_ALLOWED_ORIGINS`; sanctioned mutation targets ONLY
(TokenWorld / reubenjohn-reposix issues / JIRA TEST/KAN); NO tag push ever; never open work over a red
main; ONE cargo invocation machine-wide; leaf test setup in /tmp clones (cd in the SAME bash call).
Relief ~100k soft / ~150k hard absolute → replace THIS file, commit+push, end turn. Resume an agent via
SendMessage, never fork. **THE LESSON (caused the P0):** TokenWorld known-good = EXACTLY 2 durable pages
— parent `7766017` + child `7798785` (child.parentId=7766017). EVERY real-backend run MUST teardown to
this state; verify `python3 scripts/confluence_tokenworld.py list` (env NOT auto-loaded in a bare shell —
`set -a; . ./.env; set +a` first, as `scripts/preflight-real-backends.sh` does; the helper refuses to
delete the 2 protected ids). A run that leaks or trashes fixture pages reds CI.

## 6. Serialization + budget
Single tree-writer at a time (owner-ratified session serialization). Heaviest cost = subagent-RESULT
weight (real-backend + cargo transcripts). Delegate every heavy run; demand compact committed-artifact
reports (SHAs + paths + key numbers only); NEVER pull CI `--log-failed` or a transcript into your own
context. Successor #5 rotated at a clean wave boundary (~140k) with D2+B3 CLOSED GREEN; the doc-lie fix
(independent) + §4 (gated on the OPEN §3 decision) deliberately handed to #6 — they don't accelerate a
tag blocked on §3.

---
History lives in git — `git log` / `git show`, not restated here.
