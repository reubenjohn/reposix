# P91 Plan Overview — `reposix attach` + `sync --reconcile` real-backend wiring + QL-001 + T2 litmus REOPEN gate

**Authored:** 2026-07-04 · **Base:** main @ 32ba856 (clean) · **Box git:** 2.25.1 (no apt ≥2.34)
**Spec authority:** `91-DECISIONS.md` (D91-01..12), ROADMAP § Phase 91 (RBF-A-01..07 + QL-001, SC-1..9),
`91-RESEARCH-code.md`, `91-RESEARCH-framework.md`, `raise-list-p90.md` §§2-3.

## Scope in one paragraph

Two code lanes + a framework/litmus lane + a docs lane. **Lane 1 (QL-001):** canonicalize the record path
shape to `issues/<id>.md` unpadded across four producer/consumer sites, fix the fast-import stream-parser
peek-LF + M/D reconciliation bugs, re-key ~10 masking fixtures RED-if-bug-returns, make the ALREADY-EXISTING
`real-git-push-e2e.sh` assertions pass + retire its waiver. **Lane 2:** expose `reposix-remote` as a lib so
`attach.rs`/`sync.rs` delegate to the mature `backend_dispatch.rs` (no fourth dispatch copy; OP-3 `with_audit`
inherited for free), scrub phase-ID tokens, resolve ForkAsNew/Abort, ship `#[ignore]` real-backend smokes.
**Framework/litmus:** populate the `dvcs-third-arm` harness (RBF-A-05) and rewrite the
`milestone-close-vision-litmus.sh` stub into a real sanctioned-target run — leaving substrate ready for the
COORDINATOR's fresh-agent T2 REOPEN gate. **Docs:** honest REQUIREMENTS/CLAUDE flips, durable-fixture doc,
intake drains.

## Wave structure (STRICTLY SERIAL — one tree-writer, one cargo machine-wide)

| Wave | Plan | Title | Cargo footprint | Model |
|------|------|-------|-----------------|-------|
| 1 | 91-01 | Catalog-first mint (2 new rows, NO implementation) | none | sonnet |
| 2 | 91-02 | Lane 1: QL-001 canonical path-shape + stream-parser fix + fixture re-key | `-p reposix-core`, `-p reposix-remote`, `-p reposix-cache`, `check -p reposix-cli` | opus |
| 3 | 91-03 | Lane 2: attach/sync real-backend dispatch + reconciliation + smokes | `-p reposix-remote` (new [lib]), `-p reposix-cli`, `-p reposix-cache` | opus |
| 4 | 91-04 | dvcs-third-arm populate (RBF-A-05) + Confluence hierarchy self-seed + testing-targets doc | `-p reposix-confluence` (compile) + `build --workspace --bins` (harness, unavoidable) | sonnet |
| 5 | 91-05 | Litmus verifier rewrite (D91-06) + substrate prep (mirror repo, transcript) | `build -p reposix-cli` (bin) | opus |
| 6 | 91-06 | Docs (REQUIREMENTS/CLAUDE/comments-attachments/intake) + phase close | none (mkdocs only) | sonnet |

**Serial rationale:** every wave after 91-01 touches the workspace via cargo. No two plans may compile
concurrently (VM has OOM-crashed twice on parallel cargo). Waves 2→3→4→5→6 run one at a time. 91-01 is
cargo-free and could physically run beside a cargo plan, but it is the catalog-first prerequisite for all
others so it goes first alone.

## Dependency graph

```
91-01 (catalog mint)
   └─> 91-02 (Lane 1 QL-001)            [litmus SC-6 + real-git-push-e2e depend on this]
          └─> 91-03 (Lane 2 attach/sync) [ForkAsNew D91-04 checks Lane 1's fix first]
                 └─> 91-04 (dvcs-third-arm + confluence)  [reconciliation cases need attach wiring]
                        └─> 91-05 (litmus + substrate)    [attach real-backend must exist]
                               └─> 91-06 (docs + close)   [pushes only at phase close]
```

91-04's Confluence-hierarchy leg is file-disjoint from its dvcs leg but shares the cargo machine → same plan,
serial internally.

## Catalog-first contract (91-01 mints, later plans cite)

**New rows minted NOT-VERIFIED in 91-01** (each with `minted_at`, `coverage_kind`, `claim_vs_assertion_audit`
per D91-11; get `coverage_kind` right FIRST TRY or catalog load SystemExits — `_audit_field.py:236-249`):

1. `agent-ux/ql-001-canonical-path-shape` — `kind: mechanical`, `transport_claim: false`, NO `coverage_kind`
   (transport_claim: false suppresses the gate; `"sim"` is not a valid enum member — {real-backend, sim-only,
   mechanical, manual} per _audit_field.py:54; amended per PLAN-CHECK MF-1/N-1). Box-independent cargo/grep
   proof per D91-02. Cited by 91-02.
2. `agent-ux/attach-sync-real-backend` — `kind: shell-subprocess`, `coverage_kind: real-backend`,
   `cadences: [pre-release-real-backend]`, transcript contract declared, NO `waiver` block. Cited by 91-03.

