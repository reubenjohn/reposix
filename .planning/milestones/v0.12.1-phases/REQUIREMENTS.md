# v0.12.1 Requirements

_Carry-forward bundle from v0.12.0 (P63 MIGRATE-03 — 2026-04-28). Every item below is anchored to a v0.12.0 source: a stub catalog row, a SURPRISES entry, or a v0.11.x carry-forward._

## Scope

v0.12.1 closes:
1. **Docs-alignment cleanup (P72-P75)** — bind 22 `MISSING_TEST` rows to single-purpose verifiers; close out 23 `RETIRE_PROPOSED` rows (owner TTY); fix the bind-verb hash-overwrite bug surfaced during P67's cluster sweeps. **This is the autonomous run.**
2. **+2 phase reservation (P76-P77)** — surprises absorption + good-to-haves polish per CLAUDE.md OP-8.
3. **v0.12.0 carry-forwards (P67-P71, deferred to follow-up session)** — perf-dimension full implementation, security-dimension stubs->real, cross-platform rehearsals, MSRV / binstall / latest-pointer fixes, the `Error::Other` 156->144 completion (POLISH2-09 from v0.11.1), and three subjective-dimension runner-invariant fixes from P61 Wave G.

## Requirements

### Docs-alignment coverage metric (P66)
- [x] **COVERAGE-01** — `Summary` struct grows 4 fields with serde back-compat: `coverage_ratio` (f64; default 0.0), `lines_covered` (u64; default 0), `total_eligible_lines` (u64; default 0), `coverage_floor` (f64; default 0.10 via `default_coverage_floor` fn). All four `#[serde(default)]` so legacy populated catalogs deserialize. Source: P66 prompt § 1.
- [x] **COVERAGE-02** — `crates/reposix-quality/src/coverage.rs` exposes `eligible_files`, `line_count`, `merge_ranges`, `covered_lines_for_file`, `compute_per_file`, `compute_global`. Range-merge unions overlapping AND adjacent inclusive ranges; multi-source rows attribute to each cited file independently; out-of-eligible rows warn to stderr + skip; ≥8 integration tests in `crates/reposix-quality/tests/coverage.rs` plus ≥4 unit tests for `merge_ranges`. Source: P66 prompt § 2 + § 5.
- [x] **COVERAGE-03** — `walk` populates `summary.lines_covered` + `summary.total_eligible_lines` + `summary.coverage_ratio` each run; BLOCKs (exit non-zero, stderr names `/reposix-quality-backfill` recovery path) when `coverage_ratio < coverage_floor`. The walker NEVER auto-tunes `coverage_floor`. Source: P66 prompt § 3.
- [x] **COVERAGE-04** — `status` displays a global summary block AND a per-file (worst-coverage-first) table; `--top N` (default 20), `--all`, `--json` flags supported; ZERO ROWS hint when `row_count == 0` AND `total_lines > 50`. Source: P66 prompt § 4.
- [x] **COVERAGE-05** — `quality/catalogs/README.md` documents the 2x2 alignment-vs-coverage matrix + `coverage_floor` ratchet semantics; CLAUDE.md gains a P66 H3 subsection ≤30 lines under whatever v0.12.1 in-flight section convention exists; verifier verdict GREEN at `quality/reports/verdicts/p66/VERDICT.md` with explicit note that pre-push exit non-zero on `docs-alignment/walk` is INTENDED until v0.12.1 cluster phases close enough rows. Source: P66 prompt § 7 + § 8 + § 9.

### Perf dimension (full implementation)
- [ ] **PERF-01** — Latency vs headline-copy cross-check. Wire `quality/gates/perf/latency-vs-headline-copy.sh` to (a) run `quality/gates/perf/latency-bench.sh`, (b) parse the bench output, (c) diff per-backend numbers against the headline numbers in `docs/benchmarks/latency.md` within +/-15% tolerance. Source: `quality/catalogs/perf-targets.json` row `perf/latency-bench` (P59 stub) + P63 reaffirmation in MIGRATE-03.
- [ ] **PERF-02** — Token-economy bench cross-check. Wire `quality/gates/perf/token-economy-bench.sh` to run `python3 quality/gates/perf/bench_token_economy.py` + cross-check against headline numbers in user-facing docs. Source: stub row `perf/token-economy-bench`.
- [ ] **PERF-03** — Golden-path envelope (post-release). Wire `quality/gates/perf/golden-path-envelope.sh` (or extend `headline-numbers-cross-check.py`) to observe latency against simulator + sanctioned real backends; assert envelope per `docs/benchmarks/latency.md`. Cron schedule + sanctioned creds. Source: stub row `perf/headline-numbers-cross-check`.

