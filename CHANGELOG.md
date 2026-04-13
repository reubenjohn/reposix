# Changelog

All notable changes to reposix are documented here.
The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [SemVer](https://semver.org/spec/v2.0.0.html) once the project leaves alpha.

## [Unreleased]

— Nothing yet.

## [v0.2.0-alpha] — 2026-04-13 (post-noon)

The "real GitHub" cut. v0.1.0 was simulator-only on purpose; this release lets reposix talk to a real backend.

### Added

- **`crates/reposix-github`** — read-only GitHub Issues adapter implementing `IssueBackend`. Honors the `x-ratelimit-remaining` / `x-ratelimit-reset` headers via a `Mutex<Option<Instant>>` rate-limit gate (next call sleeps until reset, capped at 60s). Allowlist-aware: callers must set `REPOSIX_ALLOWED_ORIGINS=...,https://api.github.com`. Auth via optional `Bearer` token; `None` falls back to anonymous (60/hr).
- **`reposix list --backend github`** — `reposix list --backend github --project owner/repo` reads real GitHub issues end-to-end via the CLI. The IssueBackend trait, abstract since v0.1, is now reachable from the shipped binary.
- **`scripts/demos/parity.sh`** (Tier 3 demo) — `reposix list` against the simulator and `gh api` against `octocat/Hello-World`, normalized via `jq`, diffed line-by-line. The diff IS the proof: structure identical, only content differs.
- **`crates/reposix-github/tests/contract.rs`** — same five invariants run against `SimBackend` (always) and `GithubReadOnlyBackend` (`#[ignore]`-gated, opt-in via `cargo test -- --ignored`). The simulator lying becomes a CI failure.
- **`docs/decisions/001-github-state-mapping.md`** — ADR for how reposix's 5-valued `IssueStatus` round-trips through GitHub's `state + state_reason + label` model. Documents label-precedence tiebreak (review wins over progress).
- **`crates/reposix-swarm`** — adversarial swarm harness. `reposix-swarm --clients 50 --duration 30s --mode {sim-direct,fuse}` spawns N concurrent agents, each running `list + 3×read + 1×patch`. Reports HDR-histogram p50/p95/p99 per op, error counts, and asserts the SG-06 append-only audit invariant under load. Validated locally at 132,895 ops / 0% errors.
- **`scripts/demos/swarm.sh`** (Tier 4 demo) — wraps the swarm with sane defaults; tunable via `SWARM_CLIENTS` and `SWARM_DURATION` env vars. Not in CI smoke (too expensive per push).
- **CI: `integration-contract` job** — runs the contract test against real GitHub on every push, authenticated via `${{ secrets.GITHUB_TOKEN }}` (1000/hr ceiling, ample for per-push). Load-bearing (`continue-on-error: false`).
- **CI: `demos-smoke` job** — runs the four Tier 1 demos via `scripts/demos/smoke.sh` on every push. Load-bearing.
- **Codecov badge in README** — `CODECOV_TOKEN` was wired in v0.1.x; the badge just hadn't been added.

### Changed

- **`scripts/demo.sh` is now a shim** that execs `scripts/demos/full.sh`. Old documentation links continue to work; new structure is `scripts/demos/{01..04, full, parity, swarm}.sh`.
- **`continue-on-error: true` flipped off** on `integration (mounted FS)` (passing reliably since Phase 3) and `integration-contract` (now authenticated).
- **`SimBackend::with_agent_suffix`** — Phase 9 needed per-client agent IDs to dodge per-token rate limits during swarm runs.
- **`MORNING-BRIEF.md`, `PROJECT-STATUS.md`, `docs/why.md`** updated to mention real-GitHub via CLI, new test counts, deferred items.

### Fixed

- **Phase S H-01 / H-02** — `git-remote-reposix` `ProtoReader` now reads raw bytes instead of UTF-8 strings; CRLF round-trips, non-UTF-8 doesn't tear the protocol pipe.
- **Phase S H-03** — backend errors during `import`/`export` emit a proper `error refs/heads/main <kind>` line on the protocol stream instead of `?`-propagating into a torn pipe.
- **Phase S M-03** — `git-remote-reposix` `diff::plan` now does normalized-compare on blob bytes (parses both sides as `Issue`, compares values) so trailing-newline drift no longer emits phantom PATCHes for unchanged trees.
- **Phase 8 H-01 / L-02** — `reposix-core::http::HttpClient` is a sealed newtype around `reqwest::Client`; the inner client is unreachable, blocking allowlist-bypass via `client.get(url).send()`. `OriginGlob::parse` migrated to `url::Url::parse` so bracketed IPv6 (`http://[::1]:7777`) works.
- **Phase 8 H-02** — `reposix-core::audit::open_audit_db` enables `SQLITE_DBCONFIG_DEFENSIVE`, blocking the `writable_schema=ON` + `DELETE FROM sqlite_master` schema-attack bypass on the append-only audit invariant.
- **Phase 8 M-04** — `audit.sql` schema bootstrap wrapped in `BEGIN; ... COMMIT;` so the `DROP TRIGGER IF EXISTS; CREATE TRIGGER` race window can't be exploited by a concurrent UPDATE.
- **Phase 8 H-04 (CI)** — `reposix sim --no-seed` and `--rate-limit` flags actually plumb through to the spawned subprocess (were silently dropped).
- **Phase 8 H-01 (CI)** — `reposix demo` step 5 (audit-tail) used to short-circuit because the spawned sim was ephemeral; now uses an on-disk DB so the tail prints real rows. Also fixed the SQL column name (`agent` → `agent_id`).
- **Phase 8 LR-02** — `GithubReadOnlyBackend` now actually backs off on rate-limit exhaustion, not just logs.
- **Phase 8 MR-03** — `SimBackend::update_issue` "wildcard" test now uses a custom `wiremock::Match` impl that fails if `If-Match` is sent, not just a permissive matcher.
- **Phase 8 MR-05 (CI)** — `demos-smoke` and `integration-contract` jobs have explicit `timeout-minutes` so a stuck job can't block the next push indefinitely.

### Deferred to v0.3 / future

- Write path on `GithubReadOnlyBackend` — currently `create_issue` / `update_issue` / `delete_or_close` return `NotSupported`. v0.3.
- FUSE daemon and `git-remote-reposix` still hardcode the simulator. The `IssueBackend` trait makes the rewire mechanical; v0.3 lands "FUSE-mount real GitHub".
- Adversarial swarm in CI (small-scope) — currently `swarm.sh` is excluded from `demos-smoke` because 30s per push is expensive.
- Jira and Linear adapters.

## [v0.1.0] — 2026-04-13 (~05:38)

Initial autonomous overnight build. Tagged at `v0.1.0`.

### Shipped

- Five-crate Rust workspace (`reposix-{core,sim,fuse,remote,cli}`).
- 8 security guardrails (SG-01 through SG-08), each enforced and tested.
- FUSE daemon (`fuser` 0.17, no `libfuse-dev` linkage required) supporting read + write.
- `git-remote-reposix` git remote helper supporting `import` / `export` capability with bulk-delete cap (SG-02).
- In-process axum REST simulator with rate-limit middleware, ETag-based 409 path, append-only SQLite audit log.
- `scripts/demo.sh` end-to-end walkthrough + `script(1)` recording showing three guardrails firing on camera (SG-01 allowlist refusal, SG-02 bulk-delete cap, SG-03 sanitize-on-egress).
- MkDocs documentation site at <https://reubenjohn.github.io/reposix/> with 11 mermaid architecture diagrams.
- `benchmarks/RESULTS.md` — measured **92.3% reduction** in input-context tokens (reposix vs MCP for the same task).
- 139 workspace tests passing, `cargo clippy --workspace --all-targets -- -D warnings` clean, `#![forbid(unsafe_code)]` at every crate root.
- CI green: rustfmt + clippy + test + coverage + integration mount.

### Notable design decisions

- Simulator-first per the StrongDM "dark factory" pattern (see `AgenticEngineeringReference.md` §1).
- Adversarial red-team subagent at T+0 — security guardrails became first-class requirements before any code was written.
- Rust over Python for the production substrate; FUSE perf matters under swarm load.
- `fuser` with `default-features = false` to avoid the `libfuse-dev` apt dep on hosts without sudo.

See [PROJECT-STATUS.md](PROJECT-STATUS.md) for the timeline + outstanding items, and [MORNING-BRIEF.md](MORNING-BRIEF.md) for the executive summary.

---

[Unreleased]: https://github.com/reubenjohn/reposix/compare/v0.2.0-alpha...HEAD
[v0.2.0-alpha]: https://github.com/reubenjohn/reposix/compare/v0.1.0...v0.2.0-alpha
[v0.1.0]: https://github.com/reubenjohn/reposix/releases/tag/v0.1.0
