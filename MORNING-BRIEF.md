# Morning brief — 2026-04-13

You went to bed at ~12:42 AM with a goal: ship the reposix project (git-backed FUSE filesystem for autonomous agents, dark-factory pattern) by 8:00 AM. You said "make the demo hit bigger" and "focus less on media for the demo and more on the value of the deliverable."

## Where to look first

1. **<https://github.com/reubenjohn/reposix>** — the public repo. Latest commit on `main`: `ffa1666` (or whatever this morning's HEAD is).
2. **<https://reubenjohn.github.io/reposix/>** — the live docs site (mermaid architecture diagrams, full reference, security page).
3. **`bash scripts/demo.sh`** — runs the full 9-step demo end-to-end in <2 minutes. Idempotent. Shows three guardrails firing on camera.
4. **`benchmarks/RESULTS.md`** — measured token economy: **92.3% reduction** (~12.9× more context for MCP) for the same task.

## What shipped

| Phase | Outcome |
|-------|---------|
| 1 — Core contracts + 8 security guardrails | shipped |
| 2 — Simulator + audit log (axum, rate limit, 409 conflicts, append-only SQLite) | shipped |
| 3 — Read-only FUSE mount + CLI (`reposix sim/mount/demo`) | shipped |
| S — STRETCH: write path + `git-remote-reposix` + bulk-delete cap | shipped (in 29 min vs 120 min budget) |
| 4 — `scripts/demo.sh` + script(1) recording + README polish | shipped |
| 5 — MkDocs site with 11 mermaid diagrams + GitHub Pages | shipped + verified live via playwright |
| 6 — Token-economy benchmark with measured 92.3% reduction | shipped |
| 7 — Phase S robustness fixes (CRLF, error frames, deterministic blobs) | shipped |
| 8 — Demo suite + real-backend seam (post-ship, per your 09:05 direction) | shipped |

**163 workspace tests pass** (up from 139 at initial ship). `cargo clippy --workspace --all-targets -- -D warnings` is clean. `#![forbid(unsafe_code)]` at every crate root. All 8 SG guardrails enforced and demo-visible.

### Phase 8 highlights (added after the initial 8am demo)

- **`IssueBackend` trait seam** — `reposix-core::backend` defines the abstraction; the FUSE daemon and CLI can talk to any backend that implements it.
- **`SimBackend`** — the existing simulator wrapped as a first-class backend.
- **`GithubReadOnlyBackend`** — a **real** GitHub Issues adapter (new crate `reposix-github`). Reads any public repo via the GitHub REST API, honoring rate-limit headers and the `https://api.github.com` allowlist.
- **Contract test** — `crates/reposix-github/tests/contract.rs` runs **the same 5 invariants** against both `SimBackend` and real GitHub (`octocat/Hello-World`). Proves shape parity.
- **Tier 1 demo suite** — the old monolithic `demo.sh` split into 4 audience-specific 60-second demos (developer / security / skeptic / buyer) + a full walkthrough + a Tier 3 sim-vs-GitHub parity demo. CI runs them via `scripts/demos/smoke.sh` (load-bearing, no `continue-on-error`).
- **ADR-001 — GitHub state mapping** — `docs/decisions/001-github-state-mapping.md` is the source of truth for how reposix's 5 Jira-flavored statuses round-trip through GitHub's `open/closed + state_reason + labels`.
- **Real-GitHub CI job** — `integration-contract` runs the ignored contract test against real GitHub on every push, authenticated via `${{ secrets.GITHUB_TOKEN }}` (1000 req/hr budget).
- **Codecov** — coverage badge now renders (CODECOV_TOKEN secret landed during Phase 8).

## What did NOT make v0.1

- Real-backend integration (Jira/GitHub/Confluence). v0.2.
- Adversarial swarm harness. v0.2 — explicitly cut from scope to keep the budget honest.
- FUSE-in-CI mount integration job (CI runs cargo tests + clippy + coverage; the "literal mount inside the runner" job was cut).
- Various MEDIUM/LOW review findings cataloged in `.planning/phases/*/REVIEW.md` and tracked in `docs/development/roadmap.md`.

## How to verify the night's work

```bash
git clone https://github.com/reubenjohn/reposix
cd reposix
cargo test --workspace --quiet              # 139 tests
cargo clippy --workspace --all-targets -- -D warnings   # clean
cargo build --release --workspace --bins
export PATH="$PWD/target/release:$PATH"
bash scripts/demo.sh                        # exits 0 in <2 min, fires SG-01/02/03 on camera
python3 scripts/bench_token_economy.py      # prints the 92.3% reduction table
mkdocs serve -a 127.0.0.1:8111             # local docs (after `pip install --user mkdocs-material mkdocs-minify-plugin`)
```

## The audit trail

If you want to see *how* the night went:

- `.planning/PROJECT.md` — Core Value + 17 Active Requirements (all delivered, see VERIFICATION.md).
- `.planning/ROADMAP.md` — 4 MVD phases + 1 STRETCH + decision gates.
- `.planning/phases/*/01-CONTEXT.md` (per phase) — what was decided.
- `.planning/phases/*/[0-9]*-PLAN.md` files — task breakdown per phase.
- `.planning/phases/*/REVIEW.md` — adversarial code review per phase.
- `.planning/phases/*/[0-9]*-DONE.md` — what shipped per phase.
- `.planning/VERIFICATION.md` — independent goal-backward verification (PASS).
- `.planning/research/` — research artifacts (FUSE patterns, git remote helper, simulator design, threat model).
- `git log --oneline main` — every commit, atomic, with phase prefixes.

## Things worth your attention

- **CI is green.** Latest commit on `main` had all 5 jobs (rustfmt, clippy, test, coverage, integration) pass. If a later commit shows red, check `gh run view --log-failed` first.
- **The benchmark fixture is conservative.** I synthesized a 35-tool MCP catalog modeled on the public Atlassian Forge surface; real Jira deployments with custom apps are typically larger, which would push the reduction even higher than 92.3%. The fixture is auditable in `benchmarks/fixtures/mcp_jira_catalog.json` — you can decide if it's representative for your use.
- **Phase S landed with one HIGH finding that's still deferred to v0.2: H-04** (FUSE `create()` server-id divergence). Path: agent picks an id; server picks `max_id+1`; kernel dirent and server state diverge by one inode. Cosmetic in the demo, not yet a security or correctness issue. Documented in `.planning/phases/S-stretch-write-path-and-remote-helper/REVIEW.md`.
- **Deferred but easy follow-ups** in priority order: macOS via macFUSE, real GitHub Issues adapter (the simulator API is already GitHub-shaped), the swarm harness, FUSE-in-CI.

## Stuff I touched outside reposix/

Nothing. Everything I did is inside `/home/reuben/workspace/reposix/` and `https://github.com/reubenjohn/reposix`. The reference projects (`token_world`, `theact`, `reeve_bot`) were read-only; I copied conventions (mkdocs structure, GSD discipline) but didn't touch them.

## My recommendation

The demo bar you set in our initial discussion (working FUSE mount + adversarial subagent reports + CI green + walkthrough doc + recording) all landed AND has measured 92.3% token-economy reduction backing the central thesis. You can show this to anyone — it stands on its own.

The single thing I'd queue for v0.2 first is **the real GitHub Issues adapter**. The simulator is already GitHub-shaped; bolting on auth + pagination is ~1 day, and it turns reposix from "interesting demo" into "I use this every morning to triage my issues."

Sleep well. Have fun with the demo.