**Existing rows EDITED in later plans (not 91-01, because the flip is coupled to code landing):**
- `agent-ux/real-git-push-e2e` — 91-02 retires the waiver, keeps legacy status (NO `minted_at`, per D91-11 /
  PLAN-CHECK MF-2), sets `transport_claim: false`
  (it drives the LOCAL sim, not a real backend — declaring `real-backend` would be F-K4a false; A(a) nuance),
  re-adds `pre-pr` to cadences (row's own owner_hint mandates it, D-CONV-1).
- `agent-ux/dvcs-third-arm` — 91-04 rewrites `expected.asserts` to require non-vacuous case-specific counts
  (not the shape-only regex) + adds `minted_at`.
- `agent-ux/milestone-close-vision-litmus-real-backend` — 91-05 rewrites only the SCRIPT body; the row's
  `waiver: null` and `status: NOT-VERIFIED` are NOT touched (OD-2 waiver prohibition; coordinator flips it).

## SURPRISES-INTAKE drains this phase performs

| Entry (line) | Disposition | Plan |
|--------------|-------------|------|
| QL-001 BLOCKER (318/366) | RESOLVED — canonical path fix + real-git-push-e2e waiver retired | 91-02 |
| ForkAsNew stub "TODO P82+" (150) | RESOLVED — implement-if-free-else-teaching-error (D91-04) | 91-03 |
| JIRA_TEST_PROJECT ci.yml gap (210) | RESOLVED — forward in both JIRA job env blocks (D91-09) | 91-03 |
| Confluence hierarchy fragile test (200) | RESOLVED — self-seeding + protected-fixture doc (D91-08) | 91-04 |
| Comments/attachments dead surface (~400) | ROUTED-P95 — doc-grep verify no user promise (D91-05) | 91-06 |
| Swarm write-contention (~410) | ROUTED-P95 — home updated, stays OPEN→ROUTED (D91-12) | 91-06 |

Each plan flips ONLY its own intake entry's STATUS (serial file edits — no conflict). 91-06 sweeps the two
ROUTED-P95 entries.

## Execution constraints (binding on every executor)

- **Commit trailer** on every commit: `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`. NEVER `--no-verify`.
- **Per-crate cargo only.** `cargo check/test -p <crate>` — NEVER `--workspace` except where a pre-push hook or
  the dvcs harness's `build_and_resolve_bins` already owns the workspace build (91-04).
- **Pushes happen ONLY at phase close (91-06).** Transient RED between plans is allowed only if the plan
  documents it and names the same-wave plan that restores green before the close push. (None of P91's target
  rows are pre-push-tagged — see 91-RESEARCH-framework A(d) — so status flips won't break local pre-push; the
  risk surface is `pre-pr` CI, which only runs post-push.)
- **Real-backend etiquette:** sanctioned targets ONLY (Confluence TokenWorld / `reubenjohn/reposix` issues /
  JIRA `KAN`|`TEST`); `kind=test` labels + cleanup; NEVER touch Confluence pages **7766017 / 7798785**; every
  real mutation lands DUAL audit rows (`audit_events_cache` + `audit_events`, OP-3).
- **Ownership charter is binding** (verbatim in each plan): acceptance criteria are the floor; noticing is a
  deliverable; eager-fix <1h else file to SURPRISES-INTAKE/GOOD-TO-HAVES; verify against reality; polish for adoption.

## Checkpoint-back triggers (do NOT absorb — report to coordinator)

- **ForkAsNew turns M+** (Lane 1's fix does NOT make it free AND a real impl is non-trivial) → 91-03 emits the
  teaching-error path per D91-04 and files a v0.14 intake entry; does NOT build speculative machinery.
- **reposix-remote [lib] split drags a new heavy prod dep** into reposix-cli beyond the already-noted
  `parking_lot`/`rusqlite[bundled]` (Warning #3) → 91-03 checkpoints before committing the Cargo.toml diff.
- **git ≥2.34 unavailable** blocks local e2e proof → EXPECTED; 91-02 relies on the box-independent cargo/grep
  proof + CI (D91-02); real-git-push-e2e legitimately lands NOT-VERIFIED locally, PASS in CI.

## Risk register

1. **Catalog SystemExit on wrong `coverage_kind`** (91-01) — breaks EVERY cadence's runner load. Mitigation:
   D91-11 values pre-specified above; validate with `run.py` load after the mint commit.
2. **`sync.rs:94` `P82+` caught by NO structure gate** (framework B(f)) — its removal is verified only by the
   ql-001/attach row asserts + verifier subagent reading `sync.rs`. 91-03 must grep-confirm zero phase-ID
   tokens survive in `attach.rs`/`sync.rs`.
3. **Fixture re-key that is NOT RED-if-bug-returns** (91-02) — a re-key that keeps tests green even with the
   bug defeats the regression. Each re-keyed test must be shown RED against the unfixed planner (git-stash the
   fix, run, observe RED) before landing.
4. **Litmus false-real claim** — the litmus row is P0 and real-backend; 91-05 must NOT flip its status
   (coordinator runs the actual T2). Script must exit 75 on env-unset, hard-FAIL (not 75) on non-sanctioned
   target or creds-present-but-unreachable (OD-2).
5. **`perf_l1.rs` / `bulk_delete_cap.rs` unaudited fixtures** (research NOTICED #8/#9) — 91-02 must grep
   `no_op_tree_export` + open `bulk_delete_cap.rs` before declaring the re-key inventory closed.
