## v0.14.0 Wave-2 hardening (PLANNING)

> **Status:** scoped 2026-07-11 by owner-settled decision (this document encodes the
> owner's ordering verbatim — it is NOT a discovery/research pass). Phase numbering
> continues from v0.13.1's close: v0.13.0-ext closed at **P97**, v0.13.1 (onboarding
> hotfix) claimed **P98–P101** and SHIPPED 2026-07-07 (tag v0.13.1, `04640d5`). This
> milestone claims **P102–P112**. The queued-but-not-yet-replanned v0.13.2
> "Cross-link fidelity" milestone (still scoped at its original P98–P107 placeholder
> range) shifts further when it is eventually replanned — same renumber-on-insertion
> convention used for v0.13.1 and, before it, v0.12.1's P66 insertion. This scaffold
> does NOT edit v0.13.2's files.

> **D2 anchor.** `S-260707-pr-08` (`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`
> — "agent 'worktrees' are NOT isolated; a sim-seed leaf corrupted the shared repo
> (`t <t@t>` flipped `core.bare=true`)", HIGH) is the founding incident. It recurred
> during the v0.13.1 session (`S-260707-pr-08` amendment note + the separate
> `S-260707-rbf-lr03-external-write-crash` triage) — always local, origin never
> affected, but the corruption hit **4× in one session** per STATE.md `last_activity`.
> D2 exists to make this class of incident structurally impossible before any other
> autonomous fleet run is trusted again.

**Thesis.** v0.13.0/v0.13.1 shipped a working DVCS front door but left two classes of
debt: (1) the fleet's own safety substrate (worktree isolation, identity hygiene, shared
`.git/config` protection) is documented-but-not-enforced, and a real corruption already
hit 4× in one session; (2) five carried HIGH-severity items from the v0.13.0/v0.13.1
intake (GitHub real-backend 404, RBF-LR-03 reconciliation, waived tutorials, open RUSTSEC
advisories, a pagination-truncation data-loss hazard) never got a dedicated hardening
pass. v0.14.0 closes both, in that order — the fleet must be self-safe before it is
trusted to run the rest of the backlog unattended.

**Ordering is a hard constraint, not a suggestion.** Phase 102 (D2) is a **serializing
gate**: no other phase in this milestone — and no other autonomous fleet run anywhere in
the project — starts until P102 lands GREEN, pushed, and verifier-PASS. Phases 103–109
(the carried HIGHs + early cheap wins) MAY then run in parallel with each other, subject
to the unconditional CLAUDE.md constraint that **all cargo lanes still serialize
machine-wide** (one `cargo check`/`build`/`test`/`clippy` invocation at a time, project-wide,
regardless of how many phases are logically "in flight").

**Recurring success criteria for EVERY phase (P102–P112)** — non-negotiable per CLAUDE.md
Operating Principles + the autonomous-execution protocol; NOT separate REQ-IDs:

1. **Catalog-first** — phase's FIRST commit writes/updates catalog rows under
   `quality/catalogs/<file>.json` BEFORE any implementation commit (per `quality/CLAUDE.md`).
2. **CLAUDE.md updated in the same PR** (fix-it-twice meta-rule) — any phase introducing a
   new file/convention/gate/hook revises the relevant CLAUDE.md section in the same PR.
3. **Per-phase push** — `git push origin main` BEFORE verifier-subagent dispatch; the
   verifier grades RED if the phase shipped without the push landing.
4. **Phase close = unbiased verifier subagent (OP-7)** — verdict at
   `quality/reports/verdicts/p<N>/VERDICT.md`; the phase does not close on RED.
5. **Eager-resolution preference (OP-8)** — items <1h / no new dependency fixed in-phase;
   else appended to `SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md`, never silently skipped.
6. **Simulator-first (OP-1)** — sim is the default/only backend exercised except where a
   phase's explicit charter is real-backend verification (P104, P106); real-backend
   mutation there is **owner-gated**, not self-authorized.
7. **Tainted-by-default (OP-2) + audit log non-optional (OP-3)** apply unchanged to any
   phase touching cache/backend/export code paths.
