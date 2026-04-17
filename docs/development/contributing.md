# Contributing

!!! note "reposix v0.1 was built autonomously"
    The entire v0.1 codebase was written by a single coding-agent session overnight on 2026-04-13, following the [Get Shit Done](https://github.com/reubenjohn/get-shit-done) workflow. The `.planning/` directory is the full audit trail — PROJECT.md, ROADMAP.md, per-phase CONTEXT.md + PLAN.md + REVIEW.md + DONE.md files, a threat model, and a final VERIFICATION.md.
    
    This page is written for a human (or another agent) picking the project up from here.

## Quickstart for contributors

```bash
git clone https://github.com/reubenjohn/reposix
cd reposix

# Fast type-check (no codegen — use this during development)
cargo check --workspace

# Build everything
cargo build --workspace

# Run the full test suite (133 tests; one #[ignore]'d timeout test runs via --ignored)
cargo test --workspace

# Run the lint + format gates CI enforces
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings

# Run the end-to-end demo
cargo build --release --workspace --bins
export PATH="$PWD/target/release:$PATH"
bash scripts/demo.sh
```

## What the code layout looks like

```text
reposix/
├── crates/
│   ├── reposix-core/       # shared types, HttpClient, Tainted<T>, audit schema
│   ├── reposix-sim/        # axum REST simulator
│   ├── reposix-fuse/       # FUSE daemon
│   ├── reposix-remote/     # git-remote-reposix helper
│   └── reposix-cli/        # top-level orchestrator
├── docs/                   # this MkDocs site (docs/*.md)
├── scripts/                # demo.sh + CI smoke tests + goal-backward gates
├── .planning/              # GSD planning artifacts (PROJECT, ROADMAP, per-phase)
├── .github/workflows/      # CI: fmt + clippy + test + integration + coverage
├── clippy.toml             # workspace disallowed-methods lint
└── rust-toolchain.toml     # stable channel
```

## Non-negotiable invariants

Anything in this list is test-enforced. Break it and CI (or a clippy lint) will refuse the change.

1. **`#![forbid(unsafe_code)]`** at every crate root. The FUSE callbacks are safe Rust (via `fuser`). POSIX syscalls that require `unsafe` are gated through `rustix`.
2. **`reqwest::Client` is banned outside `crates/reposix-core/src/http.rs`.** Clippy `disallowed-methods` enforces this workspace-wide. The seal test `scripts/check_clippy_lint_loaded.sh` verifies the lint is loaded.
3. **Every public `Result`-returning function has an `# Errors` doc section.** Clippy `missing-errors-doc` is on.
4. **No hand-rolled JSON / YAML serialization.** Always `serde`. Always `serde_yaml` for frontmatter.
5. **Clippy is `pedantic` with targeted allows only.** Blanket `#[allow(clippy::pedantic)]` is a code-review failure.
6. **Workspace compiles on Rust stable.** No nightly features. `rust-toolchain.toml` pins `stable`.
7. **Every outbound HTTP request passes the allowlist.** The sealed `HttpClient` newtype is the only construction path. Per-request URL recheck catches redirect follow-ups.
8. **Server-controlled frontmatter fields are stripped on egress.** `sanitize(Tainted<Issue>, ServerMetadata) -> Untainted<Issue>`.

## The GSD workflow

This project uses [GSD (Get Shit Done)](https://github.com/reubenjohn/get-shit-done) for planning. If you are extending reposix, the canonical flow is:

1. `/gsd-add-phase` — add a new phase to `.planning/ROADMAP.md`.
2. `/gsd-discuss-phase <N>` — conversational context gathering (or write `.planning/phases/XX/CONTEXT.md` directly if you know what you want).
3. `/gsd-plan-phase <N>` — produces one or more `PLAN.md` files via a planner subagent.
4. `/gsd-execute-phase <N>` — executes the plans wave-by-wave.
5. `/gsd-code-review` + `/gsd-verify-work` — quality gates before the phase closes.

For a small change (bug fix, doc tweak), skip to `/gsd-quick`.

For a larger change that requires design discussion, use `/gsd-discuss-phase` first — that's the step that was deliberately skipped during the overnight autonomous build.

## Running the simulator alone

```bash
cargo run -p reposix-sim -- \
    --bind 127.0.0.1:7878 \
    --db runtime/sim.db \
    --seed-file crates/reposix-sim/fixtures/seed.json

# In another shell
curl -s http://127.0.0.1:7878/projects/demo/issues | jq length
```

## Building the docs site

```bash
pip install --user mkdocs-material mkdocs-minify-plugin
mkdocs serve    # → http://127.0.0.1:8000
```

The deployed site lives at <https://reubenjohn.github.io/reposix/>.

## Pre-commit hooks (recommended)

```bash
cargo install pre-commit-rs  # or use your preferred runner
cat > .pre-commit-config.yaml <<'EOF'
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt --check
        entry: cargo fmt --all --check
        language: system
        pass_filenames: false
      - id: cargo-clippy
        name: cargo clippy -D warnings
        entry: cargo clippy --workspace --all-targets -- -D warnings
        language: system
        pass_filenames: false
EOF
pre-commit install
```

## Reporting bugs

Open a GitHub issue. Include:

- `cargo --version` and `uname -a`
- `fusermount3 --version`
- The exact command that failed, and the output (with `RUST_LOG=debug` ideally)
- Whether you're using the in-process simulator or a real backend

For security issues, see [SECURITY.md](https://github.com/reubenjohn/reposix/blob/main/SECURITY.md) (v0.2 — until then, email the repo owner directly).
