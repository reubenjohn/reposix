# CONSULT-DECISIONS — Portion-1 L1 coordinator ledger

Decision ledger for the v0.13.0 close-out drive (P92→P97), no-fable regime.
`[SELF]` = decided under the escalation-valve bar (below the E1–E4 threshold),
recorded not escalated. `[CONSULT]` = fable-consult was invoked (E-tier).

Format: `## <ID> — <one-line> [SELF|CONSULT] <date>` then rationale + evidence.

---

## D-P92-01 — Do NOT split P92 into P92a/P92b [SELF] 2026-07-04

**Situation (DP-4-adjacent sizing call):** Charter pre-authorized a P92a/P92b split
IF day-1 recon sized RBF-B-01 (rebase-ancestry, debugger-flagged) at >16h.

**Decision:** No split. Run P92 as a single phase.

**Evidence (recon agent a96e2c74, 2026-07-04):** The heavy mechanism fixes already
landed on `main` BEFORE P92 started —
- `cb630e5` scrubs `GIT_DIR`/`GIT_WORK_TREE`/`GIT_INDEX_FILE`/`GIT_COMMON_DIR`/
  `GIT_OBJECT_DIRECTORY`/`GIT_NAMESPACE` before the bare-cache `git config` shell-outs
  (`crates/reposix-cache/src/cache.rs:649-673`) — this was the root cause of the
  cache-open failure → fresh-root / no-audit path.
- `a0c84a3` chains `.with_audit()` on the Confluence + JIRA connectors
  (`crates/reposix-cache/src/backend_dispatch.rs:303,322`).

RBF-B-01 residual = author a T4 two-writer/pull-rebase ancestry regression test
(prove-before-fix: the test IS the deliverable; debugger only if RED against current
main) + TokenWorld smoke. Sized S–M (~4-10h). OP-3 residual = upgrade
`bus_write_audit_completeness.rs` to query BOTH SQLite tables directly + behavioral
no-retry verifier replacing the source-grep. Sized S (~2-6h). Combined well under
the 16h split trigger.

**Debugger risk:** LOW — root cause already diagnosed/fixed; escalate only if the new
ancestry test grades RED against current main.

---

## D-P92-02 — T4 prove-before-fix: HIGH-1 GREEN, regression locked; two NEW findings not fixed [SELF] 2026-07-05

**Situation:** D-P92-01's debugger-escalation condition ("escalate only if the new
ancestry test grades RED against current main"). Executed the prove-before-fix step.

**Decision:** HIGH-1 (fresh root commit / no ancestry across a helper refetch) is
GREEN against current `main` — no escalation triggered. Locked with a regression test
(`quality/gates/agent-ux/t4-conflict-rebase-ancestry.sh`, catalog row
`agent-ux/t4-conflict-rebase-ancestry`). Two SEPARATE, previously-undocumented findings
surfaced during the repro; both filed (not fixed — different root causes than
`cb630e5`, Rule 4 territory) rather than hand-waved.

