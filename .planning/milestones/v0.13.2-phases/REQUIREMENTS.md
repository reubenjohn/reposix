# v0.13.2 Requirements — Cross-link fidelity

**Milestone status:** PLANNING (Phases P98–P107; formalized 2026-05-08).

**Milestone goal:** Land the project's 10th quality-gate dimension — `cross-link-fidelity` — that grades whether every markdown link `[A](B)` in the project still expresses an accurate *fidelity assertion* (does A's framing of B match what B currently teaches?). Catches the **unknown-unknowns** failure mode where progressive-disclosure parents silently lie because nobody re-graded them after the children drifted.

The litmus test: at v0.13.2 close, the reposix repo's own `default` scope is at `enforcement_mode: block-newedge` — every new edge added to the repo carries an L3 grade or blocks the push, and the project's existing ~400 edges have been bootstrapped to a non-zero coverage floor that monotonically ratchets upward.

**Mental model.** Two-sentence summary from `.planning/research/v0.13.2-cross-link-fidelity/index.md`: every markdown link `[A](B)` is treated as a *fidelity assertion* — A's framing of B should still match what B currently teaches. A new quality-gate dimension grades that assertion using a four-level scrutiny ladder (L0 link resolves → L1 anchor exists → L2 hash unchanged → L3 LLM-judged subjective fidelity), with brownfield-friendly ratcheting coverage floors per scope.

The five primitives:
- **Edge** — a `(source, target, anchors)` tuple from a markdown link.
- **Edge state** — one of `UNGRADED | GRADED | STALE | BROKEN` per ADR-11.
- **Scope** — glob pattern set + level + cadence + grading-defaults; binds an edge to a policy.
- **Catalog** — runner-readable JSON at `quality/catalogs/cross-link-fidelity.json` (~4 rows).
- **Tracker** — gate-internal JSON at `quality/state/cross-link-fidelity-tracker.json` (per-edge state).
- **Config** — human-authored TOML at `.cross-link-fidelity` (scopes + policy).

Five enforcement modes per scope (owner-controlled progression): `warn` → `block-broken` → `block-stale` → `block-floor` → `block-newedge`.