### Security dimension (stubs -> real)
- [ ] **SEC-01** — Allowlist-enforcement gate. Wire `quality/gates/security/allowlist-enforcement.sh` to run a fuzz test that attempts outbound HTTP to an origin not in `REPOSIX_ALLOWED_ORIGINS` and asserts the `http::client()` factory rejects with the documented error. Source: stub row `security/allowlist-enforcement` + CLAUDE.md threat-model section "Outbound HTTP allowlist".
- [ ] **SEC-02** — Audit-immutability test. Wire `quality/gates/security/audit-immutability.sh` to attempt UPDATE/DELETE against either audit table (`audit_events_cache`, `audit_events`) and assert the SQLite layer rejects (append-only by schema, WAL mode). Source: stub row `security/audit-immutability` + CLAUDE.md OP-3.

### Cross-platform rehearsals
- [ ] **CROSS-01** — Windows-2022 GH Actions runner rehearsal of curl-installer + cargo-binstall paths. Source: stub row `cross-platform/windows-2022-rehearsal` + REQUIREMENTS.md MIGRATE-03 (d).
- [ ] **CROSS-02** — macOS-14 GH Actions runner rehearsal. Source: stub row `cross-platform/macos-14-rehearsal`.

### MSRV / binstall / release-pipeline carry-forwards (from v0.12.0 P56 SURPRISES)
- [ ] **MSRV-01** — Bump Rust MSRV 1.82 -> 1.85 OR cap transitive `block-buffer` at `<0.12`. Currently `cargo install reposix-cli` from crates.io fails on MSRV 1.82 because `block-buffer-0.12.0` requires `edition2024`. Source: REQUIREMENTS.md MIGRATE-03 (d) + P56 SURPRISES.md.
- [ ] **BINSTALL-01** — `[package.metadata.binstall]` blocks (~10 LOC) in `crates/reposix-cli/Cargo.toml` + `crates/reposix-remote/Cargo.toml` rewritten to match `release.yml` archive shape (`reposix-cli-v${version}` tag prefix, `reposix-v${version}-${target}.tar.gz` archive basename, x86_64-unknown-linux-musl + aarch64-unknown-linux-musl target overrides). Lifts `install/cargo-binstall` PARTIAL -> PASS. Source: REQUIREMENTS.md MIGRATE-03 (c) + P56 SURPRISES.md.
- [ ] **LATEST-PTR-01** — Pin `releases/latest/download/...` pointer to the cli release after every per-crate release sequence. Either `gh release create --latest` flag OR release-plz config to publish reposix-cli last. Source: REQUIREMENTS.md MIGRATE-03 (a) + P56 SURPRISES.md row 2.
- [ ] **RELEASE-PAT-01** — release-plz workflow uses fine-grained PAT (or adds a post-tag `gh workflow run` step) so `GITHUB_TOKEN`-pushed tags trigger `release.yml` instead of being silently dropped by GH loop-prevention. Source: REQUIREMENTS.md MIGRATE-03 (b) + P56 SURPRISES.md row 1.

### v0.11.1 carry-forward
- [ ] **ERR-OTHER-01** — Complete the `Error::Other(String)` 156 -> 144 partial migration (POLISH2-09 from v0.11.1; stubbed in v0.12.0 MIGRATE-03). No NEW `Error::Other(String)` sites accepted in v0.12.0; v0.12.1 closes the migration.

### Subjective dimension runner invariants (from P61 Wave G)
- [ ] **SUBJ-RUNNER-01** — Dispatch-and-preserve runner invariant. Extend `quality/runners/run.py` `run_row` so a row with `kind=subagent-graded` AND a recent artifact whose `dispatched_via` starts with `Wave-G-Path-A` or `Path-A` is treated as authoritative — runner reads score + verdict from the artifact, sets row.status from the score-vs-threshold mapping, and does NOT overwrite. Source: REQUIREMENTS.md MIGRATE-03 (e) + P61 Wave G.
- [ ] **SUBJ-AUTODISPATCH-01** — CI auto-dispatch (Anthropic API auth on GH Actions runners). Source: REQUIREMENTS.md MIGRATE-03 (f).
- [ ] **SUBJ-HARDGATE-01** — Hard-gate chaining for `release.yml` waiting on `quality-pre-release.yml` verdict (composite workflow OR workflow_run trigger). Source: REQUIREMENTS.md MIGRATE-03 (g) + P56 SURPRISES row 1.

