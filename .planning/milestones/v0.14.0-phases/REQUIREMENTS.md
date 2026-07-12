# v0.14.0 Requirements — Wave-2 hardening

**Milestone status:** PLANNING (Phases P102–P112; scoped 2026-07-11 by owner-settled
decision — see `ROADMAP.md` header for the phase-numbering rationale and the
renumber-on-insertion convention this milestone inherits from v0.13.1/v0.12.1).

**Milestone goal:** Make the fleet's own safety substrate mechanically enforced (D2:
reject-`t@t` identity, real per-leaf worktree isolation, shared-`.git/config` write guard)
BEFORE trusting any other autonomous fleet run — then clear five carried HIGH-severity
items from the v0.13.0/v0.13.1 intake (GitHub real-backend 404, RBF-LR-03 reconciliation,
waived tutorials, open RUSTSEC advisories, a pagination-truncation data-loss hazard) plus
two carried framework-hygiene items and two early cheap wins, closing with the standard
OP-8 (+2 absorption) and OP-9 (retrospective distill) milestone-close ritual. A
scope-only stub for the OD-4 launch-readiness milestone is produced but explicitly marked
DO-NOT-START.

**The litmus test:** at v0.14.0 close, a deliberate `t@t`-identity commit attempt against
the shared repo is mechanically REJECTED (not just documented against), a deliberate
shared-`.git/config` write is mechanically BLOCKED, and all five carried HIGH intake items
carry a terminal RESOLVED/closed status with commit SHA — no re-deferral to v0.15.0
without an explicit, owner-visible rationale.

