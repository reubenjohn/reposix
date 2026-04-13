# reposix

> A git-backed FUSE filesystem that exposes REST APIs as POSIX directories so autonomous LLM agents can use `cat`, `grep`, `sed`, and `git` instead of MCP tool schemas.

[![CI](https://github.com/reubenjohn/reposix/actions/workflows/ci.yml/badge.svg)](https://github.com/reubenjohn/reposix/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](rust-toolchain.toml)

## Why

Modern coding agents have ingested vast amounts of Unix shell scripting and `git` workflows during pre-training. Asking them to use a `cat` + `git commit` workflow is asking them to do what they already know how to do. Asking them to use Model Context Protocol (MCP) is asking them to load 100k+ tokens of JSON schemas before doing anything useful.

reposix takes the second problem and reduces it to the first. A Jira board, a GitHub Issues repo, or a Confluence space becomes a directory of Markdown files with YAML frontmatter, with native `git push` synchronization and merge-conflict-as-API-conflict semantics.

See [`InitialReport.md`](InitialReport.md) for the full architectural argument.

## Status

**v0.1 (in development).** Built overnight 2026-04-13 as an autonomous-build experiment. The current scope:

- ✅ Cargo workspace with 5 crates (`-core`, `-sim`, `-fuse`, `-remote`, `-cli`).
- ✅ Issue model with YAML-frontmatter Markdown serialization.
- 🚧 In-process simulator (axum) mimicking GitHub-Issues + Jira semantics with rate limits, 409s, workflow rules.
- 🚧 FUSE daemon (`fuser`) with read+write.
- 🚧 `git-remote-reposix` helper (fast-export/fast-import based).
- 🚧 End-to-end demo: mount → edit → `git commit` → `git push` → simulator state changes.
- 🚧 CI green on GitHub Actions (lint + test + integration mount + coverage).

Tracking artifacts live in [`.planning/`](.planning/).

## Quickstart (when phases 2-5 land)

```bash
# 1. Start the simulator
cargo run -p reposix-sim &

# 2. Mount the FUSE filesystem
mkdir /tmp/reposix-mnt
cargo run -p reposix-fuse -- /tmp/reposix-mnt

# 3. Use it like a git repo
cd /tmp/reposix-mnt/projects/demo
git init
git remote add origin reposix::http://localhost:7777/projects/demo
git pull origin main
ls
# 0001.md  0002.md  0003.md  index.md
cat 0001.md
# ---
# id: 1
# title: thing is broken
# status: open
# ...
# ---
# Steps to reproduce:
# 1. ...

sed -i 's/^status: open/status: in_progress/' 0001.md
git commit -am "claim issue 1"
git push origin main
# → translated to PATCH /projects/demo/issues/1 {status: "in_progress"}
```

## Architecture

```
┌──────────┐   git    ┌──────────────────┐  HTTP   ┌──────────────────┐
│  agent   │ ───────▶ │ git-remote-      │ ──────▶ │ reposix-sim      │
│ (shell)  │          │ reposix          │         │ (or real Jira)   │
└──────────┘          └──────────────────┘         └──────────────────┘
     │                         │                            ▲
     │ POSIX                   │ tokio                      │
     ▼                         │                            │
┌──────────┐                   │                            │
│ FUSE     │ ──────────────────┴────────────────────────────┘
│ mount    │   reqwest (HTTP allowlist enforced)
└──────────┘
     ▲
     │ fusermount3
     ▼
┌──────────┐
│ kernel   │
│  VFS     │
└──────────┘
```

See [`docs/architecture.md`](docs/architecture.md) (TODO).

## Security

This project is a textbook **lethal trifecta** (Simon Willison's framing): private data + untrusted input + exfiltration channel. Mitigations are first-class and tested — see [`CLAUDE.md`](CLAUDE.md#threat-model) and [`research/threat-model-and-critique.md`](research/threat-model-and-critique.md).

**v0.1 does not connect to any real backend.** All development and CI runs against the in-process simulator. Real-credential paths require explicit user opt-in and ship with allowlist enforcement.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