**Source-of-truth handover bundle:**
- `.planning/research/v0.13.2-cross-link-fidelity/index.md` — the entry point routing chapter-by-chapter.
- `.planning/research/v0.13.2-cross-link-fidelity/01-vision-and-problem.md` — why this gate exists.
- `.planning/research/v0.13.2-cross-link-fidelity/02-architecture.md` — five primitives + four-level ladder + scope model + edge-state taxonomy + five enforcement modes.
- `.planning/research/v0.13.2-cross-link-fidelity/03-schemas.md` — config / tracker / frontmatter schemas (incl. `grading_context`).
- `.planning/research/v0.13.2-cross-link-fidelity/04-cli-and-workflow.md` — CLI verb set + integration points.
- `.planning/research/v0.13.2-cross-link-fidelity/05-edge-cases.md` — 14 named failure modes with recovery paths (incl. § 12 secret leakage, § 11 API outage).
- `.planning/research/v0.13.2-cross-link-fidelity/06-decisions-log.md` — ADRs 1–28 (load-bearing decisions; 23–28 from two design-scrutiny passes).
- `.planning/research/v0.13.2-cross-link-fidelity/07-extraction-plan.md` — standalone-tool roadmap; carry-forward target for v1.x extraction items.
- `.planning/research/v0.13.2-cross-link-fidelity/08-open-questions.md` § "Owner ratification" — Q2 / Q6 / Q14 BLOCKS-PLAN ratifications + 6 deferrable opens.
- `.planning/research/v0.13.2-cross-link-fidelity/09-brownfield-and-onboarding.md` — adoption journey day-1 to steady-state.
- `.planning/research/v0.13.2-cross-link-fidelity/examples/` — `default-config.toml`, `tracker-row.json`, `frontmatter.md`, `ladder-walkthrough.md`.
- `.planning/research/v0.13.2-cross-link-fidelity/PROPOSED-ROADMAP.md` — historical artifact (uses placeholder P97–P106; superseded by this milestone's `ROADMAP.md` at P98–P107).

**Operating-principle hooks (non-negotiable, per project CLAUDE.md):**
- **OP-1 Simulator-first.** All v0.13.2 phases exercise their gate logic against in-memory or fixture inputs. L3 dispatch tests use a mock Anthropic client by default. Real-API tests gate milestone close (P106 dogfood + P107 milestone-close), not individual phase closes.
- **OP-2 Tainted-by-default.** `grading_context` content shipped to Anthropic in P102 carries the `Tainted<T>` marker. The cred-hygiene regex pre-commit (Q6 ratification) is the only sanitization at v1; `${...}` reject + 2KB cap are deferred to v0.13.3 GOOD-TO-HAVES per Q6 ratification.
- **OP-3 Audit log non-optional.** Every L3 dispatch writes a row to the cache audit table (request → vendor → cost → verdict-id) so a forensic query can trace every shipped grading_context byte.
- **OP-7 Verifier subagent dispatch on every phase close.** L3-graded fidelity outcomes are subagent-graded artifacts; the verifier reads them as evidence.
- **OP-8 +2 phase practice.** v0.13.2 reserves last 1 phase (P107) absorbing both surprises + good-to-haves duties (or splits at execution time per Q14 ratified shape).
- **OP-9 Milestone-close ritual: distill before archiving.** P107 distills SURPRISES + GOOD-TO-HAVES + per-phase verdicts + dogfood findings into `RETROSPECTIVE.md` v0.13.2 section BEFORE archive.
- **Per-phase push cadence (codified 2026-04-30).** Every phase closes with `git push origin main` BEFORE verifier-subagent dispatch. Pre-push gate-passing is part of phase-close criterion.

### Active

#### Crate skeleton + shared-compute lift + edge model + walker + catalog + tracker schemas (P98)

- [ ] **XLINK-MD-WALKER-LIFT-01**: Refactor `crates/reposix-quality/src/coverage.rs::{walk_md, eligible_files}` into a shared `crates/reposix-quality/src/md_walker.rs` per ADR-27. `coverage.rs` re-exports for backwards compatibility; doc-alignment tests still pass against the lifted module; cross-link's walker imports from `md_walker.rs`.
- [ ] **XLINK-HEADING-HASH-01**: Add `pub fn heading_subtree_hash(file: &Path, slug: &str) -> Result<String>` next to `source_hash` (line 29) and `test_body_hash` (line 92) in `crates/reposix-quality/src/hash.rs` per ADR-28. ≥3 unit tests cover (matched-heading / unknown-slug / multi-level-nesting).
- [ ] **XLINK-SKELETON-01**: Sub-command shape settled: `reposix-quality cross-link {walk, status, ...}` (decision documented in PLAN.md). First verb wired: `reposix-quality cross-link walk` emits a JSON array of `(source, target, anchors)` tuples.
- [ ] **XLINK-EDGE-MODEL-01**: Edge identity is path-derived per ADR-19. Bulk-move recovery via `cross-link rebind --auto` is OUT of P98 (lands in P104 alongside `suggest-scopes`).
- [ ] **XLINK-WALKER-01**: `reposix-quality cross-link walk` against the reposix repo emits between 350 and 450 edges (matches Q4 measurement: ~400 edges total addressable).
- [ ] **XLINK-CATALOG-SCHEMA-01**: `quality/catalogs/cross-link-fidelity.json` lands per ADR-25 with ~4 runner-readable rows (skeleton-builds, walker-emits-edges, tracker-schema-validates, catalog-schema-validates), each conforming to the unified row schema in `quality/catalogs/README.md`. Discovered cleanly by `quality/runners/run.py:62-69`.
- [ ] **XLINK-TRACKER-SCHEMA-01**: `quality/state/cross-link-fidelity-tracker.json` lands per ADR-25; schema versioned `1.0.0` per ADR-1 strict-semver; schema validates ≥3 example tracker rows under `tests/`. Runner does NOT touch the tracker.

#### Config TOML schema + scope resolution + glob matcher (P99)

- [ ] **XLINK-CONFIG-SCHEMA-01**: `examples/default-config.toml` (already in research folder) loads without errors; `reposix-quality cross-link config show` round-trips it. `default` scope at `max_level: L3` lands in the committed `.cross-link-fidelity` per ADR-8.
- [ ] **XLINK-SCOPE-RESOLVE-01**: Each walked edge resolves to exactly one scope; resolution decisions logged for every edge in `quality/reports/cross-link/scope-resolution.jsonl`.
- [ ] **XLINK-GLOB-MATCH-01**: Ambiguous matches respect the file-vs-folder priority rule per ADR-20 (file-glob beats folder-glob; nearest folder beats outer folder).

#### L0 + L1 verifiers + pre-commit hook integration (P100)

- [ ] **XLINK-L0-01**: L0 verifier — every walked edge with a non-existent target produces a `BROKEN` verdict; UNGRADED edges with valid targets stay UNGRADED.
- [ ] **XLINK-L1-01**: L1 verifier — anchor existence checked against the target's heading slug map (markdown spec heading-to-slug rules); missing-anchor edges surface as `BROKEN` with anchor name. False-positive guard: code-fence-wrapped links + commented-out links + footnote references excluded.
- [ ] **XLINK-PRECOMMIT-01**: Pre-commit hook integration — hook calls `reposix-quality cross-link walk --pre-commit` (only edges from changed files); <2s budget on the reposix repo per cadence taxonomy; failure prints offending edge + recovery hint.

#### L2 hash-drift + edge state classifier (P101)

- [ ] **XLINK-L2-HASH-01**: L2 verifier — target's content hash (sha256 of body bytes excluding frontmatter, normalized line endings) compared to tracker's `last_graded_target_hash`; mismatch ⇒ STALE.
- [ ] **XLINK-STATE-CLASSIFIER-01**: Full edge-state classifier `UNGRADED | GRADED | STALE | BROKEN` per ADR-11. Edge-state transitions tested against fixture: NEW→UNGRADED; UNGRADED→GRADED; GRADED→STALE; STALE→GRADED; GRADED→BROKEN; BROKEN→GRADED. `reposix-quality cross-link status` summarizes scope-by-scope edge-state mix; output is the input to L3 dispatch in P102. Drift-triggered-L3 contract documented per ADR-15: only STALE edges enter the L3 queue.

#### L3 judge dispatcher + grading_context merge (P102)

- [ ] **XLINK-L3-DISPATCH-REUSE-01**: L3 dispatcher reuses `persist_artifact()` per ADR-26. Verdicts written via the canonical shape `{ts, score, verdict, rationale, evidence_files, dispatched_via, asserts_passed, asserts_failed, stale}`; artifact directory matches `quality/reports/verifications/cross-link-fidelity/`. Reuse target: `.claude/skills/reposix-quality-review/lib/persist_artifact.py:33-59`.
- [ ] **XLINK-PATH-AB-01**: Path A/B dispatcher per ADR-26 — Path A invokes the Claude Code Task tool in-session for unbiased grading; Path B is the subprocess stub returning FAIL when no API key + no in-session orchestrator. MIGRATE-03 trap (runner sweep stomping fresh Path-A artifacts) explicitly mitigated. Reuse target: `.claude/skills/reposix-quality-review/lib/dispatch_inline_subagent.sh:39-76`.
- [ ] **XLINK-GRADING-CONTEXT-MERGE-01**: L3 dispatcher takes a STALE edge → loads source + target → merges three-flavor `grading_context` (target ⊕ edge ⊕ source) per ADR-6 → ships to Anthropic Sonnet via Path A/B → parses verdict (PASS / FLAG / BLOCK + rationale).
- [ ] **XLINK-CRED-HYGIENE-01**: Cred-hygiene regex pre-commit only per Q6 ratification. Pre-commit blocks credential-pattern matches in `grading_context` frontmatter; `${VAR}` syntax and >2KB blocks pass through (knowingly, with explicit author-discipline doc in CLAUDE.md + `docs/concepts/cross-link-fidelity.md`).
- [ ] **XLINK-AUDIT-LOG-01**: Audit row per dispatch written to cache audit table per OP-3 schema (`op_type=cross_link_l3_dispatch`, payload=verdict-id + cost cents); forensic query reads both audit tables. Cost budget guard: aggregate L3 cost per phase tracked; phase close prints "L3 cost this phase: $X.YZ".

#### `bootstrap` + `plan-refresh` + cron CI integration (P103)

- [ ] **XLINK-BOOTSTRAP-01**: `bootstrap` verb — full L0+L1+L2+L3 sweep against every edge in scope (CI-only per ADR-15; never on contributor laptops); writes the resulting tracker as a single commit at end-of-run per Q3 (c); abort + audit-log on partial failure.
- [ ] **XLINK-PLAN-REFRESH-01**: `plan-refresh` verb — incremental sweep on edges where source-or-target changed since last run; updates tracker as a follow-up commit per Q3 (b); pre-push hook integration deferred to P105.
- [ ] **XLINK-CRON-CI-01**: Cron CI workflow at `.github/workflows/cross-link-cron.yml` — nightly bootstrap (full sweep) + per-PR plan-refresh; both gated by `secrets.ANTHROPIC_API_KEY`; cost-asymmetry guard ensures contributor pre-push never triggers cron-budget L3.

#### `suggest-scopes` migration assistant (P104)

- [ ] **XLINK-SUGGEST-SCOPES-01**: `reposix-quality cross-link suggest-scopes` against a brownfield fixture (the reposix repo before any `.cross-link-fidelity` exists) emits a TOML proposal that round-trips through P99's config loader. User agency preserved: verb proposes a TOML to stdout (or `--write` flag to `.cross-link-fidelity.proposal`); never overwrites existing config. Bulk-move recovery: `cross-link rebind --auto` per ADR-19 lands in P104 alongside `suggest-scopes` (related migration tooling).
- [ ] **XLINK-NAV-ONLY-HEURISTIC-01**: Nav-only heuristic — README files where ≥80% of outgoing links target siblings get a `nav-only` scope at `max_level: L0` (cheap).

#### Pre-push hook integration; phased enforcement modes; `max_l3_per_push` cap (P105)

- [ ] **XLINK-PREPUSH-01**: Pre-push hook integration — `bash scripts/install-hooks.sh` wires `cross-link prepush` into `.githooks/pre-push`; budget <60s per cadence taxonomy; failure surfaces offending edges + scope mode + recovery hint.
- [ ] **XLINK-ENFORCEMENT-MODES-01**: Five modes implemented — `warn` (no block, log only), `block-broken` (L0/L1 BROKEN blocks), `block-stale` (BROKEN + STALE blocks), `block-floor` (coverage drops below floor blocks), `block-newedge` (any new UNGRADED edge blocks).
- [ ] **XLINK-MAX-L3-CAP-01**: `max_l3_per_push` cap (default 25 per ADR-21) — when push would dispatch >cap STALE edges, refuse and stderr names `git rebase --interactive` for split-PR recovery.
- [ ] **XLINK-RESET-FLOOR-01**: Coverage floor monotonic per ADR-22 — pushes that reduce floor are rejected; `cross-link reset-floor <scope> --reason "<text>"` (Q8 (a)) is the only way down; reset writes audit-log row.

#### Reposix dogfood — bootstrap + flip default to `block-newedge` (P106)

- [ ] **XLINK-DOGFOOD-BOOTSTRAP-01**: `reposix-quality cross-link bootstrap` runs against the full reposix repo edge population; produces tracker; commits with rationale message. Cost report: total L3 dispatch count + cost in USD posted in the phase verdict.
- [ ] **XLINK-DOGFOOD-MODE-PROGRESSION-01**: Mode progression evidence — each mode flip (`warn` → `block-broken` → `block-stale` → `block-floor` → `block-newedge`) is its own commit with verifier-run output proving the prior mode was clean. Final state: `default` scope `enforcement_mode: block-newedge`; coverage_floor reflects the bootstrap's L3 coverage ratio.

### +2 reservation (per OP-8)

- [ ] **XLINK-SURPRISES-01** (P107): Surprises-absorption duty drains `.planning/milestones/v0.13.2-phases/SURPRISES-INTAKE.md`. Each entry → RESOLVED | DEFERRED | WONTFIX with commit SHA or rationale. Verifier honesty spot-check on previous phases' (P98–P106) plans + verdicts (empty intake acceptable IF phases produced explicit `Eager-resolution` decisions).
- [ ] **XLINK-GOOD-TO-HAVES-01** (P107): Good-to-haves polish duty drains `.planning/milestones/v0.13.2-phases/GOOD-TO-HAVES.md`. XS items always close; S items close-or-defer; M items default-defer to v0.13.3 (Q6 deferrals — `XLINK-SANITIZE-DOLLAR-VAR-REJECT` + `XLINK-SANITIZE-2KB-CAP` — are the canonical default-defers).
- [ ] **XLINK-MILESTONE-CLOSE-01** (P107): Milestone-close ritual — CHANGELOG `[v0.13.2]` finalized; tag-script at `.planning/milestones/v0.13.2-phases/tag-v0.13.2.sh` (≥6 safety guards mirroring v0.13.0 + v0.12.0 precedents); RETROSPECTIVE.md v0.13.2 section distilled per OP-9 BEFORE archive; milestone-close verifier subagent dispatched and GREEN at `quality/reports/verdicts/milestone-v0.13.2/VERDICT.md`. Owner runs `tag-v0.13.2.sh` — orchestrator does NOT push the tag.

### Out of Scope (deferred to v0.13.3 or later)

- **`${VAR}` syntax reject in `grading_context`** (Q6 deferral). Templated env-vars pass through at v1; M-class GOOD-TO-HAVE seeded in P102 (`XLINK-SANITIZE-DOLLAR-VAR-REJECT`); default-defers to v0.13.3 per Q6 ratification.
- **2KB length cap on `grading_context` blocks** (Q6 deferral). Accidental large log dumps pass through at v1; S-class GOOD-TO-HAVE seeded in P102 (`XLINK-SANITIZE-2KB-CAP`); defer-to-budget at P107.
- **Auto-fix sketch** for stale edges (`auto-fix` verb proposing source rewrites). Deferred to v0.13.3 per `08-open-questions.md` deferrable opens.
- **Prompt registry override** for L3 judge prompts. Deferred to v0.13.3 per `08-open-questions.md` deferrable opens.
- **`suggest-promote` verb** for ratcheting `default` scope mode automatically. Deferred to v0.13.3 per `07-extraction-plan.md`.
- **Auto-classifier heuristics for nav-only beyond README**. Deferred to v0.13.3 per `08-open-questions.md`.
- **Standalone-tool extraction** (`crates/cross-link-fidelity/`). v1.x extraction per `07-extraction-plan.md`; this milestone ships as a sub-command of `reposix-quality` per Q2 ratification.

### Traceability

Drafted 2026-05-08 (formalized from `PROPOSED-ROADMAP.md`). Coverage: **30/30 v0.13.2 REQ-IDs mapped to exactly one phase** (no orphans, no duplicates). Phases P98–P107; v0.13.2 starts at P98 (continuing after v0.13.0 extension milestone P89–P97).

| REQ-ID | Phase | Success-criterion # | Status |
|--------|-------|---------------------|--------|
| XLINK-MD-WALKER-LIFT-01 | P98 | 1 | planned |
| XLINK-HEADING-HASH-01 | P98 | 2 | planned |
| XLINK-SKELETON-01 | P98 | 3 | planned |
| XLINK-EDGE-MODEL-01 | P98 | 4 | planned |
| XLINK-WALKER-01 | P98 | 7 | planned |
| XLINK-CATALOG-SCHEMA-01 | P98 | 5 | planned |
| XLINK-TRACKER-SCHEMA-01 | P98 | 6 | planned |
| XLINK-CONFIG-SCHEMA-01 | P99 | 1, 3 | planned |
| XLINK-SCOPE-RESOLVE-01 | P99 | 2 | planned |
| XLINK-GLOB-MATCH-01 | P99 | 2 | planned |
| XLINK-L0-01 | P100 | 1 | planned |
| XLINK-L1-01 | P100 | 2, 4 | planned |
| XLINK-PRECOMMIT-01 | P100 | 3 | planned |
| XLINK-L2-HASH-01 | P101 | 1 | planned |
| XLINK-STATE-CLASSIFIER-01 | P101 | 2, 3, 4 | planned |
| XLINK-L3-DISPATCH-REUSE-01 | P102 | 1 | planned |
| XLINK-PATH-AB-01 | P102 | 2 | planned |
| XLINK-GRADING-CONTEXT-MERGE-01 | P102 | 3 | planned |
| XLINK-CRED-HYGIENE-01 | P102 | 4 | planned |
| XLINK-AUDIT-LOG-01 | P102 | 5, 6 | planned |
| XLINK-BOOTSTRAP-01 | P103 | 1 | planned |
| XLINK-PLAN-REFRESH-01 | P103 | 2 | planned |
| XLINK-CRON-CI-01 | P103 | 3 | planned |
| XLINK-SUGGEST-SCOPES-01 | P104 | 1, 3, 4 | planned |
| XLINK-NAV-ONLY-HEURISTIC-01 | P104 | 2 | planned |
| XLINK-PREPUSH-01 | P105 | 1 | planned |
| XLINK-ENFORCEMENT-MODES-01 | P105 | 2 | planned |
| XLINK-MAX-L3-CAP-01 | P105 | 3 | planned |
| XLINK-RESET-FLOOR-01 | P105 | 4 | planned |
| XLINK-DOGFOOD-BOOTSTRAP-01 | P106 | 1, 4 | planned |
| XLINK-DOGFOOD-MODE-PROGRESSION-01 | P106 | 2, 3 | planned |
| XLINK-SURPRISES-01 | P107 | 1, 2 | planned |
| XLINK-GOOD-TO-HAVES-01 | P107 | 3 | planned |
| XLINK-MILESTONE-CLOSE-01 | P107 | 4, 5, 6, 7, 8 | planned |

### Recurring success criteria across every v0.13.2 phase

These are part of every phase's definition-of-done and are NOT separate REQ-IDs (they are recurring expressions of OP-7 + the autonomous-execution protocol):
- **Catalog-first**: phase's first commit writes catalog rows BEFORE implementation. The cross-link-fidelity catalog at `quality/catalogs/cross-link-fidelity.json` is created in P98 and incremented (not duplicated) per subsequent phase.
- **CLAUDE.md update in same PR** (per QG-07 carry-over from v0.12.0). The "Quality Gates" 9-dimension table grows to 10 in P98; the dimension row is filled in incrementally as L0/L1/L2/L3 land.
- **Unbiased verifier-subagent dispatch on phase close** (per OP-7).
- **Per-phase push** — `git push origin main` BEFORE verifier-subagent dispatch; pre-push gate-passing is part of close criterion (codified 2026-04-30, closes 999.4).
- **Eager-resolution preference** per OP-8 — items < 1hr / no new dependency get fixed in the discovering phase; else appended to `SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md`.
- **Goal: pristine codebase across all dimensions** — every dimension's gates GREEN-or-WAIVED at milestone close.