**Evidence:** This dev box's system git (2.25.1) is below the project's `>= 2.34`
floor, so the scenario cannot run natively here (matches `agent-ux/real-git-push-e2e`'s
own NOT-VERIFIED precedent on this exact box). Reproduced instead via a throwaway
`docker run ubuntu:24.04` + `git-core` PPA to match CI's runner git (`gh run view
28726703296` → `actions/checkout@v7` reports `git version 2.54.0`), `--network host`
to reach a `reposix-sim` built once on the host and bind-mounted read-only. Full
transcript citations: `.planning/phases/92-push-flow-correctness/92-T4-REPRO-NOTES.md`.

Two independent working trees (A, B), each with its OWN `REPOSIX_CACHE_DIR` (the
realistic two-agent topology — matches the original May-02 T4 test's structure; a
shared-cache single-machine variant was tried first and found to NOT trigger conflict
detection at all, a third, separately-noted finding, not escalated further since it's
an artificial topology this project doesn't actually recommend). A pushes (succeeds);
B edits the same record with a stale base and pushes (correctly REJECTED: `version
mismatch: current=2 requested=1`, `[remote rejected] ... (some-actions-failed)`); B
recovers via `git fetch origin`; `git rev-list --max-parents=0
refs/reposix/origin/main` is IDENTICAL before and after the refetch, across 3
consecutive runs. **Proven to bite:** temporarily reverted `git_config_cmd`'s env
scrub in `crates/reposix-cache/src/cache.rs` (reintroducing the pre-`cb630e5` bug),
rebuilt, re-ran the container repro — the row's own gate failed RED exactly as
predicted (`git config --add transfer.hideRefs failed: fatal: not in a git
directory` → `reposix init`'s fetch fails → `git checkout -B main
refs/reposix/origin/main` fails with `'refs/reposix/origin/main' is not a commit`).
Reverted the temporary change back (`git diff` confirmed byte-identical to the
committed state), rebuilt, reconfirmed GREEN.

**NEW finding 1 (filed, not fixed):** `git rebase`'s own 3-way merge (the literal
`git pull --rebase` command, not just `git fetch`) fails separately with `fatal: git
upload-pack: not our ref <oid>` / `could not fetch <oid> from promisor remote`. Root
cause: the cache's delta-sync ("since" cursor query) reports `0 changed (of 6)` even
2+ seconds after the conflicting write landed, so the blob the rebase's merge needs
was never lazily materialized. Blocks "step 6 completes" in SC1's literal wording even
though the HIGH-1 ancestry mechanism is fixed. Different root cause than `cb630e5`
(cache delta-sync cursor logic, not `Cache::open`'s git shell-out env). Filed to
`.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md`.

**NEW finding 2 (filed, not fixed):** stock Ubuntu 24.04 git (2.43.0, this project's
current LTS-default) fails EVERY real single-backend `git push` outright — the helper
answers a `stateless-connect git-receive-pack` probe with a custom `"unsupported
service: ..."` string instead of the `git-remote-helpers(7)`-mandated `fallback`
sentinel; per that spec, any reply other than `fallback` means "don't bother trying to
fall back," so git never attempts the `export` capability push actually needs. CI's
runner (2.54.0) and old-enough git (< the version that started probing `connect`-family
capabilities for push, matching this box's 2.25.1) don't hit it. Filed to
`.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` — real-user impact on a
currently-supported LTS git version, orthogonal to T4/HIGH-1, genuinely a different
mechanism (`stateless_connect.rs`'s reply string, not `Cache::open`'s env scrub).

**Catalog-first:** `agent-ux/t4-conflict-rebase-ancestry` (sim arm, implemented +
proven GREEN this session) + `agent-ux/t4-conflict-rebase-ancestry-real-backend`
(TokenWorld arm, scaffold only) + SC2/SC3/SC4/SC5/SC6 scaffold rows minted
NOT-VERIFIED, commit `858330a` / `600755e` (post-rebase SHA), pushed before this entry.

---

## D-P92-03 — SC1 full round-trip GREEN on sim; delta-sync downgraded to suspicion; TokenWorld arm NOT-VERIFIED by design [SELF] 2026-07-05

**Situation:** Two independent executors verified T4 litmus on the sim in a git-2.54 container (host git 2.25.1 is below the >=2.34 floor).

**Decision:** SC1 = GREEN on the sim arm (ancestry locked + full round-trip completes; overlapping-edit conflict is expected git behavior). The "not our ref"/cache-delta-sync item is DOWNGRADED from a confirmed bug to an UNREPRODUCED SUSPICION (DP-2: independent runner could not reproduce) and routed to P93 (cache-coherence) to reproduce-or-close, no blind fix. SC1 real-backend (TokenWorld) arm remains NOT-VERIFIED BY DESIGN (coverage_kind: real-backend; verified at the P97 9th probe `pre-release-real-backend`).

**Evidence (Exec1 + Exec2):**

- **Exec1** locked the ancestry regression (no fresh root after refetch) GREEN and NOTICED a "not our ref <oid>/promisor remote" cache delta-sync failure during `git pull --rebase`, routed to P93.
- **Exec2** ran the FULL `pull --rebase` round-trip twice on independent per-writer caches: non-overlapping edits complete cleanly (reject → rebase → push all exit 0, ancestry preserved); overlapping edits stop at an ordinary textual `CONFLICT (content)` from a real 3-way merge (proves the blob WAS fetched) = correct git behavior, not a bug. Exec2 did NOT reproduce Exec1's "not our ref" failure.

**Rationale:** DP-2 prove-before-fix — a defect an independent runner cannot reproduce is a suspicion, not a confirmed bug; P92 must not carry a blind P93 fix. SC1's designed coverage split (sim GREEN now, real-backend at P97) matches ROADMAP SC7 (rows minted NOT-VERIFIED, coverage_kind real-backend).

**Reversibility:** Reversible — if P93 reproduces the delta-sync failure it re-escalates as a confirmed P93 finding; the suspicion note preserves Exec1's transcript path.

### UPDATE 2026-07-05 (P93 recon, prove-before-fix / DP-2) — REPRODUCED / CONFIRMED [SELF]

**Verdict: REPRODUCED — a real, deterministic cache-coherence bug, NOT environmental.**
The DP-2 downgrade above ("unreproduced suspicion, reproduce-or-close in P93") resolves in
the REPRODUCE direction — the reversal the prior entry pre-authorized. Writer B's `git pull
--rebase` after a two-writer conflict dies `fatal: git upload-pack: not our ref <oid>` /
`could not fetch <oid> from promisor remote` **whenever the conflicting write shares a
truncated wall-clock second with B's cache cursor** — reproduced 4/4 in that window (1
deterministic cursor-pin + 3 natural same-second runs, git-2.54 container) and cleanly
ABSENT in the 2-second-gap negative control (`1 changed` → ordinary `CONFLICT (content)`).
P92 Exec2's non-repro was a TIMING STRADDLE (its run crossed a second boundary, like the
`gap2s` control), not evidence of falseness. **Do NOT close D-P92-03 as false.**

**Evidence pointers:**

- Repro transcript/notes + root cause: `.planning/phases/93-cache-coherence/93-DP2-REPRO-NOTES.md`.
- FAILING RED regression (prove-before-fix, `#[ignore]`d so CI stays green):
  `crates/reposix-cache/tests/delta_sync.rs::delta_sync_tree_references_only_resolvable_oids`
  (run `cargo test -p reposix-cache --test delta_sync -- --ignored` to see it bite).
- Repro commit `9c46e49` (RED test + container litmus harness + notes + GOOD-TO-HAVES flip;
  NO production fix — coordinator-gated). Harness: `.planning/phases/93-cache-coherence/repro/run-repro.sh`.

**Root cause (a trigger + a latent amplifier):**

- **TRIGGER (sim):** `crates/reposix-sim/src/routes/issues.rs` seconds-truncates `updated_at`
  (`SecondsFormat::Secs`, L138-139) and filters `list_changed_since` with a seconds-truncated
  cursor under a STRICT `updated_at > ?` (L180-183), so a same-truncated-second write is
  invisible to `list_changed_since` even though `list_records` (unfiltered) still returns its
  new content.
- **AMPLIFIER (the actual defect, cache layer):** `Cache::sync` builds the git TREE from the
  full `list_records` set (`crates/reposix-cache/src/builder.rs:293-328`) while
  blob-materialization + `oid_map` cover only the `list_changed_since` delta → a dangling tree
  entry → `read_blob` `UnknownOid` → the helper leaves the `want` → `git upload-pack: not our
  ref`. Invariant violated: every blob OID the HEAD tree references MUST be resolvable by
  `read_blob`.

**Disposition:** fix DEFERRED to the P93 ADR (RBF-LR-01) + fix wave — no blind fix here.
Recommended fix is at the **CACHE layer** (restore the tree↔`oid_map` invariant), NOT
sim-precision-only: real Confluence/JIRA/GitHub `updated_at` are second-resolution too, so a
pure sim-timestamp tightening leaves the latent amplifier live for any backend whose clock
resolution or skew re-creates the disagreement. Scope is **FETCH/`sync`** only — the PUSH
precheck path (`read_last_fetched_at`) is NOT vulnerable (it tolerates the same condition; see
the separate LOW degradation-asymmetry finding filed in GOOD-TO-HAVES 2026-07-05). Acceptance
gate: the catalog row `agent-ux/p93-delta-sync-coherence-invariant` (this session's mint) flips
RED→GREEN (the `#[ignore]` removed) once the cache fix lands.

**Reversibility:** this is the TERMINAL adjudication of the reproduce-or-close fork —
CONFIRMED, not reversible back to "suspicion." What remains open is only the FIX (P93), not
the existence of the bug.

---

## D-P93-01 — Deleted-record ghost `oid_map` row forces false `SotPartialFail` — CONFIRMED via execution (DP-2 prove-before-fix) [SELF] 2026-07-05

**Situation:** A code-reviewer raised a HIGH by code-reading only (never executed): an
upstream-DELETED record's `oid_map` row is never pruned (`INSERT OR REPLACE`, never
`DELETE`, in both `Cache::build_from` and `Cache::sync`); `Cache::list_record_ids()`
(`SELECT DISTINCT issue_id FROM oid_map`, unfiltered) resurrects the dead id;
`precheck.rs`'s steady-state branch (reached once every `oid_map` blob is already
materialized — the NORMAL case after an agent has read its issues) trusts that stale id
set as `diff::plan`'s `prior`; `plan()` emits a phantom `PlannedAction::Delete` for the
gone id; `execute_action` -> `delete_or_close` 404s (already gone) -> `Error::NotFound`;
`write_loop`'s `failed_ids` turns that into `SotPartialFail` + a FALSE
`helper_push_partial_fail_sot` audit row, on every subsequent push, forever, even though
the agent did nothing wrong. Per DP-2, this is a HYPOTHESIS until executed — a code-read
chain is not evidence.

