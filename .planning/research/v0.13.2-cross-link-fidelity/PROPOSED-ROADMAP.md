## PROPOSED — Cross-link fidelity (NOT YET FORMALIZED AS A MILESTONE)

> **Status.** PROPOSAL only — owner has not yet formalized this work as its own milestone. May ship as v0.13.2, may absorb into another milestone, may renumber. Treat the phase numbers (P97–P106) and the milestone tag (`v0.13.2`) used below as placeholders that the formalization decision will fix. The design folder ([`index.md`](./index.md)) is the source of truth; this roadmap proposes one shape for sequencing the work.
>
> **What's decided.** The 28 ADRs in [`06-decisions-log.md`](./06-decisions-log.md) and the 3 BLOCKS-PLAN ratifications in [`08-open-questions.md`](./08-open-questions.md) § "Owner ratification" (Q2 = Rust sub-command of `reposix-quality`; Q6 = cred-hygiene regex pre-commit only; Q14 = 10-phase decomposition) ARE settled. Two design-scrutiny subagent passes seeded ADRs 23–28: tag-filter (framework-orchestration decoupling), per-scope `max_l3_per_push` + derived discovery, catalog/tracker file split, L3 dispatcher reuse via `persist_artifact`, shared markdown walker, shared `heading_subtree_hash`.
>
> **What's NOT decided.** Whether this becomes its own milestone (e.g., v0.13.2), gets absorbed into a larger one, or interleaves with other in-flight work. The phase numbers below assume sequential-after-v0.13.0-absorption ordering for sizing intuition; they are not commitments. Path references like `.planning/milestones/v0.13.2-phases/...` and `.planning/phases/97-*/...` are placeholders pending formalization.

**Thesis + mental model.** See `.planning/research/v0.13.2-cross-link-fidelity/index.md` for the full statement. Two-sentence summary: every markdown link `[A](B)` is treated as a *fidelity assertion* — A's framing of B should still match what B currently teaches. A new quality-gate dimension (the 10th, joining the 9 in CLAUDE.md § "Quality Gates") grades that assertion using a four-level scrutiny ladder (L0 link resolves → L1 anchor exists → L2 hash unchanged → L3 LLM-judged subjective fidelity), with brownfield-friendly ratcheting coverage floors per scope. Catches the unknown-unknowns failure mode where progressive-disclosure parents silently lie because nobody re-graded them after the children drifted.

**Recurring success criteria for EVERY phase (P97–P106)** — non-negotiable per CLAUDE.md Operating Principles + the v0.12.0/v0.13.0 autonomous-execution protocol; NOT separate REQ-IDs:

1. **Catalog-first** — phase's FIRST commit writes catalog rows under `quality/catalogs/<file>.json` BEFORE any implementation commit. The new dimension lives at `quality/catalogs/cross-link-fidelity.json` (created in P97).
2. **CLAUDE.md updated in same PR** (QG-07) — every phase that introduces a new file/convention/gate revises the relevant CLAUDE.md section in the same PR. The "Quality Gates" 9-dimension table grows to 10 in P97; the dimension row is filled in incrementally as L0/L1/L2/L3 land.
3. **Per-phase push** (codified 2026-04-30, closes backlog 999.4) — `git push origin main` BEFORE verifier-subagent dispatch; pre-push gate-passing is part of phase-close criterion.
4. **Phase close = unbiased verifier subagent dispatch (OP-7)** — isolated subagent grades all catalog rows for the phase against artifacts under `quality/reports/verifications/`; verdict at `quality/reports/verdicts/p<N>/VERDICT.md`; phase does not close on RED.
5. **Eager-resolution preference (OP-8)** — items < 1hr / no new dependency get fixed in the discovering phase; else appended to `SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md`. The +2 reservation (P106) drains them.
6. **Simulator-first (OP-1)** — all phases exercise their gate logic against in-memory or fixture inputs; L3 dispatch tests use a mock Anthropic client by default. Real-API tests gate milestone close (P105 dogfood + P106 milestone-close), not individual phase closes.
7. **Tainted-by-default (OP-2)** — `grading_context` content shipped to Anthropic in P101 carries the `Tainted<T>` marker; the cred-hygiene regex pre-commit (Q6 ratification) is the only sanitization at v1; `${...}` reject + 2KB cap are deferred to v0.13.3 GOOD-TO-HAVES per Q6 ratification.
8. **Audit log non-optional (OP-3)** — every L3 dispatch writes a row to the cache audit table (request → vendor → cost → verdict-id) so a forensic query can trace every shipped grading_context byte.

