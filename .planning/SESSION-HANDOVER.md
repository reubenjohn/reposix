# SESSION-HANDOVER.md — v0.14.0 TAG BLOCKED on an OWNER DECISION (B1) — 2026-07-12 (→ successor #5)

For the incoming top-level workhorse (L0). Map, not territory — detail lives in git + linked
files. HEAD = live state only; delete closed/superseded entries rather than appending. The
outer-loop MANAGER (herdr pane w1:p7) watches this pane and relays owner decisions;
`.planning/MANAGER-HANDOVER.md` is the live owner-directive channel.

## 0. 🟢 Main is GREEN. Tag is BLOCKED on ONE owner decision (B1). Do NOT fake it green.
- **PRIORITY ZERO (red main) is RESOLVED.** Old red (CI 29220925797, confluence contract exit
  101) was NOT the hypothesised orphan pages — the two **protected durable fixture pages**
  (parent `7766017`, child `7798785`) had been **trashed** by this session's churn; a trashed
  page still resolves `Ok` (status trashed, parentId null) so `contract_confluence_live_hierarchy`'s
  self-seed fallback was bypassed → hard-panic. Repaired (restore+reparent+sweep) via the new
  committed helper `scripts/confluence_tokenworld.py`. Full green confirmed: run 29222139204 =
  all 15 jobs success. Evidence: `.../evidence/PRIORITY-ZERO-red-ci-sweep-2026-07-12.md`.
- **THE TAG GATE IS NOW A SINGLE OWNER DECISION on B1** (§3). Everything else is either done or
  cleanly queued (§4). Do NOT re-mint VERDICT GREEN / reach READY-TO-TAG until the owner rules.

## 1. State (verify: `git log --oneline -8`, `git rev-parse origin/main`, `gh run list --branch main -L3`)
- HEAD = origin/main = `be00016`. Clean tree. Main CI GREEN at be00016 (CI all 15 jobs + CodeQL
  both `success`, confirmed this session).
- **NO v0.14 tag** (correct — never push it; tag is the MANAGER's/owner's, only after an HONEST
  gate). VERDICT.md still honestly **RED** at `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md`.
- Session commits: `68131dc` P0 sweep · `d413432` D1 self-heal+doc-lie · `dd995bc` B1 finding+OPEN
  owner decision · `18bc083` 5 noticings · `be00016` FABLE verdict.

## 2. What happened (verified this session)
- **DECISION-1 was IMPLEMENTED and its premise FALSIFIED by reality.** The blessed bus-fetch-rebase
  self-heal (litmus-flow.sh, commit `d413432`) fires correctly (proven: marker once; push rejected
  "fetch first"; rebases the BUS remote only; bounded; fails closed) but **cannot green the litmus**.
  The `sync --reconcile` doc-lie in root CLAUDE.md was fixed in the same commit (landed).
- **FABLE consult (be00016) verdict: GENUINE PRODUCT GAP, not a harness artifact. Recommend Option B.**
  Root cause with code cites: `reposix attach` (`crates/reposix-cli/src/attach.rs:259`) does a plain
  `git remote add` and NEVER seeds `refs/reposix/origin/main`; `resolve_import_parent`
  (`crates/reposix-remote/src/main.rs:400-418`) chains the bus's synthetic history onto that unseeded
  ref → every documented Pattern-C round-tripper (`docs/concepts/dvcs-topology.md:133-156`) gets a
  parentless synthetic bus root (`main.rs:380`) → the documented `git pull --rebase` recovery is a
  cross-root add/add wall. **A real attach user hits the same wall the litmus does.** Option A
  (harness bus-ref checkout) is DISHONEST (greens by ceasing to test the broken path). No honest C.

## 3. THE OWNER DECISION (tag gate) — B1 vision-litmus
Recorded OPEN at `.planning/CONSULT-DECISIONS.md` (2026-07-12 `[OWNER]` + `[FABLE]` entries); full
evidence `.../evidence/B1-litmus-selfheal-INSUFFICIENT-FINDINGS-2026-07-12.md`; litmus transcript
`quality/reports/transcripts/milestone-close-vision-litmus-real-backend.txt`.
- **The bind:** the manager set B1 = **non-waivable P0, NO caveat escape** AND **no product/defect
  fix mid-tag-sequence** — and the one sanctioned mid-sequence change (the self-heal) is proven
  insufficient. Unsatisfiable together → only the OWNER can break it.