**Decision: CONFIRMED.** Built and ran a minimal sim-backed repro exercising the REAL
`git-remote-reposix export` path (not a unit-level shortcut — `precheck`/`diff`/
`write_loop` are all `pub(crate)`, so only the compiled helper binary can drive the full
chain from an integration test). Both load-bearing links execute-verified true:

- **LINK (a):** `Cache::list_record_ids()` DOES still return the deleted id after a real
  `Cache::sync()` delta-sync cycle post-upstream-delete (`[1, 2]` — printed at repro
  runtime, not just asserted).
- **LINK (b):** `diff::plan` DOES emit + execute a phantom Delete for the gone id — a
  real DELETE request lands at the sim's `DELETE /projects/demo/issues/2` route (already
  404, matching `SimBackend::delete_or_close`'s real double-delete contract), forcing
  `error refs/heads/main some-actions-failed` and (in the buggy build) a
  `helper_push_partial_fail_sot` audit row for a push that had zero real work left to do.

**Evidence:** Repro commit `0b20c6c` adds
`crates/reposix-remote/tests/deleted_record_ghost_oid_map_row_forces_false_partial_fail.rs`,
`#[ignore]`d (asserts the DESIRED/correct behavior so it currently FAILS RED against the
buggy code — confirmed via `cargo test -p reposix-remote --test
deleted_record_ghost_oid_map_row_forces_false_partial_fail -- --ignored --nocapture`,
panic: `a no-real-work push must succeed, not false-fail on a ghost id;
stdout=error refs/heads/main some-actions-failed`; default `cargo test` run without
`--ignored` shows `1 ignored`, so CI stays green until the fix lands).

