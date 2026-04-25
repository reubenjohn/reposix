# Contributing to reposix

Thanks for showing up. reposix is built around a small set of strong constraints — pure-git agent UX, simulator-first testing, and the lethal-trifecta threat model. Most of what makes a contribution land cleanly is understanding those constraints; the rest is mechanics. This file covers the mechanics.

## Code of conduct

By participating in this project you agree to abide by the [Code of Conduct](CODE_OF_CONDUCT.md). In short: be patient, assume good faith, and direct conduct concerns to **reubenvjohn@gmail.com**.

## Welcome — what kind of contribution fits

| You want to… | Path |
|---|---|
| Report a bug | Open a [bug report issue](.github/ISSUE_TEMPLATE/bug_report.md) |
| Propose a feature | Open a [feature request issue](.github/ISSUE_TEMPLATE/feature_request.md) (read the threat-model section first) |
| Add support for a new backend | Open a [connector proposal](.github/ISSUE_TEMPLATE/connector_proposal.md), then read [`docs/guides/write-your-own-connector.md`](docs/guides/write-your-own-connector.md) |
| Report a vulnerability | **Do not** open an issue. See [SECURITY.md](SECURITY.md) |
| Fix a typo, tighten a doc | Open a PR; small docs PRs skip the GSD phase machinery |

## Development setup

### Prerequisites

- Rust stable (≥ 1.82) — pinned via `rust-toolchain.toml`, so a fresh `rustup` install will pick the right toolchain automatically.
- `git ≥ 2.34` (required for partial-clone protocol v2).
- `pre-commit` (Python) for the docs lint hook — `pip install pre-commit`.

### Clone and verify

```bash
git clone https://github.com/reubenjohn/reposix
cd reposix

# Fast type-check (use this during development — no codegen)
cargo check --workspace

# Full test suite
cargo test --workspace

# CI lint and format gates — these MUST be clean before you push
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# Auto-format if needed
cargo fmt --all
```

If any of those four commands fail on a clean checkout of `main`, that's a bug — please open an issue.

### Pre-commit hooks

Two layers of hooks run on this repo:

1. **`pre-commit` (Python)** — runs `scripts/banned-words-lint.sh` against `docs/` on every commit.
   ```bash
   pip install pre-commit
   pre-commit install
   ```
2. **Project git hooks** — `scripts/install-hooks.sh` symlinks `scripts/hooks/*` into `.git/hooks/`. Currently this installs a `pre-push` check.
   ```bash
   bash scripts/install-hooks.sh
   ```

Both should be installed once per fresh clone. CI re-runs the same scripts, so you can't bypass them by skipping local install — your PR will just fail CI instead.

## The GSD workflow