- **Owner must choose:** (A) authorize a product+harness fix now (seed `refs/reposix/origin/main`
  on attach + parseable durable fixture) — honors non-waivable, costs v0.15.0-class work mid-tag;
  **(B, fable+coord recommend)** relax B1's non-waivable constraint → ship v0.14.0 GREEN-with-B1-
  documented-caveat, product fix (attach ref-seed + ADF round-trip) routed to v0.15.0 (precedent:
  v0.13.0 GREEN-with-caveats). **Surface this to the manager/owner FIRST; it gates §4.**

## 4. Remaining work (queued; D2/B3 are INDEPENDENT of B1 — safe to do while awaiting the decision)
- **D2 — honest p93 rewrite (manager DECISION-2b; NO product-code).** Rewrite
  `crates/reposix-cli/tests/agent_flow_real.rs::partial_failure_recovery_real_confluence`
  (CREATE L633-661, recovery loop L663-697) to test **UPDATE-recovery**, mirroring the GREEN sim twin
  `crates/reposix-remote/tests/partial_failure_recovery.rs::partial_fail_then_next_push_replans_only_remainder_and_converges`
  (L179+; pre-seed at v1 → converges). **Add teardown** (delete created pages; reuse
  `scripts/confluence_tokenworld.py`). **Correct the lying catalog assert** `quality/catalogs/agent-ux.json:2001`
  (row 1986-2024). Gate: `quality/gates/agent-ux/p93-partial-failure-recovery-real-confluence.sh`.
  Diagnosis: `.planning/quick/260712-phc-.../B2-p93-DIAGNOSIS.md`. CREATE-recovery gap → documented
  caveat in VERDICT, routed v0.15.0. Verify against the REAL backend; teardown to known-good.
- **B3 — attach-sync re-run + re-persist.** `bash quality/gates/agent-ux/attach-sync-real-backend.sh`
  → `quality/reports/verifications/agent-ux/attach-sync-real-backend.json`. Prior: exit 1, empty
  asserts (VERDICT L114-119). **CHECK FIRST:** B3's failure may be the SAME unseeded-`refs/reposix/origin/main`
  product gap fable found → if so it is ALSO caveat-able (v0.15.0), not an independent defect.
- **Doc-lie fix (HIGH, filed noticing #1) — REQUIRED before tag.** `docs/guides/troubleshooting.md:259-272`
  + `docs/concepts/dvcs-topology.md:93` over-claim recovery for the attach topology. Correct them
  honestly (attach round-trip recovery is a known v0.15.0 gap). Bundle with the B1 caveat write-up.
- **§4 mechanicals — GATED on the §3 owner decision.** IF owner approves (B): the honest
  `pre-release-real-backend` probe WILL exit non-zero on the P0 litmus row — **never soften it**;
  instead re-mint VERDICT as **GREEN-with-owner-accepted-B1-caveat** (+ B2 CREATE-recovery, + B3 if
  same gap), each routed v0.15.0 → FRESH unbiased ratification subagent (template
  `quality/PROTOCOL.md` § Verifier subagent prompt / `quality/dispatch/milestone-close-verdict.md`)
  → author `.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh` (pattern
  `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh`) → STOP at READY-TO-TAG. Manager/owner pushes.

## 5. Constraints (unchanged) + THE LESSON
sim-first for code; real backends only via `REPOSIX_ALLOWED_ORIGINS`; sanctioned mutation targets
ONLY (TokenWorld / reubenjohn-reposix issues / JIRA TEST/KAN); NO tag push ever; never open work
over a red main; ONE cargo invocation machine-wide; leaf test setup in /tmp clones (cd in the SAME
bash call). Relief ~100k soft / ~150k hard absolute → replace THIS file, commit+push, end turn.
Resume an agent via SendMessage, never fork. **THE LESSON (caused the P0):** TokenWorld known-good =
EXACTLY 2 durable pages — parent `7766017` + child `7798785` (child.parentId=7766017). EVERY
real-backend run MUST teardown to this state; verify `python3 scripts/confluence_tokenworld.py list`
(helper refuses to delete the 2 protected ids). A run that leaks or trashes fixture pages reds CI.

## 6. Serialization + budget
Single tree-writer at a time (owner-ratified session serialization). Heaviest cost = subagent-RESULT
weight (real-backend + cargo transcripts). Delegate every heavy run; demand compact committed-artifact
reports (SHAs + paths + key numbers only); NEVER pull CI `--log-failed` or a transcript into your own
context. Predecessor #4 rotated at a clean wave boundary (~105k) with B1 isolated as the sole
owner-gated blocker — D2/B3/§4 deliberately handed off (they don't accelerate a tag blocked on §3).

---
History lives in git — `git log` / `git show`, not restated here.