**Fix sketch (NOT implemented this lane — repro only, per DP-2):** two candidate
strategies, either flips the repro test to GREEN unmodified (assertions deliberately do
not pin `oid_map`'s row count, only the observable contract):
1. Prune `oid_map` rows for ids absent from the full `list_records` set as part of
   `Cache::sync`'s Step 5 transaction (and `build_from`'s equivalent) — restores the
   tree↔`oid_map` coherence invariant symmetrically for deletions, not just additions.
2. Reclassify a delete-time `Error::NotFound` from `delete_or_close` as idempotent
   success in `execute_action`'s `PlannedAction::Delete` arm (the record is already in
   the desired end state) rather than a failure.

**Load-bearing / E2-boundary assessment for the coordinator:** Strategy 1 changes
`Cache::sync`/`build_from`'s write contract (adds a `DELETE FROM oid_map` to a
previously INSERT-only path) — touches the same coherence invariant ADR-010
(L2/L3 cache-coherence) already ratified for the fetch-side amplifier in this same
ledger's prior entry, so it is IN-FAMILY with that ADR's scope, not a fresh
architectural surface. Strategy 2 changes a public error-to-outcome mapping in the
write/push path (`NotFound` on delete goes from "failure" to "success") — smaller
blast radius (one match arm) but changes what `SotPartialFail` means at the write
boundary generally (any future NotFound-on-delete, not just the ghost-row case, would
also go quiet). Recommend routing the actual fix decision through the same P93
fix-wave / ADR-010 follow-up the fetch-side amplifier used, rather than a blind
inline patch — both strategies are REVERSIBLE (internal cache/write-path behavior,
no public API break), so this does not need E2 escalation, but the CHOICE between them
is a design call the coordinator should make explicitly, not default silently.

**Reversibility:** Reversible — repro-only, no production code touched this lane.

---

