← [back to index](./index.md)

## Phases

### Phase 66: coverage_ratio metric — docs-alignment second axis

**Goal:** Add the `coverage_ratio` metric to the docs-alignment dimension as the second axis alongside `alignment_ratio`. Where `alignment_ratio` answers "of the claims we extracted, how many bind to passing tests?", `coverage_ratio` answers "of the prose we said we'd mine, what fraction of lines did we actually cover with at least one row?". The two together make the 2x2 (high/low alignment vs high/low coverage) the agent's mental model for gap-closure work. The `status` verb gains a per-file table sorted worst-coverage-first so an agent looking to widen coverage reads `status --top 10` and knows which files to mine next.

**Requirements:** COVERAGE-01, COVERAGE-02, COVERAGE-03, COVERAGE-04, COVERAGE-05.

**Depends on:** v0.12.0 P64 (catalog + walker live) + P65 (388 rows populated). No dependency on the other v0.12.1 phases.

**Execution mode:** `executor`.

**Success criteria:**
- `Summary` struct grows `coverage_ratio` + `lines_covered` + `total_eligible_lines` + `coverage_floor` (all `#[serde(default)]` for back-compat).
- `coverage::compute_per_file` + `compute_global` correctly union overlapping/adjacent ranges; multi-source rows attribute to each cited file independently; out-of-eligible-set rows warn + skip.
- `walk` populates the global summary fields each run AND BLOCKs when `coverage_ratio < coverage_floor` (default 0.10) with structured stderr naming the slash-command recovery path.
- `status` displays a global summary block + a per-file (worst-first) table; `--top N`, `--all`, `--json` flags supported.
- `quality/catalogs/README.md` documents the 2x2 alignment-vs-coverage matrix + `coverage_floor` ratchet semantics.
- Verifier subagent verdict GREEN at `quality/reports/verdicts/p66/VERDICT.md`.

**Critical invariant:** `coverage_floor` stays at 0.10 in the shipping commit. A future deliberate human commit ratchets it up as gap-closure phases (P72+) widen coverage.

### Phase 67: Perf-dimension full implementation

**Goal:** Wire the 3 perf-targets stub rows (`perf/latency-bench`, `perf/token-economy-bench`, `perf/headline-numbers-cross-check`) into real verifiers that cross-check bench output against doc headline numbers; flip waivers to active enforcement.

**Requirements:** PERF-01, PERF-02, PERF-03.

**Execution mode:** `executor`.

**Success criteria:** All 3 perf-targets rows PASS at runner time; waivers removed; cron-driven verification posts evidence under `quality/reports/verifications/perf/`.

### Phase 68: Security-dimension stubs -> real

**Goal:** Wire `security/allowlist-enforcement` + `security/audit-immutability` to real Rust integration tests + bash verifier wrappers. Lift waivers.

**Requirements:** SEC-01, SEC-02.

**Execution mode:** `executor`.

**Success criteria:** Both rows PASS; outbound-HTTP allowlist enforced at runtime; audit-table immutability tested at SQLite schema layer.

### Phase 69: Cross-platform rehearsals

**Goal:** Add windows-2022 + macos-14 GH Actions runners to `release.yml` matrix; rehearse curl-installer + cargo-binstall on each.

**Requirements:** CROSS-01, CROSS-02.

**Execution mode:** `executor`.

**Success criteria:** Both `cross-platform` catalog rows PASS at pre-release cadence; release artifacts proven cross-platform.

### Phase 70: MSRV / binstall / release-pipeline + Error::Other completion

**Goal:** Close the v0.12.0 P56 SURPRISES carry-forward set + the v0.11.1 POLISH2-09 partial migration.

**Requirements:** MSRV-01, BINSTALL-01, LATEST-PTR-01, RELEASE-PAT-01, ERR-OTHER-01.

**Execution mode:** `executor`.

