# v0.12.1 Requirements

_Carry-forward bundle from v0.12.0 (P63 MIGRATE-03 — 2026-04-28). Every item below is anchored to a v0.12.0 source: a stub catalog row, a SURPRISES entry, or a v0.11.x carry-forward._

## Scope

v0.12.1 closes the v0.12.0 carry-forward debts: perf-dimension full implementation, security-dimension stubs->real, cross-platform rehearsals, MSRV / binstall / latest-pointer fixes, the `Error::Other` 156->144 completion (POLISH2-09 from v0.11.1), and three subjective-dimension runner-invariant fixes from P61 Wave G.

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

## Out of Scope (deferred beyond v0.12.1)

- Threat-model rewrite for v0.9.0 architecture (deferred to a separate security-focused milestone).

## Traceability

18 requirements -> >= 5 phases (P66 coverage_ratio added 2026-04-28; original 13 carry-forwards distributed across P67-P71 per ROADMAP.md).