## D-P93-02 — Fix the ghost `oid_map` row via Strategy 1 (prune on sync), NOT Strategy 2 (reclassify delete-NotFound) [SELF] 2026-07-05

**Situation:** D-P93-01 CONFIRMED the HIGH by execution and left TWO candidate fix
strategies for the coordinator to choose between (deliberately not patched in the repro
lane):
1. **Prune `oid_map` on sync** — DELETE rows for ids absent from the full `list_records`
   set inside `Cache::sync`'s Step-5 transaction (and `build_from`'s equivalent).
2. **Reclassify delete-time `NotFound`** — treat `Error::NotFound` from `delete_or_close`
   as idempotent success in `execute_action`'s `PlannedAction::Delete` arm.

**Decision: Strategy 1 (prune).** Implemented and shipped this lane.

**Rationale (Strategy 1 over Strategy 2):**
- **Fixes the root cause, not the symptom.** The defect is a cache-coherence gap: the
  tree is rebuilt from `list_records` but `oid_map` was insert-only, so a deleted
  record's row lingered and `list_record_ids()` resurrected it. Strategy 1 restores the
  tree↔`oid_map` invariant symmetrically for deletions — the exact mirror of Lane 1's
  addition-direction fix (`299ade0`), IN-FAMILY with ratified ADR-010. Strategy 2 leaves
  the cache incoherent: the ghost row survives, `list_record_ids()` still lies, and the
  planner still emits a phantom Delete on every push — it merely swallows the resulting
  404.
- **No spurious outbound side-effect.** Under Strategy 2 a real `DELETE` still hits the
  SoT for an already-gone id on every push (wasted request, audit noise, and — against a
  real backend with soft-delete/restore semantics — a latent correctness hazard).
  Strategy 1 emits ZERO phantom Deletes because the id never re-enters `prior`.
- **Narrower semantic blast radius.** Strategy 2 broadens what `SotPartialFail` /
  delete-`NotFound` means at the write boundary GENERALLY — ANY future NotFound-on-delete
  (not just the ghost-row case) would silently reclassify to success, masking genuine
  "the record I meant to delete isn't there" bugs. Strategy 1 changes only the cache's
  own write contract.

**Evidence (executed, not asserted):**
- Fix commit `272882c` (`meta::prune_oid_map` + `Cache::sync` Step-5 txn + `build_from`
  wrapped in one txn, covering `reposix sync --reconcile`).
- `cargo test -p reposix-remote --test deleted_record_ghost_oid_map_row_forces_false_partial_fail -j 2`
  → **1 passed, 0 ignored** (the D-P93-01 repro, un-ignored, now a permanent GREEN
  regression guard).
- `cargo test -p reposix-cache --test cache_coherence -j 2` → **3 passed** (incl. new
  `ghost_oid_map_row_pruned_after_upstream_delete` DELETION-direction case).
- `cargo test -p reposix-cache --test delta_sync -j 2` → **4 passed**;
  `cargo test -p reposix-remote --test partial_failure_recovery -j 2` → **1 passed**
  (no regression). `meta` unit tests (`prune_*`) → **2 passed**.

**Reversibility:** Reversible — internal cache/write-path behavior only, no public API
break; `git revert 272882c` restores the prior insert-only path. No E2 escalation (stays
below the threshold, in-family with ADR-010).

**Deferred:** Strategy 2 filed to `.planning/GOOD-TO-HAVES.md` as considered
defense-in-depth (a second, independent layer that would also swallow a genuine
double-delete race) — a deliberate deferral, not an oversight.

---

## D-P93-03 — Owner decisions: PR #62 merged, stale release-plz branch deleted, PR #61 HELD to P97 [SELF] 2026-07-05