This project uses [GSD (`get-shit-done`)](https://github.com/reubenjohn/get-shit-done) for planning and execution. The full state of the project lives in [`.planning/`](.planning/) — that directory is the audit trail.

> **Always enter through a GSD command.** Don't edit code outside a GSD-tracked phase or quick.

Entry points (slash-commands when running inside Claude Code; bare names otherwise):

- `/gsd-quick` — small fix or doc tweak (no phase artifacts).
- `/gsd-add-phase` → `/gsd-discuss-phase <N>` → `/gsd-plan-phase <N>` → `/gsd-execute-phase <N>` → `/gsd-code-review` + `/gsd-verify-work` — the full phase development cycle.
- `/gsd-progress` — show the project's current state.
- `/gsd-debug` — investigate a bug with persistent state across context resets.

Read [`.planning/STATE.md`](.planning/STATE.md) before starting work — it tells you what phase the project is mid-flight on.

### Phase development cycle

```
plan-phase  →  execute-phase  →  code-review-fix  →  verify-work  →  ship
   |               |                  |                 |
   |               |                  |                 └── User-acceptance loop
   |               |                  └── Auto-fix issues from REVIEW.md
   |               └── Wave-based parallel execution (subagents)
   └── PLAN.md authored by gsd-planner subagent
```

You almost never want to write `PLAN.md` by hand. The planner subagent is opinionated about scope, risk, and verification structure for a reason.

## Coding conventions

These rules are enforced by clippy, the test suite, and CI — they're not aspirational. Breaking them fails CI; CI is non-negotiable.

### Hard rules (test-enforced)

1. **`#![forbid(unsafe_code)]`** at every crate root. POSIX syscalls go through `rustix` if needed.
2. **`#![warn(clippy::pedantic)]`** at every crate root. Allow-list specific lints with rationale comments; never blanket-allow.
3. **All public `Result`-returning functions have an `# Errors` doc section.** Clippy `missing-errors-doc` is on.
4. **`reqwest::Client::new()` is banned outside `crates/reposix-core/src/http.rs`.** Use `reposix_core::http::client()` — the egress allowlist depends on it. Clippy `disallowed-methods` enforces this workspace-wide.
5. **No hand-rolled JSON / YAML serialization.** Always `serde`. Frontmatter uses `serde_yaml` 0.9 + Markdown body.
6. **Times are `chrono::DateTime<Utc>`.** No `SystemTime` in serialized form.
7. **Tests live next to the code.** `#[cfg(test)] mod tests` for unit; `tests/` for integration.
8. **Server-controlled frontmatter fields are stripped on egress.** `id`, `created_at`, `version`, `updated_at` cannot be overridden by client writes — see `Tainted<T>` / `Untainted<T>` in `reposix-core`.

### Soft conventions (reviewer-enforced)

- `thiserror` for typed crate errors; `anyhow` only at binary boundaries.
- `tokio` for async; no other runtimes in the workspace.
- `axum` for servers, `reqwest` (rustls only — never `openssl-sys`) for clients.
- `rusqlite` with `bundled` feature; do not depend on system `libsqlite3`.

When in doubt, search for an existing pattern before inventing a new one. The simulator at `crates/reposix-sim/` is a good reference.

## Adding a new BackendConnector

If your contribution is "support `<system>`", read [`docs/guides/write-your-own-connector.md`](docs/guides/write-your-own-connector.md) first. The trait surface is small but specific: list, get, create, update, delete, plus conflict semantics. Open a [connector proposal issue](.github/ISSUE_TEMPLATE/connector_proposal.md) before implementing — the proposal captures conflict-detection assumptions that are hard to retrofit.

The simulator (`reposix-sim`) is the reference behaviour. New connectors must pass the same conformance tests against a fixture or live sandbox.

## Submitting a PR

1. **Branch off `main`.** Use a descriptive name: `feat/jira-bulk-list`, `fix/push-conflict-leak`, `docs/contributing-clarify-hooks`.
2. **Atomic commits.** One logical change per commit. Conventional commit prefixes (`feat:`, `fix:`, `docs:`, `chore:`, `test:`, `refactor:`) are encouraged but not required by CI.
3. **Run the local gates before pushing:**
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   pre-commit run --all-files
   ```
4. **Fill out the PR template.** The threat-model-impact table is the section reviewers care most about — be honest if a change adds new egress, new tainted-byte paths, or new audit ops.
5. **Update `CHANGELOG.md`** if your change is user-visible.
6. **Don't merge your own PR** unless you're a maintainer and the change is `docs:` only.

## License

reposix is dual-licensed under [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE) (the standard Rust ecosystem dual-license). By submitting a PR you agree your contribution is licensed under both.

## Communication channels

- **Bugs / features / connector proposals** — GitHub Issues (templates linked above).
- **Open-ended questions, agent-integration patterns** — GitHub Discussions: <https://github.com/reubenjohn/reposix/discussions>.
- **Security** — `reubenvjohn@gmail.com` or GitHub Security Advisories. See [SECURITY.md](SECURITY.md).

Discord / chat is not currently set up. If contribution volume reaches the point where async issues are the bottleneck, that's a good problem to have and we'll add a chat channel then.