8. **Verify against reality** — hooks/guards claimed as "enforced" must be proven
   triggered (a rejected commit, a blocked write) — not just code-read-asserted.
9. **Cargo serialization** — at most ONE cargo invocation machine-wide, project-wide,
   across every phase in flight; prefer `-p <crate>` over `--workspace`.

---

### Phase 102: D2 self-safe dark-factory hardening (HARD SERIALIZING GATE)

**Goal:** Make the fleet's own safety substrate mechanically enforced instead of
documented-prose. Three deliverables, all fail-closed:

(a) **Reject-`t@t`-identity hook** — a commit/push-time hook that REJECTS any commit
authored by the throwaway sim-fixture identity (`t <t@t>`, or any known test-fixture
identity) from reaching real history (main working tree / origin), closing the exact
failure mode in `S-260707-pr-08`.

(b) **Real per-leaf worktree isolation** — mechanical enforcement (not prose) that leaf
test setup (`reposix init` / sim-seed / test `git commit`/`git config`) runs in a
throwaway `/tmp` clone, never the shared repo/worktree. Explicitly do NOT build the
enforcement on `git worktree remove --force` — that command is itself a corruption
vector (per owner note) and must not become part of the guard's own recovery path.

(c) **Shared-config write guard** — a `PreToolUse` hook that BLOCKS any leaf-scoped
process from writing `core.bare` or `user.email` into the shared `.git/config`.

**Requirements:** D2-TAT-IDENTITY-HOOK-01, D2-LEAF-ISOLATION-01, D2-SHARED-CONFIG-GUARD-01
· **Depends on:** none (milestone entry point) · **Plan:** TBD (`/gsd-plan-phase 102`)

**Success criteria:**
1. **t@t rejection proven, not asserted:** a deliberate attempt to commit/push under
   identity `t <t@t>` (or another known test-fixture identity) against the shared repo is
   observed to FAIL with a clear rejection message — captured as evidence (command +
   output), not just a code read of the hook logic.