**Situation:** The 2026-07-05 debt-drain branch-hygiene + PR triage (see
`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` § "2026-07-05 debt-drain:
branch hygiene + PR triage") staged three owner-gated external mutations rather than
executing them directly — per CLAUDE.md's "Uncommitted = didn't happen... External
mutations need owner-named-target approval." The owner has since reviewed and approved
all three.

**Decisions (owner-approved 2026-07-05):**

1. **PR #62** (`codecov-action` 6→7, all 16 checks green, mergeable) — **MERGED**
   (squash) at commit `5118ed1`. This is the ONE commit `origin/main` carries that this
   session's `main` lacked, resolved via Task B's `git pull --rebase`.
2. **Branch `release-plz-2026-05-01T03-32-29Z`** — **DELETED** from `origin`. Its PR #32
   was already CLOSED/superseded by PR #61; the branch was a 2-month-stale orphaned
   release-plz artifact, the sole item on the debt-drain triage's "safe-to-delete" list.
3. **PR #61** (release-plz v0.13.0 → crates.io publish) — **HELD until P97 GREEN**.
   Rationale: a crates.io publish is functionally irreversible (yanking exists but does
   not un-publish; downstream consumers can already resolve a yanked version once
   published), and `STATE.md`'s `blocks_tag: true` for workstream A explicitly reserves
   the v0.13.0 tag/publish gate for P97's GREEN verdict. Merging PR #61 now would
   pre-empt that gate. Honors the existing `blocks_tag` contract; not a new decision so
   much as a confirmation that the contract still holds.

**Reversibility:** #62-merge and branch-deletion are the only two irreversible actions
here (a squash-merge and a remote branch delete), both narrowly scoped (CI-action version
bump; an already-superseded, unmerged branch) and both owner-approved before execution.
Holding #61 is the reversible, conservative choice — it can be merged the moment P97
goes GREEN.

---

## 2026-07-05 [FABLE] pagination-truncation prune-safety fork (A/B/C/D)

- **Decision:** **A** (primary) — add a completeness signal to
  `BackendConnector::list_records`'s return (e.g. `Listing { records, is_complete }`)
  and gate BOTH `prune_oid_map` call sites (`builder.rs:138-139` full-rebuild,
  `builder.rs:442` delta) on `is_complete == true` — **plus B as cheap
  defense-in-depth** (reclassify delete-time `NotFound` as idempotent success in
  `write_loop.rs`, per the already-filed GOOD-TO-HAVE), WITHOUT reverting 272882c.
- **Rationale:** A is the only fork that meets the SUCCESS bar. (1) It eliminates
  the data-loss: prune runs only on known-complete listings; on truncation we fall
  back to the pre-272882c accepted HARD-02 posture (under-populated tree, rows
  retained). (2) It preserves ADR-010's invariant-at-source: `builder.rs:120-128`
  upserts an `oid_map` row for every tree entry BEFORE the prune, so tree→oid_map
  resolvability holds whether or not the prune fires — skipping prune on incomplete
  input leaves a superset, never a dangling tree ref; nothing is deferred to
  read-path. (3) Completeness is already known-but-dropped at every truncation exit
  (`github/lib.rs:498-499` early-return at cap with pages remaining, `:502-510` raw
  valve; intake confirms JIRA/Confluence likewise), so the flag is additive plumbing,
  not new pagination logic; sim always returns `is_complete=true`, so existing sim
  gates keep exercising the prune path, and a capped-mock connector test pins the
  skip branch — not sim-blind. Reversible (drop the gate).
  **C rejected:** `--reconcile` forces `build_from`, whose `list_records`
  (`builder.rs:65`) hits the same cap — C's prune-at-reconcile still deletes live
  rows beyond the cap; it turns the documented recovery command into the data-loss
  vector. **B-alone rejected:** reverting 272882c abandons ADR-010's coherence
  enforcement and leaves ghost rows firing phantom SoT DELETEs every push; filed as
  defense-in-depth, not root fix. **D rejected:** `MAX_RAW_ITEMS_PER_LIST` can
  truncate far below 500 on PR-heavy repos (`:502-510` breaks on raw count with
  `returned` « cap), so the count-margin heuristic has structural false negatives.
- **Risks + what would change this answer:** If any connector genuinely cannot
  determine completeness (no has-more/link signal), A degrades to warn-and-skip for
  that connector — evidence says all three know their truncation point, so this is
  theoretical. If the owner vetoes the E2 return-type change, the fallback is a
  sibling method (`list_records_complete()`) — same signal, uglier surface — not C/D.
  Residual: prune-skipped-on-truncation lets genuine upstream deletes linger as
  ghost rows in over-cap projects; B (the complementary defense) converts their
  phantom DELETEs into idempotent successes, bounding the blast to audit noise.
- **Spot-checks performed:** `crates/reposix-cache/src/meta.rs:75-114`;
  `crates/reposix-cache/src/builder.rs:55-149`;
  `crates/reposix-github/src/lib.rs:118-132,488-515`;
  `docs/decisions/010-l2-l3-cache-coherence.md:195-239`;
  `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md:891-939`.

---