### Phase 97: Crate skeleton + shared-compute lift + edge model + walker + catalog + tracker schemas

**Goal:** Stand up the cross-link-fidelity dimension as a sub-command of `reposix-quality` (Q2 ratified). LIFT FIRST: refactor `crates/reposix-quality/src/coverage.rs::{walk_md, eligible_files}` into a shared `crates/reposix-quality/src/md_walker.rs` (ADR-27) and add `pub fn heading_subtree_hash` next to `source_hash` + `test_body_hash` in `hash.rs` (ADR-28) — both are precursors to cross-link's edge walker and L2 hash detection. THEN land cross-link's edge data model (path-derived edge identity per ADR-19), the markdown walker that emits `(source, target, anchors)` tuples on top of the lifted module, the **catalog schema** at `quality/catalogs/cross-link-fidelity.json` (~4 runner-readable rows per ADR-25), and the **tracker schema** at `quality/state/cross-link-fidelity-tracker.json` (per-edge state per ADR-25) — all WITHOUT any L0/L1/L2/L3 verifier wired yet.

**Requirements:** XLINK-MD-WALKER-LIFT-01, XLINK-HEADING-HASH-01, XLINK-SKELETON-01, XLINK-EDGE-MODEL-01, XLINK-WALKER-01, XLINK-CATALOG-SCHEMA-01, XLINK-TRACKER-SCHEMA-01 · **Depends on:** v0.13.0 milestone GREEN (P88 + absorbed P89–P96) · **Plan:** TBD (P97 plan-overview not yet authored)

**Success criteria:**
1. **md_walker.rs lift (ADR-27):** `coverage.rs::walk_md` and `eligible_files` move to `crates/reposix-quality/src/md_walker.rs`; `coverage.rs` re-exports for backwards compatibility; doc-alignment tests still pass against the lifted module; cross-link's walker imports from `md_walker.rs`.
2. **heading_subtree_hash (ADR-28):** `hash.rs::heading_subtree_hash(file: &Path, slug: &str) -> Result<String>` lands next to `source_hash` (line 29) and `test_body_hash` (line 92); ≥3 unit tests cover (matched-heading / unknown-slug / multi-level-nesting).
3. Sub-command shape settled: `reposix-quality cross-link {walk, status, ...}` (decision documented in PLAN.md). First verb wired: `reposix-quality cross-link walk` emits a JSON array of `(source, target, anchors)` tuples.
4. Edge identity is path-derived per ADR-19; bulk-move recovery via `cross-link rebind --auto` is OUT of P97 (lands in P104 alongside `suggest-scopes`).
5. **Catalog file (ADR-25):** `quality/catalogs/cross-link-fidelity.json` lands with ~4 runner-readable rows (skeleton-builds, walker-emits-edges, tracker-schema-validates, catalog-schema-validates), each conforming to the unified row schema in `quality/catalogs/README.md`. Discovered cleanly by `quality/runners/run.py:62-69`.
6. **Tracker file (ADR-25):** `quality/state/cross-link-fidelity-tracker.json` lands; schema versioned `1.0.0` per ADR-1 strict-semver; schema validates ≥3 example tracker rows under `tests/`. Runner does NOT touch the tracker.
7. `reposix-quality cross-link walk` against the reposix repo emits between 350 and 450 edges (matches Q4 measurement: ~400 edges total addressable).
8. CLAUDE.md § "Quality Gates" grows from 9 to 10 dimensions; new row "cross-link" lands; `quality/PROTOCOL.md` updated with the dimension's runtime contract AND with the catalog/tracker file-split convention.
9. Phase close: `git push origin main`; verifier subagent grades GREEN; verdict at `quality/reports/verdicts/p97/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.2-cross-link-fidelity/02-architecture.md` § "Five primitives"; `03-schemas.md` § "Catalog schema" + § "Tracker schema (gate-internal)"; `06-decisions-log.md` ADR-1 + ADR-19 + ADR-25 (catalog/tracker split) + ADR-27 (md_walker lift) + ADR-28 (heading_subtree_hash); `examples/tracker-row.json`; existing `crates/reposix-quality/src/coverage.rs:46,73` (lift target) + `hash.rs:29,92` (extension target).

