# 91-DECISIONS.md — P91 gray-area decisions (coordinator authority, per OD-3/OD-4)

Decided 2026-07-04 by the P91 phase coordinator after reading `91-RESEARCH-code.md` +
`91-RESEARCH-framework.md`. Planner and executors treat these as ratified unless
execution surfaces premise-invalidating evidence (then: stop, report, do not absorb).

## D91-01 — Canonical record path shape: `issues/<id>.md`, unpadded

The canonical on-disk spelling for a record is `issues/<id>.md` with the id unpadded
(`issues/42.md`). Rationale: `reposix-cache/src/builder.rs:90,131-138` (the
stateless-connect production read path) already produces exactly this; stateless-connect
serves the cache tree verbatim; `real-git-push-e2e.sh:120` already assumes it; CLAUDE.md
documents `issues/42.md` UX. Implementation contract (QL-001 criterion 6):
- New `reposix_core::path::record_path(id) -> String` ("issues/<id>.md") + a rewritten
  shared `issue_id_from_path(&str) -> Option<u64>` that strips the `issues/` prefix,
  tolerates any zero-padding (build on existing padding-agnostic
  `validate_record_filename`, `path.rs:57-72`), returns `None` for non-`issues/*.md` paths.
- All four producer/consumer sites route through it: `builder.rs` (no-op/adopt),
  `refresh.rs` (11-pad producer change), `fast_import.rs` (emit side), `diff.rs`
  (prior-key + parse). The `main.rs:432-435` duplicate (QL-157) is deleted in favor of
  the core function.
- Grep-verifiable: after the change, no `format!("{:04}.md"` / `format!("{:011}.md"`
  record-path construction survives outside reposix-core.

## D91-02 — QL-001 proof strategy: cargo-level proof locally, e2e proof in CI; no git upgrade as a phase gate

This box has git 2.25.1 with no apt candidate ≥2.34 (research §1.5). Strategy:
1. Box-independent proof: new cargo tests drive `parse_export_stream` + `diff::plan` +
   precheck against canonical `issues/<id>.md` trees, directly falsifying BUG-1/2/3
   (QL-001 criteria 1–4, 6 at unit/integration level). Existing bare-shape fixtures
   (~10 files, research §1.2) are RE-KEYED to the canonical shape so they'd go RED if
   the bug returned.
2. Full-stack proof: the EXISTING `quality/gates/agent-ux/real-git-push-e2e.sh` (do NOT
   write a second harness) goes green in CI — ubuntu-latest has git ≥2.34 and
   `ci.yml:120-121` runs `run.py --cadence pre-pr` for real. Retire the waiver on
   `agent-ux/real-git-push-e2e` and re-add `pre-pr` to its cadences (its own owner_hint
   directs this). Locally the script's git-version gate exits 75 loud (criterion 5
   satisfied: asserts precondition, fails loud, never silent-skips).
3. OPTIONAL, non-gating, timeboxed 30min: attempt a local git ≥2.34 (prebuilt binary or
   source build into ~/.local). Nice for local full-stack runs; failure is not a phase
   blocker and must not consume executor budget beyond the timebox.
Phase-close proof obligation: `gh run watch` green INCLUDING the pre-pr quality job
actually executing real-git-push-e2e (verify in the run log, not by inference).

## D91-03 — Backend dispatch consolidation, not a fourth copy

