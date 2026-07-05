# crates/CLAUDE.md — Rust workspace rules (auto-loaded under crates/)

Extends root `CLAUDE.md`. Long-form build/memory doctrine lives here; root keeps a
pointer. Full orchestration doctrine: `.planning/ORCHESTRATION.md`.

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
