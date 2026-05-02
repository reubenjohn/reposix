# Crate layout, clean areas, and net ROI

[← index](./index.md)

## Crate-layout sanity check

The 9-crate workspace (`reposix-core`, `reposix-sim`, `reposix-remote`, `reposix-cli`, `reposix-github`, `reposix-jira`, `reposix-swarm`, `reposix-confluence`, `reposix-cache`) is **structurally clean post-v0.9.0**. Each crate has a single Cargo identity, a `description` line, and a clear role per `CLAUDE.md`. Recommendations:

- **No merge candidates.** The four backend connectors (`reposix-{github,confluence,jira,sim}`) are split for the right reason (each pulls a backend-specific dep tree; `reqwest` features differ).
- **No split candidates.** `reposix-cli` is the largest (`history.rs`, `gc.rs`, `tokens.rs`, `doctor.rs`, `worktree_helpers.rs` etc.) but each module is <500 lines; not yet a split-pressure point.
- **Potential future merge: `reposix-swarm` → `reposix-cli` testbench module** — Swarm is a one-binary harness; but its `chaos_audit.rs` + `contention_e2e.rs` + `confluence_real_tenant.rs` integration tests *are* the v0.7.0+ hardening evidence. Keep separate until we know whether the upcoming `reposix-bench` crate (mentioned in STATE.md v0.11.0 plans) lands; that may absorb swarm.
- **`reposix-bench` was forecast** in STATE.md v0.10.0 entry ("`cargo run -p reposix-bench`") but not present. Either the plan changed (likely — `scripts/v0.9.0-latency.sh` filled the role) or it's deferred. Track in backlog.
- **Fixtures live next to each crate** (`crates/*/fixtures/`) — correct convention; avoids `tests/fixtures/`-style cross-crate sharing.

`docs/reference/crates.md` is the public surface map — verify it covers all 9 (CATALOG-v2 says it's `rewrite-needed`; v0.11.0 may have addressed this).

## What's clean and shouldn't be touched

- **`.github/`** — 4 issue templates, 1 PR template, 1 dependabot.yml, 6 workflow files (`audit`, `bench-latency-cron`, `ci`, `docs`, `release`, `release-plz`). All current; only one stale comment in `ci.yml:55` ("v0.9.0 architecture pivot: the FUSE-backed `integration (mounted FS)`") and that's correctly historical.
- **`crates/reposix-cache/`** — clean post-Phase 31 build.
- **`crates/reposix-core/`** — single source of truth for `Tainted`, `Untainted`, `BackendConnector`. Compile-fail tests prove the type-system invariants.
- **`crates/reposix-cli/tests/`** (12 integration test files) — one test per subcommand surface; matches CLI dispatcher 1:1.
- **`crates/reposix-remote/tests/{stateless_connect,push_conflict,bulk_delete_cap,protocol}.rs`** — these are the v0.9.0 architecture's test moat.
- **`docs/concepts/`, `docs/how-it-works/`, `docs/tutorials/`, `docs/guides/`, `docs/reference/`, `docs/benchmarks/`** — Diátaxis IA from Phase 43; clean.
- **`runtime/`** — gitignored; no `git ls-files` matches under `runtime/`. The 3 sqlite files on disk (`sim.db{,-shm,-wal}`) are correctly ignored.
- **`target/`, `site/`, `.pytest_cache/`, `.playwright-mcp/`** — all gitignored; verified clean.
- **`.claude/skills/{reposix-agent-flow,reposix-banned-words}/`** — committed per OP-4 carve-out; aligned with `CLAUDE.md`.
- **`examples/`** — 5 example dirs, each with `RUN.md` + script + `expected-output.md`. Matches the launch-checklist count.
- **`benchmarks/fixtures/`** — 4 fixture pairs; `check_fixtures.py` validates them.

---

**Net cleanup ROI:** rec #1 (delete `scripts/demos/` + `docs/demos/recordings/`) and rec #5 (condense 8 milestone phase dirs) together remove ~280 files and ~2 MB of stale planning text. Combined with rec #2, #3, #4, #7, #9, #10 the repo loses ~310 files (~45% reduction in `.planning/` markdown count) without touching active code or shipped requirements.

**Sequencing.** Do recs #1–#4 + #7 + #9–#10 first (mechanical, low-risk). Defer rec #5 until v0.11.0 tag pushes (don't reshape `.planning/` while a milestone is mid-flight). Defer rec #6 + the `MILESTONE-AUDIT.md` reshuffle until immediately after v0.11.0 audit ships. Rec #8 (rename version-pinned latency artifacts) waits for v0.12.0 first regen.