### Phase 98: Config TOML schema + scope resolution + glob matcher

**Goal:** Land the human-authored `.cross-link-fidelity` TOML config (per ADR-3 scope-only configuration, ADR-5 separate from tracker). Implement scope resolution (a glob pattern set + level + cadence + grading-defaults binds an edge to a scope) and the glob matcher with file-vs-folder priority per ADR-20. Project ships with a `default` scope at `max_level: L3` per ADR-8 (dark-factory ethos); the standalone tool's eventual default is L1, but THIS project commits to L3 from day one.

**Requirements:** XLINK-CONFIG-SCHEMA-01, XLINK-SCOPE-RESOLVE-01, XLINK-GLOB-MATCH-01 · **Depends on:** P97 GREEN · **Plan:** TBD

**Success criteria:**
1. `examples/default-config.toml` (already in research folder) loads without errors; `reposix-quality cross-link config show` round-trips it.
2. Scope resolution: each walked edge resolves to exactly one scope; ambiguous matches respect the file-vs-folder priority rule (ADR-20: file-glob beats folder-glob; nearest folder beats outer folder); resolution decisions logged for every edge in `quality/reports/cross-link/scope-resolution.jsonl`.
3. `default` scope at `max_level: L3` lands in the committed `.cross-link-fidelity`; CLAUDE.md "this project default is L3 (dark-factory)" sentence makes the project commitment legible.
4. Catalog rows: `cross-link-fidelity/config-loads`, `cross-link-fidelity/scope-resolves-deterministically`, `cross-link-fidelity/glob-priority-correct`.
5. CLAUDE.md cross-link dimension row updated with config + scopes detail.
6. Phase close: `git push origin main`; verifier GREEN; verdict at `quality/reports/verdicts/p98/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.2-cross-link-fidelity/02-architecture.md` § "Scope model"; `03-schemas.md` § "Config schema"; `06-decisions-log.md` ADR-3 + ADR-8 + ADR-20; `examples/default-config.toml`.

### Phase 99: L0 + L1 verifiers + pre-commit hook integration

**Goal:** Ship the bottom rungs of the four-level scrutiny ladder. L0 = link target file exists. L1 = `#anchor` exists in the target file. Wire both into the pre-commit hook (`.githooks/pre-commit`) so authors get fast local feedback (<2s budget per the cadence taxonomy). Pre-commit failure surfaces the offending edge IDs + suggested-fix message ("create the file or remove the link"). No L2/L3 in this phase.

**Requirements:** XLINK-L0-01, XLINK-L1-01, XLINK-PRECOMMIT-01 · **Depends on:** P98 GREEN · **Plan:** TBD

**Success criteria:**
1. L0 verifier: every walked edge with a non-existent target produces a `BROKEN` verdict; UNGRADED edges with valid targets stay UNGRADED.
2. L1 verifier: anchor existence checked against the target's heading slug map (markdown spec heading-to-slug rules); missing-anchor edges surface as `BROKEN` with anchor name.
3. Pre-commit hook integration: hook calls `reposix-quality cross-link walk --pre-commit` (only edges from changed files); <2s on the reposix repo; failure prints offending edge + recovery hint.
4. False-positive guard: code-fence-wrapped links + commented-out links + footnote references are excluded from the walker.
5. Catalog rows: `cross-link-fidelity/l0-broken-detection`, `cross-link-fidelity/l1-anchor-detection`, `cross-link-fidelity/pre-commit-budget` (cadence pre-commit, kind mechanical).
6. CLAUDE.md cross-link dimension row updated; `bash scripts/install-hooks.sh` re-run note added if needed.
7. Phase close: `git push origin main`; verifier GREEN; verdict at `quality/reports/verdicts/p99/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.2-cross-link-fidelity/02-architecture.md` § "Four-level scrutiny ladder"; `04-cli-and-workflow.md` § "Pattern: pre-commit fast feedback"; `examples/ladder-walkthrough.md`; `06-decisions-log.md` ADR-15 (CI/local cost split — pre-commit runs L0+L1 only).