**Source-of-truth handover bundle:**
- `.planning/milestones/v0.14.0-phases/ROADMAP.md` — the full P102–P112 phase
  decomposition (this REQUIREMENTS file's companion).
- `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` — origin of every carried
  anchor: `S-260707-pr-08` (D2 founding incident), `S-260707-gh404` (GitHub 404),
  `S-260707-rbf-lr03-external-write-crash` (RBF-LR-03), the RUSTSEC entry, the
  `prune_oid_map` entry, the RBF-FW-11 entry, the swarm write-contention entry.
- `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` — origin of the doc-alignment
  persist-split entries (P103) and the `structure/file-size-limits` waiver row context.
- `quality/catalogs/docs-reproducible.json` — origin of the WAIVED tutorial/example rows
  (P106) and their `claim_vs_assertion_audit` text.
- `.planning/phases/93-cache-coherence/93-RELIEF-HANDOFF.md` §4-6 — full `prune_oid_map`
  decision tree (P108).
- `.planning/STATE.md` frontmatter + `last_activity` — live cursor confirming v0.13.1
  SHIPPED and v0.14.0 as the reserved next milestone (`04640d5`/`a20c063`
  "chore: release v0.14.0" reserving the version).
- `.planning/phases/89-framework-fixes-cadence-shell-kind/89-OWNER-DECISIONS.md` §
  "DECISION OD-4" — source of the launch-readiness scope named in Phase 112's stub.

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**
- **OP-1 Simulator-first.** All phases exercise sim-backed logic by default; P104/P106/P109
  real-backend verification steps are explicitly owner-gated, not self-authorized.
- **OP-2 Tainted-by-default / OP-3 Audit-log-non-optional.** Apply unchanged to any phase
  touching cache/backend/export code paths (P104, P105, P108, P109).
- **OP-7 Verifier subagent dispatch on every phase close.**
- **OP-8 +2 phase practice.** v0.14.0 reserves the last 2 phases (P110 surprises, P111
  good-to-haves + milestone close) per the standard split shape (mirrors v0.13.2's P107
  single-slot precedent, expanded to 2 slots here per this milestone's larger carried-debt
  volume).
- **OP-9 Milestone-close ritual: distill before archiving.** P111 distills into
  `RETROSPECTIVE.md` v0.14.0 section BEFORE archive.
- **Per-phase push cadence.** Every phase closes with `git push origin main` BEFORE
  verifier-subagent dispatch.
- **Milestone-close 9th probe (non-skippable).** P111's milestone-close verifier MUST run
  `pre-release-real-backend`; reads NOT-VERIFIED honestly if env unset, never
  FAIL/skip-as-PASS (per `.planning/CLAUDE.md`).

### Active

#### D2 self-safe dark-factory hardening (P102) — HARD SERIALIZING GATE

- [ ] **D2-TAT-IDENTITY-HOOK-01**: A commit/push-time hook REJECTS any commit authored by
  the throwaway sim-fixture identity `t <t@t>` (or another known test-fixture identity)
  from reaching real history (shared repo / origin). Proven via a deliberate rejected
  attempt, not just code-read.
- [ ] **D2-LEAF-ISOLATION-01**: Mechanical enforcement that leaf test setup (`reposix
  init`/sim-seed/test `git commit`/`git config`) runs in a throwaway `/tmp` clone, never
  the shared repo/worktree. Must NOT be built on `git worktree remove --force`. Proven via
  a deliberate leaf-shaped setup that "forgets" to `cd` into `/tmp` being blocked or
  redirected before touching the shared repo.
- [ ] **D2-SHARED-CONFIG-GUARD-01**: A `PreToolUse` hook BLOCKS any leaf-scoped process
  from writing `core.bare` or `user.email` into the shared `.git/config`. Proven via a
  deliberate blocked write attempt; shared `.git/config` unchanged after.

#### Early cheap wins — doc-alignment persist split + file-size waiver split (P103)

- [ ] **D2-DOC-ALIGNMENT-PERSIST-SPLIT-01**: `reposix-quality doc-alignment walk`
  (validate-only) no longer mutates `quality/catalogs/doc-alignment.json`; a `--persist`
  flag (mirroring `run.py`'s GRADE/PERSIST split, D-P96-01 precedent) is required to mint.
- [ ] **D2-FILE-SIZE-WAIVER-SPLIT-01**: `SURPRISES-INTAKE.md` and `GOOD-TO-HAVES.md`
  under `.planning/milestones/v0.13.0-phases/` are split (per-chapter or per-theme child
  docs) to clear the `structure/file-size-limits` waiver's per-file violation for these two
  named files before the **2026-08-08** hard deadline.

#### GitHub-v09 helper-path 404 fix (P104)

- [ ] **D2-GH404-CACHE-PROJECT-SPLIT-01**: `Cache`'s cache-path key is separated from its
  backend-project identifier; `crates/reposix-remote/src/main.rs:299`'s `Cache::open` call
  and `crates/reposix-cli/src/attach.rs`'s equivalent both pass the slashed `owner/repo`
  form to backend REST calls and the sanitized dash form only to on-disk path resolution.
  Regression test proves the constructed REST path is `repos/owner/repo/issues`.
- [ ] **D2-GH404-REAL-VERIFY-01**: Real-backend verification against `reubenjohn/reposix`
  issues is owner-gated (named-target ask, not self-authorized); the `github-v09`
  `continue-on-error` CI marker is removed once verified.

#### RBF-LR-03 rebase-recovery reconciliation redesign (P105)

- [ ] **D2-RBF-LR03-DESCENDANT-COMMITS-01**: The cache-side "Sync from REST snapshot"
  commit is parented on the prior `refs/reposix/origin/main` tip (descendant lineage);
  `git pull --rebase` no longer aborts with `fatal: error while running fast-import` after
  an external REST write. Leaf-isolated regression test proves the full recovery sequence.
- [ ] **D2-RBF-LR03-RECONCILE-HONESTY-01**: `sync --reconcile` no longer produces a
  teaching-free false-success (exit 0 while leaving the cache in an unusable
  non-descendant state) — either by making the non-descendant state unreachable, or by
  failing loudly when it cannot produce a rebaseable state.

#### Waived tutorials reproduce (P106)

- [ ] **D2-TUTORIAL-REPLAY-CONTAINER-BUDGET-01**: `docs-repro/tutorial-replay` runs green
  inside a fresh `ubuntu:24.04` container within budget (pre-warmed cargo cache or an
  explicitly documented, owner-visible budget raise).
- [ ] **D2-TUTORIAL-REPLAY-QL001-01**: The QL-001 push-step path-shape bug blocking
  tutorial step 7 is fixed (or cross-referenced and closed via P104/P105 if duplicated
  there).
- [ ] **D2-EXAMPLES-SIM-REACHABLE-01**: Examples 01 and 02's "sim not reachable" abort is
  fixed (both `run.sh` exit 0, expected `helper_push_*` audit row present); examples 04 and
  05 are re-verified and their WAIVED status cleared or explicitly re-scoped before the
  **2026-09-15** hard deadline.

#### RUSTSEC memmap2 + quinn-proto advisories (P107)

- [ ] **D2-RUSTSEC-CARGO-AUDIT-CONFIRM-01**: `cargo audit` explicitly confirms
  RUSTSEC-2026-0186 (memmap2) and RUSTSEC-2026-0185 (quinn-proto) are cleared in
  `Cargo.lock`.
- [ ] **D2-DEPENDABOT-PR-MERGE-01**: Dependabot PRs #64 (tower-http), #65 (gix), #66
  (rusqlite) are steward-reviewed; merges are owner-gated (named-target ask).

#### `prune_oid_map` pagination-truncation data-loss hazard (P108)

- [ ] **D2-PRUNE-TRUNCATION-REPRO-01**: A mock/capped-backend regression test directly
  reproduces the data-loss: a truncated `list_records()` response feeding
  `meta::prune_oid_map` deletes a live-record's `oid_map` row before the fix.
- [ ] **D2-PRUNE-COMPLETENESS-CONTRACT-01**: One of the two architecturally-sound fixes
  from the `93-RELIEF-HANDOFF.md` decision tree lands (A: completeness/`has_more` signal
  added to `BackendConnector::list_records`, gating the prune; or B: pruning restricted to
  the dedicated full-paginated `reposix sync --reconcile` path only), recorded as a dated
  ratified decision (Rule 4 territory). GitHub, JIRA, and Confluence connectors updated
  consistently.

#### Carried RBF-FW-11 + quality-convergence write-contention (P109)

- [ ] **D2-RBF-FW11-GRANDFATHER-FIX-01**: RBF-FW-11's grandfather rule keys off "predates
  the RBF-FW-11-landing commit SHA" rather than "has an explicit pre-cutoff `last_verified`
  value"; the 5 previously-misclassified rows are correctly classified;
  `pytest quality/runners/test_freshness_synth.py` passes.
- [ ] **D2-SWARM-WRITE-CONTENTION-01**: A sim-first swarm write-contention scenario (N
  agents racing `update_record` on shared records, asserting version-conflict handling +
  audit-row completeness) lands in `reposix-swarm`; a real-Confluence `--ignored` variant
  against TokenWorld is added (owner-gated real-backend mutation per usual); the stale
  "Phase 17 read-only by design" comment in `confluence_direct.rs` is corrected.

#### +2 reservation Slot 1 — surprises absorption (P110)

- [ ] **D2-SURPRISES-01**: Every entry in `.planning/milestones/v0.14.0-phases/
  SURPRISES-INTAKE.md` has terminal STATUS (RESOLVED + commit SHA / DEFERRED + target
  milestone / WONTFIX + rationale). Verifier honesty spot-check samples ≥3 P102–P109
  plan/verdict pairs.

#### +2 reservation Slot 2 — good-to-haves polish + milestone close (P111)

- [ ] **D2-GOOD-TO-HAVES-01**: Every entry in `.planning/milestones/v0.14.0-phases/
  GOOD-TO-HAVES.md` has terminal STATUS.
- [ ] **D2-RETROSPECTIVE-DISTILL-01**: `.planning/RETROSPECTIVE.md` gains a v0.14.0
  section distilled BEFORE archive (OP-9): What Was Built / What Worked / What Was
  Inefficient / Patterns Established / Key Lessons.
- [ ] **D2-MILESTONE-CLOSE-01**: CHANGELOG `[v0.14.0]` finalized; tag-script at
  `.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh` (≥6 safety guards); milestone-close
  verifier subagent dispatched and GREEN at
  `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md`, including the non-skippable 9th
  probe (`pre-release-real-backend`). Orchestrator does NOT push the tag — owner-gated.

#### OD-4 launch-readiness — SCOPE-BUT-DO-NOT-START stub (P112)

- [ ] **D2-OD4-STUB-01**: A stub scope file exists naming the four OD-4 deliverables
  (asciinema hero demo, honest headline numbers, install-path excellence, Show-HN
  positioning kit) with a bold DO-NOT-START-until-v0.14.0-GREEN banner. No phase
  decomposition, no REQ-IDs, no execution detail — those are authored by a dedicated later
  scoping session.

### Out of Scope (deferred beyond v0.14.0)

- **Full-corpus `structure/file-size-limits` zero-violation.** P103 clears only the two
  named oversize planning ledgers (`SURPRISES-INTAKE.md`, `GOOD-TO-HAVES.md`); the
  remaining ~48 files in the waiver's live violation count are NOT this milestone's scope.
- **OD-4 launch-readiness execution** (asciinema recording, headline-number copy,
  install-path polish, Show-HN kit drafting). Scoped-but-explicitly-blocked by P112's stub;
  execution is a FOLLOWING milestone, gated on v0.14.0 (P102–P111) closing GREEN.
- **v0.13.2 Cross-link fidelity** (10th quality-gate dimension). Remains queued/
  not-yet-replanned per `.planning/STATE.md` workstream B; its phase numbers shift to
  accommodate this milestone's P102–P112 claim when it is eventually replanned (same
  renumber-on-insertion convention).
- **`v1.0.0` + ADR-009 semver activation, plugin ecosystem.** Later rungs of the OD-3
  research-only ladder (`.planning/STATE.md` OD-3 mandate); out of scope until v0.14.0 and
  the launch-readiness milestone both close.

### Traceability

Drafted 2026-07-11 (owner-settled ordering encoded directly, not researched). Coverage:
**23/23 v0.14.0 REQ-IDs mapped to exactly one phase** (no orphans, no duplicates). Phases
P102–P112; v0.14.0 continues after v0.13.1's P98–P101 (v0.13.1 SHIPPED 2026-07-07).

| REQ-ID | Phase | Status |
|--------|-------|--------|
| D2-TAT-IDENTITY-HOOK-01 | P102 | planned |
| D2-LEAF-ISOLATION-01 | P102 | planned |
| D2-SHARED-CONFIG-GUARD-01 | P102 | planned |
| D2-DOC-ALIGNMENT-PERSIST-SPLIT-01 | P103 | planned |
| D2-FILE-SIZE-WAIVER-SPLIT-01 | P103 | planned |
| D2-GH404-CACHE-PROJECT-SPLIT-01 | P104 | planned |
| D2-GH404-REAL-VERIFY-01 | P104 | planned |
| D2-RBF-LR03-DESCENDANT-COMMITS-01 | P105 | planned |
| D2-RBF-LR03-RECONCILE-HONESTY-01 | P105 | planned |
| D2-TUTORIAL-REPLAY-CONTAINER-BUDGET-01 | P106 | planned |
| D2-TUTORIAL-REPLAY-QL001-01 | P106 | planned |
| D2-EXAMPLES-SIM-REACHABLE-01 | P106 | planned |
| D2-RUSTSEC-CARGO-AUDIT-CONFIRM-01 | P107 | planned |
| D2-DEPENDABOT-PR-MERGE-01 | P107 | planned |
| D2-PRUNE-TRUNCATION-REPRO-01 | P108 | planned |
| D2-PRUNE-COMPLETENESS-CONTRACT-01 | P108 | planned |
| D2-RBF-FW11-GRANDFATHER-FIX-01 | P109 | planned |
| D2-SWARM-WRITE-CONTENTION-01 | P109 | planned |
| D2-SURPRISES-01 | P110 | planned |
| D2-GOOD-TO-HAVES-01 | P111 | planned |
| D2-RETROSPECTIVE-DISTILL-01 | P111 | planned |
| D2-MILESTONE-CLOSE-01 | P111 | planned |
| D2-OD4-STUB-01 | P112 | planned |

### Recurring success criteria across every v0.14.0 phase

Part of every phase's definition-of-done, NOT separate REQ-IDs (recurring expressions of
OP-7 + the autonomous-execution protocol — see `ROADMAP.md` header for the full list):
catalog-first, CLAUDE.md updated in the same PR, per-phase push before verifier dispatch,
unbiased verifier-subagent dispatch on phase close, eager-resolution preference (OP-8),
simulator-first with owner-gated real-backend exceptions (OP-1), cargo serialization
(one invocation machine-wide).