2. **Leaf isolation proven:** a deliberate leaf-shaped setup routine that "forgets" to
   `cd` into `/tmp` before running `reposix init`/seed/`git commit`/`git config` is
   observed to be BLOCKED or REDIRECTED before it can touch the shared repo/worktree — not
   merely documented in `.planning/ORCHESTRATION.md` § Leaf isolation (which remains, but
   is now backstopped mechanically). The guard's own implementation MUST NOT invoke `git
   worktree remove --force` as part of setup, cleanup, or recovery.
3. **Shared-config guard proven:** a deliberate attempt to write `core.bare=true` or
   `user.email` into the shared `.git/config` from a leaf-scoped process is BLOCKED by the
   PreToolUse hook before the write lands; the shared `.git/config` is unchanged after the
   attempt.
4. All three guards are fail-closed (default-deny on ambiguous state, not default-allow).
5. `.claude/hooks/` gains the new hook(s); `CLAUDE.md` § "Leaf isolation" +
   `.planning/ORCHESTRATION.md` § "Leaf isolation" are revised in the same PR to describe
   the now-mechanical enforcement (fix-it-twice meta-rule) — the prior prose hard-stop is
   marked superseded-by-mechanism, not deleted (historical record).
6. Catalog rows land for all three guards (dimension TBD in PLAN — likely `agent-ux` or a
   new `fleet-safety` row group) conforming to the unified row schema in
   `quality/catalogs/README.md`.
7. Phase close: `git push origin main`; verifier subagent GREEN; verdict at
   `quality/reports/verdicts/p102/VERDICT.md`. **No other phase in this milestone, and no
   other autonomous fleet run project-wide, starts until this verdict is GREEN.**

**Context anchor:** `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`
§ `S-260707-pr-08` (founding incident + sketch) and its amendment note near
`S-260707-rbf-lr03-external-write-crash`; `.planning/ORCHESTRATION.md` § "Leaf isolation";
root `CLAUDE.md` § "Leaf test setup runs in a throwaway `/tmp` clone"; `.claude/hooks/`
(existing cargo-mutex / stop-on-dirty / precompact-persist precedents to mirror).

### Phase 103: Early cheap wins — doc-alignment grade/persist split + structure waiver OP-8 split

**Goal:** Two low-risk, non-cargo, Python-only fixes slotted early because they are safe
alongside/after D2 and cheap to land. (a) `reposix-quality doc-alignment walk` currently
mutates `quality/catalogs/doc-alignment.json` in place with no `--persist` gate, dirtying
the tree on every validate-only run (bit 3+ lanes across the v0.13.1 session per
`GOOD-TO-HAVES.md` 2026-07-07 entry) — add the same GRADE/PERSIST split `run.py` already
has (D-P96-01 precedent). (b) the `structure/file-size-limits` waiver expires
**2026-08-08**; `SURPRISES-INTAKE.md` and `GOOD-TO-HAVES.md` are themselves 5–6x over the
file-size budget (per the waiver row's own violation breakdown) — split them per OP-8
before the waiver lapses.

**Requirements:** D2-DOC-ALIGNMENT-PERSIST-SPLIT-01, D2-FILE-SIZE-WAIVER-SPLIT-01 ·
**Depends on:** P102 GREEN · **Plan:** TBD (`/gsd-plan-phase 103`)

**Success criteria:**
1. `reposix-quality doc-alignment walk` (validate-only) no longer writes
   `quality/catalogs/doc-alignment.json`; a new `--persist` flag (mirroring `run.py`'s
   GRADE/PERSIST split) is required to mint. A validate-only run against a clean tree
   leaves `git status --short` empty.
2. `SURPRISES-INTAKE.md` and `GOOD-TO-HAVES.md` under
   `.planning/milestones/v0.13.0-phases/` are split into per-chapter or per-theme child
   docs (or otherwise brought under the `structure/file-size-limits` budget) before
   2026-08-08; the waiver row's live violation count drops for these two files
   specifically (full-corpus zero-violation is NOT required this phase — only these two
   named files, which are this milestone's carried debt).
3. Catalog rows updated to reflect the fixes; `quality/CLAUDE.md` documents the
   grade/persist split convention now shared by `run.py` and `doc-alignment walk`.
4. Phase close: `git push origin main`; verifier subagent GREEN; verdict at
   `quality/reports/verdicts/p103/VERDICT.md`.

**Context anchor:** `.planning/milestones/v0.13.0-phases/GOOD-TO-HAVES.md` 2026-07-07
"doc-alignment `walk` mutates the catalog with no `--persist` gate" entries (two
companion filings — dedupe when addressing); `quality/catalogs/freshness-invariants.json`
`structure/file-size-limits` row (waiver expiry + full violation breakdown);
`quality/runners/run.py` (D-P96-01 GRADE/PERSIST precedent).

### Phase 104: GitHub-v09 helper-path 404 fix (S-260707-gh404)

**Goal:** Fix the real-GitHub-via-helper 404. Root cause: `Cache::open` /
`Cache::build_from` forward the filesystem-sanitized project string (`owner-repo`, dashes)
to `backend.list_records_complete`, which needs the slashed `owner/repo` form for the REST
call — same latent bug in both `crates/reposix-remote/src/main.rs:299` and
`crates/reposix-cli/src/attach.rs`. The safe fix separates the cache-path key from the
backend-project identifier (or sanitizes only at path-resolution time) rather than a naive
arg swap, since `Cache` currently holds one `project` field serving both concerns.

**Requirements:** D2-GH404-CACHE-PROJECT-SPLIT-01, D2-GH404-REAL-VERIFY-01 · **Depends
on:** P102 GREEN · **Plan:** TBD (`/gsd-plan-phase 104`)

**Success criteria:**
1. `Cache` (or its callers) carry a cache-path key distinct from the backend-project
   identifier; `main.rs:299`'s `Cache::open` call and `attach.rs`'s equivalent both pass
   the slashed `owner/repo` form to backend REST calls and the sanitized dash form only to
   on-disk path resolution.
2. Regression test: a mock/fixture backend proving the helper path constructs
   `repos/owner/repo/issues` (not `repos/owner-repo/issues`).
3. **Real-backend verification is OWNER-GATED** — real GitHub `reubenjohn/reposix` issues
   is the sanctioned target (per root CLAUDE.md OP-6); this phase surfaces a named-target
   ask for the owner rather than self-authorizing the mutation. The `github-v09`
   `continue-on-error` CI marker (added `b067e21`) is removed once real-backend
   verification confirms the fix.
4. `S-260707-gh404` in `SURPRISES-INTAKE.md` closes with commit SHA; the paired
   `S-260707-desync` MEDIUM entry (confusing PATCH-404 instead of the D-01
   `sync --reconcile` recovery hint) is folded in if in-scope, else re-filed with an
   explicit v0.14.0-continuation note.
5. Catalog rows updated; phase close: `git push origin main`; verifier subagent GREEN;
   verdict at `quality/reports/verdicts/p104/VERDICT.md`.

**Context anchor:** `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`
§ `S-260707-gh404` (root cause + impact) and § `S-260707-desync` (paired MEDIUM);
`crates/reposix-remote/src/main.rs:141,299`; `crates/reposix-cli/src/attach.rs`;
`docs/reference/testing-targets.md` (GitHub sanctioned target + cleanup conventions).

### Phase 105: RBF-LR-03 rebase-recovery reconciliation redesign

**Goal:** Fix the documented post-conflict recovery crash when the SoT moved via an
**external REST write** (not a git-side push). `reposix sync --reconcile` currently mints
a fresh "Sync from REST snapshot" commit that is NOT a descendant of the prior tracking
tip, so the follow-on `git pull --rebase` aborts with `fatal: error while running
fast-import` — and the documented recovery (fresh `reposix init` into a new dir) loses
unpushed local commits. Also close the false-success sub-risk: `sync --reconcile` exits 0
even when it leaves the cache in this unusable non-descendant state.

**Requirements:** D2-RBF-LR03-DESCENDANT-COMMITS-01, D2-RBF-LR03-RECONCILE-HONESTY-01 ·
**Depends on:** P102 GREEN · **Plan:** TBD (`/gsd-plan-phase 105`)

**Success criteria:**
1. The cache-side "Sync from REST snapshot" commit is parented on the prior
   `refs/reposix/origin/main` tip (descendant lineage), so `git pull --rebase` sees a
   fast-forwardable/rebaseable history instead of aborting with `fatal: error while
   running fast-import`.
2. A leaf-isolated regression test (per Phase 102's isolation guard) proves: external
   `PATCH` on the SoT → local commit → `sync --reconcile` → `git pull --rebase` →
   `git push` recovers cleanly, with the external edit present and no silent overwrite of
   the external writer.
3. `sync --reconcile` no longer produces a teaching-free false-success: either it fails
   loudly when it cannot produce a rebaseable state, or the redesign in criterion 1 makes
   the non-descendant state impossible to reach.
4. `S-260707-rbf-lr03-external-write-crash` closes in `SURPRISES-INTAKE.md` with commit
   SHA; `docs/guides/troubleshooting.md` + ADR-010 §3's WAIVED-known-limitation marker is
   updated to reflect the fix (or, if only partially fixed, honestly re-scoped).
5. Catalog rows updated; phase close: `git push origin main`; verifier subagent GREEN;
   verdict at `quality/reports/verdicts/p105/VERDICT.md`.

**Context anchor:** `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`
§ `S-260707-rbf-lr03-external-write-crash` (full repro + sketched resolution + sub-risks);
`docs/guides/troubleshooting.md` § "DVCS push/pull issues"; `docs/decisions/010-*.md`
§3 (ADR-010 known-limitation marker); `docs/concepts/dvcs-topology.md`.

### Phase 106: Waived tutorials reproduce (docs-repro/tutorial-replay + examples 01/02/04/05)

**Goal:** Clear the WAIVED docs-repro rows for `docs-repro/tutorial-replay` and examples
01, 02, 04, 05 before their **HARD DEADLINE 2026-09-15**. Root causes already identified:
tutorial-replay fails on (i) cold cargo build inside a fresh `ubuntu:24.04` container
exceeding the 5-min container budget when not pre-warmed, and (ii) a push-step failure
(QL-001 path-shape bug). Examples 01/02 fail on "sim not reachable" aborts.

**Requirements:** D2-TUTORIAL-REPLAY-CONTAINER-BUDGET-01, D2-TUTORIAL-REPLAY-QL001-01,
D2-EXAMPLES-SIM-REACHABLE-01 · **Depends on:** P102 GREEN · **Plan:** TBD
(`/gsd-plan-phase 106`)

**Success criteria:**
1. `docs-repro/tutorial-replay` runs green inside a fresh `ubuntu:24.04` container within
   budget — either by pre-warming the cargo build cache before the timed window starts, or
   by raising the budget with an explicit documented rationale (owner-visible tradeoff, not
   a silent extension).
2. The QL-001 push-step path-shape bug blocking tutorial step 7 is fixed (or, if it
   duplicates work already routed to Phase 104/105, cross-referenced and closed there).
3. Examples 01 and 02's "sim not reachable" abort is fixed — both examples' `run.sh` exits
   0 and the simulator's audit log shows the expected `helper_push_*` row.
4. Examples 04 and 05 are re-verified against current binaries/docs and their WAIVED
   status is cleared (or, if a distinct root cause is found, it is fixed or explicitly
   re-scoped with a fresh waiver + rationale — never silently left WAIVED past the
   deadline).
5. `quality/catalogs/docs-reproducible.json` rows for `tutorial-replay` and
   `example-01`/`02`/`04`/`05` flip from WAIVED to PASS; the 2026-09-15 hard deadline is
   met with margin (this phase does not run at the deadline).
6. Catalog rows updated; phase close: `git push origin main`; verifier subagent GREEN;
   verdict at `quality/reports/verdicts/p106/VERDICT.md`.

**Context anchor:** `quality/catalogs/docs-reproducible.json` (WAIVED rows: `example-01`,
`example-02`, `tutorial-replay`, and `example-04`/`05` if present — read the live catalog
at plan time for the exact row set + `claim_vs_assertion_audit` text); `.planning/phases/
90-*/90-RESEARCH-*.md` (prior P90 90-05 re-check); `quality/gates/docs-repro/`.

### Phase 107: RUSTSEC memmap2 + quinn-proto advisories in `Cargo.lock`

**Goal:** Close RUSTSEC-2026-0186 (memmap2, issue #57) and RUSTSEC-2026-0185
(quinn-proto, issue #56). The original bundled dependabot PR #55 is CLOSED/superseded by
individual PRs **#64** (tower-http), **#65** (gix), **#66** (rusqlite), all OPEN and
mergeable as of the last check. `memmap2 0.9.11` and `quinn-proto 0.11.15` are already
present in `Cargo.lock`; a definitive re-confirmation requires a `cargo audit` run naming
those specific advisory IDs.

**Requirements:** D2-RUSTSEC-CARGO-AUDIT-CONFIRM-01, D2-DEPENDABOT-PR-MERGE-01 ·
**Depends on:** P102 GREEN · **Plan:** TBD (`/gsd-plan-phase 107`)

**Success criteria:**
1. `cargo audit` run (single cargo invocation, respecting the machine-wide serialization
   rule) explicitly names RUSTSEC-2026-0186 and RUSTSEC-2026-0185 as CLEARED (or, if still
   present, the phase does not close until they are resolved — this is a steward-and-merge
   phase, not a defer-again phase).
2. Dependabot PRs #64, #65, #66 are reviewed by a steward agent; **merges are
   owner-gated** (per root CLAUDE.md dependency/spend discipline) — the phase surfaces a
   named-target merge ask rather than self-merging.
3. The "Security audit" (cargo-audit) cron stays GREEN on `main` post-merge.
4. `SURPRISES-INTAKE.md`'s RUSTSEC entry closes with commit SHA / merge PR references.
5. Catalog rows updated; phase close: `git push origin main`; verifier subagent GREEN;
   verdict at `quality/reports/verdicts/p107/VERDICT.md`.

**Context anchor:** `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` RUSTSEC
entry (PR #55 supersession history); `gh pr view 64/65/66`; `.github/workflows/` security
audit cron definition.

### Phase 108: `prune_oid_map` pagination-truncation data-loss hazard

**Goal:** Fix the data-loss hazard where `meta::prune_oid_map` (commit `272882c`/
`e246e84`) can DELETE `oid_map` rows for LIVE records when a real connector's
`list_records()` silently truncates at a pagination/size cap (GitHub
`MAX_ISSUES_PER_LIST=500`, JIRA non-strict listing, Confluence equivalent) — the sim
backend never truncates, so this was structurally invisible to every sim-backed test that
gated the original fix. Per the already-drafted 4-option decision tree
(`93-RELIEF-HANDOFF.md` §4-6), this phase makes the E2 connector-contract call (Rule 4
territory — architectural) rather than re-deferring.

**Requirements:** D2-PRUNE-TRUNCATION-REPRO-01, D2-PRUNE-COMPLETENESS-CONTRACT-01 ·
**Depends on:** P102 GREEN · **Plan:** TBD (`/gsd-plan-phase 108`)

**Success criteria:**
1. A mock/capped-backend regression test proves the data-loss reproduction directly (not
   just a code-read assertion) — a truncated `list_records()` response feeding
   `prune_oid_map` must be shown deleting a live-record's `oid_map` row before the fix, and
   preserving it after.
2. One of the two architecturally-sound fixes from the decision tree lands: **(A)** add a
   completeness/`has_more` signal to `BackendConnector::list_records`'s return type (or a
   sibling method) and gate the prune on "listing is known-complete", OR **(B)** restrict
   pruning to the dedicated full-paginated reconcile path (`reposix sync --reconcile`,
   which already forces a full rebuild) and skip pruning entirely on the normal delta path.
   The choice is recorded as a dated decision (owner/coordinator ratified) with rationale —
   this is Rule 4 (architectural change) territory, not a same-phase mechanical fix without
   sign-off.
3. GitHub, JIRA, and Confluence connectors are updated consistently with the chosen fix.
4. `SURPRISES-INTAKE.md`'s `prune_oid_map` entry closes with commit SHA + decision
   reference.
5. Catalog rows updated; phase close: `git push origin main`; verifier subagent GREEN;
   verdict at `quality/reports/verdicts/p108/VERDICT.md`.

**Context anchor:** `.planning/phases/93-cache-coherence/93-RELIEF-HANDOFF.md` §4-6 (full
analysis + 4-option decision tree); `.planning/milestones/v0.13.0-phases/
SURPRISES-INTAKE.md` § `prune_oid_map` entry; `crates/reposix-*/src/github/lib.rs`,
`jira/lib.rs`, Confluence equivalent (`list_records` truncation points).

### Phase 109: Carried RBF-FW-11 + quality-convergence write-contention

**Goal:** Two carried framework-hygiene items. (a) RBF-FW-11's "grandfathered" date-cutoff
design misclassifies legitimately-null `last_verified` rows (the runner's
`catalog_dirty()` deliberately never persists `last_verified` for unchanged-status rows,
by design, to avoid git-diff timestamp churn) — fix per the sketched resolution: treat
"grandfathered" as "predates the RBF-FW-11-landing commit" (recorded SHA/OID), not "has an
explicit pre-cutoff value." (b) `reposix-swarm`'s write-contention coverage gap —
`confluence_direct.rs` never exercises `create_record`/`update_record` even though
Confluence writes have been live for over a milestone; the harness's entire purpose is
multi-agent contention testing and the risky write-contention path (version conflicts,
push-time drift) is untested.

**Requirements:** D2-RBF-FW11-GRANDFATHER-FIX-01, D2-SWARM-WRITE-CONTENTION-01 ·
**Depends on:** P102 GREEN · **Plan:** TBD (`/gsd-plan-phase 109`)

**Success criteria:**
1. RBF-FW-11's grandfather rule keys off "predates the RBF-FW-11-landing commit SHA" (or
   the docs-alignment-dimension permanent-exemption, already applied) rather than "has an
   explicit pre-cutoff `last_verified` value"; the 5 previously-misclassified rows
   (`docs-reproducible.json::benchmark-claim/{8ms-cached-read,89.1-percent-token-reduction}`,
   `freshness-invariants.json::structure/release-plz-disables-gh-releases`,
   `subjective-rubrics.json::subjective/dvcs-cold-reader`,
   `freshness-invariants.json::structure/file-size-limits`) are correctly classified under
   the new rule.
2. `pytest quality/runners/test_freshness_synth.py` passes against the fix; a direct
   `load_catalog()` sweep over all discovered catalogs confirms no new false-positives.
3. A sim-first swarm write-contention scenario lands (N agents racing `update_record` on
   shared records, asserting version-conflict handling + audit-row completeness), per OP-1
   default; a real-Confluence `--ignored` variant against TokenWorld is added per
   `docs/reference/testing-targets.md` cleanup conventions — real-backend mutation there is
   owner-gated per usual.
4. `confluence_direct.rs`'s stale "Phase 17 read-only by design" comment is corrected to
   reflect the new write-contention coverage.
5. Catalog rows updated; phase close: `git push origin main`; verifier subagent GREEN;
   verdict at `quality/reports/verdicts/p109/VERDICT.md`.

**Context anchor:** `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md`
§ RBF-FW-11 entry (full two-part breakage + sketched resolution) and § swarm
write-contention entry (`ROUTED-P95`, now landing here); `quality/runners/run.py`
(`catalog_dirty()`); `crates/reposix-swarm/` (`confluence_direct.rs`).

### Phase 110: +2 reservation Slot 1 — surprises absorption (OP-8)

**Goal:** Drain `.planning/milestones/v0.14.0-phases/SURPRISES-INTAKE.md` (seeded
in-flight by P102–P109) per OP-8. Every entry gets a terminal STATUS (RESOLVED + commit
SHA / DEFERRED + target milestone / WONTFIX + rationale). No `STATUS: TBD` at phase close.
Verifier honesty spot-check samples ≥3 P102–P109 plan/verdict pairs.

**Requirements:** D2-SURPRISES-01 · **Depends on:** P102 + P103 + P104 + P105 + P106 +
P107 + P108 + P109 ALL GREEN · **Plan:** TBD (`/gsd-plan-phase 110`)

**Success criteria:**
1. Every `SURPRISES-INTAKE.md` entry has terminal STATUS; empty intake acceptable ONLY if
   phases produced explicit `Eager-resolution` decisions (verified, not assumed).
2. Verifier honesty spot-check report at
   `quality/reports/verdicts/p110/honesty-spot-check.md`.
3. Phase close: `git push origin main`; verifier subagent GREEN; verdict at
   `quality/reports/verdicts/p110/VERDICT.md`.

**Context anchor:** CLAUDE.md OP-8; `.planning/PRACTICES.md` § OP-8 long-form;
`.planning/milestones/v0.13.2-phases/SURPRISES-INTAKE.md` (P107 precedent shape).

### Phase 111: +2 reservation Slot 2 — good-to-haves polish + milestone close (OP-9)

**Goal:** Drain `.planning/milestones/v0.14.0-phases/GOOD-TO-HAVES.md` per OP-8, then
close the milestone per OP-9: distill SURPRISES + GOOD-TO-HAVES + per-phase verdicts into
a new `.planning/RETROSPECTIVE.md` v0.14.0 section BEFORE archive. Milestone-close ritual:
CHANGELOG `[v0.14.0]` finalized; tag-script authored at
`.planning/milestones/v0.14.0-phases/tag-v0.14.0.sh` (≥6 safety guards mirroring v0.13.0/
v0.12.0 precedents); milestone-close verifier subagent dispatched and GREEN, including the
non-skippable 9th probe (`pre-release-real-backend`, per `.planning/CLAUDE.md`
"Milestone-close 9th probe" — reads NOT-VERIFIED honestly if env unset, never FAIL/skip-as-
PASS). Owner runs the tag script — orchestrator does NOT push the tag.

**Requirements:** D2-GOOD-TO-HAVES-01, D2-MILESTONE-CLOSE-01, D2-RETROSPECTIVE-DISTILL-01
· **Depends on:** P110 GREEN · **Plan:** TBD (`/gsd-plan-phase 111`)

**Success criteria:**
1. Every `GOOD-TO-HAVES.md` entry has terminal STATUS (closed with commit SHA / deferred
   with named carry-forward target).
2. `RETROSPECTIVE.md` gains a v0.14.0 section (What Was Built / What Worked / What Was
   Inefficient / Patterns Established / Key Lessons) distilled BEFORE archive — the
   ratification subagent grades RED if missing (per root CLAUDE.md OP-9).
3. CHANGELOG `[v0.14.0]` finalized; tag-script authored with ≥6 safety guards; guards
   re-run cleanly post-P111.
4. Milestone-close verifier dispatched and GREEN at
   `quality/reports/verdicts/milestone-v0.14.0/VERDICT.md`, including the non-skippable
   9th probe.
5. STOP at tag boundary: orchestrator does NOT push the tag. STATE.md cursor updated to
   "v0.14.0 ready-to-tag; owner pushes tag."
6. Phase close: `git push origin main`; milestone-close verifier GREEN; verdict at
   `quality/reports/verdicts/p111/VERDICT.md` + the milestone verdict above.

**Context anchor:** CLAUDE.md OP-9; `.planning/PRACTICES.md` § OP-9 long-form;
`.planning/milestones/v0.13.2-phases/ROADMAP.md` Phase 107 (structural precedent);
`.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` + `v0.12.0-phases/tag-v0.12.0.sh`
(tag-script precedents); `.planning/CLAUDE.md` § "Milestone-close 9th probe is
non-skippable".

### Phase 112: OD-4 launch-readiness — SCOPE-BUT-DO-NOT-START stub

**Goal:** Produce a scoped placeholder ONLY for the OD-4 launch-readiness milestone
(asciinema hero demo, honest headline numbers, install-path excellence, Show-HN
positioning kit). This phase authors a stub roadmap/scope pointer — **NOT execution
detail** — and marks it clearly DO-NOT-START until wave-2 hardening (P102–P111) closes
GREEN. No implementation, no asciinema recording, no copy drafting happens in this phase.

**Requirements:** D2-OD4-STUB-01 · **Depends on:** none for scoping (this phase can be
authored any time), but **execution against it is blocked until P111 GREEN** · **Plan:**
N/A — this phase produces a scope stub, not an execution plan.

**Success criteria:**
1. A stub file exists (suggested: `.planning/milestones/v0.15.0-launch-readiness-phases/
   ROADMAP.md` or equivalent, named per the OD-4 mandate — exact milestone version number
   TBD at scoping time, do not guess it here) containing ONLY: the four OD-4 deliverable
   names (asciinema demo, honest headline numbers, install excellence, Show-HN kit), a
   one-line description each, and a bold "DO NOT START until wave-2 hardening (v0.14.0,
   P102–P111) closes GREEN" banner.
2. No phase decomposition, no REQ-IDs, no success criteria, no execution detail — those
   are authored later by a dedicated OD-4 scoping session, not by this stub.
3. `.planning/STATE.md` records the OD-4 stub's existence and its DO-NOT-START gate in the
   cursor narrative.
4. Phase close: `git push origin main`; this phase does NOT require a verifier-subagent
   catalog-row dispatch (it produces planning prose only, no code/gates) — a lightweight
   human/owner acknowledgment that the stub correctly says "do not start" suffices.

**Context anchor:** `.planning/STATE.md` OD-4 decision reference (`89-OWNER-DECISIONS.md`
§ "DECISION OD-4"); the user's owner-settled scope for this milestone (asciinema demo,
honest headline numbers, install excellence, Show-HN kit — verbatim from the OD-4
mandate).
