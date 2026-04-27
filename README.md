# reposix

> Edit issues as files. `git push` to sync. Zero MCP schema tokens.

reposix exposes REST-based issue trackers (and similar SaaS systems) as a
git-native partial clone, served by `git-remote-reposix` from a local
bare-repo cache built from REST responses. Agents use `cat`, `grep`, `sed`,
and `git` on real workflows — no MCP tool schemas, no custom CLI.

[![CI](https://github.com/reubenjohn/reposix/actions/workflows/ci.yml/badge.svg)](https://github.com/reubenjohn/reposix/actions/workflows/ci.yml)
[![Docs](https://github.com/reubenjohn/reposix/actions/workflows/docs.yml/badge.svg)](https://reubenjohn.github.io/reposix/)
[![codecov](https://codecov.io/gh/reubenjohn/reposix/branch/main/graph/badge.svg)](https://codecov.io/gh/reubenjohn/reposix)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](rust-toolchain.toml)
[![Release](https://img.shields.io/github/v/release/reubenjohn/reposix?include_prereleases)](https://github.com/reubenjohn/reposix/releases)

**Full docs and narrative:** <https://reubenjohn.github.io/reposix/>

## Three measured numbers

- **`8 ms`** — read one issue from the local cache after first fetch ([`docs/benchmarks/latency.md`](docs/benchmarks/latency.md)).
- **`24 ms`** — `reposix init` cold bootstrap against the simulator (soft threshold `500 ms`).
- **`89.1%`** — input-context-token reduction vs a synthesized MCP-tool-catalog baseline for the same task, measured in [`docs/benchmarks/token-economy.md`](docs/benchmarks/token-economy.md) (v0.7 token-economy benchmark, recalibrated to real Anthropic tokenization in v0.10.0; the architectural argument is unchanged in v0.9.0). The MCP comparison fixture is synthesized from public Atlassian Forge tool surfaces — see the artifact for methodology.

## What it is

A git remote helper plus an on-disk cache. After `reposix init <backend>::<project> <path>`, the working tree is a real partial-clone git checkout. Reading is `cat` / `grep -r`; writing is `sed` + `git commit`; syncing is `git push`. `git pull --rebase` recovers from conflicts the standard way. reposix complements REST — complex JQL, bulk imports, and admin operations stay on the API.

The 5-minute first-run tutorial lives at [`docs/tutorials/first-run.md`](docs/tutorials/first-run.md). The architectural argument and progressive-disclosure narrative live at <https://reubenjohn.github.io/reposix/>.

## Install

All eight crates ship to crates.io and prebuilt binaries land on every GitHub Release. Pick whichever fits your platform — these are the supported install paths.

```bash
# macOS / Linux: Homebrew (tap is reubenjohn/reposix)
brew install reubenjohn/reposix/reposix

# Cross-platform: cargo binstall (no compile)
cargo binstall reposix-cli reposix-remote

# curl | sh (Linux/macOS)
curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.sh | sh

# Windows: PowerShell + irm
powershell -ExecutionPolicy Bypass -c "irm https://github.com/reubenjohn/reposix/releases/latest/download/reposix-installer.ps1 | iex"
```

Other paths (Docker, prebuilt archives, source build) are documented in [`docs/tutorials/first-run.md`](docs/tutorials/first-run.md).

## Quick start (5 min)

Once you have `reposix` and `git-remote-reposix` on `PATH`:

```bash
# Start the simulator.
reposix sim --bind 127.0.0.1:7878 &

# Bootstrap a partial-clone working tree.
reposix init sim::demo /tmp/reposix-demo
cd /tmp/reposix-demo

# Agent UX is pure git from here.
git checkout -B main refs/reposix/origin/main   # helper namespaces fetched refs
ls issues/                          # 0001.md  0002.md  ...
sed -i 's/TODO/DONE/' issues/0001.md
git commit -am 'mark issue 1 done'
git push                            # round-trips through the helper to the backend
```

The full walkthrough — including the `git pull --rebase` conflict cycle and the `git sparse-checkout` blob-limit recovery — is in [`docs/tutorials/first-run.md`](docs/tutorials/first-run.md).

<details>
<summary><strong>Build from source (advanced)</strong></summary>

Linux. Requires Rust stable 1.82+ and `git >= 2.34`.

```bash
git clone https://github.com/reubenjohn/reposix && cd reposix
cargo build --release --workspace --bins
export PATH="$PWD/target/release:$PATH"
```

</details>

## How it works (one paragraph)

`git-remote-reposix` is a hybrid promisor remote. It advertises `stateless-connect` for reads (tunnelling protocol-v2 fetch traffic to a local bare-repo cache built lazily from REST responses) and `export` for push (parsing the fast-import stream, doing push-time conflict detection against backend state, applying writes via REST). Tree metadata syncs eagerly (cheap); blobs materialize on demand and are capped by `REPOSIX_BLOB_LIMIT`. Every byte is `Tainted<T>` until an explicit `sanitize()` strips server-controlled fields (`id`, `created_at`, `version`, `updated_at`). The architecture trio — cache layer, git layer, trust model — is in [`docs/how-it-works/`](docs/how-it-works/).

## Connectors

| Backend | Crate | Read | Write | Status |
|---------|-------|------|-------|--------|
| Simulator | `crates/reposix-sim` | yes | yes | default for tests / autonomous loops |
| GitHub Issues | `crates/reposix-github` | yes | no | live against `reubenjohn/reposix`; write path deferred |
| Confluence Cloud | `crates/reposix-confluence` | yes | yes | live against `reuben-john.atlassian.net` (TokenWorld space) |
| JIRA Cloud | `crates/reposix-jira` | yes | yes | live against `JIRA_TEST_PROJECT` (default `TEST`) |

Real-backend test targets and env-var setup: [`docs/reference/testing-targets.md`](docs/reference/testing-targets.md).

## Project status

- **v0.9.0 — shipped 2026-04-24.** Architecture pivot to git-native partial clone. `crates/reposix-fuse/` deleted; `git-remote-reposix` is now a hybrid promisor remote (`stateless-connect` reads + `export` push). Migration: `reposix mount /tmp/m --backend sim --project demo` becomes `reposix init sim::demo /tmp/m`. See [`CHANGELOG.md`](CHANGELOG.md#v090--2026-04-24).
- **v0.10.0 — landing 2026-04-25.** Diátaxis-structured docs site, 5-minute tutorial verified by `scripts/tutorial-runner.sh`, mental-model + vs-MCP concept pages, banned-words linter, README rewritten for v0.9.0 surface.

`cargo test --workspace` is green; `cargo clippy --workspace --all-targets -- -D warnings` is clean; `bash scripts/dark-factory-test.sh sim` passes the dark-factory regression. `#![forbid(unsafe_code)]` at every crate root.

Treat as alpha per Simon Willison's "proof of usage, not proof of concept" rule — the v0.9.0 quickstart above is reproducible on a stock Ubuntu host in under five minutes against the in-process simulator, with no system packages required beyond `git >= 2.34` and a Rust toolchain.

## Security

reposix is a textbook lethal-trifecta machine (private remote data + untrusted ticket text + `git push` exfiltration). Cuts that are mandatory and tested:

- **Outbound HTTP allowlist** (`REPOSIX_ALLOWED_ORIGINS`) — the helper and cache refuse origins not listed; one factory (`reposix_core::http::client()`), one clippy lint.
- **Push-time conflict detection** — stale-base pushes are rejected with the standard `error refs/heads/main fetch first` line; agents recover via `git pull --rebase`.
- **Blob-limit guardrail** — fetches over `REPOSIX_BLOB_LIMIT` (default `200`) are refused with a stderr message that names `git sparse-checkout` (self-teaching dark-factory pattern).
- **Frontmatter field allowlist** — server-controlled fields are stripped from inbound writes before the REST call.
- **Audit log append-only** — every blob materialization, every `command=fetch`, every push (accept and reject) writes a row; SQLite `BEFORE UPDATE/DELETE RAISE` triggers enforce immutability.

The trust-model page at [`docs/how-it-works/trust-model.md`](docs/how-it-works/trust-model.md) walks through these end-to-end.

## Contributing

- Build & test: `cargo check --workspace && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`.
- Docs: `mkdocs serve` (Python 3.8+; install `mkdocs-material`).
- Project workflow uses GSD (`get-shit-done`) — see [`CLAUDE.md`](CLAUDE.md) and [`.planning/`](.planning/).
- Threat model: [`docs/how-it-works/trust-model.md`](docs/how-it-works/trust-model.md) plus the original red-team report in `.planning/research/v0.1-fuse-era/threat-model-and-critique.md`.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