### Lint-config invariants (P72)
- [ ] **LINT-CONFIG-01** — `forbid-unsafe-code` verifier walks `crates/*/src/{lib,main}.rs` and asserts every file contains `#![forbid(unsafe_code)]`. Same verifier covers `forbid-unsafe-per-crate`. Source: catalog rows `README-md/forbid-unsafe-code` + `docs-development-contributing-md/forbid-unsafe-per-crate`.
- [ ] **LINT-CONFIG-02** — MSRV pin verifier asserts `rust-version = "1.82"` (or current MSRV) appears in workspace `Cargo.toml`. Source: `README-md/rust-1-82-requirement`.
- [ ] **LINT-CONFIG-03** — `tests-green` verifier compiles workspace tests (`cargo test --workspace --no-run`) and asserts exit 0. One cargo invocation per memory budget. Source: `README-md/tests-green`.
- [ ] **LINT-CONFIG-04** — `errors-doc-section-required` verifier asserts every `pub fn` returning `Result<_, _>` has a `# Errors` rustdoc section (clippy `missing_errors_doc` lint, or grep). Source: `docs-development-contributing-md/errors-doc-section-required`.
- [ ] **LINT-CONFIG-05** — `rust-stable-no-nightly` verifier asserts `rust-toolchain.toml` has `channel = "stable"`. Source: `docs-development-contributing-md/rust-stable-no-nightly`.
- [ ] **LINT-CONFIG-06** — `cargo-check-workspace-available` verifier runs `cargo check --workspace -q` and asserts exit 0. Source: `docs-development-contributing-md/cargo-check-workspace-available`.
- [ ] **LINT-CONFIG-07** — `cargo-test-count` verifier counts test binaries via `cargo test --workspace --no-run --message-format=json` and asserts ≥ documented threshold. PROSE update if count drifted. Source: `docs-development-contributing-md/cargo-test-133-tests`.
- [ ] **LINT-CONFIG-08** — `demo-script-exists` verifier asserts `[ -x scripts/dark-factory-test.sh ]`. Source: `docs-development-contributing-md/demo-script-exists`.
- [ ] **LINT-CONFIG-09** — All P72 verifier files live under `quality/gates/code/lint-invariants/` (single dimension home); CLAUDE.md gains a P72 H3 subsection ≤30 lines.

### Connector contract gaps (P73)
- [ ] **CONNECTOR-GAP-01** — Wiremock-based byte-exact `Authorization` header test in `crates/reposix-confluence/tests/auth_header.rs` (`Basic <base64>`) AND in `crates/reposix-github/tests/auth_header.rs` (`Bearer <token>`). Bind `docs/connectors/guide/auth-header-exact-test`. Source: `docs/guides/write-your-own-connector.md:158`.
- [ ] **CONNECTOR-GAP-02** — Bind `docs/connectors/guide/real-backend-smoke-fixture` to existing `crates/reposix-cli/tests/agent_flow_real.rs::dark_factory_real_{confluence,github,jira}` `#[ignore]` smoke fixtures. NO new code. Source: same write-your-own-connector.md:158 area.
- [ ] **CONNECTOR-GAP-03** — Rust test in `crates/reposix-jira/tests/list_records_excludes_attachments.rs` seeds wiremock with `fields.attachment` + `fields.comment.comments`; asserts `list_records` markdown output excludes both. Bind `docs/decisions/005-jira-issue-mapping/attachments-comments-excluded`. Source: `docs/decisions/005-jira-issue-mapping.md:79-87`.
- [ ] **CONNECTOR-GAP-04** — Resolve `docs/benchmarks/token-economy/jira-real-adapter-not-implemented` (STALE prose; JIRA adapter shipped in v0.11.x). Path (a): update prose row + bind to existence test; Path (b): `propose-retire` with rationale "superseded by JIRA adapter Phase 29". Executor picks per ROI threshold. Source: `docs/benchmarks/token-economy.md:23-28`.