**Success criteria:** `cargo install reposix-cli` from crates.io succeeds; `cargo binstall reposix-cli` PASS; `releases/latest/download/...` pointer pinned to cli release; release-plz tag triggers `release.yml`; zero `Error::Other(String)` sites remain.

### Phase 71: Subjective-dimension runner invariants (P61 Wave G carry-forward)

**Goal:** Implement the 3 subjective-runner invariants from P61 Wave G.

**Requirements:** SUBJ-RUNNER-01, SUBJ-AUTODISPATCH-01, SUBJ-HARDGATE-01.

**Execution mode:** `executor`.

**Success criteria:** Subjective-graded rows preserve dispatched verdicts; CI auto-dispatch works on Anthropic API auth'd GH Actions runners; `release.yml` waits on `quality-pre-release.yml` verdict.

### Phase 72: Lint-config invariants — bind 9 contributing.md / README.md claims

**Goal:** Bind 9 `MISSING_TEST` rows asserting workspace-level lint/configuration invariants to single-purpose shell verifiers under `quality/gates/code/lint-invariants/`. The verifier files are themselves hashed by the walker (P71 schema 2.0 `--test` accepts shell paths), so drift on either prose or verifier fires `STALE_DOCS_DRIFT` and an agent reviews. Concretizes "lint-config rows ARE testable" for the next maintainer.

**Requirements:** LINT-CONFIG-01..LINT-CONFIG-09.