`attach.rs` and `sync.rs` real-backend dispatch MUST delegate to a single shared
factory derived from `crates/reposix-remote/src/backend_dispatch.rs` (which already
holds sim/github/confluence/jira instantiation + OP-3 `with_audit` wiring). Default
shape: give `reposix-remote` a `[lib]` target exposing the dispatch module and add a
`reposix-cli` → `reposix-remote` path dependency; planner MAY instead choose a small
shared crate if the dependency diff proves ugly (research warning #3: note any NEW prod
dep for reposix-cli, e.g. parking_lot, in the plan's dependency diff). Hard constraint
either way: Confluence/JIRA connectors keep their `.with_audit(...)` chain
(backend_dispatch.rs:284-324 pattern) — attach/sync mutations MUST land `audit_events`
rows (OP-3); a hand-rolled arm that drops audit wiring is a RED.

## D91-04 — ForkAsNew: investigate-then-implement-or-error; never a silent no-op

First check whether the D91-01 path fix makes `OrphanPolicy::ForkAsNew`
(`reconciliation.rs:182` stub) work for free (research NOTICED #3). If a genuine
implementation is S-sized: implement (orphan mirror record → `create_record` on SoT
with new id, audit row, reconciliation report counts it). If larger: replace the
logged no-op with an explicit, teaching error ("fork-as-new is not supported yet; use
--orphans=keep or delete <path>; see docs/guides/troubleshooting.md") + file intake for
v0.14. Also fix `OrphanPolicy::Abort` never actually aborting (research NOTICED).
The "TODO P82+" comment goes away either way.

## D91-05 — Confluence comments/attachments dead surface: not a v0.13 capability

Executive decision on the OPEN HIGH intake (quality-convergence re-audit): reposix
v0.13 ships WITHOUT comment/attachment working-tree surface. P91 does not wire it and
does not delete it (deletion + CommentSupport downgrade + capability-matrix
reconciliation is M-sized and belongs with P95's doc/catalog drain — route there).
P91's only obligation: verify no user-facing doc promises comment/attachment access
(grep docs/; fix any promise found as an XS doc edit). Update the intake entry STATUS
to ROUTED-P95 with this decision recorded.

## D91-06 — Litmus verifier becomes real (D90-06)

`quality/gates/agent-ux/milestone-close-vision-litmus.sh` stops being an unconditional
exit-75 stub. New body per 91-RESEARCH-framework.md B(c) 8 steps: (1) resolve + assert
sanctioned target IN-BODY (TokenWorld / reubenjohn/reposix / JIRA KAN-or-TEST per
docs/reference/testing-targets.md) — hard FAIL, not 75, on non-sanctioned; (2)
preflight; (3) vanilla clone + `reposix attach confluence::TokenWorld` + edit + commit +
`git push reposix` flow; (4) the 5 T2 pass boxes asserted mechanically; (5) dual-table
audit assert (audit_events_cache AND audit_events); (6) hard-FAIL (OD-2) when substrate
exists but cannot execute — unreachable-with-creds is RED, not 75; (7) shell-subprocess
transcript artifact via lib/transcript.sh; (8) cleanup: kind=test labels swept, durable
fixture pages 7766017/7798785 NEVER touched. Env unset → exit 75 stays legitimate.
Push path works on git 2.25 (export capability; QL-001 repro proved real pushes run);
any step requiring helper stateless-connect fetch must degrade explicitly, not fake.

## D91-07 — Phase REOPEN gate = fresh-agent T2 run, mechanically anchored

The P91 REOPEN gate has two layers: (a) the mechanical litmus script (D91-06) exits 0
against TokenWorld; (b) a FRESH dark-factory-style subprocess agent (zero repo context
beyond the documented UX) executes the T2-attach flow against TokenWorld and its
transcript is graded for frictions (HIGH = documented happy path disagrees with binary
behavior, per T2-attach.md severity rubric). ≥1 HIGH ⇒ phase REOPENS, fix, re-run.
Friction counts per run are report data. The mirror repo
(reubenjohn/reposix-tokenworld-mirror) existence/population is verified before the run;
if it is empty/stale, populating it is in-scope litmus prep (it is also RBF-A-05's
populated-mirror analog for real).

## D91-08 — Fragile Confluence hierarchy test goes self-seeding

`contract_confluence_live_hierarchy` (reposix-confluence/tests/contract.rs:752-797)
becomes self-seeding: create parent+child pages via `create_record` with `parentId`
(lib.rs:288) labeled kind=test, assert hierarchy, delete them in teardown. The durable
fixture pages 7766017/7798785 stop being load-bearing for the test but are DOCUMENTED
in docs/reference/testing-targets.md as named, protected fixtures (fixes the doc gap:
today nothing tells a cleanup sweep to spare them).

## D91-09 — XS absorptions sanctioned in-phase

- ci.yml JIRA integration job forwards `JIRA_TEST_PROJECT` (intake entry 12; env blocks
  at ci.yml:267-290, 292-349).
- `agent_flow_real.rs` stale module doc ("helper hardcodes SimBackend") corrected in the
  same commit that deepens its tests.
- Phase-ID token scrub: `attach.rs`/`sync.rs` `P79-02 scaffold`/`P79-03`/`P82+`
  production strings replaced with teaching errors (recovery-oriented, no phase IDs).
  Note the `sync.rs:94` bare `P82+` is caught by NO current gate (framework research
  B(f)) — its removal must be verified by the phase's own catalog row asserts, and the
  banned-token story for the no-suffix `P\d+\+` shape gets an intake entry if not
  cheaply extendable.

## D91-10 — refresh.rs stale-padded-file hazard

`reposix refresh` regenerates the record dir deterministically: before writing, remove
(or rename-migrate) any `issues/*.md` whose stem parses to a record id but whose
spelling differs from canonical (`00000000042.md` → superseded by `42.md`). Never
touch non-record files. Covered by a test with a pre-seeded stale-padded file.

## D91-11 — Catalog honesty for P91 rows

All NEW rows carry `minted_at` + `coverage_kind` + `claim_vs_assertion_audit` per P90
rules; anything agent-ux with transport-shaped ids/claims gets `coverage_kind` correct
FIRST TRY (catalog load SystemExits otherwise — framework research A(a)/A(d)).
Specifically: rows verified only against sim/wiremock say so (`coverage_kind: sim-only` — amended per
PLAN-CHECK MF-1: the valid enum is {real-backend, sim-only, mechanical, manual} per `_audit_field.py:54`;
the original ratified text said `sim`, which is not a member and would SystemExit the catalog load);
`coverage_kind: real-backend` is reserved for rows whose verifier genuinely hits
TokenWorld/GitHub/JIRA (attach_real_*/sync_real_* family, litmus row). The edited
legacy `real-git-push-e2e` row keeps legacy status (no minted_at added), retires its
waiver, re-adds pre-pr, and its description says sim explicitly — the REAL-backend push
row remains P92's RBF-B-05 (do not overclaim it here).

## D91-12 — Routed OUT of P91 (do not absorb)

- Swarm write-contention workload → P95 (intake stays OPEN, home updated).
- Comments/attachments deletion → P95 (D91-05).
- Subjective-rubrics bare-slug vs full-id dispatch mismatch → P92 (already routed).
- Any L2/L3 cache-desync work → v0.14.0 (per CLAUDE.md).

## D91-13 — Bucket-aware canonical path layer (extends/supersedes D91-01)

Ratified de facto in Wave-5.5 (commits `3090499`/`d6e1411`) while fixing the litmus-REOPEN
BLOCKER (mass-delete of real Confluence pages — see SURPRISES-INTAKE 2026-07-04 21:00
"discovered-by: P91 91-05 (vision-litmus real-run) | severity: BLOCKER"); formalized here
per 91-06 Task 6 so the decision has a proper D-number, not just a buried intake STATUS line.

**What changed:** D91-01 ratified a single universal canonical shape, `issues/<id>.md`,
unpadded, for every backend. That was wrong for Confluence: `docs/reference/confluence.md`
already documented (and the repopulated TokenWorld mirror already carried) `pages/<id>.md`
as the Confluence UX, while `reposix-cache`'s builder/diff/refresh sites had been silently
forced onto `issues/` for confluence too. The mismatch meant a `pages/`-shaped working tree
(the real, documented shape) diffed against an `issues/`-shaped cache read every one of its
records as DELETED — the mass-delete BLOCKER that trashed the durable fixture pages
(`7766017`, `7798785`) and the space Home (`2818063`) during 91-05 litmus development
(all three restored; see the intake entry for the restore log).

**Resolution (inverts the originally-sketched fix):** rather than forcing Confluence onto
`issues/` to match D91-01, the path layer went **bucket-aware** and the diff planner went
**id-keyed**:

- `reposix_core::path::bucket_for_backend(backend) -> &'static str` — `confluence` maps to
  `"pages"`; every other backend (`sim`, `github`, `jira`) maps to `"issues"`. The sanctioned
  bucket set is `RECORD_BUCKETS = {"issues", "pages"}` — never hand-pick a bucket string
  outside this function.
- `reposix_core::path::record_path(bucket, id) -> String` — `"<bucket>/<id>.md"`, unpadded
  (D91-01's unpadded-id rule is UNCHANGED and extends to both buckets).
- `record_id_from_path` accepts either sanctioned bucket prefix and returns the id; unknown
  buckets return `None`.
- `diff::plan` matches prior↔tree by **record id**, never by path string — a mis-bucketed or
  padding-variant tree can no longer be misclassified as prior-Delete + tree-Create.
  Duplicate ids across paths (e.g. one record at `issues/42.md` and another at `pages/42.md`)
  refuse loud as a protocol error rather than silently picking one. This is strictly stronger
  than the originally-sketched count-based delete guard; the SG-02 bulk-delete cap
  (>5 deletes without `[allow-bulk-delete]`) remains as defense-in-depth on top of it.
- `builder.rs`, `fast_import`'s emit side, and `refresh.rs` all route through
  `bucket_for_backend` — the Confluence CACHE tree now also says `pages/` (previously the
  cache said `issues/` even for Confluence, which is what caused the mismatch in the first
  place: cache and refresh/docs disagreed with each other, not just docs vs. code).

**Relationship to D91-01:** D91-01's core claim (canonical paths are unpadded, single
source of truth in `reposix_core::path`, no ad hoc `format!` construction outside
`reposix-core`) is UNCHANGED and still binding. What D91-13 supersedes is D91-01's
*universal-`issues/`* claim — that part was premature; the real canonical shape is
bucket-aware, with `issues/` and `pages/` as the two sanctioned buckets, selected per
backend via `bucket_for_backend`, not hardcoded.

**CLAUDE.md follow-up:** the "agent UX is pure git" bullet + bootstrap example now note the
bucket split (`issues/<id>.md` vs `pages/<id>.md`) alongside the `record_path`/
`bucket_for_backend` reference (91-06).