### Phase 100: L2 hash-drift + edge state classifier

**Goal:** Add L2 (target's content hash unchanged since last grade) and the full edge-state classifier `UNGRADED | GRADED | STALE | BROKEN` per ADR-11. Hash drift downgrades a `GRADED` edge to `STALE`; broken target downgrades to `BROKEN`; new edge starts `UNGRADED`. Classifier is the input to L3 dispatch in P101 — only `STALE` edges trigger L3 (drift-triggered-L3 per ADR-15).

**Requirements:** XLINK-L2-HASH-01, XLINK-STATE-CLASSIFIER-01 · **Depends on:** P99 GREEN · **Plan:** TBD

**Success criteria:**
1. L2 verifier: target's content hash (sha256 of body bytes excluding frontmatter, normalized line endings) compared to tracker's `last_graded_target_hash`; mismatch ⇒ STALE.
2. Edge-state transitions tested against fixture: NEW→UNGRADED; UNGRADED→GRADED (manual L3 grade); GRADED→STALE (target hash drifts); STALE→GRADED (re-grade); GRADED→BROKEN (target deleted); BROKEN→GRADED (target restored, re-grade).
3. `reposix-quality cross-link status` summarizes scope-by-scope edge-state mix; output is the input to L3 dispatch in P101.
4. Drift-triggered-L3 contract documented: only STALE edges enter the L3 queue; GRADED/UNGRADED/BROKEN do not.
5. Catalog rows: `cross-link-fidelity/l2-hash-detection`, `cross-link-fidelity/state-classifier-correct`, `cross-link-fidelity/drift-triggered-l3-contract`.
6. CLAUDE.md cross-link dimension row updated; brownfield UNGRADED-is-legitimate clause cited per ADR-11.
7. Phase close: `git push origin main`; verifier GREEN; verdict at `quality/reports/verdicts/p100/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.2-cross-link-fidelity/02-architecture.md` § "Edge state taxonomy"; `06-decisions-log.md` ADR-11 + ADR-15; `examples/ladder-walkthrough.md` (state transition trace).

### Phase 101: L3 judge dispatcher (via persist_artifact + Path A/B) + grading_context merge

**Goal:** Top of the scrutiny ladder. L3 = LLM judge grades "does the source still adequately forecast the target?" **Dispatch reuses the existing subjective-rubric infrastructure (ADR-26)** — verdicts persist via `lib/persist_artifact.py:33-59` (`persist_artifact()`); dispatch follows the Path A/B pattern from `lib/dispatch_inline_subagent.sh:39-76`. Judge receives the source-link-context, target-content, and a three-flavor grading_context (target ⊕ edge ⊕ source) merged from frontmatter per ADR-6. **Sanitization at v1 is cred-hygiene regex pre-commit only per Q6 ratification — `${...}` reject and 2KB cap are NOT in scope (deferred to v0.13.3 GOOD-TO-HAVES).** Audit log writes one row per dispatch (request → vendor → cost → verdict-id) per OP-3.

**Requirements:** XLINK-L3-DISPATCH-REUSE-01, XLINK-PATH-AB-01, XLINK-GRADING-CONTEXT-MERGE-01, XLINK-CRED-HYGIENE-01, XLINK-AUDIT-LOG-01 · **Depends on:** P100 GREEN · **Plan:** TBD

**Success criteria:**
1. **L3 dispatcher reuses persist_artifact (ADR-26):** verdicts written via the canonical `persist_artifact()` shape `{ts, score, verdict, rationale, evidence_files, dispatched_via, asserts_passed, asserts_failed, stale}`; artifact directory matches `quality/reports/verifications/cross-link-fidelity/`.
2. **Path A/B dispatcher (ADR-26):** Path A invokes the Claude Code Task tool in-session for unbiased grading; Path B is the subprocess stub returning FAIL when no API key + no in-session orchestrator. MIGRATE-03 trap (runner sweep stomping fresh Path-A artifacts) explicitly mitigated — only `cross-link grade <id>` writes L3 verdicts; the runner reads them.
3. L3 dispatcher: takes a STALE edge → loads source + target → merges three-flavor grading_context per ADR-6 → ships to Anthropic Sonnet via Path A/B → parses verdict (PASS / FLAG / BLOCK + rationale).
4. **Cred-hygiene regex pre-commit only (Q6 ratified)**: pre-commit blocks credential-pattern matches in `grading_context` frontmatter; `${VAR}` syntax and >2KB blocks pass through (knowingly, with explicit author-discipline doc in CLAUDE.md + `docs/concepts/cross-link-fidelity.md`).
5. Audit row per dispatch written to cache audit table per OP-3 schema (op_type=`cross_link_l3_dispatch`, payload=verdict-id + cost cents); forensic query reads both audit tables.
6. Cost budget guard: aggregate L3 cost per phase tracked; phase close prints "L3 cost this phase: $X.YZ" (sanity check against ADR-9 fail-closed cap design).
7. **GOOD-TO-HAVES.md initialized at `.planning/milestones/v0.13.2-phases/GOOD-TO-HAVES.md` with TWO entries (Q6 deferrals): `XLINK-SANITIZE-DOLLAR-VAR-REJECT` (M, default-defer to v0.13.3) and `XLINK-SANITIZE-2KB-CAP` (S, defer-to-budget at P106).**
8. Catalog rows updated in `quality/catalogs/cross-link-fidelity.json` (the SAME catalog created in P97 — incremented, not duplicated): `cross-link-fidelity/l2-l3-fidelity-graded` lights up; `cross-link-fidelity/cred-hygiene-blocks`, `cross-link-fidelity/audit-log-row-per-dispatch` add per-row tests.
9. CLAUDE.md cross-link dimension row updated; "Q6 sanitization is regex-only at v1; templated-secret + log-dump leak vectors are author-discipline" warning added to `docs/concepts/cross-link-fidelity.md` author guide.
10. Phase close: `git push origin main`; verifier GREEN; verdict at `quality/reports/verdicts/p101/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.2-cross-link-fidelity/02-architecture.md` § "Four-level scrutiny ladder" L3; `03-schemas.md` § "Frontmatter schema" (grading_context); `05-edge-cases.md` § 12 (secret leakage); `06-decisions-log.md` ADR-6 + ADR-9 + ADR-15 + ADR-26 (dispatcher reuse); `08-open-questions.md` § "Owner ratification" Q6; `.claude/skills/reposix-quality-review/lib/persist_artifact.py:33-59` + `dispatch_inline_subagent.sh:39-76` (reuse targets); `examples/frontmatter.md`.

### Phase 102: `bootstrap` + `plan-refresh` + cron CI integration

**Goal:** Ship the two CI-cadence verbs. `bootstrap` runs the full L0+L1+L2+L3 sweep against every edge in scope (CI-only per ADR-15; never on contributor laptops); writes the resulting tracker as a single commit per Q3 recommendation (b)+(c) (per-push for refresh; daily batch for bootstrap). `plan-refresh` runs the drift-triggered-L3 sweep on changed edges only and updates the tracker. Cron CI workflow at `.github/workflows/cross-link-cron.yml` triggers nightly bootstrap + per-PR plan-refresh.

**Requirements:** XLINK-BOOTSTRAP-01, XLINK-PLAN-REFRESH-01, XLINK-CRON-CI-01 · **Depends on:** P101 GREEN · **Plan:** TBD

**Success criteria:**
1. `bootstrap` verb: full sweep; writes tracker in one commit at end-of-run (Q3 (c)); cron-only via `.github/workflows/cross-link-cron.yml`; abort + audit-log on partial failure.
2. `plan-refresh` verb: incremental sweep on edges where source-or-target changed since last run; updates tracker as a follow-up commit per Q3 (b); pre-push hook integration deferred to P104.
3. Cron CI workflow: nightly bootstrap (full sweep) + per-PR plan-refresh; both gated by `secrets.ANTHROPIC_API_KEY`; cost-asymmetry guard ensures contributor pre-push never triggers cron-budget L3.
4. Catalog rows: `cross-link-fidelity/bootstrap-roundtrip`, `cross-link-fidelity/plan-refresh-incremental`, `cross-link-fidelity/cron-ci-runs`.
5. CLAUDE.md updated: cron lives in CI not laptops (cost-asymmetry guard) — explicit clause; `docs/guides/cross-link-fidelity.md` walks through bootstrap UX.
6. Phase close: `git push origin main`; verifier GREEN; verdict at `quality/reports/verdicts/p102/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.2-cross-link-fidelity/04-cli-and-workflow.md` § "Pattern: CI bootstrap" + § "Pattern: pre-push refresh"; `06-decisions-log.md` ADR-15; `08-open-questions.md` Q3.

### Phase 103: `suggest-scopes` migration assistant

**Goal:** Adoption-aid verb per ADR-3 + Q5(b). `suggest-scopes` analyzes a brownfield repo's existing edge population (link-density per folder, anchor frequency, doc-type heuristics) and proposes a starter `.cross-link-fidelity` config the user reviews and commits. Heuristics identify nav-only links (READMEs that index siblings) so they get a `nav-only` scope at `max_level: L0` (cheap). User retains agency: tool proposes; user reviews/edits/commits.

**Requirements:** XLINK-SUGGEST-SCOPES-01, XLINK-NAV-ONLY-HEURISTIC-01 · **Depends on:** P102 GREEN · **Plan:** TBD

**Success criteria:**
1. `reposix-quality cross-link suggest-scopes` against a brownfield fixture (the reposix repo before any `.cross-link-fidelity` exists) emits a TOML proposal that round-trips through P98's config loader.
2. Nav-only heuristic: README files where ≥80% of outgoing links target siblings get a `nav-only` scope at `max_level: L0`.
3. User agency preserved: verb proposes a TOML to stdout (or `--write` flag to `.cross-link-fidelity.proposal`); never overwrites existing config.
4. Bulk-move recovery: `cross-link rebind --auto` (per ADR-19 — bulk file moves require this to preserve grade) lands in P103 alongside `suggest-scopes` (related migration tooling).
5. Catalog rows: `cross-link-fidelity/suggest-scopes-roundtrip`, `cross-link-fidelity/nav-only-heuristic-correct`, `cross-link-fidelity/rebind-auto-preserves-grade`.
6. CLAUDE.md updated; `docs/guides/cross-link-fidelity.md` § "Brownfield onboarding" walks through `suggest-scopes` → review → commit.
7. Phase close: `git push origin main`; verifier GREEN; verdict at `quality/reports/verdicts/p103/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.2-cross-link-fidelity/04-cli-and-workflow.md` § "suggest-scopes"; `09-brownfield-and-onboarding.md` § "Day 1: suggest-scopes"; `06-decisions-log.md` ADR-3 + ADR-19; `08-open-questions.md` Q5.

### Phase 104: Pre-push hook integration; phased enforcement modes; `max_l3_per_push` cap

**Goal:** Wire the gate into pre-push. Implement the five enforcement modes per scope (`warn` → `block-broken` → `block-stale` → `block-floor` → `block-newedge`) and the `max_l3_per_push` cap with split-PR guidance per ADR-21 (when the cap is hit, helper stderr names `git rebase --interactive` to split). Coverage floor never silently lowers per ADR-22; `cross-link reset-floor --reason` is the only way down (per Q8 (a)). Enforcement is per-scope and progressive — adopters move through the modes by editing config.

**Requirements:** XLINK-PREPUSH-01, XLINK-ENFORCEMENT-MODES-01, XLINK-MAX-L3-CAP-01, XLINK-RESET-FLOOR-01 · **Depends on:** P103 GREEN · **Plan:** TBD

**Success criteria:**
1. Pre-push hook integration: `bash scripts/install-hooks.sh` wires `cross-link prepush` into `.githooks/pre-push`; budget <60s per cadence taxonomy; failure surfaces offending edges + scope mode + recovery hint.
2. Five modes implemented: `warn` (no block, log only), `block-broken` (L0/L1 BROKEN blocks), `block-stale` (BROKEN + STALE blocks), `block-floor` (coverage drops below floor blocks), `block-newedge` (any new UNGRADED edge blocks).
3. `max_l3_per_push` cap (default 25 per ADR-21): when push would dispatch >cap STALE edges, refuse and stderr names `git rebase --interactive` for split-PR recovery.
4. Coverage floor monotonic per ADR-22: pushes that reduce floor are rejected; `cross-link reset-floor <scope> --reason "<text>"` (Q8 (a)) is the only way down; reset writes audit-log row.
5. Catalog rows: `cross-link-fidelity/prepush-budget`, `cross-link-fidelity/enforcement-modes-correct`, `cross-link-fidelity/max-l3-cap-blocks`, `cross-link-fidelity/floor-never-silently-lowers`, `cross-link-fidelity/reset-floor-audit-row`.
6. CLAUDE.md updated: pre-push wires cross-link gate; floor-never-lowers and split-PR-on-cap clauses cited; `docs/guides/cross-link-fidelity.md` covers mode progression.
7. Phase close: `git push origin main`; verifier GREEN; verdict at `quality/reports/verdicts/p104/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.2-cross-link-fidelity/02-architecture.md` § "Five enforcement modes"; `04-cli-and-workflow.md` § "Pattern: pre-push refresh"; `06-decisions-log.md` ADR-21 + ADR-22; `08-open-questions.md` Q8.

### Phase 105: Reposix dogfood — bootstrap + flip default to `block-newedge`

**Goal:** Eat our own dogfood. Run `bootstrap` (P102) against the reposix repo's full edge population (~400 edges per Q4 measurement), commit the resulting tracker, and progress the `default` scope from `warn` through `block-broken` → `block-stale` → `block-floor` → `block-newedge` over the phase. Final state: reposix's `default` scope is at `block-newedge`. This phase BLOCKS milestone close on the gate's real-world performance against this codebase. Real Anthropic API calls happen here (gated by `ANTHROPIC_API_KEY`) — first time we hit production cost.

**Requirements:** XLINK-DOGFOOD-BOOTSTRAP-01, XLINK-DOGFOOD-MODE-PROGRESSION-01 · **Depends on:** P104 GREEN · **Plan:** TBD

**Success criteria:**
1. `reposix-quality cross-link bootstrap` runs against the full reposix repo edge population; produces tracker at `quality/catalogs/cross-link-fidelity-tracker.json` (path TBD in P97 PLAN); commits with rationale message.
2. Mode progression evidence: each mode flip is its own commit with verifier-run output proving the prior mode was clean.
3. Final state: `default` scope `enforcement_mode: block-newedge`; coverage_floor floor reflects the bootstrap's L3 coverage ratio.
4. Cost report: total L3 dispatch count + cost in USD posted in the phase verdict; sanity-check against ADR-9 fail-closed cap design + ~$50/month estimate from index.md.
5. RETROSPECTIVE-FULL.md draft entry on dogfood findings (what L3 surfaced that L0/L1/L2 missed; what the cost actually was vs estimate; what edge cases bit).
6. Catalog rows: `cross-link-fidelity/dogfood-bootstrap-passes`, `cross-link-fidelity/dogfood-block-newedge-active`, `cross-link-fidelity/dogfood-cost-within-envelope`.
7. CLAUDE.md updated: cross-link dimension is now "10/10 — production at `block-newedge` for `default` scope"; `docs/concepts/cross-link-fidelity.md` finalized with real numbers.
8. Phase close: `git push origin main`; verifier GREEN; verdict at `quality/reports/verdicts/p105/VERDICT.md`.

**Context anchor:** `.planning/research/v0.13.2-cross-link-fidelity/index.md` § "Why now" (cost estimate); `09-brownfield-and-onboarding.md` § "Steady state"; `06-decisions-log.md` ADR-8 + ADR-9 + ADR-12; `08-open-questions.md` Q4.

### Phase 106: +2 reservation slot — surprises absorption + good-to-haves polish + milestone close

**Goal:** Drain `.planning/milestones/v0.13.2-phases/SURPRISES-INTAKE.md` AND `GOOD-TO-HAVES.md` per OP-8 + OP-9. P106 absorbs both reservation duties (a single closing phase rather than P10 split into two — finalized at execution time per intake volume). Each SURPRISES entry → RESOLVED | DEFERRED | WONTFIX with commit SHA or rationale. GOOD-TO-HAVES entries: XS always close, S close if budget, M default-defer to v0.13.3 (Q6 deferrals — `${...}` reject + 2KB cap — are the canonical M-class default-defers per Q6 ratification). Verifier honesty spot-check on previous phases' plans + verdicts ("did P97–P105 honestly look for out-of-scope items?"). Milestone-close ritual: CHANGELOG `[v0.13.2]` finalized; tag-script at `.planning/milestones/v0.13.2-phases/tag-v0.13.2.sh` (≥6 safety guards); RETROSPECTIVE.md v0.13.2 section distilled per OP-9 BEFORE archive; milestone-close verifier subagent dispatched and GREEN. Owner runs `tag-v0.13.2.sh` — orchestrator does NOT push the tag.

**Requirements:** XLINK-SURPRISES-01, XLINK-GOOD-TO-HAVES-01, XLINK-MILESTONE-CLOSE-01 · **Depends on:** P97 + P98 + P99 + P100 + P101 + P102 + P103 + P104 + P105 ALL GREEN · **Plan:** TBD

**Success criteria:**
1. Every entry in `SURPRISES-INTAKE.md` has terminal STATUS (RESOLVED + commit SHA / DEFERRED + target milestone / WONTFIX + rationale). No `STATUS: TBD` at phase close.
2. Verifier honesty spot-check samples ≥3 P97–P105 plan/verdict pairs; spot-check report at `quality/reports/verdicts/p106/honesty-spot-check.md`. Empty intake acceptable IF phases produced explicit `Eager-resolution` decisions; empty intake when verdicts show skipped findings → RED.
3. `GOOD-TO-HAVES.md` drained: every entry terminal STATUS — XS closed (commit SHA), S closed-or-deferred (rationale), M default-deferred to v0.13.3 (carry-forward target named). The two Q6 deferrals (`XLINK-SANITIZE-DOLLAR-VAR-REJECT` M-class, `XLINK-SANITIZE-2KB-CAP` S-class) seeded in P101 close per their default disposition.
4. CHANGELOG `[v0.13.2]` finalized: summarizes P97–P106 + lists every shipped REQ-ID by category + names v0.13.3 carry-forwards (Q6 deferrals + any `suggest-promote` / `auto-fix` / prompt-override deferrals from extraction-plan).
5. Tag-script authored at `.planning/milestones/v0.13.2-phases/tag-v0.13.2.sh` with ≥6 safety guards (clean tree, on `main`, version match, CHANGELOG entry exists, tests green, signed tag); tag-gate guards re-run cleanly post-P106.
6. RETROSPECTIVE.md v0.13.2 section distilled (OP-9) BEFORE archive: What Was Built / What Worked / What Was Inefficient / Patterns Established / Key Lessons. Source: SURPRISES-INTAKE + GOOD-TO-HAVES + per-phase verdicts + dogfood findings (P105) + autonomous-run findings if any.
7. Milestone-close verifier dispatched and GREEN at `quality/reports/verdicts/milestone-v0.13.2/VERDICT.md`: P97–P106 catalog rows all GREEN-or-WAIVED; reposix dogfood at `block-newedge` is real; no expired waivers without follow-up; RETROSPECTIVE v0.13.2 section exists; +2 reservation operational.
8. STOP at tag boundary: orchestrator does NOT push the tag. STATE.md cursor updated to "v0.13.2 ready-to-tag; owner pushes tag."
9. Catalog rows + CLAUDE.md v0.13.2-shipped historical-milestone subsection land first.
10. Phase close: `git push origin main`; milestone-close verifier GREEN; verdict at `quality/reports/verdicts/p106/VERDICT.md` + `quality/reports/verdicts/milestone-v0.13.2/VERDICT.md`.

**Context anchor:** CLAUDE.md § "Operating Principles" OP-8 + OP-9; `.planning/milestones/v0.13.2-phases/SURPRISES-INTAKE.md` (created in-flight); `.planning/milestones/v0.13.2-phases/GOOD-TO-HAVES.md` (seeded P101 with Q6 deferrals); `.planning/RETROSPECTIVE.md`; `.planning/milestones/v0.13.0-phases/tag-v0.13.0.sh` + `.planning/milestones/v0.12.0-phases/tag-v0.12.0.sh` (tag-script precedents); `.planning/research/v0.13.2-cross-link-fidelity/07-extraction-plan.md` (carry-forward target for v1.x extraction items).
