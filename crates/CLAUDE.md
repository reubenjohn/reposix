# crates/CLAUDE.md — Rust workspace rules (auto-loaded under crates/)

Extends root `CLAUDE.md`. Long-form build/memory doctrine + tech stack + workspace
layout live here; root keeps a pointer. Full orchestration doctrine:
`.planning/ORCHESTRATION.md`.

## Workspace layout

```
crates/
├── reposix-core/        # Shared types: Record, Project, RemoteSpec, Error, Tainted<T>.
├── reposix-sim/         # In-process axum HTTP simulator (default backend).
├── reposix-cache/       # On-disk bare-repo cache backed by gix; lazy blob materialization.
├── reposix-remote/      # git-remote-reposix binary (stateless-connect + export).
├── reposix-cli/         # Top-level `reposix` CLI (init, attach, sim, list, refresh, spaces, sync).
├── reposix-github/      # GitHub Issues BackendConnector.
├── reposix-confluence/  # Confluence Cloud BackendConnector.
├── reposix-jira/        # JIRA Cloud BackendConnector.
└── reposix-swarm/       # Multi-agent contention/swarm test harness.
```
(Repo-root siblings: `.planning/` GSD state — do not hand-edit; `docs/` user-facing;
`research/` long-form notes; `runtime/` gitignored sim DB + scratch trees.)

## Tech stack

- Rust stable (1.82+ via `rust-toolchain.toml`).
- Async: `tokio` 1. Web: `axum` 0.7 + `reqwest` 0.12 (rustls only, never openssl-sys).
- Git: `gix` 0.83 (pinned with `=`, gix is pre-1.0). **Git `2.34+` recommended**
  for reliable partial-clone reads / `stateless-connect` (`extensions.partialClone`);
  the simulator flow runs on older git (verified down to 2.25) — `doctor` WARNs, not
  ERRORs, below 2.34.
- Storage: `rusqlite` 0.32 with `bundled` (no system libsqlite3).
- Errors: `thiserror` for typed crate errors, `anyhow` only at binary boundaries.

## Build memory budget (load-bearing — read before parallelizing)

The VM has OOM-crashed **three times** this project from parallel cargo workspace
builds. The workspace links large crates (`gix` chain, `rusqlite-bundled`,
`reqwest+rustls`); a single `cargo check --workspace` peaks ~4–6 GB; `cargo test
--workspace` is worse (rustc + N test binaries link in parallel).

**Hard rules (both apply, no exception without explicit owner override):**
1. **Never more than one cargo invocation at a time, machine-wide** — check/build/test/
   clippy. Two subagents needing compile run SEQUENTIALLY. At most one phase-executor
   doing cargo work at a time. This is now ALSO enforced by
   `.claude/hooks/cargo-mutex.sh` (PreToolUse/Bash → exit 2 if a live cargo/rustc build
   is detected) — the hook is a backstop, orchestration discipline is the primary control.
2. **Prefer per-crate over workspace-wide** — `cargo check -p reposix-cli`, not
   `--workspace`, when the change is scoped. Pre-push covers the workspace gate.

**Pre-push is a hidden cargo lane — serialize pushes machine-wide (fix-twice, v0.14.0 P111).**
`.githooks/pre-push` runs a FULL workspace quality validate — `cargo fmt --check` +
`cargo clippy` unconditionally, plus gitleaks — on EVERY push, regardless of diff size
(measured **~102s including gitleaks on a docs-only commit**; the cost is a fixed
whole-repo walk, not per-changed-file — see `quality/CLAUDE.md` § "Runtime does NOT scale
with diff size"). So a code-landing lane that pushes while ANOTHER session already holds
the machine-wide cargo token spawns a SECOND concurrent workspace compile — the exact OOM
trap Hard rule 1 exists to prevent, just triggered by `git push` instead of a bare `cargo`.
Treat a push like a cargo invocation: **serialize pushes** across all lanes (at most one
`git push` in flight machine-wide, and never push while another lane is mid-cargo). When a
lane only needs the change LANDED (not locally re-linted), prefer relying on post-push CI
(`code/ci-green-on-main` is the authoritative green) over a redundant local full-workspace
clippy — it keeps the single cargo slot free for the lane that actually needs it.

**Soft rules:** `CARGO_BUILD_JOBS=2` (committed in `.cargo/config.toml`); `cargo nextest
run` links test binaries one at a time (prefer for full-workspace tests); close idle
rust-analyzer editors (2–3 GB); schedule docs/playwright work in a no-cargo window.

**If the VM crashes again:** suspect parallel cargo, rust-analyzer, or leftover
background processes (`ps aux | grep -E "cargo|rustc"`). Blame the orchestration that
let two run at once, not the linker.

## Code conventions

- `#![forbid(unsafe_code)]` and `#![warn(clippy::pedantic)]` in every crate; allow-list
  specific lints with rationale, never blanket-allow pedantic.
- All public items documented (missing-doc lint on for `reposix-core`); every
  `Result`-returning fn has a `# Errors` section.
- Tests next to code (`#[cfg(test)] mod tests`); integration tests in `tests/`.
- Frontmatter = `serde_yaml` 0.9 + Markdown body; never JSON-on-disk for issues.
- Times are `chrono::DateTime<Utc>`; no `SystemTime` in serialized form.
- Banned in production `crates/**/*.rs` (outside `tests/`): `\bP\d{2,3}-\d+\b` phase-ID
  tokens (`banned-production-tokens.sh`); use `// banned-words: ok` for justified refs.
  The regex CATCHES v0.13+ phase numbers (`P79-02`, `P150-01`) and INTENTIONALLY MISSES
  v0.8/v0.9-era audit IDs `P\d-\d` (`P1-1` in `error.rs` — code-quality refs, not phase
  IDs). Forward convention: new audit-ID schemes adopt `P\d{2,3}-` numbering or a distinct
  prefix (`AUD-1`). Full rationale in the script header. Separately,
  `deferral-pointer-linter.sh` (pre-push) requires every deferral pointer in `crates/`
  (phrases like "lands in P<N>") to name a real downstream phase with a PLAN artifact
  under `.planning/phases/N-*/`; see the script header for the exact patterns.

## Error-message convention (Rust-compiler-grade UX)

Every user-facing error — every `reposix` CLI subcommand AND the `reposix-remote` git
helper — meets a three-part bar: (1) **teach the fix**, (2) **suggest the alternative**,
(3) **give a copy-paste recovery command**. The pattern to copy is
`reposix-cli/src/init.rs::refuse_existing_repo_root`: it refuses fail-closed, names the
corruption shape, points at `reposix attach` as the alternative, and prints runnable
recovery lines. A bare `bail!("usage: …")` or a terse `.context("parse remote url")` that
surfaces raw to the user does NOT meet the bar — wrap it in a teaching message. Dimension:
**agent-ux / docs-alignment** (route audits to `quality/gates/agent-ux/`). Scheduled as a
first-class v0.15.0 phase — see `.planning/milestones/v0.15.0-phases/ROADMAP.md`.