**Catalog rows touched (9):**
- `README-md/forbid-unsafe-code` — verifier walks workspace, asserts every `crates/*/src/lib.rs` (or `main.rs` for binaries) contains `#![forbid(unsafe_code)]`.
- `README-md/rust-1-82-requirement` — grep `rust-version = "1.82"` (or current MSRV) in workspace `Cargo.toml`.
- `README-md/tests-green` — `cargo test --workspace --no-run` succeeds (compile-only; one cargo invocation per memory budget).
- `docs-development-contributing-md/forbid-unsafe-per-crate` — same verifier as `forbid-unsafe-code`; both rows bind to the SAME verifier file (single source of truth).
- `docs-development-contributing-md/errors-doc-section-required` — rustdoc check OR grep for `# Errors` doc section in every `pub fn` returning `Result`.
- `docs-development-contributing-md/rust-stable-no-nightly` — assert `rust-toolchain.toml` `channel = "stable"`.
- `docs-development-contributing-md/cargo-check-workspace-available` — assert `cargo check --workspace --message-format=json -q` exits 0 (compile-only; cheap).
- `docs-development-contributing-md/cargo-test-133-tests` — count `cargo test --workspace --no-run --message-format=json` test binaries; assert ≥ documented count. PROSE: re-measure and update README/contributing.md to the current count if drifted (don't bind to a stale number).
- `docs-development-contributing-md/demo-script-exists` — `[ -x scripts/dark-factory-test.sh ]`.

**Execution mode:** `executor`.

**Success criteria:**
- 9 verifier scripts under `quality/gates/code/lint-invariants/` exist, executable, exit 0 GREEN against current workspace.
- 9 catalog rows transition `MISSING_TEST` → `BOUND` after `target/release/reposix-quality doc-alignment refresh README.md docs/development/contributing.md`.
- CLAUDE.md gains a P72 H3 subsection ≤30 lines under "v0.12.1 — in flight".
- Verifier subagent verdict GREEN at `quality/reports/verdicts/p72/VERDICT.md` (Path A via `Task` from top-level orchestrator preferred; Path B in-session if executor depth-1 lacks Task per P56-P64 precedent).

### Phase 73: Connector contract gaps — bind 4 wiremock/decision rows

**Goal:** Close the 4 `MISSING_TEST` rows asserting connector contract behavior. Two bind to existing Rust tests (`agent_flow_real.rs::dark_factory_real_*`); two require NEW Rust tests (wiremock-based auth header byte-exact assertion + JIRA `list_records` excludes attachments/comments). One row's source prose is STALE (JIRA real adapter exists; doc says "not implemented") — fix prose first, then bind.

**Requirements:** CONNECTOR-GAP-01..CONNECTOR-GAP-04.

**Catalog rows touched (4):**
- `docs/connectors/guide/auth-header-exact-test` (docs/guides/write-your-own-connector.md:158) — write Rust test in `crates/reposix-confluence/tests/auth_header.rs` (or extend existing) using wiremock; assert `Authorization` header is byte-exact `Basic <base64>` for Confluence and `Bearer <token>` for GitHub. Bind both halves of the test pair.
- `docs/connectors/guide/real-backend-smoke-fixture` (line 158 area) — bind to existing `crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_{confluence,github,jira}` (`#[ignore]` smoke fixtures already shipped in v0.11.x). NO new code; pure rebind to existing test.
- `docs/decisions/005-jira-issue-mapping/attachments-comments-excluded` (decisions/005:79-87) — write Rust test in `crates/reposix-jira/tests/list_records_excludes_attachments.rs`; seed wiremock with JIRA issue containing `fields.attachment` + `fields.comment.comments`; assert `list_records` returns issues with NEITHER field rendered into the markdown body.
- `docs/benchmarks/token-economy/jira-real-adapter-not-implemented` (benchmarks/token-economy.md:23-28) — **prose is stale** (JIRA adapter shipped in v0.11.x). Two paths: (a) update prose row to acknowledge adapter exists + bench numbers pending re-measurement, then bind to existence test; OR (b) `propose-retire` with rationale "superseded by JIRA adapter shipping in Phase 29". Executor picks (a) if measurement is cheap (< 30 min), (b) otherwise.

**Execution mode:** `executor`.

**Success criteria:**
- 2 new Rust tests pass (`cargo test -p reposix-confluence -p reposix-jira`); both `agent_flow_real.rs` smoke tests still pass with `--ignored` against a sanctioned target (or skipped if creds absent — pre-push doesn't require real creds).
- 4 rows transition `MISSING_TEST` → `BOUND` (or 1 → `RETIRE_CONFIRMED` if path (b) chosen) after `refresh`.
- CLAUDE.md gains a P73 H3 subsection ≤30 lines.
- Verifier subagent verdict GREEN at `quality/reports/verdicts/p73/VERDICT.md`.

### Phase 74: Narrative cleanup + docs/index UX bindings + linkedin prose fix

**Goal:** Close the remaining 9 `MISSING_TEST` rows + 1 prose-fix in `docs/social/linkedin.md`. Four narrative rows propose-retire as untestable design-framing; four UX claims bind to hash-shape verifiers (P71 `--test <path>` pattern); `spaces-01` binds to a `reposix spaces --help` smoke test (subcommand confirmed live during prep); polish2-06-landing binds to a doc-grep asserting the connector capability matrix exists in `docs/index.md`. Linkedin FUSE drift gets fixed via prose edit (no catalog change — the BOUND row at line 21 stays BOUND; walker re-hashes after refresh).

**Requirements:** NARRATIVE-RETIRE-01..04, UX-BIND-01..05, PROSE-FIX-01.

**Catalog actions (10):**
- **Propose-retire** (4 rows; rationale "qualitative design framing; no behavioral assertion possible"):
  - `use-case-20-percent-rest-mcp`
  - `use-case-80-percent-routine-ops`
  - `mcp-fixture-synthesized-not-live`
  - `mcp-schema-discovery-100k-tokens`
- **Bind to hash-shape verifiers** (4 rows; verifier file under `quality/gates/docs-alignment/verifiers/`):
  - `docs/index/5-line-install` — `install-snippet-shape.sh` asserts `docs/index.md:19` is one line + matches the install-channel pattern (`curl|brew|cargo binstall|irm`); 5-line claim is shape-checked in `tutorials/first-run.md`.
  - `docs/index/audit-trail-git-log` — `audit-trail-git-log.sh` asserts `git log --grep` shows audit-relevant commits (the claim is "git log IS the audit trail" — verifier asserts the claim's premise via shell).
  - `docs/index/tested-three-backends` — `three-backends-tested.sh` asserts the 3 sanctioned `dark_factory_real_*` test functions exist in `crates/reposix-cli/tests/agent_flow_real.rs`.
  - `planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing` — `connector-matrix-on-landing.sh` asserts `docs/index.md` contains a connector capability matrix (grep heading + table marker).
- **Bind to CLI smoke test** (1 row):
  - `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01` — bind to a verifier running `target/release/reposix spaces --help` and asserting exit 0 + expected help text. Subcommand confirmed live in `crates/reposix-cli/src/spaces.rs`.
- **Prose fix** (1 file, no row state change):
  - `docs/social/linkedin.md:21` — drop "FUSE filesystem" framing; replace with "git-native partial clone + git-remote-helper" (v0.9.0 architecture). Walker re-hashes the existing BOUND row at refresh; expect STALE_DOCS_DRIFT then immediate re-bind.

**Execution mode:** `executor`.

**Success criteria:**
- 4 rows `RETIRE_PROPOSED` (owner confirms in step 1 of HANDOVER bulk-confirm — NOT in this phase; agents cannot run `confirm-retire`).
- 5 rows `BOUND` after `refresh`.
- `docs/social/linkedin.md` updated; existing BOUND row at line 21 re-hashed and remains BOUND.
- CLAUDE.md gains a P74 H3 subsection ≤30 lines.
- Verifier subagent verdict GREEN at `quality/reports/verdicts/p74/VERDICT.md`.

### Phase 75: Bind-verb hash-overwrite fix (HANDOVER §4)

**Goal:** Fix the bind-verb / walker hash-asymmetry bug that caused false `STALE_DOCS_DRIFT` after every cluster sweep this session. Either preserve first-source semantics in `bind` (don't overwrite `source_hash` when adding `Source::Multi`) OR have walker hash all sources from a `Multi` and AND them.

**Requirements:** BIND-VERB-FIX-01.

**Files touched:**
- `crates/reposix-quality/src/commands/doc_alignment.rs` — `verbs::bind` (line ~295 area).
- `crates/reposix-quality/src/commands/doc_alignment.rs` — walker hash check (`walker::check_source_hash` or equivalent).
- `crates/reposix-quality/tests/walk.rs` — add regression test: row with `Source::Multi([A, B])` survives walker hash check after both A's prose AND B's prose stay byte-identical.

**Execution mode:** `executor`.

**Success criteria:**
- Regression test added + passes.
- Live catalog walked clean: zero rows flip to `STALE_DOCS_DRIFT` from a no-op walk after this fix lands.
- CLAUDE.md gains a P75 H3 subsection ≤20 lines.
- Verifier subagent verdict GREEN at `quality/reports/verdicts/p75/VERDICT.md`.

### Phase 76: Surprises absorption (the +2 reservation, slot 1)

**Goal:** Close the surprises surfaced by P72-P75 that were too out-of-scope to fix eagerly inside their discovering phase but too important to drop. Per the project's `+2 phase practice` operating principle (CLAUDE.md OP-8), every in-flight phase appends to `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` when it finds an issue rather than silently skipping or expanding scope. P76 reads that intake file and resolves each entry.

**Requirements:** SURPRISES-ABSORB-01 (umbrella; sub-items grow during P72-P75 execution).

**Intake mechanism:**
- File: `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` (created at prep; populated during P72-P75).
- Entry shape per surprise: `## YYYY-MM-DD HH:MM | discovered-by: P<N> | severity: BLOCKER|HIGH|MEDIUM|LOW`, then a paragraph describing what was found, why it was out-of-scope to fix in P<N>, and a sketched resolution.
- The discovering phase's plan MUST emit the entry as its first action when triaging an out-of-scope issue. Phase verdict cannot grade GREEN if `git diff` doesn't show a SURPRISES-INTAKE.md update for at least one identified out-of-scope issue when the phase observed any.
- **Eager-resolution preference:** if a surprise can be closed inside its discovering phase without doubling the phase's scope (rough heuristic: < 1 hour incremental work, no new dependency introduced, no new file created outside the phase's planned set), do it there. The intake file is for items that genuinely don't fit.

**Execution mode:** `executor` if intake is short (< 5 entries OR all LOW-severity); `top-level` if intake contains BLOCKER/HIGH (orchestrator triages, dispatches per item).

**Success criteria:**
- Every entry in SURPRISES-INTAKE.md transitions to one of: `RESOLVED` (commit SHA), `DEFERRED` (with new requirement filed under v0.13.0 carry-forward), or `WONTFIX` (with rationale).
- Empty SURPRISES-INTAKE.md is acceptable (a phase that surfaces zero surprises is honest, not suspicious — but the verifier checks the running phases actually looked).
- CLAUDE.md gains a P76 H3 subsection ≤30 lines summarizing what was absorbed.
- Verifier subagent verdict GREEN at `quality/reports/verdicts/p76/VERDICT.md`.

### Phase 77: Good-to-haves polish (the +2 reservation, slot 2)

**Goal:** Close the good-to-have polish items surfaced by P72-P75 that were too discretionary to fix eagerly. Same intake mechanism as P76, separate file: `GOOD-TO-HAVES.md`. Distinct from surprises (which fix something broken or risky); good-to-haves are improvements that make the next maintainer's life easier (better error messages, clearer doc cross-refs, redundant test consolidation, helper extractions, naming polish).

**Requirements:** GOOD-TO-HAVES-01 (umbrella; sub-items grow during P72-P75 execution).

**Intake mechanism:**
- File: `.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md` (created at prep; populated during P72-P75).
- Entry shape: `## discovered-by: P<N> | size: XS|S|M | impact: clarity|perf|consistency|grounding`, then a one-paragraph description and a one-line proposed fix.
- **Eager-resolution preference:** same threshold as P76 — if it's genuinely XS work, fold into the discovering phase. The intake captures items that need their own focused attention.

**Execution mode:** `executor`. ROI-aware: P77's scope is whatever fits in the time remaining at 5pm. A backlog of M-sized items is fine to defer; XS+S items are the priority.

**Success criteria:**
- All XS items closed; all S items either closed or moved to v0.13.0 backlog with explicit reason.
- Each closed item has its own atomic commit referencing the GOOD-TO-HAVES.md entry.
- CLAUDE.md gains a P77 H3 subsection ≤30 lines.
- Verifier subagent verdict GREEN at `quality/reports/verdicts/p77/VERDICT.md`.

### Phases 67-71 — DEFERRED to a follow-up v0.12.1 session

The carry-forward bundle (perf full implementation, security stubs→real, cross-platform, MSRV/binstall, subjective-runner) is too large to ship in the same autonomous run as P72-P77. P67-P71 remain on this milestone but are scoped for a SEPARATE session after the docs-alignment cleanup + +2 reservation ships and v0.12.1.0 is tagged.

**Milestone-completion contract:** v0.12.1 closes when:
1. P72-P77 ship GREEN (docs-alignment claims_missing_test → 0 within rounding; HANDOVER step 1 retire-confirms close the RETIRE_PROPOSED queue; SURPRISES-INTAKE + GOOD-TO-HAVES drained or explicitly deferred) — **this autonomous run.**
2. P67-P71 ship GREEN (carry-forward bundle) — **follow-up session.**
3. `quality/reports/verdicts/milestone-v0.12.1/VERDICT.md` graded GREEN by an unbiased subagent.
4. `floor_waiver` (if any was re-added during cluster sweeps) expires; `alignment_ratio` floor ratchets up to whatever the closing measurement is (rounded down to nearest 0.05).