### Narrative + UX cleanup + prose-fix (P74)
- [ ] **NARRATIVE-RETIRE-01** — `propose-retire use-case-20-percent-rest-mcp` (qualitative design framing).
- [ ] **NARRATIVE-RETIRE-02** — `propose-retire use-case-80-percent-routine-ops` (qualitative design framing).
- [ ] **NARRATIVE-RETIRE-03** — `propose-retire mcp-fixture-synthesized-not-live` (qualitative design framing).
- [ ] **NARRATIVE-RETIRE-04** — `propose-retire mcp-schema-discovery-100k-tokens` (qualitative design framing; the 100k figure is approximate by construction).
- [ ] **UX-BIND-01** — Bind `docs/index/5-line-install` to `quality/gates/docs-alignment/verifiers/install-snippet-shape.sh`. Verifier shape-checks the install line at `docs/index.md:19` matches `(curl|brew|cargo binstall|irm)` channel pattern.
- [ ] **UX-BIND-02** — Bind `docs/index/audit-trail-git-log` to `quality/gates/docs-alignment/verifiers/audit-trail-git-log.sh`. Verifier asserts `git log --oneline` returns ≥1 line for the repo (claim premise).
- [ ] **UX-BIND-03** — Bind `docs/index/tested-three-backends` to `quality/gates/docs-alignment/verifiers/three-backends-tested.sh`. Verifier asserts the 3 `dark_factory_real_*` test fns exist in `crates/reposix-cli/tests/agent_flow_real.rs`.
- [ ] **UX-BIND-04** — Bind `planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish2-06-landing` to `quality/gates/docs-alignment/verifiers/connector-matrix-on-landing.sh`. Verifier greps `docs/index.md` for the connector capability matrix heading + table.
- [ ] **UX-BIND-05** — Bind `planning-milestones-v0-8-0-phases-REQUIREMENTS-md/spaces-01` to `quality/gates/docs-alignment/verifiers/cli-spaces-smoke.sh`. Verifier runs `target/release/reposix spaces --help` and asserts exit 0 + the "List all readable Confluence spaces" header text.
- [ ] **PROSE-FIX-01** — Update `docs/social/linkedin.md:21` to drop "FUSE filesystem" framing; replace with "git-native partial clone + git-remote-helper". The existing BOUND row at line 21 (`docs/social/linkedin/token-reduction-92pct`) re-hashes via walker after refresh; expected STALE_DOCS_DRIFT then auto-rebind.

### Bind-verb hash-overwrite fix (P75)
- [ ] **BIND-VERB-FIX-01** — Fix `crates/reposix-quality/src/commands/doc_alignment.rs::verbs::bind` to preserve `source_hash` semantics for `Source::Multi` rows OR have the walker hash all sources from a `Multi`. Add regression test in `crates/reposix-quality/tests/walk.rs` covering the multi-source no-op walk path. Live catalog `walk` reports zero false STALE_DOCS_DRIFT after fix lands.

### Surprises absorption (P76, +2 reservation slot 1)
- [ ] **SURPRISES-ABSORB-01** — Drain `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md`. Each entry transitions to RESOLVED (commit SHA), DEFERRED (filed under v0.13.0 carry-forward), or WONTFIX (with rationale). Empty intake is acceptable IF the running phases honestly observed no out-of-scope items; verifier subagent checks honesty by spot-checking phase plans for skipped findings.

### Good-to-haves polish (P77, +2 reservation slot 2)
- [ ] **GOOD-TO-HAVES-01** — Drain `.planning/milestones/v0.12.1-phases/GOOD-TO-HAVES.md`. All XS items closed; S items closed if they fit before 5pm; M items deferred to v0.13.0 backlog with explicit reason. Each closed item has its own atomic commit referencing the GOOD-TO-HAVES.md entry.

## Out of Scope (deferred beyond v0.12.1)

- Threat-model rewrite for v0.9.0 architecture (deferred to a separate security-focused milestone).

## Traceability

40 requirements -> 12 phases (P66 shipped; P67-P71 deferred carry-forwards; P72-P77 = the autonomous-run cluster including the +2 reservation per CLAUDE.md OP-8).
